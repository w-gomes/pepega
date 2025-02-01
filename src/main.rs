use std::{
    env::current_dir,
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
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
#[command(about = "Smol video and audio tool that uses ffmpeg.")]
#[command(version, long_about = None)]
struct Pepega {
    /// Inputs files.
    #[arg(short, required = true)]
    inputs: Vec<String>,

    /// Output file.
    #[arg(short, required = true)]
    output: String,

    /// Options for the program.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Creates a clip of a video with START and END positions.
    /// All streams and timestamp are copied.
    /// You might want to encode after this with encode command.
    Clip { start: String, end: String },

    /// Merges two or more videos.
    Merge,

    /// Creates a video from images with default of 5 seconds each.
    /// This time can be changed with FRAMERATE option, BETWEEN 1 and 10.
    Video {
        #[arg(short, long, value_name = "FRAMERATE")]
        framerate: Option<i16>,
    },

    /// Extracts the audio stream from a video.
    Audio,

    /// Encodes a video with default value of 23 for CRF.
    /// This value can be changed with CRF option, BETWEEN 0 and 51.
    Encode {
        #[arg(short, long, value_name = "CRF")]
        crf: Option<i16>,
    },

    /// Encodes a video with options specifically for youtube.
    Youtube,
}

enum Encoders {
    H264 { crf: Option<i16> },
    H265,
    AV1,
}

// ffmpeg args
static CLIP: &str = "-y -ss START -i INPUTS -to END -c copy -copyts OUTPUT";

static MERGE: &str = "-y -f concat -safe 0 -i INPUTS -c:v libx264 -pix_fmt yuv420p OUTPUT";

static VIDEO: &str = "-y -f concat -safe 0 -i INPUTS -c:v libx264 -r 30 -pix_fmt yuv420p OUTPUT";

static AUDIO: &str = "-y -i INPUTS -vn -c:a mp3 OUTPUT";

// H264 encoder
static ENCODE_H264: &str =
    "-y -i INPUTS -c:v libx264 -crf CRF -c:a aac -b:a 192k -pix_fmt yuv420p OUTPUT";

// av1_nvenc encoder, default to cq 20
static ENCODE_AV1: &str =
    "-y -i INPUTS -c:v av1_nvenc -preset fast -cq 20 -c:a aac -b:a 192k -pix_fmt yuv420p OUTPUT";

// H265 encoder, defaults to cq 20
static ENCODE_H265: &str =
    "-y -i INPUTS -c:v hevc_nvenc -preset fast -cq 20 -c:a aac -b:a 192k -pix_fmt yuv420p OUTPUT";

static YOUTUBE: &str = "-y -i INPUTS -c:v libx264 -crf 18 -preset ultrafast -c:a aac -b:a 384k -pix_fmt yuv420p OUTPUT";

fn run_ffmpeg(args: Vec<&str>) -> Result<&'static str> {
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
        Commands::Clip { start, end } => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_inputs = ctx.input.single();
            let args = CLIP
                .replace("START", &start)
                .replace("INPUTS", &actual_inputs)
                .replace("END", &end)
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Creating a clip of {actual_inputs} [{start}...{end}] -> {0}",
                ctx.output
            );
            println!("{}", run_ffmpeg(args)?);
        }
        Commands::Merge => {
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

        Commands::Video { framerate } => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::Directory {
                bail!("input is not a directory.");
            }

            let framerate = match framerate {
                Some(framerate) => {
                    if !(1..=10).contains(&framerate) {
                        println!(
                            "Framerate ({}) value out of range [1..10]. Defaulting to 5.",
                            framerate
                        );
                        5
                    } else {
                        framerate
                    }
                }
                None => 5,
            };

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
        Commands::Audio => {
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
        Commands::Encode { crf } => {
            if ctx.input_size > 1 {
                bail!("Too many inputs");
            }

            if ctx.input_type != InputType::File {
                bail!("Input is not a file.");
            }

            let crf = match crf {
                Some(crf) => {
                    if !(0..=51).contains(&crf) {
                        println!(
                            "CRF ({}) value out of range [0..51]. Defaulting to 23.",
                            crf
                        );
                        23
                    } else {
                        crf
                    }
                }
                None => 23,
            };

            let actual_inputs = ctx.input.single();
            let args = ENCODE_H264
                .replace("INPUTS", &actual_inputs)
                .replace("CRF", &crf.to_string())
                .replace("OUTPUT", &ctx.output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Encoding {actual_inputs} with libx264 crf={crf} -> {0}",
                ctx.output
            );
            println!("{}", run_ffmpeg(args)?);
        }
        Commands::Youtube => {
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
    }

    Ok(())
}
