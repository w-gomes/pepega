use anyhow::Result;
use clap::Parser;

mod args;
mod commands;
mod ffmpeg;
mod utils;

use crate::args::{AudioArgs, Commands, Opts, VideoArgs, VideoCommands};
use crate::commands::{
    audio, clip, clip_gif, encode, encode_upscale, encode_youtube, merge, video,
};

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.commands {
        Commands::Audio(AudioArgs { audio_codec }) => {
            audio(opts.dry_run, &opts.input, opts.output, &audio_codec)?;
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

                clip(
                    opts.dry_run,
                    &opts.input,
                    opts.output,
                    &start,
                    &end,
                    encode_opt,
                )?;
            }

            VideoCommands::Encode { encode_opt, flip } => {
                encode_opt.check_quality_flags()?;
                encode(opts.dry_run, &opts.input, opts.output, encode_opt, flip)?;
            }

            VideoCommands::Merge { encode_opt } => {
                encode_opt.check_quality_flags()?;
                merge(opts.dry_run, &opts.input, opts.output, &encode_opt)?;
            }

            VideoCommands::Youtube { flip } => {
                encode_youtube(opts.dry_run, &opts.input, opts.output, flip)?;
            }

            VideoCommands::Upscale { flip } => {
                encode_upscale(opts.dry_run, &opts.input, opts.output, flip)?;
            }

            VideoCommands::Gif { start, end } => {
                clip_gif(opts.dry_run, &opts.input, opts.output, &start, &end)?;
            }
        },

        Commands::ToVideo { framerate } => {
            video(opts.dry_run, &opts.input, opts.output, framerate)?;
        }
    }

    Ok(())
}
