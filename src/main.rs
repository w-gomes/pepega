/* TESTED:
 *
 * clip: done
 * merge: done
 * video
        [concat @ 000001778f264140] Impossible to open 'C:\dev\pepega\.\test\images\.tmpHrOuU5\.\test\images\ffxiv_01032025_020845_930.png'
        [in#0 @ 000001778f263ec0] Error opening input: No such file or directory
        Error opening input file C:\dev\pepega\.\test\images\.tmpHrOuU5\tmp_list.txt.
        Error opening input files: No such file or directory
        Error: -- Failed to execute ffmpeg. Error code: Some(-2) --
        error: process didn't exit successfully: `target\debug\pepega.exe -i .\test\images\ video` (exit code: 1)

 * audio
 * encode: done
 * youtube
 * upscale
 * flip
*/
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use tempfile::TempDir;

mod tasks;
use crate::tasks::{Audio, Clip, Encode, Flip, Merge, Upscale, Video, Youtube};

#[derive(Parser, Debug)]
#[command(name = "pepega", about = "video and audio tool.", version)]
struct Cli {
    /// A list of tasks to choose to operate on the input.
    #[command(subcommand)]
    task: Tasks,

    /// Input files.
    #[arg(short, long, required = true, help = "One or more input files.")]
    inputs: Vec<PathBuf>,

    /// Output file.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Tasks {
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

fn run_ffmpeg<Iter>(args: Iter) -> Result<&'static str>
where
    Iter: std::iter::IntoIterator<Item = String> + std::fmt::Debug,
{
    let run_dummy = false;
    if run_dummy {
        println!("\nRunning dummy!");
        println!("- Args:\n{:?}\n", args);
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

#[derive(PartialEq)]
enum InputType {
    Directory,
    File,
}

struct Inputs {
    inputs: Vec<String>,
    inputs_type: InputType,
}

struct Output {
    output: Option<String>,
}

struct Ctx {
    inputs: Inputs,
    output: Output,
}

impl Ctx {
    fn new(inputs: Vec<PathBuf>, output: Option<PathBuf>) -> Result<Self> {
        // Determine InputType.
        let inputs_type = if inputs[0].is_dir() {
            InputType::Directory
        } else {
            InputType::File
        };

        // Convert the inputs to string.
        let inputs = inputs
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<String>>();

        let output = if let Some(out) = output {
            if out.is_dir() {
                return Err(anyhow!("The output must be a file."));
            }
            Some(out.display().to_string())
        } else {
            None
        };

        let inputs = Inputs {
            inputs: inputs,
            inputs_type: inputs_type,
        };

        let output = Output { output: output };

        Ok(Self { inputs, output })
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let ctx = Ctx::new(cli.inputs, cli.output)?;

    match cli.task {
        Tasks::Clip { start, end, encode } => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "clip_output.mp4".to_string()
            };

            let input = &ctx.inputs.inputs[0];

            let clip = Clip::new()
                .start(&start)
                .input(&input)
                .end(&end)
                .encode(encode)
                .output(&output);

            println!("Creating a clip.");
            println!("{}", run_ffmpeg(clip.args.into_iter())?);
        }
        Tasks::Merge => {
            let mut filters = String::new();
            let inputs = &ctx.inputs.inputs;

            // Check the input type and write the entries to the tmp file.
            let new_inputs = match ctx.inputs.inputs_type {
                // It's multiple files, apply the filters.
                InputType::File => {
                    if inputs.len() < 2 {
                        bail!("Not enough inputs");
                    }

                    let total_inputs = inputs.len();
                    for i in 0..total_inputs {
                        filters.push_str(format!("[{0}:v:0][{0}:a:0]", i).as_str());
                    }
                    filters.push_str(format!("concat=n={}:v=1:a=1[v][a]", total_inputs).as_str());

                    ctx.inputs.inputs
                }
                // It's a single directory, we iterator over that and read
                // each entry checking if they end with mp4 or mkv and write their
                // absolute path to a file
                InputType::Directory => {
                    // We use PathBuf to iterate the directory.
                    let input_path = PathBuf::from(inputs[0].clone());
                    let mut new_inputs = Vec::new();

                    let mut idx = 0;
                    for entry in input_path
                        .read_dir()
                        .expect("Failed to read entries in directory.")
                        .flatten()
                    {
                        let entry_path = entry.path();
                        let entry_path_str =
                            entry_path.to_str().context("Failed to convert to &str.")?;
                        if entry_path_str.ends_with("mkv") || entry_path_str.ends_with("mp4") {
                            new_inputs.push(entry_path_str.to_string());
                        }

                        filters.push_str(format!("[{0}:v:0][{0}:a:0]", idx).as_str());
                        idx += 1;
                    }
                    filters.push_str(format!("concat=n={}:v=1:a=1[v][a]", idx).as_str());

                    new_inputs
                }
            };

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "videos_merged_output.mp4".to_string()
            };

            let merge = Merge::new()
                .inputs(&new_inputs)
                .filters(&filters)
                .output(&output);

            println!("Merging {} videos.", new_inputs.len());
            println!("{}", run_ffmpeg(merge.args.into_iter())?);
        }
        Tasks::Video { framerate } => {
            if ctx.inputs.inputs_type != InputType::Directory {
                bail!("input is not a directory.");
            }

            // Creates temporary dir and file
            let source_dir = PathBuf::from(&ctx.inputs.inputs[0]);
            let tmp_dir =
                TempDir::new_in(&source_dir).expect("Failed to create a temporary folder");
            let tmp_list = tmp_dir.path().join("tmp_list.txt");
            let mut tmp_list_file =
                File::create(&tmp_list).expect("Failed to create a tmp image list file");

            let mut total_images = 0;
            for entry in source_dir
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

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "video_from_images_output.mp4".to_string()
            };

            let input = tmp_list
                .to_str()
                .context("Failed to convert to &str.")?
                .to_string();

            let video = Video::new().input(&input).output(&output);
            println!("Creating a video from {total_images} images.");
            println!("{}", run_ffmpeg(video.args.into_iter())?);
        }
        Tasks::Audio => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "encode_output.mp4".to_string()
            };

            let input = &ctx.inputs.inputs[0];

            let audio = Audio::new().input(&input).output(&output);
            println!("Extracting a audio.");
            println!("{}", run_ffmpeg(audio.args.into_iter())?);
        }
        Tasks::Encode { encoders, crf } => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "encode_output.mp4".to_string()
            };

            let input = &ctx.inputs.inputs[0];

            let encode = Encode::new()
                .input(&input)
                .encode(encoders, crf)
                .output(&output);

            println!("Encoding a video.");
            println!("{}", run_ffmpeg(encode.args.into_iter())?);
        }
        Tasks::Youtube => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "youtube_output.mp4".to_string()
            };

            let input = &ctx.inputs.inputs[0];

            let youtube = Youtube::new().input(&input).output(&output);
            println!("Encoding a video for youtube.");
            println!("{}", run_ffmpeg(youtube.args.into_iter())?);
        }
        Tasks::Upscale => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("input is not a file.");
            }

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "upscale_output.mp4".to_string()
            };

            let input = &ctx.inputs.inputs[0];

            let upscale = Upscale::new().input(&input).output(&output);

            println!("Upscaling a video.");
            println!("{}", run_ffmpeg(upscale.args.into_iter())?);
        }
        Tasks::Flip => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("input is not a file.");
            }

            let output = if let Some(out) = ctx.output.output {
                out
            } else {
                "flip_output.mp4".to_string()
            };

            let input = &ctx.inputs.inputs[0];

            let flip = Flip::new().input(&input).output(&output);

            println!("Flipping a video.");
            println!("{}", run_ffmpeg(flip.args.into_iter())?);
        }
    }

    Ok(())
}
