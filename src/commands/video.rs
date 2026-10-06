use anyhow::{Result, bail, ensure};

use crate::args::{Config, VideoFormat};
use crate::commands::loglevel_flag;
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{get_output, temp_list_for_video};

const IMAGES_TO_VIDEO: &str = "IMAGES_TO_VIDEO";

pub fn video(config: Config, framerate: u64) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    if let Ok(exists) = input.try_exists()
        && !exists
    {
        bail!("input doesn't exist.");
    }

    ensure!(input.is_dir(), "input must be a directory.");

    let output = get_output(&input, output, IMAGES_TO_VIDEO)?;
    let video_format = VideoFormat::default();
    let output = output.with_extension(video_format.to_string());

    let (temp_file, total_images) = temp_list_for_video(&input, framerate)?;
    let temp_file = temp_file.path().display().to_string();

    let mut args = Vec::new();
    args.extend(loglevel_flag(verbose));
    args.extend_from_slice(&[
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        temp_file,
        "-tune".to_string(),
        "stillimage".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-r".to_string(),
        "30".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ]);
    args.push(output.display().to_string());

    println!("Creating a video from {total_images} images.");
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
