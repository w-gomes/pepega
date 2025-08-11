use std::{env::current_dir, fs::File, io::Write, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use tempfile::tempdir_in;

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
    inputs: Vec<PathBuf>,

    /// Output file.
    #[arg(short, long)]
    output: PathBuf,

    /// Options for the program.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Creates a clip of a video with START and END times.
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

    /// Encodes a video with three differents encoders: H264, H265 and AV1.
    Encode {
        /// Choose the encoder.
        #[arg(short, long, value_enum, help = "Select the video encoder (H264, H265, AV1).", default_value_t = Encoders::H264)]
        encoders: Encoders,

        /// Constant Rate Factor (CRF) for the H.264 encoder.
        /// A lower value means higher quality. Valid range: 0-51.
        /// Defaults to 23. This option is only applicable for the H264 encoder.
        #[arg(short, long, value_name = "CRF", default_value_t = 23, value_parser = clap::value_parser!(i16).range(1..=51))]
        /// This option's used for x264 encoder. Defaults to 23.
        /// This value can be changed with CRF option, BETWEEN 0 and 51.
        crf: i16,
    },

    /// Encodes a video with options specifically for YouTube.
    Youtube,

    /// Upscales a video for higher peak quality on platforms like YouTube.
    /// Uses FFmpeg's recommended settings for upscalling.
    /// https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality
    Upscale,

    /// Flips a video clockwise.
    Flip,
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
static CLIP: &str = "-ss START -i INPUTS -to END -c copy -copyts OUTPUT";
static CLIP_REENCODE: &str = "-i INPUTS -ss START -to END -c:v libx264 -c:a aac OUTPUT";

static MERGE: &str = "-f concat -safe 0 -i INPUTS -c:v libx264 -pix_fmt yuv420p OUTPUT";

static VIDEO: &str = "-f concat -safe 0 -i INPUTS -c:v libx264 -r 30 -pix_fmt yuv420p OUTPUT";

static AUDIO: &str = "-i INPUTS -vn -c:a mp3 -b:a 192k OUTPUT";

// TODO: these encoders are so cooked.
// H264 encoder
// we're already recording using acc encode and bitrate 160k so we just copy it
static ENCODE_H264: &str =
    "-i INPUTS -c:v libx264 -crf CRF -preset ultrafast -c:a copy -pix_fmt yuv420p OUTPUT";

// av1_nvenc encoder, defaults to cq 20
// same as H264 for audio
static ENCODE_AV1: &str =
    "-i INPUTS -c:v av1_nvenc -cq 20 -preset p1 -c:a copy -pix_fmt yuv420p OUTPUT";

// H265 encoder, defaults to cq 20
// same as H264 for audio
static ENCODE_H265: &str =
    "-i INPUTS -c:v hevc_nvenc -cq 20 -preset p1 -c:a copy -pix_fmt yuv420p OUTPUT";

static YOUTUBE: &str =
    "-i INPUTS -c:v libx264 -crf 18 -preset ultrafast -c:a aac -b:a 384k -pix_fmt yuv420p OUTPUT";

static UPSCALE: &str =
    "-i INPUTS -vf scale=iw*2:ih*2:flags=neighbor -c:v libx264 -crf 18 -preset ultrafast OUTPUT";

static FLIP: &str = "-display_rotation:v:0 -90.0 -i INPUTS -c copy OUTPUT";

fn run_ffmpeg(args: Vec<&str>) -> Result<&'static str> {
    use std::process::{Command, Stdio};

    let msg = &args;
    let msg = msg.join(" ");
    print!("Calling ffmpeg with args:\n( {msg} )\n\n");

    let run_dummy = false;
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

fn full_path(path: PathBuf) -> Result<PathBuf> {
    let mut absolute_path = current_dir().unwrap();
    absolute_path.push(path);
    Ok(absolute_path)
}

#[derive(Debug)]
struct PepegaContext {
    input: Input,
    input_type: InputType,
    input_size: usize,
    output: String,
}

impl PepegaContext {
    fn new(mut input: Vec<PathBuf>, output: PathBuf) -> Result<Self> {
        let input_size = input.len();

        let (input_type, input) = if input_size == 1 {
            let single_input = input.pop().unwrap();
            let path = if single_input.is_absolute() {
                single_input
            } else {
                full_path(single_input)?
            };

            if path.is_dir() {
                (
                    InputType::Directory,
                    Input::Single(path.display().to_string()),
                )
            } else {
                println!("path: {}", path.display());
                (InputType::File, Input::Single(path.display().to_string()))
            }
        } else {
            let inputs = input
                .into_iter()
                .map(|i| {
                    let absolute_path = full_path(i).unwrap();
                    absolute_path.display().to_string()
                })
                .collect::<Vec<String>>();
            (InputType::File, Input::Multiple(inputs))
        };

        // TODO: Make sure this is a file and not a dir.
        let output = PathBuf::from(output);
        let output = full_path(output)?.display().to_string();

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
    let program = Pepega::parse();

    let ctx = PepegaContext::new(program.inputs, program.output)?;

    match program.command {
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
        Command::Flip => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }
            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();
            let args = FLIP
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Flipping the video clockwise {actual_inputs} -> {0}",
                ctx.output
            );
            println!("{}", run_ffmpeg(args)?);
        }
    }

    Ok(())
}
