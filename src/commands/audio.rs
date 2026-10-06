use anyhow::{Result, bail, ensure};

use crate::args::{AudioCodec, Config};
use crate::commands::{audio_flag, loglevel_flag};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::get_output;

const AUDIO: &str = "AUDIO";

pub fn audio(config: Config, audio_codec: AudioCodec) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    ensure!(input.is_file(), "input must be a file.");

    let output = get_output(&input, output, AUDIO)?;

    let mut args = Vec::new();
    args.extend(loglevel_flag(verbose));
    args.extend_from_slice(&[
        "-i".to_string(),
        input.display().to_string(),
        "-vn".to_string(),
    ]);

    // The output is added together with audio_codec, because the file format
    // depends on it.
    match audio_codec {
        enc @ AudioCodec::Aac => {
            let output = output.with_extension("aac");
            args.extend(audio_flag(enc));
            args.push(output.display().to_string());
        }
        enc @ AudioCodec::Mp3 => {
            let output = output.with_extension("mp3");
            args.extend(audio_flag(enc));
            args.push(output.display().to_string());
        }
        enc @ AudioCodec::Opus => {
            let output = output.with_extension("ogg");
            args.extend(audio_flag(enc));
            args.push(output.display().to_string());
        }
    }

    println!("Extracting audio...");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
