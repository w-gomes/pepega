use anyhow::{Context, Result};
use clap::Parser;

mod args;
mod commands;
mod ffmpeg;
mod utils;

use crate::args::{AudioArgs, Commands, Config, Opts, VideoArgs, VideoCommands};
use crate::commands::{
    audio, clip, clip_gif, encode, encode_upscale, encode_youtube, merge, video,
};

fn main() -> Result<()> {
    // Set rayon to use 6 threads instead of all available threads
    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .with_context(|| "Failed to create a thread pool".to_string())?;

    let opts = Opts::parse();

    let config = Config {
        input: opts.input,
        output: opts.output,
        dry_run: opts.dry_run,
        verbose: opts.verbose,
    };

    match opts.commands {
        Commands::Audio(AudioArgs { audio_codec }) => {
            audio(config, &audio_codec)?;
        }

        Commands::Video(VideoArgs { video_commands }) => match video_commands {
            VideoCommands::Clip {
                start,
                end,
                encode_opt,
            } => {
                if let Some(ref encode_opt) = encode_opt {
                    encode_opt.check_quality_flags()?;
                }

                clip(config, &start, &end, encode_opt)?;
            }

            VideoCommands::Encode { encode_opt, flip } => {
                encode_opt.check_quality_flags()?;
                encode(config, encode_opt, flip)?;
            }

            VideoCommands::Merge { encode_opt } => {
                encode_opt.check_quality_flags()?;
                merge(config, &encode_opt)?;
            }

            VideoCommands::Youtube { flip } => {
                encode_youtube(config, flip)?;
            }

            VideoCommands::Upscale { flip } => {
                encode_upscale(config, flip)?;
            }

            VideoCommands::Gif { start, end } => {
                clip_gif(config, &start, &end)?;
            }
        },

        Commands::ToVideo { framerate } => {
            video(config, framerate)?;
        }
    }

    Ok(())
}
