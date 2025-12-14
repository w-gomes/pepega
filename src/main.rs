use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context};
use chrono::Local;
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

    /// Prints args sent to ffmpeg.
    #[arg(long)]
    test: bool,
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

fn run_ffmpeg<'a, Iter>(args: Iter, test: bool) -> anyhow::Result<&'static str>
where
    Iter: std::iter::IntoIterator<Item = &'a str> + std::fmt::Debug,
{
    if test {
        println!("- Args:\n{:?}\n", args);
        let ffmpeg = Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::piped())
            .spawn()?;

        let output = ffmpeg.wait_with_output()?;
        if !output.status.success() {
            anyhow::bail!(
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
            anyhow::bail!(
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
    fn new(inputs: Vec<PathBuf>, output: Option<PathBuf>) -> anyhow::Result<Self> {
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

        let inputs = Inputs { inputs, inputs_type };

        let output = Output { output };

        Ok(Self { inputs, output })
    }

    fn inputs(&self) -> &[String] {
        &self.inputs.inputs
    }

    fn input(&self) -> &str {
        self.inputs.inputs.first().unwrap()
    }

    fn output(&self, command: &str, extension: &str) -> String {
        match &self.output.output {
            Some(out) => out.to_string(),
            None => Self::generate_output(command, extension),
        }
    }

    fn generate_output(command: &str, extension: &str) -> String {
        let now = Local::now();

        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();

        format!("{}_{}.{}", command, timestamp, extension)
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let ctx = Ctx::new(cli.inputs, cli.output)?;

    match cli.task {
        Tasks::Clip { start, end, encode } => {
            if ctx.inputs.inputs_type != InputType::File {
                anyhow::bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("clip", "mp4");

            let clip = Clip::new()
                .start(&start)
                .input(input)
                .end(&end)
                .encode(encode)
                .output(&output);

            println!("Creating a clip.");
            println!("{}", run_ffmpeg(clip.args.into_iter(), cli.test)?);
        }
        Tasks::Merge => {
            let mut filters = String::new();
            let mut new_inputs = Vec::new();
            let inputs = ctx.inputs();

            // Check the input type and write the entries to the tmp file.
            let inputs = match ctx.inputs.inputs_type {
                // It's multiple files, apply the filters.
                InputType::File => {
                    if inputs.len() < 2 {
                        anyhow::bail!("Not enough inputs");
                    }

                    let total_inputs = inputs.len();
                    for i in 0..total_inputs {
                        filters.push_str(format!("[{0}:v:0][{0}:a:0]", i).as_str());
                    }
                    filters.push_str(format!("concat=n={}:v=1:a=1[v][a]", total_inputs).as_str());

                    inputs
                }
                // It's a single directory, we iterator over that and read
                // each entry checking if they end with mp4 or mkv and write their
                // absolute path to a file
                InputType::Directory => {
                    // We use PathBuf to iterate the directory.
                    let input_path = PathBuf::from(inputs[0].clone());

                    let mut idx = 0;
                    for entry in input_path
                        .read_dir()
                        .expect("Failed to read entries in directory.")
                        .flatten()
                    {
                        let entry_path = entry.path();
                        let entry_path_str = entry_path.to_str().with_context(|| {
                            format!("Failed to convert {} to &str.", entry_path.display())
                        })?;
                        if entry_path_str.ends_with("mkv") || entry_path_str.ends_with("mp4") {
                            new_inputs.push(entry_path_str.to_string());
                        }

                        filters.push_str(format!("[{0}:v:0][{0}:a:0]", idx).as_str());
                        idx += 1;
                    }
                    filters.push_str(format!("concat=n={}:v=1:a=1[v][a]", idx).as_str());

                    &new_inputs
                }
            };

            let output = ctx.output("merge", "mp4");

            let merge = Merge::new()
                .inputs(inputs)
                .filters(&filters)
                .output(&output);

            println!("Merging {} videos.", inputs.len());
            println!("{}", run_ffmpeg(merge.args.into_iter(), cli.test)?);
        }
        Tasks::Video { framerate } => {
            if ctx.inputs.inputs_type != InputType::Directory {
                anyhow::bail!("input is not a directory.");
            }

            // Creates a temporary dir and a file
            let tmp_dir = TempDir::new_in(".").expect("Failed to create a temporary folder");
            let tmp_list = tmp_dir.path().join("tmp_list.txt");
            let mut tmp_list_file =
                File::create(&tmp_list).expect("Failed to create a tmp image list file");

            let mut total_images = 0;

            let source_dir = PathBuf::from(&ctx.inputs.inputs[0]);
            for entry in source_dir
                .read_dir()
                .expect("Failed to read entries in directory")
                .flatten()
            {
                let entry_path = entry.path();
                let entry_path_absolute = fs::canonicalize(&entry_path).with_context(|| {
                    format!("Failed to get absolute path of {}", entry_path.display())
                })?;
                let entry_path_str = entry_path_absolute.to_str().with_context(|| {
                    format!(
                        "Failed to convert {} to &str.",
                        entry_path_absolute.display()
                    )
                })?;
                if entry_path_str.ends_with("png") || entry_path_str.ends_with("jpg") {
                    dbg!(entry_path_str);
                    writeln!(tmp_list_file, "file '{}'", entry_path_str)
                        .expect("Failed to write to tmp_list_file");
                    writeln!(tmp_list_file, "duration {}", framerate)
                        .expect("Failed to write to tmp_list_file");
                    total_images += 1;
                }
            }

            let output = ctx.output("video_from_images", "mp4");
            let input = tmp_list.to_str().context("Failed to convert to &str.")?;

            let video = Video::new().input(input).output(&output);
            println!("Creating a video from {total_images} images.");
            println!("{}", run_ffmpeg(video.args.into_iter(), cli.test)?);
        }
        Tasks::Audio => {
            if ctx.inputs.inputs_type != InputType::File {
                anyhow::bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("audio", "mp3");

            let audio = Audio::new().input(input).output(&output);
            println!("Extracting a audio.");
            println!("{}", run_ffmpeg(audio.args.into_iter(), cli.test)?);
        }
        Tasks::Encode { encoders, crf } => {
            if ctx.inputs.inputs_type != InputType::File {
                anyhow::bail!("Input is not a file.");
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
                anyhow::bail!("Input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("youtube", "mp4");

            let youtube = Youtube::new().input(input).output(&output);
            println!("Encoding a video for youtube.");
            println!("{}", run_ffmpeg(youtube.args.into_iter(), cli.test)?);
        }
        Tasks::Upscale => {
            if ctx.inputs.inputs_type != InputType::File {
                anyhow::bail!("input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("upscale", "mp4");

            let upscale = Upscale::new().input(input).output(&output);

            println!("Upscaling a video.");
            println!("{}", run_ffmpeg(upscale.args.into_iter(), cli.test)?);
        }
        Tasks::Flip => {
            if ctx.inputs.inputs_type != InputType::File {
                anyhow::bail!("input is not a file.");
            }

            let input = ctx.input();
            let output = ctx.output("flip", "mp4");

            let flip = Flip::new().input(input).output(&output);

            println!("Flipping a video.");
            println!("{}", run_ffmpeg(flip.args.into_iter(), cli.test)?);
        }
    }

    Ok(())
}
