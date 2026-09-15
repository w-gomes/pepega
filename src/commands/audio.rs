use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use chrono::Local;
use log::info;

use crate::args::AudioCodec;
use crate::ffmpeg::FFmpeg;

pub struct Audio {
    pub args: Vec<String>,
    pub dry: bool,
}

impl FFmpeg for Audio {
    fn args(&self) -> &[String] {
        self.args.as_slice()
    }
}

impl Audio {
    pub fn new_with(
        input: &Path,
        output: Option<PathBuf>,
        audio_codec: &AudioCodec,
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
            let file_name = format!("AUDIO_FROM_{file_name}_{timestamp}");

            let output = Path::new(&input_clone)
                .with_file_name(file_name)
                .with_extension("mp3");
            output
        };

        let mut args = Vec::new();
        args.extend_from_slice(&["-i".to_string(), input.display().to_string()]);
        args.extend_from_slice(&["-vn".to_string(), "-c:a".to_string()]);
        args.push(audio_codec.to_string());
        args.extend_from_slice(&["-b:a".to_string(), "192k".to_string()]);
        args.push(output.display().to_string());

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
