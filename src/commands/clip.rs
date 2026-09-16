use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use chrono::Local;

use crate::args::EncodeOpt;
use crate::ffmpeg::ffmpeg;

pub fn clip(
    input: &Path,
    output: Option<PathBuf>,
    start: &str,
    end: &str,
    encode_option: Option<EncodeOpt>,
    gif: bool,
    dry_run: bool,
) -> Result<()> {
    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = if let Some(output) = output {
        output
    } else {
        let input_clone = input.to_path_buf();
        let Some(file_name) = input_clone.file_name() else {
            bail!("Error extracting file name from Input");
        };

        // Convert OsStr to &str to pass to format!
        let Some(file_name) = file_name.to_str() else {
            bail!("Error converting OsStr to &str");
        };

        let now = Local::now();
        let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
        let file_name = format!("CLIP_FROM_{file_name}_{timestamp}");

        let output = Path::new(&input_clone)
            .with_file_name(file_name)
            .with_extension("mp4");
        output
    };

    Ok(())
}
