#![feature(iter_intersperse)]
use std::{
    env::current_dir,
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use clap::{Parser, Subcommand};
use tempfile::tempdir_in;

// - Future ideas
// General Design:
//  Currently we are doing a bunch of push calls to vec.
//  Research a way to avoid that. Maybe Vec::with_capacity
//  or vec![...] and replace the fields with parameters...
//
// Options:
//  -i <INPUTS>, -o <OUTPUT>
//  At the moment we are reading the inputs and output as Vec<String> and String
//  respectively.
//  We want to start to use PathBuf that way we can check if the arguments
//  entered are actually files or a directory.
//  The idea is to use PathBuf. So that we can check if the inputs are actually
//  files or a directory. And the output we can check if the directory is also
//  valid.
//  We can also check, for example, for the video command we can enter `.`,
//  which means run the command on the current directory. So we can check for
//  this input and run std::fs::current_dir and pass that to PathBuf and get
//  the absolute path.
//  We can use an Enum and match those inputs and pass that to the Commands.
//  A bit of engineering will be required.
//
// Commands:
// * Clip
//   At the moment we are copying video and audio streams as well as timestamp.
//   Add an option to each stream or both. E.g. --reencode
//
// * Merge
//   At the moment, merge is incomplete, but after implementing video command.
//   We could implement this in the same way. Just take the the path to
//   a directory containing all the video that we want to merge.
//   Moreover, we can also check if all the files ends with .mp4 or .mkv and
//   filter out all different files without those extensions.
//
// * Video
//   Mostly complete. We should also check for images with different extensions.
//
// * Audio
//   At the moment we are extracting audio from the entire video file.
//   We could add the clip functionality and just extract a portion of the video,
//   with a given START and END.
//
// * Filters
//   What kinda of filters do we want though?
//
#[derive(Parser, Debug)]
#[command(about = "Smol video tool that uses ffmpeg under the hood.")]
#[command(version, long_about = None)]
struct Yuh {
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
    /// Creates a clip of video with given START and END
    Clip {
        start: String,
        end: String,
        #[arg(short, long)]
        encode: bool,
    },

    /// Merges two or more videos
    Merge,

    /// Creates a video from images with default framaterate of 1/5.
    /// If desired, user can pass a new value for the framerate BETWEEN 0 and 5
    Video {
        #[arg(short, long, value_name = "FRAMERATE")]
        framerate: Option<i16>,
    },

    /// Extracts the audio stream from a video
    Audio {
        start: Option<String>,
        end: Option<String>,
    },

    /// Encodes a video with default value of 23 for crf.
    /// If desired, user can pass a new value for crf between 0 and 51
    Encode {
        #[arg(short, long, value_name = "CRF")]
        crf: Option<i16>,
    },

    /// Encodes a video for youtube
    Youtube,
}

fn run_ffmpeg(args: &[String]) {
    let msg = args;
    let msg = msg.join(" ");
    print!("calling ffmpeg with args:\n\t( {msg} )\n\n");

    let run_dummy = false;
    if run_dummy {
        println!("calling ffmpeg with no args!");
        let ffmpeg = Command::new("ffmpeg")
            .arg("-version")
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
    } else {
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
}

fn full_path(file: String) -> String {
    current_dir()
        .unwrap()
        .join(file)
        .to_str()
        .unwrap()
        .to_string()
}

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

fn main() {
    let args = Yuh::parse();

    // check inputs files
    let inputs_size = args.inputs.len();

    // handle the inputs
    let actual_inputs = if inputs_size == 1 {
        // single input
        let single_input = args.inputs[0].clone();
        // if input is a '.', then we call env::current_dir()
        if single_input == "." {
            Input::Single(current_dir().unwrap().to_str().unwrap().to_string())
        } else {
            Input::Single(full_path(single_input))
        }
    } else {
        // multiple inputs
        Input::Multiple(
            args.inputs
                .iter()
                .map(|input| full_path(input.to_string()))
                .collect(),
        )
    };

    // currently we only suport single output.
    let actual_output = full_path(args.output);

    match args.command {
        Commands::Clip { start, end, encode } => {
            // we only expect ONE input.
            if inputs_size > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let actual_inputs = actual_inputs.single();
                let mut clip_args = Vec::with_capacity(50);
                clip_args.push(String::from("-y"));
                clip_args.push(String::from("-ss"));
                clip_args.push(format!("{start}"));
                clip_args.push(String::from("-i"));
                clip_args.push(format!("{actual_inputs}"));
                clip_args.push(String::from("-to"));
                clip_args.push(format!("{end}"));
                if encode {
                    clip_args.push(String::from("-c:v"));
                    clip_args.push(String::from("libx264"));
                    clip_args.push(String::from("-c:a"));
                    clip_args.push(String::from("aac"));
                    clip_args.push(String::from("-b:a"));
                    clip_args.push(String::from("384k"));
                    clip_args.push(String::from("-pix_fmt"));
                    clip_args.push(String::from("yuv420p"));
                } else {
                    clip_args.push(String::from("-c"));
                    clip_args.push(String::from("copy"));
                    clip_args.push(String::from("-copyts"));
                }
                clip_args.push(format!("{actual_output}"));

                println!("creating a clip of {actual_inputs} [{start}...{end}] -> {actual_output}");
                run_ffmpeg(&clip_args);
            }
        }
        Commands::Merge => {
            // we expect more TWO or MORE inputs.
            if inputs_size < 2 {
                eprintln!("Not enough inputs for this command!");
            } else {
                // we first append -i to every input
                let actual_inputs = actual_inputs
                    .multiple()
                    .iter()
                    .map(|video| PathBuf::from(video.clone()).to_str().unwrap().to_string())
                    .collect::<Vec<String>>();

                // Create temporary dir and file
                let tmp_dir = tempdir_in(".").expect("Failed to create a folder");
                let tmp_list = tmp_dir.path().join("tmp_list.txt");
                let mut tmp_list_file =
                    File::create(&tmp_list).expect("Failed to create an tmp list file");

                let mut total_videos = 0;

                for entry in actual_inputs {
                    writeln!(tmp_list_file, "file '{}'", entry)
                        .expect("Failed to write to tmp_img_list_file");
                    total_videos += 1;
                }

                let inputs = tmp_list.to_str().unwrap().to_string();
                let mut merge_args = Vec::with_capacity(50);
                merge_args.push(String::from("-y"));
                merge_args.push(String::from("-f"));
                merge_args.push(String::from("concat"));
                merge_args.push(String::from("-safe"));
                merge_args.push(String::from("0"));
                merge_args.push(String::from("-i"));
                merge_args.push(format!("{inputs}"));
                merge_args.push(String::from("-c:v"));
                merge_args.push(String::from("libx264"));
                merge_args.push(String::from("-pix_fmt"));
                merge_args.push(String::from("yuv420p"));
                merge_args.push(format!("{actual_output}"));

                println!("merging {total_videos} videos in {inputs}");
                run_ffmpeg(&merge_args);
            }
        }

        Commands::Video { framerate } => {
            if inputs_size > 1 {
                eprintln!("We only need the pattern.");
            } else {
                let actual_inputs = actual_inputs.single();
                let input_path = PathBuf::from(actual_inputs.clone());
                if !input_path.is_dir() {
                    eprintln!("{} is not a directory.", actual_inputs);
                    // do we return here?
                    return;
                }

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

                // FUCK, WINDOWS DOESN'T SUPPORT GLOB, OMEGALUL
                // gotta to create a temporary file with the names of every
                // images and use ffmpeg -concat

                // Create temporary dir and file
                let tmp_dir = tempdir_in(".").expect("Failed to create a folder");
                let tmp_img_list = tmp_dir.path().join("tmp_img_list.txt");
                let mut tmp_img_list_file =
                    File::create(&tmp_img_list).expect("Failed to create an tmp image list file");

                let mut total_images = 0;

                for entry in input_path
                    .read_dir()
                    .expect("Failed to read entries in directory")
                {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();
                        let entry_path_str = entry_path.to_str().unwrap();
                        if entry_path_str.ends_with("png") {
                            writeln!(tmp_img_list_file, "file '{}'", entry_path_str)
                                .expect("Failed to write to tmp_img_list_file");
                            writeln!(tmp_img_list_file, "duration {}", framerate)
                                .expect("Failed to write to tmp_img_list_file");
                            total_images += 1;
                        }
                    }
                }

                let inputs = tmp_img_list.to_str().unwrap().to_string();

                let mut video_args = Vec::with_capacity(50);
                video_args.push(String::from("-y"));
                video_args.push(String::from("-f"));
                video_args.push(String::from("concat"));
                video_args.push(String::from("-safe"));
                video_args.push(String::from("0"));
                video_args.push(String::from("-i"));
                video_args.push(format!("{inputs}"));
                video_args.push(String::from("-c:v"));
                video_args.push(String::from("libx264"));
                video_args.push(String::from("-r"));
                video_args.push(String::from("30"));
                video_args.push(String::from("-pix_fmt"));
                video_args.push(String::from("yuv420p"));
                video_args.push(format!("{actual_output}"));

                println!("creating a video from {total_images} images in {inputs} with framerate 1/{framerate}");
                run_ffmpeg(&video_args);
            }
        }
        Commands::Audio { start, end } => {
            // we only expect ONE input.
            if inputs_size > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                if start.is_none() || end.is_none() {
                    eprintln!("Need both start and end to clip audio");
                    return;
                }

                let actual_inputs = actual_inputs.single();
                let mut audio_args = Vec::with_capacity(50);
                audio_args.push(String::from("-y"));

                if let Some(start) = start {
                    audio_args.push(String::from("-ss"));
                    audio_args.push(format!("{start}"));
                }

                audio_args.push(String::from("-i"));
                audio_args.push(format!("{actual_inputs}"));

                if let Some(end) = end {
                    audio_args.push(String::from("-to"));
                    audio_args.push(format!("{end}"));
                }

                audio_args.push(String::from("-vn"));
                audio_args.push(String::from("-c:a"));
                audio_args.push(String::from("mp3"));
                audio_args.push(format!("{actual_output}"));

                println!("extracting audio of {actual_inputs} -> {actual_output}");
                run_ffmpeg(&audio_args);
            }
        }
        Commands::Encode { crf } => {
            // we only expect ONE input.
            if inputs_size > 1 {
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

                let actual_inputs = actual_inputs.single();
                let mut encode_args = Vec::with_capacity(50);
                encode_args.push(String::from("-y"));
                encode_args.push(String::from("-i"));
                encode_args.push(format!("{actual_inputs}"));
                encode_args.push(String::from("-c:v"));
                encode_args.push(String::from("-libx264"));
                encode_args.push(String::from("-crf"));
                encode_args.push(format!("{crf}"));
                encode_args.push(String::from("-c:a"));
                encode_args.push(String::from("copy"));
                encode_args.push(format!("{actual_output}"));

                println!(
                    "encoding {actual_inputs} with libx264 crf={crf} audio stream is copied -> {actual_output}"
                );
                run_ffmpeg(&encode_args);
            }
        }
        Commands::Youtube => {
            // we only expect ONE input.
            if inputs_size > 1 {
                eprintln!("Too many inputs for this command!");
            } else {
                let actual_inputs = actual_inputs.single();
                let mut youtube_args = Vec::with_capacity(50);
                youtube_args.push(String::from("-y"));
                youtube_args.push(String::from("-i"));
                youtube_args.push(format!("{actual_inputs}"));
                youtube_args.push(String::from("-c:v"));
                youtube_args.push(String::from("libx264"));
                youtube_args.push(String::from("-crf"));
                youtube_args.push(String::from("18"));
                youtube_args.push(String::from("-preset"));
                youtube_args.push(String::from("ultrafast"));
                youtube_args.push(String::from("-c:a"));
                youtube_args.push(String::from("aac"));
                youtube_args.push(String::from("-b:a"));
                youtube_args.push(String::from("384k"));
                youtube_args.push(String::from("-pix_fmt"));
                youtube_args.push(String::from("yuv420p"));
                youtube_args.push(format!("{actual_output}"));

                println!("encoding video for youtube {actual_inputs} -> {actual_output}");
                run_ffmpeg(&youtube_args);
            }
        }
    }
}
