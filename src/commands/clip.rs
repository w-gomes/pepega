use anyhow::{Result, bail};

use crate::args::{Config, EncodeOpt, VideoCodec};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{get_or_generate_output, set_flags_for_loglevel};

const CLIP: &str = "CLIP";
const CLIP_GIF: &str = "CLIP_GIF";

const DEFAULT_VIDEOCODEC: VideoCodec = VideoCodec::H264;

pub fn clip(config: Config, start: &str, end: &str, encode_opt: Option<EncodeOpt>) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = get_or_generate_output(&input, output, CLIP)?;

    let mut args = Vec::new();
    args.extend(set_flags_for_loglevel(verbose));

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
            encode_opt
                .video_codec
                .unwrap_or(DEFAULT_VIDEOCODEC)
                .to_string(),
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

    if !input.is_file() {
        bail!("Input must be a file");
    }

    let output = get_or_generate_output(&input, output, CLIP_GIF)?;

    let mut args = Vec::new();

    args.extend(set_flags_for_loglevel(verbose));
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
