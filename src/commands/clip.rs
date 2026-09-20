use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::args::EncodeOpt;
use crate::ffmpeg::ffmpeg;
use crate::utils::generate_output_with;

pub fn clip(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    start: &str,
    end: &str,
    encode_opt: EncodeOpt,
) -> Result<()> {
    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = output.clone().map_or_else(
        || generate_output_with(&input, "CLIP"),
        |_| output.ok_or(anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();
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
    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = output.clone().map_or_else(
        || generate_output_with(&input, "CLIP_GIF"),
        |_| output.ok_or(anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();
    args.extend_from_slice(&[
        "-i".to_string(),
        input.display().to_string(),
        "-ss".to_string(),
        start.to_string(),
        "-to".to_string(),
        end.to_string(),
        "-vf".to_string(),
        "fps=30,scale=1080:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse"
            .to_string(),
        "-loop".to_string(),
        "0".to_string(),
        output.with_extension("gif").display().to_string(),
    ]);

    if dry_run {
        let args = args.join(" ");
        println!("ffmpeg {args}");
    } else {
        ffmpeg(args.into_iter())?;
    }

    Ok(())
}
