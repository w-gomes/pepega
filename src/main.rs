use std::{fs::File, io::Write, path::PathBuf};

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
        help = "One or more input files. It doesn't support relative path."
    )]
    inputs: Vec<PathBuf>,

    /// Output file.
    #[arg(short, long)]
    output: Option<PathBuf>,

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

static MERGE: &str = "INPUTS -filter_complex FILTERS -map [v] -map [a] -c:v libx264 -pix_fmt yuv420p -c:a aac OUTPUT";

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

    let run_dummy = false;
    if run_dummy {
        println!("\nRunning dummy!");
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
        let msg = &args;
        let msg = msg.join(" ");
        print!("\nCalling ffmpeg with args:\n( {msg} )\n\n");
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

#[derive(Debug, PartialEq)]
enum InputType {
    Directory,
    File,
}

#[derive(Debug)]
struct PepegaContext {
    inputs: Vec<String>,
    inputs_type: InputType,
    output: String,
    output_has_extension: bool,
    is_relative: bool,
}

impl PepegaContext {
    fn new(inputs: Vec<PathBuf>, output: Option<PathBuf>) -> Result<Self> {
        // Determine InputType.
        // TODO: We assume all remaining inputs are the same.
        let inputs_type = if inputs[0].is_dir() { InputType::Directory } else { InputType::File };
        let output_has_extension;

        let is_input_rel = inputs[0].is_relative();
        let is_output_rel;

        // Check if output, else make one from inputs' parent path.
        let output = match output {
            Some(out) => {
                is_output_rel = out.is_relative();
                output_has_extension = true;
                out.display().to_string()
            }
            None => {
                // TODO: unwrap()
                output_has_extension = false;
                is_output_rel = is_input_rel;
                let out = inputs[0].parent().unwrap().display().to_string();
                out + "/output_tmp_name"
            },
        };

        // Check if both output and inputs are relative.
        // TODO: If more inputs, check if they are all absolute path.
        //       Here, we are assumming all remaning inputs are the same as
        //       the first one.
        let is_relative = is_input_rel && is_output_rel;

        // Convert the inputs to String from PathBuf.
        let inputs = inputs.iter().map(|path| {
            path.display().to_string()
        }).collect::<Vec<String>>();

        Ok(Self {
            inputs,
            inputs_type,
            output,
            output_has_extension,
            is_relative,
        })
    }

    fn output(&self, ext: &str) -> Option<String> {
        if !self.output_has_extension {
            Some(self.output.clone() + "." + ext)
        } else {
            None
        }
    }
}

fn main() -> Result<()> {
    let program = Pepega::parse();

    let ctx = PepegaContext::new(program.inputs, program.output)?;

    if ctx.is_relative {
        bail!("No support for relative path.");
    }

    match program.command {
        Command::Clip { start, end, encode } => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let actual_inputs = &ctx.inputs[0];

            let args = if encode {
                CLIP_REENCODE
                    .replace("START", &start)
                    .replace("INPUTS", &actual_inputs)
                    .replace("END", &end)
                    .replace("OUTPUT", &actual_output)
            } else {
                CLIP.replace("START", &start)
                    .replace("INPUTS", &actual_inputs)
                    .replace("END", &end)
                    .replace("OUTPUT", &actual_output)
            };

            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Creating a clip of {actual_inputs} [{start}...{end}] -> {actual_output}"
            );
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Merge => {
            let mut filters = String::new();
            let mut inputs = ctx.inputs.clone();
            let total_videos;

            // Check the input type and write the entries to the tmp file.
            match ctx.inputs_type {
                // It's multiple files, we iterate over them and write their
                // absolute path to a file.
                InputType::File => {
                    if ctx.inputs.len() < 2 {
                        bail!("Not enough inputs");
                    }

                    let total_inputs = inputs.len();

                    for i in 0..total_inputs {
                        filters.push_str(format!("[{0}:v:0][{0}:a:0]", i).as_str());
                    }
                    filters.push_str(format!("concat=n={}:v=1:a=1[v][a]", total_inputs).as_str());

                    total_videos = total_inputs;
                }
                // It's a single directory, we iterator over that and read
                // each entry checking if they end with mp4 or mkv and write their
                // absolute path to a file
                InputType::Directory => {
                    // We use PathBuf to iterate the directory.
                    let input_path = PathBuf::from(inputs[0].clone());

                    for entry in input_path
                        .read_dir()
                        .expect("Failed to read entries in directory.")
                        .flatten()
                    {
                        let entry_path = entry.path();
                        let entry_path_str =
                            entry_path.to_str().context("Failed to convert to &str.")?;
                        if entry_path_str.ends_with("mkv") || entry_path_str.ends_with("mp4") {
                            inputs.push(entry_path_str.to_string());
                        }
                    }

                    let total_inputs = inputs.len();

                    for i in 0..total_inputs {
                        filters.push_str(format!("[{0}:v:0][{0}:a:0]", i).as_str());
                    }
                    filters.push_str(format!("concat=n={}:v=1:a=1[v][a]", total_inputs).as_str());
                    total_videos = total_inputs;
                }
            }

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let mut inputs = inputs.join(" -i ");

            // TODO: so bad
            inputs.insert(0, ' ');
            inputs.insert(0, 'i');
            inputs.insert(0, '-');

            let args = MERGE
                .replace("INPUTS", &inputs)
                .replace("FILTERS", &filters)
                .replace("OUTPUT", &actual_output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Merging {total_videos} videos.");
            println!("{}", run_ffmpeg(args)?);
        }

        Command::Video { framerate } => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::Directory {
                bail!("input is not a directory.");
            }

            // Create temporary dir and file
            let tmp_dir = tempdir_in(".").expect("Failed to create a folder");
            let tmp_list = tmp_dir.path().join("tmp_list.txt");
            let mut tmp_list_file =
                File::create(&tmp_list).expect("Failed to create a tmp image list file");

            let mut total_images = 0;

            let actual_inputs = &ctx.inputs[0];

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

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let inputs = tmp_list
                .to_str()
                .context("Failed to convert to &str.")?
                .to_string();
            let args = VIDEO
                .replace("INPUTS", &inputs)
                .replace("OUTPUT", &actual_output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Creating a video from {total_images} images in {inputs} with framerate 1/{framerate}");
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Audio => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_output = if let Some(out) = ctx.output("mp3") { out } else {
                ctx.output
            };

            let actual_inputs = &ctx.inputs[0];
            let args = AUDIO
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &actual_output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Extracting audio of {actual_inputs} -> {actual_output}");
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Encode { encoders, crf } => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let actual_inputs = &ctx.inputs[0];

            let args = match encoders {
                Encoders::H264 => {
                    println!("Encoding with libx264 crf={crf}");
                    ENCODE_H264
                        .replace("INPUTS", &actual_inputs)
                        .replace("CRF", &crf.to_string())
                        .replace("OUTPUT", &actual_output)
                }
                Encoders::H265 => {
                    println!("Encoding with libx265 defaults to cq=20");
                    ENCODE_H265
                        .replace("INPUTS", &actual_inputs)
                        .replace("OUTPUT", &actual_output)
                }
                Encoders::AV1 => {
                    println!("Encoding with av1 defaults to cq=20");
                    ENCODE_AV1
                        .replace("INPUTS", &actual_inputs)
                        .replace("OUTPUT", &actual_output)
                }
            };

            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Encoding {actual_inputs} -> {actual_output}");
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Youtube => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let actual_inputs = &ctx.inputs[0];
            let args = YOUTUBE
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &actual_output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Encoding video for youtube {actual_inputs} -> {actual_output}"
            );
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Upscale => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let actual_inputs = &ctx.inputs[0];
            let args = UPSCALE
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &actual_output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!("Upscaling video {actual_inputs} -> {actual_output}");
            println!("{}", run_ffmpeg(args)?);
        }
        Command::Flip => {
            if ctx.inputs.len() > 1 {
                println!("{}", "Warning: Inputs length is greater than 1. Discarding...");
            }

            if ctx.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let actual_output = if let Some(out) = ctx.output("mp4") { out } else {
                ctx.output
            };

            let actual_inputs = &ctx.inputs[0];
            let args = FLIP
                .replace("INPUTS", &actual_inputs)
                .replace("OUTPUT", &actual_output);
            let args = args.split_whitespace().collect::<Vec<&str>>();
            println!(
                "Flipping the video clockwise {actual_inputs} -> {actual_output}"
            );
            println!("{}", run_ffmpeg(args)?);
        }
    }

    Ok(())
}
