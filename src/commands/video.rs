use anyhow::{anyhow, bail, Result};

use std::path::{Path, PathBuf};

use crate::ffmpeg::ffmpeg;
use crate::utils::{generate_output_with, temp_list_for_video};

// TODO: -tune stillimage

const IMAGES_TO_VIDEO: &str = "IMAGES_TO_VIDEO";

pub fn video(dry_run: bool, input: &Path, output: Option<PathBuf>, framerate: u64) -> Result<()> {
    if !input.is_dir() {
        bail!("Input must be a directory");
    }

    let output = output.clone().map_or_else(
        || generate_output_with(input, IMAGES_TO_VIDEO),
        |_| output.ok_or_else(|| anyhow!("Unable to get the output file")),
    )?;

    let mut args = Vec::new();

    let (temp_file, total_images) = temp_list_for_video(input, framerate)?;
    let temp_file = temp_file.path().display().to_string();
    args.extend_from_slice(&[
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        temp_file,
    ]);

    args.extend_from_slice(&[
        "-tune".to_string(),
        "stillimage".to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-r".to_string(),
        "30".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ]);
    let output = output.with_extension("mp4");
    args.push(output.display().to_string());

    println!("Creating a video from {total_images} images.");
    if dry_run {
        let args = args.join(" ");
        println!("ffmpeg {args}");
    } else {
        ffmpeg(args)?;
    }

    Ok(())
}
