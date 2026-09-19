use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::args::EncodeOpt;

pub fn encode(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    encode_opt: Option<EncodeOpt>,
) -> Result<()> {
    Ok(())
}

pub fn encode_youtube(dry_run: bool, input: &Path, output: Option<PathBuf>) -> Result<()> {
    Ok(())
}

pub fn encode_upscale(dry_run: bool, input: &Path, output: Option<PathBuf>) -> Result<()> {
    Ok(())
}
