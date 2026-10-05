use anyhow::{Result, ensure};

use crate::args::{Config, EncodeOpts, VideoFormat};
use crate::commands::{audio_flag, loglevel_flag, video_flag};
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
        verbose,
        ..
    } = config;

    ensure!(input.is_file(), "input must be a file.");

    let cmd_str = if encode_opts.is_some() {
        CLIP
    } else {
        CLIP_COPY
    };

    let output = get_output(&input, output, cmd_str)?;

    let mut args = Vec::new();
    args.extend(loglevel_flag(verbose));

    if let Some(encode_opts) = encode_opts {
        let EncodeOpts {
            video_codec,
            audio_codec,
            video_format,
            quality,
        } = encode_opts;

        let output = output.with_extension(video_format.to_string());
        let input_opt = vec![
            "-ss".to_string(),
            start.to_string(),
            "-accurate_seek".to_string(),
            "-i".to_string(),
            input.display().to_string(),
            "-to".to_string(),
            end.to_string(),
        ];
        args.extend(input_opt);
        args.extend(video_flag(video_codec, quality));
        args.extend(audio_flag(audio_codec));
        args.push(output.display().to_string());
    } else {
        let video_format = VideoFormat::default();
        let output = output.with_extension(video_format.to_string());
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
    }

    println!("Clipping a video");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}

pub fn clip_gif(config: Config, start: &str, end: &str) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    ensure!(input.is_file(), "input must be a file.");

    let output = get_output(&input, output, CLIP_GIF)?;

    let mut args = Vec::new();

    args.extend(loglevel_flag(verbose));
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

    println!("Clipping a video and saving as gif");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
