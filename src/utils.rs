use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use chrono::Local;
use tempfile::TempDir;
use walkdir::{DirEntry, WalkDir};

pub fn merge_with_inputs_and_filters(dir: &Path) -> Result<(Vec<String>, String)> {
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
    if (inputs.len() / 2) < 2 {
        return Err(anyhow!(
            "Not enough video file to merge. {} contains: {}",
            dir.display(),
            inputs.len() / 2
        ));
    }

    Ok((inputs, filters))
}

pub fn temp_list_for_video(dir: &Path, framerate: u64) -> Result<(TempDir, PathBuf, usize)> {
    // TODO: replace TempDir. It's now unmaintained.
    // Also, it's not working.
    let temp_dir = TempDir::new_in(dir)?;
    let temp_list = temp_dir.path().join("temp_list.txt");
    let mut temp_list_file = File::create(&temp_list)?;

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
        writeln!(temp_list_file, "file '{}'", entry.display())?;
        writeln!(temp_list_file, "duration {framerate}")?;
        total_images += 1;
    }

    Ok((temp_dir, temp_list, total_images))
}

// // Creates a temporary dir and a temporary file
// pub fn tmp_list_for_video(src: &Path, framerate: i16) -> Result<(TempDir, PathBuf, usize)> {
//     let tmp_dir = TempDir::new_in(".")?;
//     let tmp_list = tmp_dir.path().join("tmp_list.txt");
//     let mut tmp_list_file = File::create(&tmp_list)?;

//     let mut total_images = 0;

//     let source_dir = PathBuf::from(src);
//     for entry in source_dir.read_dir()?.flatten() {
//         let entry_path = entry.path();
//         let entry_path_absolute = fs::canonicalize(&entry_path)
//             .with_context(|| format!("Failed to get absolute path of {}", entry_path.display()))?;

//         if let Some(ext) = entry_path_absolute.extension() {
//             if let Some(ext) = ext.to_str() {
//                 if ext == "png" || ext == "jpg" {
//                     writeln!(tmp_list_file, "file '{}'", entry_path_absolute.display())?;
//                     writeln!(tmp_list_file, "duration {framerate}")?;
//                     total_images += 1;
//                 }
//             }
//         }
//     }

//     Ok((tmp_dir, tmp_list, total_images))
// }

// Generate an output name
pub fn generate_output_with(path: &Path, command_str: &str) -> Result<PathBuf> {
    let original_input = path.to_path_buf();
    let Some(file_name) = original_input.file_name() else {
        return Err(anyhow!("Error extracting file name from Input"));
    };

    // Convert OsStr to &str to pass to format!
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
        let output = generate_output_with(input, command_str)?;

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
