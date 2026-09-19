use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use tempfile::TempDir;

// use crate::InputType;

// // Generates filters for merge subcommand.
// pub fn filters_for_merge(
//     inputs: Vec<String>,
//     inputs_type: InputType,
// ) -> Result<(Vec<String>, String)> {
//     let mut filters = String::new();
//     let mut new_inputs = Vec::new();

//     match inputs_type {
//         // It's multiple files, apply the filters for each file.
//         InputType::File => {
//             if inputs.len() < 2 {
//                 return Err(anyhow!("Not enough inputs"));
//             }

//             let total_inputs = inputs.len();
//             for input in 0..total_inputs {
//                 filters.push_str(format!("[{input}:v:0][{input}:a:0]").as_str());
//             }
//             filters.push_str(format!("concat=n={total_inputs}:v=1:a=1[v][a]").as_str());

//             Ok((inputs, filters))
//         }

//         // It's a single directory, we iterator over that and read
//         // each entry checking if they end with mp4 or mkv and get their
//         // absolute path.
//         InputType::Directory => {
//             // We use PathBuf to iterate the directory.
//             let input_path = PathBuf::from(inputs[0].clone());

//             let mut idx = 0;
//             for entry in input_path.read_dir()?.flatten() {
//                 let entry_path = entry.path();

//                 if let Some(ext) = entry_path.extension() {
//                     if let Some(ext) = ext.to_str() {
//                         if ext == "mp4" || ext == "mkv" {
//                             new_inputs.push(entry_path.display().to_string());
//                             filters.push_str(format!("[{idx}:v:0][{idx}:a:0]").as_str());
//                             idx += 1;
//                         }
//                     }
//                 }
//             }
//             filters.push_str(format!("concat=n={idx}:v=1:a=1[v][a]").as_str());

//             // FIXME: Assuming there are at least two videos in the folder to merge.
//             // Add an error for this.
//             Ok((new_inputs, filters))
//         }
//     }
// }

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
pub fn generate_output(path: &Path, command_str: &str) -> Result<PathBuf> {
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
