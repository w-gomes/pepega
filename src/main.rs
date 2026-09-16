use anyhow::Result;
use clap::Parser;

mod args;
mod commands;
mod ffmpeg;
mod utils;

use crate::args::{AudioArgs, Commands, Opts, VideoArgs};
use crate::commands::{audio, clip};

// fn dry_run(&self) {
//     println!("---\nflag --dry=true printing args only");
//     println!("{:?}\n---", self.args());
// }

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.commands {
        Commands::Audio(AudioArgs { audio_codec }) => {
            if let Err(error) = audio(opts.dry_run, &opts.input, opts.output, audio_codec) {
                eprintln!("{error}");
            }
        }

        Commands::Video(VideoArgs {
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
