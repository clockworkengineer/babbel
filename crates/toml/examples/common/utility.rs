//! Utility functions for loading and listing sample TOML files in examples.

/// Returns a list of TOML file paths from the specified directory.
pub fn get_toml_file_list(dir_path: &str) -> Vec<String> {
    babbel_core::file::list_files_by_extension(dir_path, "toml")
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}
