//! Utility library for handling torrent files and related operations.
//! Provides functionality for file system operations specific to torrent files.

use std::fs;
use std::path::Path;

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Write;

/// Returns a list of torrent file paths from the specified directory.
///
/// # Arguments
///
/// * `file_path` - Path to the directory containing torrent files
///
/// # Returns
///
/// A vector of strings containing paths to all .torrent files in the directory
pub fn get_torrent_file_list(file_path: &str) -> Vec<String> {
    // Convert the input path string to a Path
    let files_dir = Path::new(file_path);
    // Create a directory if it doesn't exist
    if !files_dir.exists() {
        fs::create_dir("files").expect("Failed to create files directory");
        return vec![];
    }

    // Read the directory and collect all .torrent files
    fs::read_dir(files_dir)
        .expect("Failed to read directory")
        .filter_map(|entry| {
            // Extract entry and convert to the path
            let entry = entry.ok()?;
            let file_path = entry.path();
            // Check if the file has .torrent extension
            if file_path.extension()? == "torrent" {
                Some(file_path.to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_dir(dir: &str) {
        fs::create_dir_all(dir).expect("Failed to create test directory");
    }

    fn create_test_file(path: &str) {
        let mut file = File::create(path).expect("Failed to create test file");
        file.write_all(b"test content").expect("Failed to write to test file");
    }

    fn cleanup_test_dir(dir: &str) {
        fs::remove_dir_all(dir).expect("Failed to clean up test directory");
    }

    #[test]
    fn test_empty_directory() {
        let test_dir = "test_empty";
        create_test_dir(test_dir);

        let files = get_torrent_file_list(test_dir);
        assert!(files.is_empty());

        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_with_torrent_files() {
        let test_dir = "test_torrents";
        create_test_dir(test_dir);

        create_test_file(&format!("{}/test1.torrent", test_dir));
        create_test_file(&format!("{}/test2.torrent", test_dir));
        create_test_file(&format!("{}/not_torrent.txt", test_dir));

        let files = get_torrent_file_list(test_dir);
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.ends_with(".torrent")));

        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_nonexistent_directory() {
        let files = get_torrent_file_list("nonexistent_dir");
        assert!(files.is_empty());
        assert!(Path::new("files").exists());
        fs::remove_dir("files").expect("Failed to clean up files directory");
    }
}
