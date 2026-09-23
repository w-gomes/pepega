use anyhow::{Result, bail};

use crate::args::{Config, EncodeOpt};
use crate::commands::encode::encode_opt_to_vec;
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{generate_inputs_and_filters, generate_output, set_flags_for_loglevel};

const MERGE: &str = "MERGE";

pub fn merge(config: Config, encode_opt: &EncodeOpt) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
    } = config;

    if !input.is_dir() {
        bail!("Input must be a directory");
    }

    let output = output.map_or_else(|| generate_output(&input, MERGE), Ok)?;

    let mut args = Vec::new();

    let (inputs, filters) = generate_inputs_and_filters(&input)?;

    args.extend(set_flags_for_loglevel(verbose));
    args.extend(inputs);
    args.extend_from_slice(&[
        "-filter_complex".to_string(),
        filters,
        "-map".to_string(),
        "[v]".to_string(),
        "-map".to_string(),
        "[a]".to_string(),
    ]);
    args.extend(encode_opt_to_vec(encode_opt));

    let output = output.with_extension(encode_opt.video_format.to_string());
    args.push(output.display().to_string());

    println!("Merging multiple videos");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
