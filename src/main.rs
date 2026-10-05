mod args;
mod commands;
mod ffmpeg;
mod utils;

use anyhow::Result;
use clap::Parser;

use crate::args::{ClipCommand, Command, Config, Opts};
use crate::commands::{
    audio, clip, clip_gif, encode, encode_upscale, encode_youtube, merge, video,
};

fn run() -> Result<()> {
    let opts = Opts::parse();

    let config = Config {
        input: opts.input,
        output: opts.output,
        dry_run: opts.dry_run,
        verbose: opts.verbose,
        threads: opts.threads,
    };

    match opts.command {
        Command::Audio { audio_codec } => audio(config, audio_codec),

        Command::Create { framerate } => video(config, framerate),

        Command::Clip {
            start,
            end,
            encode_command,
        } => match encode_command {
            ClipCommand::Copy => clip(config, &start, &end, None),
            ClipCommand::Encode(encode_opts) => clip(config, &start, &end, Some(encode_opts)),
            ClipCommand::Gif => clip_gif(config, &start, &end),
        },

        Command::Encode { encode_opts, flip } => encode(config, encode_opts, flip),

        Command::Merge { encode_opts } => merge(config, encode_opts),

        Command::Youtube { flip } => encode_youtube(config, flip),

        Command::Upscale { flip } => encode_upscale(config, flip),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
