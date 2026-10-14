use std::path::{Path, PathBuf};
use std::{fs, io::Write};

use anyhow::{Context, Result, anyhow, ensure};
use chrono::Local;
use tempfile::{Builder, NamedTempFile};
use walkdir::{DirEntry, WalkDir};

// Create a list of (-i, input) and the output's filters
pub fn generate_inputs_and_filters(dir: &Path) -> Result<(Vec<String>, String)> {
    let mut inputs = Vec::new();
    let mut filters = String::new();
    let mut idx = 0;

    for entry in WalkDir::new(dir)
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
    {
        let input = entry
            .path()
            .to_str()
            .ok_or_else(|| anyhow!("failed to convert to &str"))?;
        inputs.extend(vec!["-i".to_string(), input.to_string()]);
        filters.push_str(format!("[{idx}:v:0][{idx}:a:0]").as_str());
        idx += 1;
    }
    // the real length is inputs.len() / 2 because of the pair: `-i <INPUT>`
    let len = inputs.len() / 2;

    debug_assert_eq!(idx, len);
    filters.push_str(format!("concat=n={idx}:v=1:a=1[v][a]").as_str());

    debug_assert_eq!(inputs.len() % 2, 0);
    if (len) < 2 {
        return Err(anyhow!(
            "directory `{}` doesn't have enough videos. it contains {}",
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
            .with_context(|| format!("failed to get the absolute path of {}", entry.display()))?;
        let input = entry
            .to_str()
            .ok_or_else(|| anyhow!("failed to convert to &str"))?;
        writeln!(temp_file, "file '{input}'")?;
        writeln!(temp_file, "duration {framerate}")?;
        total_images += 1;
    }

    ensure!(
        total_images > 1,
        "directory `{}` doesn't have enough images. it contains {}.",
        dir.display(),
        total_images
    );

    Ok((temp_file, total_images))
}

// unwrap the output or generate one
pub fn get_output(input: &Path, output: Option<PathBuf>, prefix: &str) -> Result<PathBuf> {
    output
        .map_or_else(|| generate_output(input, prefix), Ok)
        .with_context(|| format!("could not get the output file for `{}`", input.display()))
}

// Generate an output name
fn generate_output(path: &Path, command_str: &str) -> Result<PathBuf> {
    let original_input = path.to_path_buf();
    let Some(file_name) = original_input.file_name() else {
        return Err(anyhow!("failed to get file_name of {}", path.display()));
    };

    // Convert OsStr to &str to pass to format!()
    let Some(file_name) = file_name.to_str() else {
        return Err(anyhow!("failed to convert to &str"));
    };

    let now = Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let file_name = format!("{command_str}_{timestamp}_{file_name}");

    Ok(Path::new(&original_input).with_file_name(file_name))
}

// Generate outputs from inputs in a directory
pub fn get_inputs_and_outputs(dir: &Path, command_str: &str) -> Result<Vec<(PathBuf, PathBuf)>> {
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

    ensure!(
        !input_output_pair.is_empty(),
        "directory `{}` is empty.",
        dir.display(),
    );

    Ok(input_output_pair)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with('.'))
}
