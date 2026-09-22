use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use rayon::prelude::*;

fn ffmpeg<Iter>(args: Iter) -> Result<()>
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

pub fn try_run_ffmpeg(dry_run: bool, args: Vec<String>) -> Result<()> {
    if dry_run {
        let args = args.join(" ");
        println!("dry run... doing nothing.");
        println!("ffmpeg {args}");
        println!("------");
    } else {
        ffmpeg(args)?;
        println!("Finished!");
    }

    Ok(())
}

pub fn try_run_ffmpeg_par(dry_run: bool, args: Vec<Vec<String>>) {
    println!("{} files", args.len());
    if dry_run {
        println!("dry run...");
        for inner in args {
            let args = inner.join(" ");
            println!("ffmpeg {args}");
        }
        println!("------");
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
        println!("Finished!");
        println!("ffmpeg failed to encode {num_errors} files");
    }
}
