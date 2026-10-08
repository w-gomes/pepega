use std::process::Command;

use anyhow::{Result, bail, ensure};

use crate::args::{Config, EncodeOpts, VideoFormat};
use crate::commands::{FFMPEG, audio_flags, global_flags, push_args, video_flags};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::get_output;

const CLIP: &str = "CLIP";
const CLIP_COPY: &str = "CLIP_COPY";
const CLIP_GIF: &str = "CLIP_GIF";

pub fn clip(config: Config, start: &str, end: &str, encode_opts: Option<EncodeOpts>) -> Result<()> {
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

    let cmd_str = if encode_opts.is_some() {
        CLIP
    } else {
        CLIP_COPY
    };

    let output = get_output(&input, output, cmd_str)?;

    let mut cmd = Command::new(FFMPEG);
    global_flags(&mut cmd);

    if let Some(encode_opts) = encode_opts {
        let EncodeOpts {
            video_codec,
            audio_codec,
            video_format,
            quality,
        } = encode_opts;

        push_args![cmd => [
            "-ss", start,
            "-accurate_seek",
            "-i", input,
            "-to", end,
        ]];

        video_flags(&mut cmd, video_codec, quality);
        audio_flags(&mut cmd, audio_codec);

        let output = output.with_extension(video_format.to_string());
        cmd.arg(output);
    } else {
        let video_format = VideoFormat::default();
        let output = output.with_extension(video_format.to_string());

        push_args![cmd => [
            "-ss", start,
            "-i", input,
            "-to", end,
            "-c", "copy", "-copyts",
            output,
        ]];
    }

    println!("Clipping a video");
    try_run_ffmpeg(dry_run, &mut [cmd], None)?;

    Ok(())
}

pub fn clip_gif(config: Config, start: &str, end: &str) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        ..
    } = config;

    ensure!(input.is_file(), "input must be a file.");

    let mut cmd = Command::new(FFMPEG);

    let output = get_output(&input, output, CLIP_GIF)?;
    let output = output.with_extension("gif");

    global_flags(&mut cmd);

    push_args![cmd => [
        "-i", input,
        "-ss", start,
        "-to", end,
        "-vf",
        "fps=30,scale=1080:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
        "-loop", 0.to_string(),
        output,
    ]];

    println!("Clipping a video and saving as gif");
    try_run_ffmpeg(dry_run, &mut [cmd], None)?;

    Ok(())
}
