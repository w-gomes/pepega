use std::process::Command;

use anyhow::{Result, bail, ensure};

use crate::args::{AudioCodec, Config};
use crate::commands::{FFMPEG, audio_flags, global_flags, push_args};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::get_output;

const AUDIO: &str = "AUDIO";

pub fn audio(config: Config, audio_codec: AudioCodec) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        ..
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    ensure!(input.is_file(), "input must be a file.");

    let output = get_output(&input, output, AUDIO)?;

    let mut cmd = Command::new(FFMPEG);

    global_flags(&mut cmd);

    push_args![cmd => ["-i", input, "-vn"]];

    // The output is added together with audio_codec, because the file format
    // depends on it.
    match audio_codec {
        enc @ AudioCodec::Aac => {
            audio_flags(&mut cmd, enc);
            let output = output.with_extension("aac");
            cmd.arg(output);
        }
        enc @ AudioCodec::Mp3 => {
            audio_flags(&mut cmd, enc);
            let output = output.with_extension("mp3");
            cmd.arg(output);
        }
        enc @ AudioCodec::Opus => {
            audio_flags(&mut cmd, enc);
            let output = output.with_extension("ogg");
            cmd.arg(output);
        }
    }

    println!("Extracting audio...");
    try_run_ffmpeg(dry_run, &mut [cmd], None)?;

    Ok(())
}
