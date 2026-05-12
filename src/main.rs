use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand, ValueEnum};

mod tasks;
mod utils;

use crate::tasks::{Audio, Clip, Encode, Flip, Gif, Merge, Remux, Upscale, Video, Youtube};
use crate::utils::{filters_for_merge, tmp_list_for_video};

#[derive(Parser, Debug)]
#[command(name = "pepega", about = "video and audio tool.", version)]
struct Cli {
    /// A list of tasks to choose from to operate on the inputs.
    #[command(subcommand)]
    task: Tasks,

    /// Input files.
    #[arg(short, long, required = true, help = "One or more input files.")]
    inputs: Vec<PathBuf>,

    /// Output file.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Only print the args what sent to ffmpeg.
    #[arg(long)]
    test: bool,
}

#[derive(Subcommand, Debug)]
enum Tasks {
    /// Clip a video with START and END timestamps. All streams are copied without re-encoding.
    /// Run the `encode` subcommand on the output file afterward to re-encode.
    Clip {
        #[arg(help = "Start of the clip.")]
        start: String,

        #[arg(help = "End of the clip.")]
        end: String,
    },

    /// Merge two or more videos into one.
    Merge,

    /// Create a video slideshow from images. Each image is displayed for 5 seconds by default.
    Video {
        /// Set a custom framerate, between 1 and 10.
        #[arg(short, long, value_name = "FRAMERATE", default_value_t = 5, value_parser = clap::value_parser!(i16).range(1..=10))]
        framerate: i16,
    },

    /// Extract the audio stream from a video.
    Audio,

    /// Encode a video using H264, H265, or AV1 encoders.
    Encode {
        /// Set the encode.
        #[arg(short, long, value_enum, help = "Set the video encoder (H264, H265, AV1).", default_value_t = Encoders::H264)]
        encoders: Encoders,

        /// Set the Constant Rate Factor (CRF) for the H.264 encoder ONLY.
        /// A lower value means higher quality, between 0 and 51.
        #[arg(short, long, value_name = "CRF", default_value_t = 23, value_parser = clap::value_parser!(i16).range(1..=51))]
        crf: i16,
    },

    /// Encode a video with settings optimized for `YouTube`.
    Youtube,

    /// Upscale a video for higher peak quality on platforms like `YouTube`.
    /// Uses `FFmpeg`'s recommended settings for upscalling.
    /// See for more detail `<https://trac.ffmpeg.org/wiki/Encode/YouTube#Upscalingvideoforhigherpeakquality>`
    Upscale,

    /// Rotate a video 90 degrees clockwise.
    Flip,

    /// Create a `GIF` from a video with START and END timestamps. Note: `GIF` files are large even for short clips.
    Gif {
        #[arg(help = "Start of the gif.")]
        start: String,

        #[arg(help = "End of the gif.")]
        end: String,
    },

    /// Convert to a different format. (e.g. mkv -> mp4) Media streams are copied by default.
    /// Use --encode to re-encode.
    Remux {
        #[arg(short, long, help = "Reencode the output using h264 encoder.")]
        /// Re-encoding is similar to `ffmpeg -i input.mkv output.mp4`
        encode: bool,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Encoders {
    H264,
    H265,
    AV1,
}

fn run_ffmpeg<'a, Iter>(args: Iter, test: bool) -> Result<&'static str>
where
    Iter: std::iter::IntoIterator<Item = &'a str> + std::fmt::Debug,
{
    if test {
        println!("- Args:\n{args:?}\n");
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

#[derive(PartialEq, Copy, Clone)]
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
    fn new(inputs: &[PathBuf], output: Option<PathBuf>) -> Result<Self> {
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
            inputs,
            inputs_type,
        };

        let output = Output { output };

        Ok(Self { inputs, output })
    }

    fn inputs(&self) -> &[String] {
        &self.inputs.inputs
    }

    fn input(&self) -> &str {
        &self.inputs.inputs[0]
    }

    fn output(&self, command: &str, extension: &str) -> String {
        self.output
            .output
            .as_ref()
            .map_or_else(|| Self::generate_output(command, extension), String::clone)
    }

    fn generate_output(command: &str, extension: &str) -> String {
        let now = Local::now();

        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();

        format!("{command}_{timestamp}.{extension}")
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    let ctx = Ctx::new(&cli.inputs, cli.output)?;

    match cli.task {
        Tasks::Clip { start, end } => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("clip", "mp4");

            let clip = Clip::new()
                .start(&start)
                .input(input)
                .end(&end)
                .output(&output);

            println!("Creating a clip.");
            println!("{}", run_ffmpeg(clip.args.into_iter(), cli.test)?);
        }
        Tasks::Merge => {
            let inputs = ctx.inputs();

            match filters_for_merge(inputs.to_vec(), ctx.inputs.inputs_type) {
                Ok((inputs, filters)) => {
                    let output = ctx.output("merge", "mp4");
                    let merge = Merge::new()
                        .inputs(&inputs)
                        .filters(&filters)
                        .output(&output);

                    println!("Merging {} videos.", inputs.len());
                    println!("{}", run_ffmpeg(merge.args.into_iter(), cli.test)?);
                }
                Err(e) => return Err(e),
            }
        }
        Tasks::Video { framerate } => {
            if ctx.inputs.inputs_type != InputType::Directory {
                bail!("input is not a directory.");
            }

            let path = Path::new(&ctx.inputs.inputs[0]);
            let (_tmp_dir, tmp_list, total_images) = tmp_list_for_video(path, framerate)?;

            let input = tmp_list.to_str().context("Failed to convert to &str.")?;
            let output = ctx.output("video_from_images", "mp4");

            let video = Video::new().input(input).output(&output);
            println!("Creating a video from {total_images} images.");
            println!("{}", run_ffmpeg(video.args.into_iter(), cli.test)?);
        }
        Tasks::Audio => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("audio", "mp3");

            let audio = Audio::new().input(input).output(&output);
            println!("Extracting a audio.");
            println!("{}", run_ffmpeg(audio.args.into_iter(), cli.test)?);
        }
        Tasks::Encode { encoders, crf } => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("encode", "mp4");

            let crf = crf.to_string();
            let encode = Encode::new()
                .input(input)
                .encode(encoders, &crf)
                .output(&output);

            println!("Encoding a video.");
            println!("{}", run_ffmpeg(encode.args.into_iter(), cli.test)?);
        }
        Tasks::Youtube => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("youtube", "mp4");

            let youtube = Youtube::new().input(input).output(&output);
            println!("Encoding a video for youtube.");
            println!("{}", run_ffmpeg(youtube.args.into_iter(), cli.test)?);
        }
        Tasks::Upscale => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("upscale", "mp4");

            let upscale = Upscale::new().input(input).output(&output);

            println!("Upscaling a video.");
            println!("{}", run_ffmpeg(upscale.args.into_iter(), cli.test)?);
        }
        Tasks::Flip => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("flip", "mp4");

            let flip = Flip::new().input(input).output(&output);

            println!("Flipping a video.");
            println!("{}", run_ffmpeg(flip.args.into_iter(), cli.test)?);
        }

        Tasks::Gif { start, end } => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("gif", "gif");

            let gif = Gif::new()
                .input(input)
                .start(&start)
                .end(&end)
                .flags()
                .output(&output);

            println!("Converting a video to gif.");
            println!("{}", run_ffmpeg(gif.args.into_iter(), cli.test)?);
        }

        Tasks::Remux { encode } => {
            if ctx.inputs.inputs_type != InputType::File {
                bail!("input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("remux", "mp4");

            let remux = Remux::new().input(input).encode(encode).output(&output);

            println!("Converting a media format.");
            println!("{}", run_ffmpeg(remux.args.into_iter(), cli.test)?);
        }
    }

    Ok(())
}
