use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use chrono::Local;

use crate::args::EncodeOption;
use crate::ffmpeg::FFmpeg;

pub struct Clip {
    pub args: Vec<String>,
    pub dry: bool,
}

impl FFmpeg for Clip {
    fn args(&self) -> &[String] {
        self.args.as_slice()
    }
}

impl Clip {
    pub fn new_with(
        input: &Path,
        output: Option<PathBuf>,
        start: &str,
        end: &str,
        encode_option: Option<EncodeOption>,
        gif: bool,
        dry: bool,
    ) -> Result<Self> {
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

        let mut args = Vec::new();

        Ok(Self { args, dry })
    }

    pub fn run(&self) -> Result<()> {
        if self.dry {
            self.dry();
        } else {
            self.ffmpeg()?;
        }
        Ok(())
    }
}
