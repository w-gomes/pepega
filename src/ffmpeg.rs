use std::process::{Command, Stdio};

use anyhow::{bail, Result};

pub fn ffmpeg<'a, Iter>(args: Iter) -> Result<()>
where
    Iter: std::iter::IntoIterator<Item = String>,
{
    let ffmpeg = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;

    let output = ffmpeg.wait_with_output()?;
    if !output.status.success() {
        bail!(
            "-- Failed to execute FFmpeg. --\n-- Error code: --\n{:?}",
            output.status.code()
        );
    }

    Ok(())
}
