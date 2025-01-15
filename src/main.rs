use clap::{Parser, Subcommand};

use std::{path::PathBuf, process::Command};

#[derive(Parser, Debug)]
#[command(about = "Smol ffmpeg wrapper")]
#[command(version, long_about = None)]
struct Pepega {
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

fn run_ffmpeg(arg: &str) {
    let output = Command::new("ffmpeg")
        .arg(arg)
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

fn main() {
    let args = Pepega::parse();

    match args.command {
        Commands::Clip { start, end } => {
            run_ffmpeg("-version");
        }
        Commands::Merge => {}
        Commands::Video { framerate } => {}
        Commands::Audio => {}
        Commands::Encode { crf } => {}
        Commands::Youtube => {}
    }
}
