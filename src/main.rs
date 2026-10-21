use anyhow::{Result, anyhow};
use clap::Parser;
use tracing::Level;
use tracing_appender::{non_blocking, rolling};

use crate::args::{ClipCommand, Command, Config, Opts};
use crate::commands::{
    audio, clip, clip_gif, encode, encode_upscale, encode_youtube, merge, video,
};

mod args;
mod commands;
mod ffmpeg;
mod utils;

fn main() {
    let (writer, _guard) = match setup_log() {
        Ok((writer, guard)) => (writer, guard),
        Err(err) => {
            eprintln!("Error: {err:?}");
            std::process::exit(1);
        }
    };

    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(Level::ERROR)
        .init();

    if let Err(err) = run() {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let opts = Opts::parse();

    let config = Config {
        input: opts.input,
        output: opts.output,
        dry_run: opts.dry_run,
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

        Command::Encode {
            encode_opts,
            flip,
            crop,
        } => encode(config, encode_opts, flip, crop.as_ref()),

        Command::Merge { encode_opts } => merge(config, encode_opts),

        Command::Youtube { flip } => encode_youtube(config, flip),

        Command::Upscale { flip } => encode_upscale(config, flip),
    }
}

fn setup_log() -> Result<(non_blocking::NonBlocking, non_blocking::WorkerGuard)> {
    let dir = dirs_next::document_dir()
        .map(|path| path.join("pepega"))
        .ok_or_else(|| anyhow!("failed to setup a directory for log"))?;
    let file = rolling::never(dir, "pepega.log");
    Ok(non_blocking(file))
}
