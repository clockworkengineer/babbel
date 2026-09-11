//! Utility library for handling JSON files and related operations.
//! Provides functionality for file system operations specific to JSON files.

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
    babbel_core::file::list_files_by_extension(file_path, "json")
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}
