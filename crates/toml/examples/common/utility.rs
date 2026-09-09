//! Utility functions for loading and listing sample TOML files in examples.

use std::fs;
use std::path::Path;

/// Returns a list of TOML file paths from the specified directory.
pub fn get_toml_file_list(dir_path: &str) -> Vec<String> {
    let dir = Path::new(dir_path);
    if !dir.exists() {
        let _ = fs::create_dir_all(dir);
        return vec![];
    }

    fs::read_dir(dir)
        .map(|entries| {
            let mut paths: Vec<String> = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "toml") {
                        Some(path.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
                .collect();
            paths.sort();
            paths
        })
        .unwrap_or_default()
}
