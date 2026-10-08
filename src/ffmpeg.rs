use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use indicatif::{
    HumanDuration, MultiProgress, ParallelProgressIterator, ProgressBar, ProgressStyle,
};
use rayon::prelude::*;

const DEFAULT_NUM_THREADS: usize = 1;

pub fn try_run_ffmpeg(dry_run: bool, cmds: &mut [Command], threads: Option<usize>) -> Result<()> {
    println!("{} files", cmds.len());
    if dry_run {
        println!("------");
        println!("dry run... doing nothing.");
        for cmd in cmds {
            let args = cmd
                .get_args()
                .map(|flags| flags.to_str().unwrap_or_default())
                .collect::<Vec<_>>()
                .join(" ");
            println!("ffmpeg {args}");
        }
        println!("------");
    } else {
        let threads = threads.unwrap_or(DEFAULT_NUM_THREADS);
        let num_of_cpus_available = num_cpus::get();
        if threads > num_of_cpus_available {
            println!(
                "Number of threads chosen ({threads}) is greater than available threads ({num_of_cpus_available}).",
            );
            println!("Using the default config for number of threads ({DEFAULT_NUM_THREADS})");
        }
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .with_context(|| "failed to create a thread pool".to_string())?;

        let started = Instant::now();

        let parent_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len}")
            .with_context(|| anyhow!("failed to create total_style"))?;

        let multi = MultiProgress::new();

        let parent_bar = ProgressBar::new(cmds.len() as u64);
        parent_bar.set_style(parent_style);
        parent_bar.enable_steady_tick(Duration::from_millis(100));

        let parent_bar = multi.add(parent_bar);

        let results = cmds
            .into_par_iter()
            .map(|cmd| run_ffmpeg(cmd, &multi))
            .progress_with(parent_bar.clone())
            .collect::<Vec<Result<()>>>();

        let error_count = results
            .into_iter()
            .filter_map(Result::err)
            .inspect(|e| eprintln!("Error: {e}"))
            .count();

        parent_bar.finish();
        multi.println(format!("All done in {}", HumanDuration(started.elapsed())))?;
        multi.println(format!("FFmpeg failed to encode {error_count} files"))?;
    }

    Ok(())
}

fn run_ffmpeg(cmd: &mut Command, multi: &MultiProgress) -> Result<()> {
    let prefix = cmd
        .get_args()
        .last()
        .map_or_default(|last| last.to_string_lossy().into_owned());

    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {wide_msg}")
        .with_context(|| anyhow!("failed to create style in bar_style"))?
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈");

    let spinner = multi.add(ProgressBar::new_spinner().with_style(spinner_style));
    spinner.enable_steady_tick(Duration::from_millis(100));

    let started = Instant::now();

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to get stdout"))?;

    ffmpeg_stdout(&spinner, stdout)?;

    let status = child.wait()?;

    spinner.finish();

    if !status.success() {
        multi.println(format!("{prefix} failed ({status})"))?;
        return Err(anyhow!("failed to execute FFmpeg!\n"));
    }

    multi.println(format!(
        "{prefix} done in {}",
        HumanDuration(started.elapsed())
    ))?;

    Ok(())
}

fn ffmpeg_stdout(spinner: &ProgressBar, stdout: impl Read) -> Result<()> {
    let reader = BufReader::new(stdout);

    let mut frame_buf = String::with_capacity(16);
    let mut fps_buf = String::with_capacity(16);
    let mut bitrate_buf = String::with_capacity(16);
    let mut total_size_buf = String::with_capacity(16);
    let mut time_buf = String::with_capacity(16);

    for line in reader.lines() {
        let line = line?;
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "frame" => value.trim().clone_into(&mut frame_buf),
            "fps" => value.trim().clone_into(&mut fps_buf),
            "bitrate" => value.trim().clone_into(&mut bitrate_buf),
            "total_size" => value.trim().clone_into(&mut total_size_buf),
            "out_time" => value.trim().clone_into(&mut time_buf),
            "progress" => {
                let message = format!(
                    "frame={frame_buf} fps={fps_buf} total_size={total_size_buf} bitrate={bitrate_buf} time={time_buf}"
                );
                spinner.set_message(message);
                if value == "end" {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
