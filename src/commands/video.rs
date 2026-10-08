use std::process::Command;

use anyhow::{Result, bail, ensure};

use crate::args::{Config, VideoFormat};
use crate::commands::{FFMPEG, global_flags, push_args};
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{get_output, temp_list_for_video};

const IMAGES_TO_VIDEO: &str = "IMAGES_TO_VIDEO";

pub fn video(config: Config, framerate: u64) -> Result<()> {
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

    ensure!(input.is_dir(), "input must be a directory.");

    let mut cmd = Command::new(FFMPEG);

    let output = get_output(&input, output, IMAGES_TO_VIDEO)?;
    let video_format = VideoFormat::default();
    let output = output.with_extension(video_format.to_string());

    let (temp_file, total_images) = temp_list_for_video(&input, framerate)?;
    let input = temp_file.path();

    global_flags(&mut cmd);
    push_args![cmd => [
        "-f", "concat",
        "-safe", 0.to_string(),
        "-i", input,
        "-tune", "stillimage",
        "-c:v", "libx264",
        "-r", 30.to_string(),
        "-pix_fmt", "yuv420p",
        output,
    ]];

    println!("Creating a video from {total_images} images.");
    try_run_ffmpeg(dry_run, &mut [cmd], None)?;

    Ok(())
}
