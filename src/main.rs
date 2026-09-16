use anyhow::Result;
use clap::Parser;

mod args;
mod commands;
mod ffmpeg;

use crate::args::Commands::{Audio, Video};
use crate::args::{AudioArgs, Commands, Opts, VideoArgs};
use crate::commands::{audio, clip};

// fn dry_run(&self) {
//     println!("---\nflag --dry=true printing args only");
//     println!("{:?}\n---", self.args());
// }

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.commands {
        Audio(AudioArgs {
            audio_codec,
            audio_format,
        }) => {
            if let Err(error) = audio(&opts.input, opts.output, audio_codec, audio_format) {
                eprintln!("{error}");
            }
        }

        Video(VideoArgs {
            flip,
            encode_opt,
            youtube,
            upscale,
            video_cmd,
        }) => {
            todo!();
        }
    }

    Ok(())
}
