use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use rayon::prelude::*;

use crate::args::{EncodeOpt, VideoCodec};
use crate::ffmpeg::ffmpeg;
use crate::utils::{generate_multiple_inputs_and_outputs, generate_output_with};

const ENCODE: &str = "ENCODE";
const ENCODE_YOUTUBE: &str = "ENCODE_YOUTUBE";
const ENCODE_UPSCALE: &str = "ENCODE_UPSCALE";

pub fn encode(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    encode_opt: &EncodeOpt,
    flip: bool,
) -> Result<()> {
    if input.is_file() {
        println!("Running encode on a single file");
        let args = single_file(input, output, encode_opt, flip)?;

        if dry_run {
            let args = args.join(" ");
            println!("ffmpeg {args}");
        } else {
            ffmpeg(args)?;
        }
    } else if input.is_dir() {
        println!("Running encode on multiple files");
        let args = multiple_file(input, encode_opt, "ENCODE", flip)?;
        println!("{} files", args.len());

        if dry_run {
            for inner in args {
                let args = inner.join(" ");
                println!("ffmpeg {args}");
            }
        } else {
            let mut num_errors = 0;
            let results = args
                .into_par_iter()
                .map(ffmpeg)
                .collect::<Vec<Result<()>>>();

            for result in results {
                if let Err(error) = result {
                    num_errors += 1;
                    eprintln!("{error:#}");
                }
            }
            println!("ffmpeg failed to encode {num_errors} files");
        }
    }

    Ok(())
}

pub fn encode_youtube(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    flip: bool,
) -> Result<()> {
    if input.is_file() {
        println!("Running encode on a single file");

        let output = output.clone().map_or_else(
            || generate_output_with(input, ENCODE_YOUTUBE),
            |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
        )?;

        let mut args = Vec::new();
        args.extend(with_flip_or_default(input, flip));
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

        if dry_run {
            let args = args.join(" ");
            println!("ffmpeg {args}");
        } else {
            ffmpeg(args)?;
        }
    } else if input.is_dir() {
        println!("Running encode on multiple files");

        let inputs_outputs_pair = generate_multiple_inputs_and_outputs(input, ENCODE_YOUTUBE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let mut inner_args = Vec::new();
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

        println!("{} files", args.len());

        if dry_run {
            for inner in args {
                let args = inner.join(" ");
                println!("ffmpeg {args}");
            }
        } else {
            let mut num_errors = 0;
            let results = args
                .into_par_iter()
                .map(ffmpeg)
                .collect::<Vec<Result<()>>>();

            for result in results {
                if let Err(error) = result {
                    num_errors += 1;
                    eprintln!("{error:#}");
                }
            }
            println!("ffmpeg failed to encode {num_errors} files");
        }
    }
    Ok(())
}

pub fn encode_upscale(
    dry_run: bool,
    input: &Path,
    output: Option<PathBuf>,
    flip: bool,
) -> Result<()> {
    if input.is_file() {
        println!("Running encode on a single file");

        let output = output.clone().map_or_else(
            || generate_output_with(input, ENCODE_UPSCALE),
            |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
        )?;

        let mut args = Vec::new();
        args.extend(with_flip_or_default(input, flip));
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

        if dry_run {
            let args = args.join(" ");
            println!("ffmpeg {args}");
        } else {
            ffmpeg(args)?;
        }
    } else if input.is_dir() {
        println!("Running encode on multiple files");

        let inputs_outputs_pair = generate_multiple_inputs_and_outputs(input, ENCODE_UPSCALE)?;

        let mut args = Vec::new();
        for (input, output) in inputs_outputs_pair {
            let mut inner_args = Vec::new();
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

        println!("{} files", args.len());

        if dry_run {
            for inner in args {
                let args = inner.join(" ");
                println!("ffmpeg {args}");
            }
        } else {
            let mut num_errors = 0;
            let results = args
                .into_par_iter()
                .map(ffmpeg)
                .collect::<Vec<Result<()>>>();

            for result in results {
                if let Err(error) = result {
                    num_errors += 1;
                    eprintln!("{error:#}");
                }
            }
            println!("ffmpeg failed to encode {num_errors} files");
        }
    }
    Ok(())
}

fn single_file(
    input: &Path,
    output: Option<PathBuf>,
    encode_opt: &EncodeOpt,
    flip: bool,
) -> Result<Vec<String>> {
    let output = output.clone().map_or_else(
        || generate_output_with(input, ENCODE),
        |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();

    args.extend(with_flip_or_default(input, flip));
    args.extend(encode_opt_to_vec(encode_opt));

    let output = output.with_extension(encode_opt.video_format.to_string());
    args.push(output.display().to_string());

    Ok(args)
}

fn multiple_file(
    input: &Path,
    encode_opt: &EncodeOpt,
    command_str: &str,
    flip: bool,
) -> Result<Vec<Vec<String>>> {
    let mut args = Vec::new();

    let inputs_ouputs_pair = generate_multiple_inputs_and_outputs(input, command_str)?;

    for (input, output) in inputs_ouputs_pair {
        let mut inner_args = Vec::new();

        inner_args.extend(with_flip_or_default(&input, flip));
        inner_args.extend(encode_opt_to_vec(&encode_opt));

        let output = output.with_extension(encode_opt.video_format.to_string());
        inner_args.push(output.display().to_string());

        args.push(inner_args);
    }

    Ok(args)
}

pub fn encode_opt_to_vec(encode_opt: &EncodeOpt) -> Vec<String> {
    match encode_opt.video_codec {
        ref enc @ (VideoCodec::Av1 | VideoCodec::Hevc) => {
            vec![
                enc.to_string(),
                "-cq".to_string(),
                encode_opt.cq.unwrap_or(19).to_string(),
                "-preset".to_string(),
                "p1".to_string(),
            ]
        }
        ref enc @ (VideoCodec::H264 | VideoCodec::H265) => {
            vec![
                enc.to_string(),
                "-crf".to_string(),
                encode_opt.crf.unwrap_or(23).to_string(),
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
            "-c:v".to_string(),
        ]
    } else {
        vec![
            "-i".to_string(),
            input.display().to_string(),
            "-c:v".to_string(),
        ]
    }
}
