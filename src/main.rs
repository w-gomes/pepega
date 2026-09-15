use anyhow::Result;
use clap::Parser;
use log::error;

mod args;
mod commands;
mod ffmpeg;

use crate::args::{Cli, Commands};
use crate::commands::{Audio, Clip};

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.commands {
        Commands::Audio {
            input,
            output,
            audio_codec,
        } => {
            let audio = Audio::new_with(&input, output, &audio_codec, cli.dry)?;
            if let Err(err) = audio.run() {
                error!("{err}");
            }
        }
        Commands::Clip {
            input,
            output,
            start,
            end,
            encode_option,
            gif,
        } => {
            let clip = Clip::new_with(&input, output, &start, &end, encode_option, gif, cli.dry)?;
            if let Err(err) = clip.run() {
                error!("{err}");
            }
        }
        Commands::Encode { .. } => {
            todo!()
        }
        Commands::Flip { .. } => {
            todo!()
        }
        Commands::Merge { .. } => {
            todo!()
        }
        Commands::Upscale { .. } => {
            todo!()
        }
        Commands::Video { .. } => {
            todo!()
        }
        Commands::Youtube { .. } => {
            todo!()
        }
    }

    Ok(())
}
