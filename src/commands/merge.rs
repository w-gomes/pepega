use std::process::Command;

use anyhow::{Result, bail, ensure};

use crate::args::{Config, EncodeOpts};
use crate::commands::{FFMPEG, audio_flags, loglevel_flags, push_args, video_flags};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{generate_inputs_and_filters, get_output};

const MERGE: &str = "MERGE";

pub fn merge(config: Config, encode_opts: EncodeOpts) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    let EncodeOpts {
        video_codec,
        audio_codec,
        video_format,
        quality,
    } = encode_opts;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    ensure!(input.is_dir(), "input must be a directory.");

    let mut cmd = Command::new(FFMPEG);

    let output = get_output(&input, output, MERGE)?;
    let output = output.with_extension(video_format.to_string());

    let (inputs, filters) = generate_inputs_and_filters(&input)?;

    loglevel_flags(&mut cmd, verbose);

    cmd.args(inputs);

    push_args![cmd => [
        "-filter_complex", filters.as_str(),
        "-map", "[v]",
        "-map", "[a]",
    ]];

    video_flags(&mut cmd, video_codec, quality);
    audio_flags(&mut cmd, audio_codec);
    cmd.arg(output);

    println!("Merging multiple videos");
    try_run_ffmpeg(dry_run, cmd)?;

    Ok(())
}
