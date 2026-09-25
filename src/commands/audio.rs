use anyhow::{Result, bail};

use crate::args::{AudioCodec, Config};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{get_or_generate_output, set_flags_for_loglevel};

const AUDIO: &str = "AUDIO";

pub fn audio(config: Config, audio_codec: &AudioCodec) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = get_or_generate_output(&input, output, AUDIO)?;

    let mut args = Vec::new();
    args.extend(set_flags_for_loglevel(verbose));
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

    println!("Extracting audio...");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
