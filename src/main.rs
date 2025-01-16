#![feature(iter_intersperse)]
use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(about = "Smol video tool that uses ffmpeg under the hood.")]
#[command(version, long_about = None)]
struct Yuh {
    /// Inputs files.
    #[arg(short, required = true)]
    inputs: Vec<PathBuf>,

    /// Output file.
    #[arg(short, required = true)]
    output: PathBuf,

    /// Options for the program.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // TODO: add and option to encode when it clips instead of copying.
    /// Creates a clip of video with given START and END
    Clip { start: String, end: String },

    /// Merges two or more videos
    Merge,

    /// Creates a video from images with default framaterate of 1/5.
    /// If desired, user can pass a new value for the framerate BETWEEN 0 and 5
    Video {
        #[arg(short, value_name = "FRAMERATE")]
        framerate: Option<i16>,
    },

    /// Extracts the audio stream from a video
    Audio,

    /// Encodes a video with default value of 23 for crf.
    /// If desired, user can pass a new value for crf between 0 and 51
    Encode {
        #[arg(short, value_name = "CRF")]
        crf: Option<i16>,
    },

    /// Encodes a video for youtube
    Youtube,
}

fn run_ffmpeg(args: &[String]) {
    let msg = args;
    let msg = msg.join(" ");
    print!("calling ffmpeg with args:\n\t( {msg} )\n\n");

    let ffmpeg = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute ffmpeg.");

    let output = ffmpeg
        .wait_with_output()
        .expect("Failed to get the output.");
    if !output.status.success() {
        eprint!(
            "-- Failed to execute ffmpeg. Error code: {} -- ",
            output.status.code().unwrap()
        );
    } else {
        println!("\nDone!");
    }
}

fn abs_to_string(path: &PathBuf) -> String {
    dbg!(path);
    path.canonicalize().unwrap().to_str().unwrap().to_string()
}

fn main() {
    let args = Yuh::parse();

    match args.command {
        Commands::Clip { start, end } => {
            // we only expect ONE input.
            if args.inputs.len() > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let input = abs_to_string(&args.inputs[0]);
                let output = abs_to_string(&args.output);

                let mut clip_args = Vec::new();

                // overwrites file if it already exists.
                clip_args.push(String::from("-y"));
                clip_args.push(String::from("-ss"));
                clip_args.push(format!("{start}"));
                clip_args.push(String::from("-i"));
                clip_args.push(format!("{input}"));
                clip_args.push(String::from("-to"));
                clip_args.push(format!("{end}"));
                clip_args.push(String::from("-c"));
                clip_args.push(String::from("copy"));
                clip_args.push(String::from("-copyts"));
                clip_args.push(format!("{output}"));

                println!("creating a clip of {input} [{start}...{end}] -> {output}");
                run_ffmpeg(&clip_args);
            }
        }
        Commands::Merge => {
            //TODO: Fix this
            println!("Not implemented!");
            /*
            // we expect more TWO or MORE inputs.
            if args.inputs.len() < 2 {
                eprintln!("Not enough inputs for this command!");
            } else {
                // we first append -i to every input
                let mut merge_args: Vec<_> = args
                    .inputs
                    .into_iter()
                    .intersperse(String::from("-i"))
                    .collect();

                // TODO: HACK.
                // using insert here for the first argument.
                // maybe figure something out better than intersperse.
                merge_args.insert(0, String::from("-i"));

                merge_args.push(String::from("-y"));
                merge_args.push(String::from("-vcodec"));
                merge_args.push(String::from("copy"));
                merge_args.push(String::from("-acodec"));
                merge_args.push(String::from("copy"));

                let output = args.output.clone();
                merge_args.push(format!("{output}"));

                run_ffmpeg(&merge_args);
            }
            */
        }

        Commands::Video { framerate } => {
            if args.inputs.len() > 1 {
                // TODO: check this.
                // Actually we only need a path to the folder containing the images.
                eprintln!("We only need the pattern.");
            } else {
                let framerate = match framerate {
                    Some(framerate) => {
                        if framerate < 1 && framerate > 10 {
                            println!(
                                "framerate ({}) value out of range [1..10]. Defaulting to 5.",
                                framerate
                            );
                            5
                        } else {
                            framerate
                        }
                    }
                    None => 5,
                };

                let mut video_args = Vec::new();
                let input = &args.inputs[0];

                let absolute_path = abs_to_string(input);

                if !input.is_dir() {
                    eprintln!("{} is not a directory.", input.display());
                    // do we return here?
                    return;
                }

                // this should never failed?
                let total_images = input
                    .read_dir()
                    .expect("Failed to read directory")
                    .filter(|entry| {
                        entry
                            .as_ref()
                            .unwrap()
                            .file_name()
                            .into_string()
                            .unwrap()
                            .ends_with("png")
                    })
                    .count();

                let output = abs_to_string(&args.output);

                video_args.push(String::from("-y"));
                video_args.push(String::from("-framerate"));
                video_args.push(format!("1/{framerate}"));
                video_args.push(String::from("-pattern_type"));
                video_args.push(String::from("glob"));
                video_args.push(String::from("-i"));
                video_args.push(format!("{}/*.png", absolute_path));
                video_args.push(String::from("-c:v"));
                video_args.push(String::from("libx264"));
                video_args.push(String::from("-r"));
                video_args.push(String::from("30"));
                video_args.push(String::from("-pix_fmt"));
                video_args.push(String::from("yuv420p"));
                video_args.push(format!("{output}"));

                let input = input.to_str().unwrap();
                println!("creating a video from {total_images} images in {input} with framerate 1/{framerate}");
                run_ffmpeg(&video_args);
            }
        }
        Commands::Audio => {
            // we only expect ONE input.
            if args.inputs.len() > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let input = abs_to_string(&args.inputs[0]);
                let output = abs_to_string(&args.output);

                let mut audio_args = Vec::new();

                // overwrites file if it already exists.
                audio_args.push(String::from("-y"));
                audio_args.push(String::from("-i"));
                audio_args.push(format!("{input}"));
                audio_args.push(String::from("-vn"));
                audio_args.push(String::from("-c:a"));
                audio_args.push(String::from("mp3"));
                audio_args.push(format!("{output}"));

                println!("extracting audio of {input} -> {output}");
                run_ffmpeg(&audio_args);
            }
        }
        Commands::Encode { crf } => {
            // we only expect ONE input.
            if args.inputs.len() > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let crf = match crf {
                    Some(crf) => {
                        if crf < 0 && crf > 51 {
                            println!(
                                "crf ({}) value out of range [0..51]. Defaulting to 23.",
                                crf
                            );
                            23
                        } else {
                            crf
                        }
                    }
                    None => 23,
                };

                let input = abs_to_string(&args.inputs[0]);
                let output = abs_to_string(&args.output);

                let mut encode_args = Vec::new();

                // overwrites file if it already exists.
                encode_args.push(String::from("-y"));
                encode_args.push(String::from("-i"));
                encode_args.push(format!("{input}"));
                encode_args.push(String::from("-c:v"));
                encode_args.push(String::from("-libx264"));
                encode_args.push(String::from("-crf"));
                encode_args.push(format!("{crf}"));
                encode_args.push(String::from("-c:a"));
                encode_args.push(String::from("copy"));
                encode_args.push(format!("{output}"));

                println!(
                    "encoding {input} with libx264 crf={crf} audio stream is copied -> {output}"
                );
                run_ffmpeg(&encode_args);
            }
        }
        Commands::Youtube => {
            // we only expect ONE input.
            if args.inputs.len() > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let input = abs_to_string(&args.inputs[0]);
                let output = abs_to_string(&args.output);

                let mut youtube_args = Vec::new();

                // overwrites file if it already exists.
                youtube_args.push(String::from("-y"));
                youtube_args.push(String::from("-i"));
                youtube_args.push(format!("{input}"));
                youtube_args.push(String::from("-c:v"));
                youtube_args.push(String::from("libx264"));
                youtube_args.push(String::from("-crf"));
                youtube_args.push(String::from("18"));
                youtube_args.push(String::from("-preset"));
                youtube_args.push(String::from("-c:a"));
                youtube_args.push(String::from("aac"));
                youtube_args.push(String::from("-b:a"));
                youtube_args.push(String::from("384k"));
                youtube_args.push(String::from("-pix_fmt"));
                youtube_args.push(String::from("yuv420p"));
                youtube_args.push(format!("{output}"));

                println!("encoding video for youtube {input} -> {output}");
                run_ffmpeg(&youtube_args);
            }
        }
    }
}
