use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::args::EncodeOpt;
use crate::ffmpeg::ffmpeg;
use crate::utils::generate_output;

pub fn clip(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    start: &str,
    end: &str,
    encode_opt: Option<EncodeOpt>,
) -> Result<()> {
    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = output.clone().map_or_else(
        || generate_output(&input, "CLIP"),
        |_| output.ok_or(anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();

    if let Some(encode_opt) = encode_opt {
        let output = output.with_extension(encode_opt.video_format.to_string());
        args.extend_from_slice(&[
            "-ss".to_string(),
            start.to_string(),
            "-accurate_seek".to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-to".to_string(),
            end.to_string(),
            "-c:v".to_string(),
            encode_opt.video_codec.to_string(),
            "-c:a".to_string(),
            encode_opt.audio_codec.to_string(),
            output.display().to_string(),
        ]);
    } else {
        let output = output.with_extension("mp4");
        args.extend_from_slice(&[
            "-ss".to_string(),
            start.to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-to".to_string(),
            end.to_string(),
            "-c".to_string(),
            "copy".to_string(),
            "-copyts".to_string(),
            output.display().to_string(),
        ]);
    };

    println!("wtf");

    if dry_run {
        let args = args.join(" ");
        println!("ffmpeg {args}");
    } else {
        ffmpeg(args.into_iter())?;
    }

    Ok(())
}

pub fn clip_gif(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    start: &str,
    end: &str,
) -> Result<()> {
    Ok(())
}
