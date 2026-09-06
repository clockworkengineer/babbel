//! File Destination for JSON
//!
//! Re-exports the unified `FileDestination` from `babbel_core::io`.

pub use babbel_core::io::FileDestination as File;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::traits::IDestination;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_file_path() -> String {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        format!("test_json_file_dest_{pid}_{id}.txt")
    }

    fn cleanup_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn new_creates_file_and_initializes_fields() {
        let path = test_file_path();
        let file = File::new(&path).unwrap();
        assert_eq!(file.file_length(), 0);
        assert_eq!(file.file_name(), path);
        cleanup_file(&path);
    }

    #[test]
    fn add_byte_appends_single_byte() {
        let path = test_file_path();
        let mut file = File::new(&path).unwrap();
        file.add_byte(b'a');
        assert_eq!(file.file_length(), 1);
        assert_eq!(file.last(), Some(b'a'));
        cleanup_file(&path);
    }

    #[test]
    fn add_bytes_appends_multiple_bytes() {
        let path = test_file_path();
        let mut file = File::new(&path).unwrap();
        file.add_bytes("hello");
        assert_eq!(file.file_length(), 5);
        assert_eq!(file.last(), Some(b'o'));
        cleanup_file(&path);
    }

    #[test]
    fn clear_resets_file() {
        let path = test_file_path();
        let mut file = File::new(&path).unwrap();
        file.add_bytes("hello");
        file.clear();
        assert_eq!(file.file_length(), 0);
        assert_eq!(file.last(), None);
        cleanup_file(&path);
    }

    #[test]
    fn last_on_empty_returns_none() {
        let path = test_file_path();
        let file = File::new(&path).unwrap();
        assert_eq!(file.last(), None);
        cleanup_file(&path);
    }
}
