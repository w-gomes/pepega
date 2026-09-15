use std::process::{Command, Stdio};

use anyhow::{bail, Result};

pub trait FFmpeg {
    fn args(&self) -> &[String];

    fn ffmpeg(&self) -> Result<()> {
        let ffmpeg = Command::new("ffmpeg")
            .args(self.args())
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

    fn dry(&self) {
        println!("---\nflag --dry=true printing args only");
        println!("{:?}\n---", self.args());
    }
}
