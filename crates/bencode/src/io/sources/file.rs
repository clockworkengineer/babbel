//! File Source for Bencode
//!
//! Re-exports the unified `FileSource` from `babbel_core::io`.

pub use babbel_core::io::FileSource as File;
use crate::io::traits::{ISource, RewindableRead};

impl RewindableRead for File {
    fn reset(&mut self) {
        babbel_core::io::IRewindable::reset(self);
    }
}

impl ISource for File {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn create_test_file(content: &str) -> String {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("test_bencode_file_src_{pid}_{id}.txt"));
        let path = temp_path.to_str().unwrap().to_string();
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    fn cleanup_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn create_source_file_works() {
        let path = create_test_file("i32e");
        let source = File::new(&path);
        assert!(source.is_ok());
        cleanup_file(&path);
    }

    #[test]
    fn read_byte_from_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.read_byte(), Some(b'i'));
        cleanup_file(&path);
    }

    #[test]
    fn peek_byte_does_not_advance() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.peek_byte(), Some(b'i'));
        assert_eq!(source.peek_byte(), Some(b'i'));
        assert_eq!(source.read_byte(), Some(b'i'));
        assert_eq!(source.peek_byte(), Some(b'3'));
        cleanup_file(&path);
    }

    #[test]
    fn advance_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        source.advance();
        assert_eq!(source.read_byte(), Some(b'3'));
        cleanup_file(&path);
    }

    #[test]
    fn reset_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        source.advance();
        assert_eq!(source.read_byte(), Some(b'3'));
        source.reset();
        assert_eq!(source.read_byte(), Some(b'i'));
        cleanup_file(&path);
    }
}
