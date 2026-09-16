use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use chrono::Local;

use crate::args::AudioCodec;
use crate::ffmpeg::ffmpeg;

pub fn audio(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    audio_codec: AudioCodec,
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
        let file_name = format!("AUDIO_{timestamp}_{file_name}");

        let output = Path::new(&input_clone).with_file_name(file_name);
        output
    };

    let mut args = Vec::new();
    args.extend_from_slice(&[
        "-i".to_string(),
        input.display().to_string(),
        "-vn".to_string(),
    ]);

    match audio_codec {
        AudioCodec::Aac => {
            let output = output.with_extension("aac");
            args.extend_from_slice(&[
                "-c:a".to_string(),
                audio_codec.to_string(),
                "-b:a".to_string(),
                "160k".to_string(),
                output.display().to_string(),
            ]);
        }
        AudioCodec::Mp3 => {
            let output = output.with_extension("mp3");
            args.extend_from_slice(&[
                "-c:a".to_string(),
                audio_codec.to_string(),
                "-b:a".to_string(),
                "192k".to_string(),
                output.display().to_string(),
            ]);
        }
        AudioCodec::Opus => {
            let output = output.with_extension("ogg");
            args.extend_from_slice(&[
                "-c:a".to_string(),
                audio_codec.to_string(),
                "-b:a".to_string(),
                "96k".to_string(),
                output.display().to_string(),
            ]);
        }
    }

    if dry_run {
        let args = args.join(" ");
        println!("ffmpeg {args}");
    } else {
        ffmpeg(args.into_iter())?;
    }

    Ok(())
}
