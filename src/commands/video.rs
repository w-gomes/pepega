use anyhow::{Result, bail};

use crate::args::Config;
use crate::ffmpeg::try_run_ffmpeg;
use crate::utils::{get_or_generate_output, set_flags_for_loglevel, temp_list_for_video};

const IMAGES_TO_VIDEO: &str = "IMAGES_TO_VIDEO";

pub fn video(config: Config, framerate: u64) -> Result<()> {
    let Config {
        input,
        output,
        dry_run,
        verbose,
        ..
    } = config;

    if !input.is_dir() {
        bail!("Input must be a directory");
    }

    let output = get_or_generate_output(&input, output, IMAGES_TO_VIDEO)?;

    let mut args = Vec::new();
    args.extend(set_flags_for_loglevel(verbose));

    let (temp_file, total_images) = temp_list_for_video(&input, framerate)?;
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
    try_run_ffmpeg(dry_run, args)?;

    Ok(())
}
