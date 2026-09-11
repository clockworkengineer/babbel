//! Utility library for handling yaml files and related operations.
//! Provides functionality for file system operations specific to yaml files.

/// Returns a list of yaml file paths from the specified directory.
///
/// # Arguments
///
/// * `file_path` - Path to the directory containing yaml files
///
/// # Returns
///
/// A vector of strings containing paths to all .yaml files in the directory
pub fn get_yaml_file_list(file_path: &str) -> Vec<String> {
    babbel_core::file::list_files_by_extension(file_path, "yaml")
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}
