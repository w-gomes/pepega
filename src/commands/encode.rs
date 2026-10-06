use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::args::{Config, EncodeOpts, VideoFormat};
use crate::commands::{audio_flag, flip_flag, loglevel_flag, video_flag};
use crate::ffmpeg::{try_run_ffmpeg, try_run_ffmpeg_par};
use crate::utils::{get_inputs_and_outputs, get_output};

const ENCODE: &str = "ENCODE";
const ENCODE_YOUTUBE: &str = "ENCODE_YOUTUBE";
const ENCODE_UPSCALE: &str = "ENCODE_UPSCALE";

pub fn encode(config: Config, encode_opts: EncodeOpts, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        threads,
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    if input.is_file() {
        let args = single_file(&input, output, encode_opts, flip, verbose)?;
        println!("Encoding a single file");
        try_run_ffmpeg(dry_run, args)?;
    } else if input.is_dir() {
        let args = multiple_file(&input, encode_opts, ENCODE, flip, verbose)?;
        println!("Encoding multiple files");
        try_run_ffmpeg_par(dry_run, args, threads)?;
    }

    Ok(())
}

pub fn encode_youtube(config: Config, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        threads,
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    let output_flags = vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-crf".to_string(),
        "18".to_string(),
        "-preset".to_string(),
        "medium".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "384k".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ];

    if input.is_file() {
        let output = get_output(&input, output, ENCODE_YOUTUBE)?;
        let video_format = VideoFormat::default();
        let output = output.with_extension(video_format.to_string());

        let mut args = Vec::new();
        args.extend(loglevel_flag(verbose));
        args.extend(flip_flag(&input, flip));
        args.extend(output_flags);
        args.push(output.display().to_string());

        println!("Encoding a single file for youtube");
        try_run_ffmpeg(dry_run, args)?;
    } else if input.is_dir() {
        let inputs_outputs_pair = get_inputs_and_outputs(&input, ENCODE_YOUTUBE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let video_format = VideoFormat::default();
            let output = output.with_extension(video_format.to_string());

            let mut inner_args = Vec::new();
            inner_args.extend(loglevel_flag(verbose));
            inner_args.extend(flip_flag(&input, flip));
            inner_args.extend(output_flags.clone());
            inner_args.push(output.display().to_string());
            args.push(inner_args);
        }

        println!("Encoding multiple files for youtube");
        try_run_ffmpeg_par(dry_run, args, threads)?;
    }

    Ok(())
}

pub fn encode_upscale(config: Config, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        threads,
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    let output_flags = vec![
        "-vf".to_string(),
        "scale=iw*2:ih*2:flags=neighbor".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-crf".to_string(),
        "18".to_string(),
        "-preset".to_string(),
        "slow".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "384k".to_string(),
    ];

    if input.is_file() {
        let output = get_output(&input, output, ENCODE_UPSCALE)?;
        let video_format = VideoFormat::default();
        let output = output.with_extension(video_format.to_string());

        let mut args = Vec::new();
        args.extend(loglevel_flag(verbose));
        args.extend(flip_flag(&input, flip));
        args.extend(output_flags);
        args.push(output.display().to_string());

        println!("Encoding a single file upscaled");
        try_run_ffmpeg(dry_run, args)?;
    } else if input.is_dir() {
        let inputs_outputs_pair = get_inputs_and_outputs(&input, ENCODE_UPSCALE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let video_format = VideoFormat::default();
            let output = output.with_extension(video_format.to_string());

            let mut inner_args = Vec::new();
            inner_args.extend(loglevel_flag(verbose));
            inner_args.extend(flip_flag(&input, flip));
            inner_args.extend(output_flags.clone());

            inner_args.push(output.display().to_string());
            args.push(inner_args);
        }

        println!("Encoding multiple files upscaled");
        try_run_ffmpeg_par(dry_run, args, threads)?;
    }

    Ok(())
}

fn single_file(
    input: &Path,
    output: Option<PathBuf>,
    encode_opts: EncodeOpts,
    flip: bool,
    verbose: bool,
) -> Result<Vec<String>> {
    let EncodeOpts {
        video_codec,
        audio_codec,
        video_format,
        quality,
    } = encode_opts;

    let output = get_output(input, output, ENCODE)?;
    let output = output.with_extension(video_format.to_string());

    let mut args = Vec::new();
    args.extend(loglevel_flag(verbose));
    args.extend(flip_flag(input, flip));
    args.extend(video_flag(video_codec, quality));
    args.extend(audio_flag(audio_codec));

    args.push(output.display().to_string());

    Ok(args)
}

fn multiple_file(
    input: &Path,
    encode_opts: EncodeOpts,
    command_str: &str,
    flip: bool,
    verbose: bool,
) -> Result<Vec<Vec<String>>> {
    let EncodeOpts {
        video_codec,
        audio_codec,
        video_format,
        quality,
    } = encode_opts;

    let mut args = Vec::new();

    let inputs_ouputs_pair = get_inputs_and_outputs(input, command_str)?;

    for (input, output) in inputs_ouputs_pair {
        let output = output.with_extension(video_format.to_string());

        let mut inner_args = Vec::new();
        inner_args.extend(loglevel_flag(verbose));
        inner_args.extend(flip_flag(&input, flip));
        inner_args.extend(video_flag(video_codec.clone(), quality));
        inner_args.extend(audio_flag(audio_codec.clone()));
        inner_args.push(output.display().to_string());

        args.push(inner_args);
    }

    Ok(args)
}
