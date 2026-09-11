/// Returns the current version of the package as specified in Cargo.toml.
/// Uses CARGO_PKG_VERSION environment variable that is set during compilation
/// from the version field in Cargo.toml.
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// File I/O functions are only available with std feature
#[cfg(feature = "std")]
mod file_io {
    use std::fs;
    use std::path::Path;

    /// Writes bencode string to a file
    ///
    /// # Arguments
    /// * `path` - The file path where the content will be written
    /// * `content` - The bencode string content to write to the file
    pub fn write_bencode_file(path: &str, content: &str) -> Result<(), std::io::Error> {
        fs::write(Path::new(path), content)
    }

    /// Reads bencode string from a file
    ///
    /// # Arguments
    /// * `path` - The file path to read from
    ///
    /// # Returns
    /// * `Ok(String)` - The content of the file as a string if successful
    /// * `Err(std::io::Error)` - The error if reading fails
    pub fn read_bencode_file(path: &str) -> Result<String, std::io::Error> {
        fs::read_to_string(Path::new(path))
    }
}

#[cfg(feature = "std")]
pub use file_io::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    // ── helper ────────────────────────────────────────────────────────────────

    fn unique_path(tag: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("misc_test_{}_{}.torrent", tag, ns)
    }

    // ── get_version ───────────────────────────────────────────────────────────

    #[test]
    fn test_get_version() {
        assert_eq!(get_version(), "0.2.0");
    }

    #[test]
    fn get_version_is_not_empty() {
        assert!(!get_version().is_empty());
    }

    #[test]
    fn get_version_looks_like_semver() {
        // Expect at least two dots: major.minor.patch
        let parts: Vec<&str> = get_version().split('.').collect();
        assert!(
            parts.len() >= 2,
            "version should be semver-ish: {}",
            get_version()
        );
        for part in &parts {
            assert!(
                part.chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == '+' || c.is_ascii_alphabetic()),
                "unexpected version component: {}",
                part
            );
        }
    }

    // ── read_bencode_file ─────────────────────────────────────────────────────

    #[test]
    fn test_read_bencode_file_success() {
        let test_content = "d8:announce15:http://test.come";
        let test_file = "test.torrent";

        File::create(test_file)
            .and_then(|mut file| file.write_all(test_content.as_bytes()))
            .expect("Failed to create test file");

        let result = read_bencode_file(test_file);
        fs::remove_file(test_file).expect("Failed to cleanup test file");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_content);
    }

    #[test]
    fn test_read_bencode_file_error() {
        let result = read_bencode_file("nonexistent.torrent");
        assert!(result.is_err());
    }

    #[test]
    fn read_bencode_file_empty_file_returns_empty_string() {
        let path = unique_path("empty_read");
        File::create(&path).unwrap();
        let result = read_bencode_file(&path);
        fs::remove_file(&path).unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn read_bencode_file_binary_content_roundtrip() {
        let path = unique_path("binary");
        let content = "i-42el4:spame3:fooe";
        fs::write(&path, content).unwrap();
        let result = read_bencode_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn read_bencode_file_multiline_content() {
        let path = unique_path("multiline");
        let content = "line1\nline2\nline3";
        fs::write(&path, content).unwrap();
        let result = read_bencode_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn read_bencode_file_missing_returns_error() {
        let path = unique_path("definitely_does_not_exist");
        let _ = fs::remove_file(&path); // ensure gone
        assert!(read_bencode_file(&path).is_err());
    }

    // ── write_bencode_file ────────────────────────────────────────────────────

    #[test]
    fn test_write_bencode_file() {
        let test_content = "d8:announce15:http://test.come";
        let test_file = "test_write.torrent";

        let write_result = write_bencode_file(test_file, test_content);
        assert!(write_result.is_ok());

        let read_result = read_bencode_file(test_file);
        fs::remove_file(test_file).expect("Failed to cleanup test file");

        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);
    }

    #[test]
    fn write_bencode_file_creates_file_on_disk() {
        let path = unique_path("creates");
        write_bencode_file(&path, "i1e").unwrap();
        let exists = fs::metadata(&path).is_ok();
        fs::remove_file(&path).unwrap();
        assert!(exists);
    }

    #[test]
    fn write_bencode_file_empty_content() {
        let path = unique_path("empty_write");
        write_bencode_file(&path, "").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn write_bencode_file_overwrites_existing_content() {
        let path = unique_path("overwrite");
        write_bencode_file(&path, "i1e").unwrap();
        write_bencode_file(&path, "i99e").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(content, "i99e");
    }

    #[test]
    fn write_bencode_file_invalid_path_returns_error() {
        let result = write_bencode_file("/invalid/directory/that/does/not/exist/file.ben", "i1e");
        assert!(result.is_err());
    }

    #[test]
    fn write_then_read_roundtrip_bencode_dict() {
        let path = unique_path("dict_roundtrip");
        let content = "d3:cow3:moo4:spam4:eggse";
        write_bencode_file(&path, content).unwrap();
        let read_back = read_bencode_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn write_then_read_roundtrip_bencode_list() {
        let path = unique_path("list_roundtrip");
        let content = "l4:spami42ee";
        write_bencode_file(&path, content).unwrap();
        let read_back = read_bencode_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn write_then_read_roundtrip_large_content() {
        let path = unique_path("large");
        let content = "i42e".repeat(1000);
        write_bencode_file(&path, &content).unwrap();
        let read_back = read_bencode_file(&path).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(read_back, content);
    }
}
