use anyhow::{Result, bail, ensure};

use crate::args::{Config, EncodeOpts};
use crate::commands::{audio_flag, loglevel_flag, video_flag};
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

    let output = get_output(&input, output, MERGE)?;
    let output = output.with_extension(video_format.to_string());
    let (inputs, filters) = generate_inputs_and_filters(&input)?;

    let mut args = Vec::new();
    args.extend(loglevel_flag(verbose));
    args.extend(inputs);
    args.extend_from_slice(&[
        "-filter_complex".to_string(),
        filters,
        "-map".to_string(),
        "[v]".to_string(),
        "-map".to_string(),
        "[a]".to_string(),
    ]);
    args.extend(video_flag(video_codec, quality));
    args.extend(audio_flag(audio_codec));
    args.push(output.display().to_string());

    println!("Merging multiple videos");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
