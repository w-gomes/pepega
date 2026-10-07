use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use indicatif::{HumanDuration, ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

const DEFAULT_NUM_THREADS: usize = 1;

pub fn try_run_ffmpeg(dry_run: bool, cmd: Command) -> Result<()> {
    if dry_run {
        let args = cmd.get_args().collect::<Vec<_>>();
        println!("dry run... doing nothing.");
        println!("ffmpeg {args:?}");
        println!("------");
    } else {
        let started = Instant::now();

        let style = ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .with_context(|| anyhow!("Failed to create ProgressStyle"))?;

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(style);
        spinner.enable_steady_tick(Duration::from_millis(200));
        spinner.set_message("Waiting...");

        run_ffmpeg(cmd)?;

        spinner.finish_and_clear();
        println!("Done in {}", HumanDuration(started.elapsed()));
    }

    Ok(())
}

pub fn try_run_ffmpeg_par(dry_run: bool, cmds: Vec<Command>, threads: Option<usize>) -> Result<()> {
    println!("{} files", cmds.len());
    if dry_run {
        println!("dry run... doing nothing.");
        for cmd in cmds {
            let args = cmd.get_args().collect::<Vec<_>>();
            println!("ffmpeg {args:?}");
        }
        println!("------");
    } else {
        let threads = threads.unwrap_or(DEFAULT_NUM_THREADS);
        if threads > num_cpus::get() {
            println!(
                "Number of threads chosen ({}) is greater than available threads ({}).",
                threads,
                num_cpus::get()
            );
            println!("Using the default config for number of threads ({DEFAULT_NUM_THREADS})");
        }
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .with_context(|| "failed to create a thread pool".to_string())?;

        let started = Instant::now();

        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .with_context(|| anyhow!("failed to create ProgressStyle"))?
            .progress_chars("#>-");

        let bar = ProgressBar::new(cmds.len() as u64);
        bar.set_style(style);
        bar.enable_steady_tick(Duration::from_millis(100));

        let results = cmds
            .into_par_iter()
            .progress_with(bar.clone())
            .map(run_ffmpeg)
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

fn run_ffmpeg(mut cmd: Command) -> Result<()> {
    let cmd = cmd.stderr(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    let result = cmd.wait_with_output()?;
    if !result.status.success() {
        let error_msg = String::from_utf8(result.stderr)?;
        return Err(anyhow!(
            "failed to execute FFmpeg!\nFFmpeg Error: {error_msg:?}"
        ));
    }

    Ok(())
}
