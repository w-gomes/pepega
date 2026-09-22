use anyhow::{anyhow, bail, Result};

use crate::args::EncodeOpt;
use crate::commands::encode::encode_opt_to_vec;
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{generate_flags_for_loglevel, generate_inputs_and_filters, generate_output};
use crate::Config;

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

    let output = output.clone().map_or_else(
        || generate_output(&input, MERGE),
        |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();

    let (inputs, filters) = generate_inputs_and_filters(&input)?;

    args.extend(generate_flags_for_loglevel(verbose));
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
