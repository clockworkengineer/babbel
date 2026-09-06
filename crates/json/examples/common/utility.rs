//! Utility library for handling JSON files and related operations.
//! Provides functionality for file system operations specific to JSON files.

use std::fs;
use std::path::Path;

/// Returns a list of JSON file paths from the specified directory.
///
/// # Arguments
///
/// * `file_path` - Path to the directory containing JSON files
///
/// # Returns
///
/// A vector of strings containing paths to all .json files in the directory
pub fn get_json_file_list(file_path: &str) -> Vec<String> {
    let files_dir = Path::new(file_path);
    if !files_dir.exists() {
        let _ = fs::create_dir_all("files");
        return vec![];
    }

    fs::read_dir(files_dir)
        .map(|entries| {
            entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()? == "json" {
                        Some(path.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
