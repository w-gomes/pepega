#![feature(iter_intersperse)]
use clap::{Parser, Subcommand};

use std::process::Command;

#[derive(Parser, Debug)]
#[command(about = "Smol ffmpeg wrapper")]
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
    /// Creates a clip of video with given start and end
    Clip { start: String, end: String },

    /// Merges two or more videos
    Merge,

    /// Creates a video from images with default framaterate to 1/5
    /// If desired, user can pass a new value for framerate between 0 and 5
    Video {
        #[arg(short, value_name = "FRAMERATE")]
        framerate: Option<u16>,
    },

    /// Extracts an audio stream from a video
    Audio,

    /// Encodes a video with default value for crf of 23
    /// If desired, user can pass a new value for crf between 0 and 51
    Encode {
        #[arg(short, value_name = "CRF")]
        crf: Option<u16>,
    },

    /// Encodes a video for youtube
    Youtube,
}

fn run_ffmpeg(args: &[String]) {
    dbg!(&args);
    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .expect("Error executing ffmpeg");

    let status = &output.status;

    if status.success() {
        println!("{}", "Success calling ffmpeg");
    } else {
        eprint!(
            "-- Failed to execute ffmpeg. Error code: {} -- ",
            status.code().unwrap()
        );
        eprintln!("[\n\n{}\n] --", String::from_utf8_lossy(&output.stderr));
        return;
    }

    // Will this handle all the stdout from ffmpeg?
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    let args = Pepega::parse();

    match args.command {
        Commands::Clip { start, end } => {
            // we only expect ONE input.
            if args.inputs.len() > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let input = args.inputs[0].clone();
                let output = args.output.clone();

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

                run_ffmpeg(&clip_args);
            }
        }
        Commands::Merge => {
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

                merge_args.push(String::from("-vcodec"));
                merge_args.push(String::from("copy"));
                merge_args.push(String::from("-acodec"));
                merge_args.push(String::from("copy"));

                let output = args.output.clone();
                merge_args.push(format!("{output}"));
                run_ffmpeg(&merge_args);
            }
        }

        Commands::Video { framerate } => {}
        Commands::Audio => {}
        Commands::Encode { crf } => {}
        Commands::Youtube => {}
    }
}
