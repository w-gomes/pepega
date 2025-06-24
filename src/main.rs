use std::{env::current_dir, fs::File, io::Write, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use tempfile::tempdir_in;

// - Future ideas
// Commands:
// * Clip
//   At the moment we are copying video and audio streams as well as timestamp.
//   Add an option to each stream or both. E.g. --reencode
//
// * Merge
//   At the moment we are multiple videos from the command line using -i flags.
//   It can be a bit tedious if we have a lot of videos.
//   Implement passing the directory and just merge everything in there that
//   has .mp4 or .mkv extensions.
//
// * Video
//   Mostly complete. We should also check for images with different extensions.
//   Currently, for Video command we only support directory as inputs.
//   The video is created from all the images with .png extension inside the dir.
//   Do we want to support specified files like the Merge command?
//
// * Audio
//   At the moment we are extracting audio from the entire video file.
//   We could add the clip functionality and just extract a portion of the video,
//   with a given START and END.
//
// * Filters
//   What kinda of filters do we want though?
//   - Options
//     Scaling e.g. 1280x720 to 320x240
//     Padding
//     Fading (maybe good for Merge command)
//     Drawing Text (top-left, top-right, center, bottom-left, bottom-right,
//                   background, foreground)
//     Timeline Editing, enable filters with specific START and END
//     Speed up at specific sections in the video, slow and fast motions.
//
#[derive(Parser, Debug)]
#[command(
    name = "pepega",
    about = "Smol video and audio tool that uses ffmpeg.",
    version
)]
#[command(
    long_about = "pepega is a small program utility to simplify common video and audio tasks. From clipping and merging videos to extracting audio and encoding for various platforms. pepega leverages the power of FFmpeg"
)]
struct Pepega {
    /// Inputs files.
    /// These are the primary files the command will operate on.
    #[arg(
        short,
        long,
        required = true,
        help = "One or more input files. If extracting audio, it takes only ONE input."
    )]
    inputs: Vec<String>,

    /// Output file.
    #[arg(short, long)]
    output: String,

    /// Options for the program.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Creates a clip of a video with START and END positions.
    /// All streams and timestamps are copied by default.
    /// You can reencode with --encode flag.
    Clip {
        /// Start time of the clip (e.g. "00:01:30.000" for 1 minute 30 seconds.
        #[arg(help = "Start time of the clip.")]
        start: String,

        /// End time of the clip (e.g. "00:02:00.000")
        #[arg(help = "End time of the clip.")]
        end: String,

        /// Reencode the clip.
        #[arg(short, long, help = "Reencode the clip using h264 encoder.")]
        encode: bool,
    },

    /// Merges two or more videos.
    Merge,

    /// Creates a video from images.
    /// By default, each image will be displayed for 5 seconds.
    Video {
        /// This duration can be changed with FRAMERATE option.
        /// Must be between 1 and 10.
        #[arg(short, long, value_name = "FRAMERATE", default_value_t = 5, value_parser = clap::value_parser!(i16).range(1..=10))]
        framerate: i16,
    },

    /// Extracts the audio stream from a video.
    Audio,

    /// Encodes a video with three differents encoders: h264, h265 and av1.
    Encode {
        /// Choose the encoder to use.
        #[arg(short, long, value_enum, help = "Select the video encoder (h264, h265, av1).", default_value_t = Encoders::H264)]
        encoders: Encoders,

        /// Constant Rate Factor (CRF) for H.264 encoding.
        /// A lower value means higher quality. Valid range: 0-51.
        /// Defaults to 23. This option is only applicable for the H264 encoder.
        #[arg(short, long, value_name = "CRF", default_value_t = 23, value_parser = clap::value_parser!(i16).range(1..=51))]
        /// This option's used for x264 encoder. Defaults to 23.
        /// This value can be changed with CRF option, BETWEEN 0 and 51.
        crf: i16,
    },

    /// Encodes a video with options specifically for YouTube.
    Youtube,

    /// Upscale video for higher peak quality on platforms like YouTube.
    /// Uses FFmpeg's recommended settings for upscalling.
    /// https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality
    Upscale,
}

// TODO: Why do we need Clone here?
#[derive(ValueEnum, Debug, Clone)]
enum Encoders {
    H264,
    H265,
    AV1,
}

// TODO: We might not need -pix_fmt anymore?
// ffmpeg args
static CLIP: &str = "-y -ss START -i INPUTS -to END -c copy -copyts OUTPUT";
static CLIP_REENCODE: &str = "-y -i INPUTS -ss START -to END -c:v libx264 -c:a aac OUTPUT";

static MERGE: &str = "-y -f concat -safe 0 -i INPUTS -c:v libx264 -pix_fmt yuv420p OUTPUT";

static VIDEO: &str = "-y -f concat -safe 0 -i INPUTS -c:v libx264 -r 30 -pix_fmt yuv420p OUTPUT";

static AUDIO: &str = "-y -i INPUTS -vn -c:a mp3 -b:a 192k OUTPUT";

// TODO: these encoders are so cooked.
// H264 encoder
// we're already recording using acc encode and bitrate 160k so we just copy it
static ENCODE_H264: &str =
    "-y -i INPUTS -c:v libx264 -crf CRF -preset ultrafast -c:a copy -pix_fmt yuv420p OUTPUT";

// av1_nvenc encoder, defaults to cq 20
// same as H264 for audio
static ENCODE_AV1: &str =
    "-y -i INPUTS -c:v av1_nvenc -preset ultrafast -cq 20 -c:a copy -pix_fmt yuv420p OUTPUT";

// H265 encoder, defaults to cq 20
// same as H264 for audio
static ENCODE_H265: &str =
    "-y -i INPUTS -c:v hevc_nvenc -preset ultrafast -cq 20 -c:a copy -pix_fmt yuv420p OUTPUT";

static YOUTUBE: &str =
    "-y -i INPUTS -c:v libx264 -crf 18 -preset ultrafast -c:a aac -b:a 384k -pix_fmt yuv420p OUTPUT";

static UPSCALE: &str =
    "-y -i INPUTS -vf scale=iw*2:ih*2:flags=neighbor -c:v libx264 -crf 18 -preset ultrafast OUTPUT";

fn run_ffmpeg(args: Vec<&str>) -> Result<&'static str> {
    use std::process::{Command, Stdio};

    let msg = &args;
    let msg = msg.join(" ");
    print!("Calling ffmpeg with args:\n( {msg} )\n\n");

    let run_dummy = true;
    if run_dummy {
        println!("Calling ffmpeg with no args!");
        let ffmpeg = Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = ffmpeg.wait_with_output()?;
        if !output.status.success() {
            bail!(
                "-- Failed to execute ffmpeg. Error code: {:?} -- ",
                output.status.code()
            );
        }
    } else {
        let ffmpeg = Command::new("ffmpeg")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;

        let output = ffmpeg.wait_with_output()?;
        if !output.status.success() {
            bail!(
                "-- Failed to execute ffmpeg. Error code: {:?} -- ",
                output.status.code()
            );
        }
    }
    Ok("\nSuccessfully ran ffmpeg!")
}

// TODO: handle file name with spaces
fn full_path(file: Option<String>) -> Result<String> {
    let dir = current_dir()?;
    if let Some(file) = file {
        Ok(dir
            .join(file)
            .to_str()
            .context("Failed to convert to &str.")?
            .to_string())
    } else {
        Ok(dir
            .to_str()
            .context("Failed to convert to &str.")?
            .to_string())
    }
}

#[derive(Debug)]
struct PepegaContext {
    input: Input,
    input_type: InputType,
    input_size: usize,
    output: String,
}

impl PepegaContext {
    fn new(input: Vec<String>, output: String) -> Result<Self> {
        let input_size = input.len();

        // handle the inputs
        let (input_type, input) = if input_size == 1 {
            // single input can contain "." or directory
            let single_input = input[0].clone();
            if single_input.ends_with(".") {
                let full_path = full_path(None)?;
                (InputType::Directory, Input::Single(full_path))
            } else if PathBuf::from(single_input.clone()).is_dir() {
                let full_path = full_path(Some(single_input))?;
                (InputType::Directory, Input::Single(full_path))
            } else {
                let full_path = full_path(Some(single_input))?;
                (InputType::File, Input::Single(full_path))
            }
        } else {
            // multiple inputs
            let mut inputs = Vec::new();
            for i in &input {
                match full_path(Some(i.to_string())) {
                    Ok(value) => inputs.push(value),
                    Err(err) => bail!("Error getting the full path: {}", err),
                }
            }
            (InputType::File, Input::Multiple(inputs))
        };

        // currently we only suport single output.
        let output = full_path(Some(output))?;

        Ok(Self {
            input,
            input_type,
            input_size,
            output,
        })
    }
}

#[derive(Debug)]
enum Input {
    Single(String),
    Multiple(Vec<String>),
}

impl Input {
    fn single(self) -> String {
        match self {
            Input::Single(value) => value,
            _ => panic!("Tried to take Multiple from Single."),
        }
    }

    fn multiple(self) -> Vec<String> {
        match self {
            Input::Multiple(values) => values,
            _ => panic!("Tried to take Single from Multiple."),
        }
    }
}

#[derive(Debug, PartialEq)]
enum InputType {
    Directory,
    File,
}

fn main() -> Result<()> {
    let args = Pepega::parse();

    let ctx = PepegaContext::new(args.inputs, args.output)?;

    match args.command {
        Command::Clip { start, end, encode } => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();

            let args = if encode {
                CLIP_REENCODE
                    .replace("START", &start)
                    .replace("INPUTS", &actual_inputs)
                    .replace("END", &end)
                    .replace("OUTPUT", &ctx.output)
            } else {
                CLIP.replace("START", &start)
                    .replace("INPUTS", &actual_inputs)
                    .replace("END", &end)
                    .replace("OUTPUT", &ctx.output)
            };

            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Creating a clip of {actual_inputs} [{start}...{end}] -> {0}",
                ctx.output
            );
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Merge => {
            // TODO: Research concatenating streams with filters.
            // we expect more TWO or MORE inputs.

            // Create temporary dir and file
            let tmp_dir = tempdir_in(".").expect("Failed to create a folder");
            let tmp_list = tmp_dir.path().join("tmp_list.txt");
            let mut tmp_list_file =
                File::create(&tmp_list).expect("Failed to create an tmp list file");

            let mut total_videos = 0;

            // Check the input type and write the entries to the tmp file.
            match ctx.input_type {
                // It's multiple files, we iterate over them and write their
                // absolute path to a file.
                InputType::File => {
                    if ctx.input_size < 2 {
                        bail!("Not enough inputs");
                    }

                    let actual_inputs = ctx.input.multiple();
                    for entry in actual_inputs {
                        writeln!(tmp_list_file, "file '{}'", entry)
                            .expect("Failed to write to tmp_list_file");
                        total_videos += 1;
                    }
                }
                // It's a single directory, we iterator over that and read
                // each entry checking if they end with mp4 or mkv and write their
                // absolute path to a file
                InputType::Directory => {
                    // We use PathBuf to iterate the directory.
                    let actual_inputs = ctx.input.single();
                    let input_path = PathBuf::from(actual_inputs);
                    for entry in input_path
                        .read_dir()
                        .expect("Failed to read entries in directory.")
                        .flatten()
                    {
                        let entry_path = entry.path();
                        let entry_path_str =
                            entry_path.to_str().context("Failed to convert to &str.")?;
                        if entry_path_str.ends_with("mkv") || entry_path_str.ends_with("mp4") {
                            writeln!(tmp_list_file, "file '{}'", entry_path_str)
                                .expect("Failed to write to tmp_img_list_file");
                            total_videos += 1;
                        }
                    }
                }
            }

            let inputs = tmp_list
                .to_str()
                .context("Failed to convert to &str.")?
                .to_string();
            let args = MERGE
                .replace("INPUTS", &inputs)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Merging {total_videos} videos in {inputs}");
            println!("{}", run_ffmpeg(args)?);
        }

        Command::Video { framerate } => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::Directory {
                bail!("input is not a directory.");
            }

            // Create temporary dir and file
            let tmp_dir = tempdir_in(".").expect("Failed to create a folder");
            let tmp_list = tmp_dir.path().join("tmp_list.txt");
            let mut tmp_list_file =
                File::create(&tmp_list).expect("Failed to create a tmp image list file");

            let mut total_images = 0;

            let actual_inputs = ctx.input.single();
            let input_path = PathBuf::from(actual_inputs);
            for entry in input_path
                .read_dir()
                .expect("Failed to read entries in directory")
                .flatten()
            {
                let entry_path = entry.path();
                let entry_path_str = entry_path.to_str().context("Failed to convert to &str.")?;
                if entry_path_str.ends_with("png") || entry_path_str.ends_with("jpg") {
                    writeln!(tmp_list_file, "file '{}'", entry_path_str)
                        .expect("Failed to write to tmp_list_file");
                    writeln!(tmp_list_file, "duration {}", framerate)
                        .expect("Failed to write to tmp_list_file");
                    total_images += 1;
                }
            }

            let inputs = tmp_list
                .to_str()
                .context("Failed to convert to &str.")?
                .to_string();
            let args = VIDEO
                .replace("INPUTS", &inputs)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Creating a video from {total_images} images in {inputs} with framerate 1/{framerate}");
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Audio => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();
            let args = AUDIO
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Extracting audio of {actual_inputs} -> {0}", ctx.output);
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Encode { encoders, crf } => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();

            let args = match encoders {
                Encoders::H264 => {
                    println!("Encoding with libx264 crf={crf}");
                    ENCODE_H264
                        .replace("INPUTS", &actual_inputs)
                        .replace("CRF", &crf.to_string())
                        .replace("OUTPUT", &ctx.output)
                }
                Encoders::H265 => {
                    println!("Encoding with libx265 defaults to cq=20");
                    ENCODE_H265
                        .replace("INPUTS", &actual_inputs)
                        .replace("OUTPUT", &ctx.output)
                }
                Encoders::AV1 => {
                    println!("Encoding with av1 defaults to cq=20");
                    ENCODE_AV1
                        .replace("INPUTS", &actual_inputs)
                        .replace("OUTPUT", &ctx.output)
                }
            };

            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Encoding {actual_inputs} -> {0}", ctx.output);
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Youtube => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();
            let args = YOUTUBE
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Encoding video for youtube {actual_inputs} -> {0}",
                ctx.output
            );
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Upscale => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();
            let args = UPSCALE
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Upscaling video {actual_inputs} -> {0}", ctx.output);
            println!("{}", run_ffmpeg(args)?);
        }
    }

    Ok(())
}
