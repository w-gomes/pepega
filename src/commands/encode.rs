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
        let output = output.clone().map_or_else(
            || generate_output_with(input, ENCODE),
            |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
        )?;

        println!("Running encode on a single file");
        let args = single_file(input, &output, encode_opt, flip);

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
    _dry_run: bool,
    _input: &Path,
    _output: Option<PathBuf>,
    _flip: bool,
) -> Result<()> {
    Ok(())
}

pub fn encode_upscale(
    _dry_run: bool,
    _input: &Path,
    _output: Option<PathBuf>,
    _flip: bool,
) -> Result<()> {
    Ok(())
}

fn single_file(input: &Path, output: &Path, encode_opt: &EncodeOpt, flip: bool) -> Vec<String> {
    let mut args = Vec::new();

    let output = output.with_extension(encode_opt.video_format.to_string());

    let encoders = match encode_opt.video_codec {
        ref enc @ (VideoCodec::Av1 | VideoCodec::H265 | VideoCodec::Hevc) => {
            vec![
                enc.to_string(),
                "-cq".to_string(),
                encode_opt.cq.to_string(),
                "-preset".to_string(),
                "p1".to_string(),
            ]
        }
        ref enc @ VideoCodec::H264 => {
            vec![
                enc.to_string(),
                "-crf".to_string(),
                encode_opt.crf.to_string(),
                "-preset".to_string(),
                "ultrafast".to_string(),
            ]
        }
    };

    if flip {
        args.extend_from_slice(&[
            "-display_rotation:v:0".to_string(),
            "-90.0".to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-c:v".to_string(),
        ]);
    } else {
        args.extend_from_slice(&[
            "-i".to_string(),
            input.display().to_string(),
            "-c:v".to_string(),
        ]);
    }

    args.extend(encoders);
    args.push(output.display().to_string());

    args
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
        let output = output.with_extension(encode_opt.video_format.to_string());

        let encoders = match encode_opt.video_codec {
            ref enc @ (VideoCodec::Av1 | VideoCodec::H265 | VideoCodec::Hevc) => {
                vec![
                    enc.to_string(),
                    "-cq".to_string(),
                    encode_opt.cq.to_string(),
                    "-preset".to_string(),
                    "p1".to_string(),
                ]
            }
            ref enc @ VideoCodec::H264 => {
                vec![
                    enc.to_string(),
                    "-crf".to_string(),
                    encode_opt.crf.to_string(),
                    "-preset".to_string(),
                    "ultrafast".to_string(),
                ]
            }
        };

        if flip {
            inner_args.extend_from_slice(&[
                "-display_rotation:v:0".to_string(),
                "-90.0".to_string(),
                "-i".to_string(),
                input.display().to_string(),
                "-c:v".to_string(),
            ]);
        } else {
            inner_args.extend_from_slice(&[
                "-i".to_string(),
                input.display().to_string(),
                "-c:v".to_string(),
            ]);
        }

        inner_args.extend(encoders);
        inner_args.push(output.display().to_string());

        args.push(inner_args);
    }

    Ok(args)
}
