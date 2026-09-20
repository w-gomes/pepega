use anyhow::Result;

use std::path::{Path, PathBuf};

pub fn video(
    _dry_run: bool,
    _input: &Path,
    _output: Option<PathBuf>,
    _framerate: u64,
) -> Result<()> {
    Ok(())
}
