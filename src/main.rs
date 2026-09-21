use anyhow::Result;
use clap::Parser;

mod args;
mod commands;
mod ffmpeg;
mod utils;

use crate::args::{AudioArgs, Commands, EncodeCommands, Opts, VideoArgs, VideoCommands};
use crate::commands::{
    audio, clip, clip_gif, encode, encode_upscale, encode_youtube, merge, video,
};

// fn dry_run(&self) {
//     println!("---\nflag --dry=true printing args only");
//     println!("{:?}\n---", self.args());
// }

fn main() -> Result<()> {
    let opts = Opts::parse();

    match opts.commands {
        Commands::Audio(AudioArgs { audio_codec }) => {
            audio(opts.dry_run, &opts.input, opts.output, &audio_codec)?;
        }

        Commands::Video(VideoArgs { video_subcommand }) => match video_subcommand {
            VideoCommands::Encode {
                encode_opt,
                encode_subcommand,
            } => {
                encode_opt.check_quality_flags()?;

                if let Some(encode_subcommand) = encode_subcommand {
                    match encode_subcommand {
                        EncodeCommands::Clip { start, end } => {
                            clip(
                                opts.dry_run,
                                &opts.input,
                                opts.output,
                                &start,
                                &end,
                                &encode_opt,
                            )?;
                        }
                        EncodeCommands::Flip => {
                            encode(opts.dry_run, &opts.input, opts.output, &encode_opt, true)?;
                        }
                        EncodeCommands::Merge => {
                            merge(opts.dry_run, &opts.input, opts.output, &encode_opt)?;
                        }
                    }
                } else {
                    encode(opts.dry_run, &opts.input, opts.output, &encode_opt, false)?;
                }
            }
            VideoCommands::Youtube => {
                encode_youtube(opts.dry_run, &opts.input, opts.output, false)?;
            }
            VideoCommands::Upscale => {
                encode_upscale(opts.dry_run, &opts.input, opts.output, false)?;
            }
            VideoCommands::Gif { start, end } => {
                clip_gif(opts.dry_run, &opts.input, opts.output, &start, &end)?;
            }
        },

        Commands::Image { framerate } => {
            video(opts.dry_run, &opts.input, opts.output, framerate)?;
        }
    }

    Ok(())
}
