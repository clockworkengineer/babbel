//! Integration tests for JSON parser
//! Tests parsing of real JSON files from the files directory

#[cfg(test)]
mod tests {

    use crate::{FileSource, parse};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn test_file(relative: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("files")
            .join(relative)
    }

    /// Helper function to get all JSON file paths from a directory
    fn get_json_file_paths(directory: impl AsRef<Path>) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    paths.push(path);
                }
            }
        }
        paths
    }

    #[test]
    fn test_parse_json_files() {
        let files_dir = test_file("");
        let json_files = get_json_file_paths(files_dir);
        assert!(
            !json_files.is_empty(),
            "No JSON files found in files directory"
        );
        for file_path in json_files {
            match FileSource::new(&file_path) {
                Ok(mut source) => {
                    let result = parse(&mut source);
                    assert!(
                        result.is_ok(),
                        "Failed to parse {}: {:?}",
                        file_path.display(),
                        result.err()
                    );
                }
                Err(e) => panic!("Failed to open {}: {}", file_path.display(), e),
            }
        }
    }
}
