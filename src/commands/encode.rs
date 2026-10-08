use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};

use crate::args::{Config, EncodeOpts, VideoFormat};
use crate::commands::{FFMPEG, audio_flags, flip_flags, global_flags, push_args, video_flags};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{get_inputs_and_outputs, get_output};

const ENCODE: &str = "ENCODE";
const ENCODE_YOUTUBE: &str = "ENCODE_YOUTUBE";
const ENCODE_UPSCALE: &str = "ENCODE_UPSCALE";

pub fn encode(config: Config, encode_opts: EncodeOpts, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        threads,
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    if input.is_file() {
        let args = single_file(&input, output, encode_opts, flip)?;
        println!("Encoding a single file");
        try_run_ffmpeg(dry_run, &mut [args], None)?;
    } else if input.is_dir() {
        let mut args = multiple_file(&input, encode_opts, ENCODE, flip)?;
        println!("Encoding multiple files");
        try_run_ffmpeg(dry_run, &mut args, threads)?;
    }

    Ok(())
}

pub fn encode_youtube(config: Config, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        threads,
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    let output_flags = &[
        "-c:v", "libx264", "-crf", "18", "-preset", "medium", "-c:a", "aac", "-b:a", "384k",
        "-pix_fmt", "yuv420p",
    ];

    if input.is_file() {
        let output = get_output(&input, output, ENCODE_YOUTUBE)?;

        let mut cmd = Command::new(FFMPEG);

        global_flags(&mut cmd);

        if flip {
            flip_flags(&mut cmd);
        }

        push_args![cmd => ["-i", input]];

        cmd.args(output_flags);

        let video_format = VideoFormat::default();
        let output = output.with_extension(video_format.to_string());
        cmd.arg(output);

        println!("Encoding a single file for youtube");
        try_run_ffmpeg(dry_run, &mut [cmd], None)?;
    } else if input.is_dir() {
        let inputs_outputs_pair = get_inputs_and_outputs(&input, ENCODE_YOUTUBE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let mut cmd = Command::new(FFMPEG);

            global_flags(&mut cmd);

            if flip {
                flip_flags(&mut cmd);
            }

            push_args![cmd => ["-i", input]];

            cmd.args(output_flags);

            let video_format = VideoFormat::default();
            let output = output.with_extension(video_format.to_string());
            cmd.arg(output);

            args.push(cmd);
        }

        println!("Encoding multiple files for youtube");
        try_run_ffmpeg(dry_run, &mut args, threads)?;
    }

    Ok(())
}

pub fn encode_upscale(config: Config, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        threads,
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    let output_flags = &[
        "-vf",
        "scale=iw*2:ih*2:flags=neighbor",
        "-c:v",
        "libx264",
        "-crf",
        "18",
        "-preset",
        "slow",
        "-c:a",
        "aac",
        "-b:a",
        "384k",
    ];

    if input.is_file() {
        let output = get_output(&input, output, ENCODE_UPSCALE)?;

        let mut cmd = Command::new(FFMPEG);

        global_flags(&mut cmd);

        if flip {
            flip_flags(&mut cmd);
        }

        push_args![cmd => ["-i", input]];

        cmd.args(output_flags);

        let video_format = VideoFormat::default();
        let output = output.with_extension(video_format.to_string());
        cmd.arg(output);

        println!("Encoding a single file upscaled");
        try_run_ffmpeg(dry_run, &mut [cmd], None)?;
    } else if input.is_dir() {
        let inputs_outputs_pair = get_inputs_and_outputs(&input, ENCODE_UPSCALE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let mut cmd = Command::new(FFMPEG);

            global_flags(&mut cmd);

            if flip {
                flip_flags(&mut cmd);
            }

            push_args![cmd => ["-i", input]];

            cmd.args(output_flags);

            let video_format = VideoFormat::default();
            let output = output.with_extension(video_format.to_string());
            cmd.arg(output);

            args.push(cmd);
        }

        println!("Encoding multiple files upscaled");
        try_run_ffmpeg(dry_run, &mut args, threads)?;
    }

    Ok(())
}

fn single_file(
    input: &Path,
    output: Option<PathBuf>,
    encode_opts: EncodeOpts,
    flip: bool,
) -> Result<Command> {
    let EncodeOpts {
        video_codec,
        audio_codec,
        video_format,
        quality,
    } = encode_opts;

    let mut cmd = Command::new(FFMPEG);

    global_flags(&mut cmd);

    if flip {
        flip_flags(&mut cmd);
    }

    push_args![cmd => ["-i", input]];

    video_flags(&mut cmd, video_codec, quality);
    audio_flags(&mut cmd, audio_codec);

    let output = get_output(input, output, ENCODE)?;
    let output = output.with_extension(video_format.to_string());
    cmd.arg(output);

    Ok(cmd)
}

fn multiple_file(
    input: &Path,
    encode_opts: EncodeOpts,
    command_str: &str,
    flip: bool,
) -> Result<Vec<Command>> {
    let EncodeOpts {
        video_codec,
        audio_codec,
        video_format,
        quality,
    } = encode_opts;

    let mut args = Vec::new();

    let inputs_ouputs_pair = get_inputs_and_outputs(input, command_str)?;

    for (input, output) in inputs_ouputs_pair {
        let mut cmd = Command::new(FFMPEG);

        global_flags(&mut cmd);

        if flip {
            flip_flags(&mut cmd);
        }

        push_args![cmd => ["-i", input]];

        video_flags(&mut cmd, video_codec.clone(), quality);
        audio_flags(&mut cmd, audio_codec.clone());

        let output = output.with_extension(video_format.to_string());
        cmd.arg(output);

        args.push(cmd);
    }

    Ok(args)
}
