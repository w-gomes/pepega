use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

pub fn ffmpeg<Iter>(args: Iter) -> Result<()>
where
    Iter: std::iter::IntoIterator<Item = String>,
{
    let ffmpeg = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?;

    let output = ffmpeg.wait_with_output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "-- Failed to execute FFmpeg.\n\t[ Error code: {:?} ]",
            output.status.code()
        ));
    }

    Ok(())
}
