use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use indicatif::{HumanDuration, ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

fn ffmpeg<Iter>(args: Iter) -> Result<()>
where
    Iter: std::iter::IntoIterator<Item = String>,
{
    let ffmpeg = Command::new("ffmpeg")
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let result = ffmpeg.wait_with_output()?;
    if !result.status.success() {
        return Err(anyhow!(
            "-- Failed to execute FFmpeg.\n\t[ Error code: {:?} ]",
            result.status.code()
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

        let style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .with_context(|| anyhow!("Failed to create ProgressStyle"))?;

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(style);
        spinner.enable_steady_tick(Duration::from_millis(200));
        spinner.set_message("Waiting...");

        ffmpeg(args)?;

        spinner.finish_and_clear();
        println!("Done in {}", HumanDuration(started.elapsed()));
    }

    Ok(())
}

pub fn try_run_ffmpeg_par(dry_run: bool, args: Vec<Vec<String>>) -> Result<()> {
    println!("{} files", args.len());
    if dry_run {
        println!("dry run... doing nothing.");
        for inner in args {
            let args = inner.join(" ");
            println!("ffmpeg {args}");
        }
        println!("------");
    } else {
        let started = Instant::now();

        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .with_context(|| anyhow!("Failed to create ProgressStyle"))?
            .progress_chars("#>-");

        let bar = ProgressBar::new(args.len() as u64);
        bar.set_style(style);
        bar.enable_steady_tick(Duration::from_millis(100));

        let results = args
            .into_par_iter()
            .progress_with(bar.clone())
            .map(ffmpeg)
            .collect::<Vec<Result<()>>>();

        let error_count = results
            .into_iter()
            .filter_map(Result::err)
            .inspect(|e| eprintln!("Error: {e}"))
            .count();

        bar.finish_and_clear();
        println!("ffmpeg failed to encode {error_count} files");
        println!("Done in {}", HumanDuration(started.elapsed()));
    }

    Ok(())
}
