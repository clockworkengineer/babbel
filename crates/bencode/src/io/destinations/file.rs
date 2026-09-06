//! File Destination for Bencode
//!
//! Re-exports the unified `FileDestination` from `babbel_core::io`.

pub use babbel_core::io::FileDestination as File;
use crate::io::traits::{BencodeWrite, BufferedWrite, IDestination};

impl BencodeWrite for File {
    fn write_byte(&mut self, byte: u8) {
        self.add_byte(byte);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        let _ = babbel_core::io::IByteWriter::write_bytes(self, bytes);
    }
}

impl BufferedWrite for File {
    fn clear(&mut self) {
        self.clear();
    }

    fn last_byte(&self) -> Option<u8> {
        self.last()
    }
}

impl IDestination for File {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_file_path() -> String {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        format!("test_bencode_file_dest_{pid}_{id}.txt")
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
    fn write_byte_appends_single_byte() {
        let path = test_file_path();
        let mut file = File::new(&path).unwrap();
        file.write_byte(b'a');
        assert_eq!(file.file_length(), 1);
        assert_eq!(file.last_byte(), Some(b'a'));
        cleanup_file(&path);
    }

    #[test]
    fn write_bytes_appends_multiple_bytes() {
        let path = test_file_path();
        let mut file = File::new(&path).unwrap();
        file.write_bytes(b"hello");
        assert_eq!(file.file_length(), 5);
        assert_eq!(file.last_byte(), Some(b'o'));
        cleanup_file(&path);
    }

    #[test]
    fn clear_resets_file() {
        let path = test_file_path();
        let mut file = File::new(&path).unwrap();
        file.write_bytes(b"hello");
        file.clear();
        assert_eq!(file.file_length(), 0);
        assert_eq!(file.last_byte(), None);
        cleanup_file(&path);
    }
}
