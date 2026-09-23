use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use indicatif::{HumanDuration, ParallelProgressIterator, ProgressBar, ProgressStyle};
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
        let started = Instant::now();
        let spinner = ProgressBar::new_spinner();
        let style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?;
        spinner.set_style(style);
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message("Waiting...");
        ffmpeg(args)?;
        spinner.finish_with_message("Finished!");
        println!("Done in {}", HumanDuration(started.elapsed()));
    }

    Ok(())
}

pub fn try_run_ffmpeg_par(dry_run: bool, args: Vec<Vec<String>>) -> Result<()> {
    println!("{} files", args.len());
    if dry_run {
        println!("dry run...");
        for inner in args {
            let args = inner.join(" ");
            println!("ffmpeg {args}");
        }
        println!("------");
    } else {
        let started = Instant::now();
        let bar = ProgressBar::new(args.len() as u64);
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .with_context(|| anyhow!("Failed to create ProgressStyle"))?
            .progress_chars("#>-");
        bar.set_style(style);
        bar.enable_steady_tick(Duration::from_millis(100));

        let results = args
            .into_par_iter()
            .progress_with(bar.clone())
            .map(ffmpeg)
            .collect::<Vec<Result<()>>>();
        bar.finish_with_message("Finished!");

        let mut num_errors = 0;
        for result in results {
            if let Err(error) = result {
                num_errors += 1;
                eprintln!("{error:#}");
            }
        }

        println!("ffmpeg failed to encode {num_errors} files");
        println!("Done in {}", HumanDuration(started.elapsed()));
    }

    Ok(())
}
