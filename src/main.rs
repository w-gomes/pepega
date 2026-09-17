use anyhow::Result;
use clap::Parser;

mod args;
mod commands;
mod ffmpeg;
mod utils;

use crate::args::{AudioArgs, Commands, Opts, VideoArgs, VideoCmd};
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
            audio(opts.dry_run, &opts.input, opts.output, audio_codec)?;
        }

        Commands::Video(VideoArgs {
            flip,
            encode_opt,
            youtube,
            upscale,
            video_cmd,
        }) => {
            if let Some(video_cmd) = video_cmd {
                match video_cmd {
                    VideoCmd::Clip { start, end, gif } => {
                        if gif {
                            clip_gif(opts.dry_run, &opts.input, opts.output, &start, &end)?;
                        } else {
                            clip(
                                opts.dry_run,
                                &opts.input,
                                opts.output,
                                &start,
                                &end,
                                encode_opt,
                            )?;
                        }
                    }
                    VideoCmd::Merge => {
                        merge(opts.dry_run, &opts.input, opts.output)?;
                    }
                    VideoCmd::Create { framerate } => {
                        video(opts.dry_run, &opts.input, opts.output, framerate)?;
                    }
                }
            } else {
                if youtube {
                    encode_youtube(opts.dry_run, &opts.input, opts.output)?;
                } else if upscale {
                    encode_upscale(opts.dry_run, &opts.input, opts.output)?;
                } else {
                    encode(opts.dry_run, &opts.input, opts.output, encode_opt)?;
                }
            }
        }
    }

    Ok(())
}
