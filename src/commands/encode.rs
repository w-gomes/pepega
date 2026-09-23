use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::args::{EncodeOpt, VideoCodec, DEFAULT_CQ, DEFAULT_CRF};
use crate::ffmpeg::{try_run_ffmpeg, try_run_ffmpeg_par};
use crate::utils::{
    generate_flags_for_loglevel, generate_multiple_inputs_and_outputs, generate_output,
};
use crate::Config;

const ENCODE: &str = "ENCODE";
const ENCODE_YOUTUBE: &str = "ENCODE_YOUTUBE";
const ENCODE_UPSCALE: &str = "ENCODE_UPSCALE";

const DEFAULT_VIDEOCODEC: VideoCodec = VideoCodec::H264;

pub fn encode(config: Config, encode_opt: EncodeOpt, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
    } = config;

    if input.is_file() {
        let args = single_file(&input, output, Some(encode_opt), flip, verbose)?;
        println!("Encoding a single file");
        try_run_ffmpeg(dry_run, args)?;
    } else if input.is_dir() {
        let args = multiple_file(&input, Some(encode_opt), "ENCODE", flip, verbose)?;
        println!("Encoding multiple files");
        try_run_ffmpeg_par(dry_run, args)?;
    }

    Ok(())
}

pub fn encode_youtube(config: Config, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
    } = config;

    if input.is_file() {
        let output = output.clone().map_or_else(
            || generate_output(&input, ENCODE_YOUTUBE),
            |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
        )?;

        let mut args = Vec::new();
        args.extend(generate_flags_for_loglevel(verbose));
        args.extend(with_flip_or_default(&input, flip));
        args.extend_from_slice(&[
            "-c:v".to_string(),
            "libx264".to_string(),
            "-crf".to_string(),
            "18".to_string(),
            "-preset".to_string(),
            "ultrafast".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "384k".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
        ]);

        let output = output.with_extension("mp4");
        args.push(output.display().to_string());

        println!("Encoding a single file for youtube");
        try_run_ffmpeg(dry_run, args)?;
    } else if input.is_dir() {
        let inputs_outputs_pair = generate_multiple_inputs_and_outputs(&input, ENCODE_YOUTUBE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let mut inner_args = Vec::new();
            inner_args.extend(generate_flags_for_loglevel(verbose));
            inner_args.extend(with_flip_or_default(&input, flip));
            inner_args.extend_from_slice(&[
                "-c:v".to_string(),
                "libx264".to_string(),
                "-crf".to_string(),
                "18".to_string(),
                "-preset".to_string(),
                "ultrafast".to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                "384k".to_string(),
                "-pix_fmt".to_string(),
                "yuv420p".to_string(),
            ]);
            let output = output.with_extension("mp4");
            inner_args.push(output.display().to_string());
            args.push(inner_args);
        }

        println!("Encoding multiple files for youtube");
        try_run_ffmpeg_par(dry_run, args)?;
    }

    Ok(())
}

pub fn encode_upscale(config: Config, flip: bool) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
    } = config;

    if input.is_file() {
        let output = output.clone().map_or_else(
            || generate_output(&input, ENCODE_UPSCALE),
            |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
        )?;

        let mut args = Vec::new();
        args.extend(generate_flags_for_loglevel(verbose));
        args.extend(with_flip_or_default(&input, flip));
        args.extend_from_slice(&[
            "-vf".to_string(),
            "scale=iw*2:ih*2:flags=neighbor".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-crf".to_string(),
            "18".to_string(),
            "-preset".to_string(),
            "ultrafast".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "384k".to_string(),
        ]);

        let output = output.with_extension("mp4");
        args.push(output.display().to_string());

        println!("Encoding a single file upscaled");
        try_run_ffmpeg(dry_run, args)?;
    } else if input.is_dir() {
        let inputs_outputs_pair = generate_multiple_inputs_and_outputs(&input, ENCODE_UPSCALE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let mut inner_args = Vec::new();
            inner_args.extend(generate_flags_for_loglevel(verbose));
            inner_args.extend(with_flip_or_default(&input, flip));
            inner_args.extend_from_slice(&[
                "-vf".to_string(),
                "scale=iw*2:ih*2:flags=neighbor".to_string(),
                "-c:v".to_string(),
                "libx264".to_string(),
                "-crf".to_string(),
                "18".to_string(),
                "-preset".to_string(),
                "ultrafast".to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                "384k".to_string(),
            ]);
            let output = output.with_extension("mp4");
            inner_args.push(output.display().to_string());
            args.push(inner_args);
        }

        println!("Encoding multiple files upscaled");
        try_run_ffmpeg_par(dry_run, args)?;
    }

    Ok(())
}

fn single_file(
    input: &Path,
    output: Option<PathBuf>,
    encode_opt: Option<EncodeOpt>,
    flip: bool,
    verbose: bool,
) -> Result<Vec<String>> {
    let output = output.clone().map_or_else(
        || generate_output(input, ENCODE),
        |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
    )?;

    let encode_opt = encode_opt.unwrap_or_default();

    let mut args = Vec::new();
    args.extend(generate_flags_for_loglevel(verbose));
    args.extend(with_flip_or_default(input, flip));
    args.extend(encode_opt_to_vec(&encode_opt));
    args.extend_from_slice(&["-c:a".to_string(), encode_opt.audio_codec.to_string()]);

    let output = output.with_extension(encode_opt.video_format.to_string());
    args.push(output.display().to_string());

    Ok(args)
}

fn multiple_file(
    input: &Path,
    encode_opt: Option<EncodeOpt>,
    command_str: &str,
    flip: bool,
    verbose: bool,
) -> Result<Vec<Vec<String>>> {
    let mut args = Vec::new();

    let inputs_ouputs_pair = generate_multiple_inputs_and_outputs(input, command_str)?;
    let encode_opt = encode_opt.unwrap_or_default();

    for (input, output) in inputs_ouputs_pair {
        let mut inner_args = Vec::new();
        inner_args.extend(generate_flags_for_loglevel(verbose));
        inner_args.extend(with_flip_or_default(&input, flip));
        inner_args.extend(encode_opt_to_vec(&encode_opt));
        inner_args.extend_from_slice(&["-c:a".to_string(), encode_opt.audio_codec.to_string()]);

        let output = output.with_extension(encode_opt.video_format.to_string());
        inner_args.push(output.display().to_string());

        args.push(inner_args);
    }

    Ok(args)
}

pub fn encode_opt_to_vec(encode_opt: &EncodeOpt) -> Vec<String> {
    match encode_opt
        .video_codec
        .as_ref()
        .unwrap_or(&DEFAULT_VIDEOCODEC)
    {
        ref enc @ (VideoCodec::Av1 | VideoCodec::Hevc) => {
            vec![
                "-c:v".to_string(),
                enc.to_string(),
                "-cq".to_string(),
                encode_opt.cq.unwrap_or(DEFAULT_CQ).to_string(),
                "-preset".to_string(),
                "p1".to_string(),
            ]
        }
        ref enc @ (VideoCodec::H264 | VideoCodec::H265) => {
            vec![
                "-c:v".to_string(),
                enc.to_string(),
                "-crf".to_string(),
                encode_opt.crf.unwrap_or(DEFAULT_CRF).to_string(),
                "-preset".to_string(),
                "ultrafast".to_string(),
            ]
        }
    }
}

fn with_flip_or_default(input: &Path, flip: bool) -> Vec<String> {
    if flip {
        vec![
            "-display_rotation:v:0".to_string(),
            "-90.0".to_string(),
            "-i".to_string(),
            input.display().to_string(),
        ]
    } else {
        vec!["-i".to_string(), input.display().to_string()]
    }
}
