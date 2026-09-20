use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::args::AudioCodec;
use crate::ffmpeg::ffmpeg;
use crate::utils::generate_output_with;

pub fn audio(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    audio_codec: &AudioCodec,
) -> Result<()> {
    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = output.clone().map_or_else(
        || generate_output_with(input, "AUDIO"),
        |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();
    args.extend_from_slice(&[
        "-i".to_string(),
        input.display().to_string(),
        "-vn".to_string(),
    ]);

    // The output is added together with audio_codec, because the file format
    // depends on it.
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
        ffmpeg(args)?;
    }

    Ok(())
}
