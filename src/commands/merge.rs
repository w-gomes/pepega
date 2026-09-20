use anyhow::{anyhow, bail, Result};

use std::path::{Path, PathBuf};

use crate::args::EncodeOpt;
use crate::commands::encode::encode_opt_to_vec;
use crate::ffmpeg::ffmpeg;
use crate::utils::{generate_output_with, merge_with_inputs_and_filters};

const MERGE: &str = "MERGE";

pub fn merge(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    encode_opt: &EncodeOpt,
) -> Result<()> {
    if !input.is_dir() {
        bail!("Input must be a directory");
    }

    let output = output.clone().map_or_else(
        || generate_output_with(input, MERGE),
        |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();

    let (inputs, filters) = merge_with_inputs_and_filters(input)?;

    args.extend(inputs);
    args.extend_from_slice(&[
        "-filter_complex".to_string(),
        filters,
        "-map".to_string(),
        "[v]".to_string(),
        "-map".to_string(),
        "[a]".to_string(),
    ]);
    args.extend(encode_opt_to_vec(&encode_opt));
    args.push(output.display().to_string());

    if dry_run {
        let args = args.join(" ");
        println!("ffmpeg {args}");
    } else {
        ffmpeg(args)?;
    }

    Ok(())
}
