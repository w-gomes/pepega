use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use chrono::Local;
use tempfile::{Builder, NamedTempFile};
use walkdir::{DirEntry, WalkDir};

// Create a list of pair: -i, input and the filters for it
pub fn generate_inputs_and_filters(dir: &Path) -> Result<(Vec<String>, String)> {
    let mut inputs = Vec::new();
    let mut filters = String::new();

    for (idx, entry) in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "mkv" || ext == "mp4")
        })
        .enumerate()
    {
        let input = entry.path();
        inputs.extend(vec!["-i".to_string(), input.display().to_string()]);
        filters.push_str(format!("[{idx}:v:0][{idx}:a:0]").as_str());
    }

    debug_assert_eq!(inputs.len() % 2, 0);
    // the real length is inputs.len() / 2 because of the pair: `-i <INPUT>`
    let len = inputs.len() / 2;
    if (len) < 2 {
        return Err(anyhow!(
            "Not enough video file to merge. {} contains: {}",
            dir.display(),
            len
        ));
    }

    Ok((inputs, filters))
}

// Create a temporary file with the list of images for ffmpeg to read from
pub fn temp_list_for_video(dir: &Path, framerate: u64) -> Result<(NamedTempFile, usize)> {
    let mut temp_file = Builder::new()
        .prefix("temp-file")
        .suffix(".txt")
        .tempfile()?;

    let mut total_images = 0;

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "png" || ext == "jpg")
        })
    {
        let entry = entry.path();
        let entry = fs::canonicalize(entry)
            .with_context(|| format!("Failed to get the absolute path of {}", entry.display()))?;
        println!("{}", entry.display());
        writeln!(temp_file, "file '{}'", entry.display())?;
        writeln!(temp_file, "duration {framerate}")?;
        total_images += 1;
    }

    Ok((temp_file, total_images))
}

// Generate an output name
pub fn generate_output(path: &Path, command_str: &str) -> Result<PathBuf> {
    let original_input = path.to_path_buf();
    let Some(file_name) = original_input.file_name() else {
        return Err(anyhow!("Error extracting file name from Input"));
    };

    // Convert OsStr to &str to pass to format!()
    let Some(file_name) = file_name.to_str() else {
        return Err(anyhow!("Error converting OsStr to &str"));
    };

    let now = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let file_name = format!("{command_str}_{timestamp}_{file_name}");

    Ok(Path::new(&original_input).with_file_name(file_name))
}

// Generate outputs from inputs in a directory
pub fn generate_multiple_inputs_and_outputs(
    dir: &Path,
    command_str: &str,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut input_output_pair = Vec::new();
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "mkv" || ext == "mp4")
        })
    {
        let input = entry.path();
        let output = generate_output(input, command_str)?;

        input_output_pair.push((input.to_path_buf(), output));
    }

    Ok(input_output_pair)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}

pub fn set_flags_for_loglevel(verbose: bool) -> Vec<String> {
    if verbose {
        vec!["-loglevel".to_string(), "info".to_string()]
    } else {
        vec!["-loglevel".to_string(), "error".to_string()]
    }
}
