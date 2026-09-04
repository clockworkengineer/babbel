//! Bencode Buffer Destination
//!
//! Re-exports the unified `Buffer` destination from `babbel_core::io`
//! and implements Bencode-specific write traits.

pub use babbel_core::io::Buffer;
use crate::io::traits::{BencodeWrite, BufferedWrite};

impl BencodeWrite for Buffer {
    fn write_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }
}

impl BufferedWrite for Buffer {
    fn clear(&mut self) {
        self.clear();
    }

    fn last_byte(&self) -> Option<u8> {
        self.buffer.last().copied()
    }
}

impl crate::io::traits::IDestination for Buffer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_buffer() {
        let buffer = Buffer::new();
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn default_creates_empty_buffer() {
        let buffer = Buffer::default();
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn write_byte_appends_to_buffer() {
        let mut buffer = Buffer::new();
        buffer.write_byte(b'a');
        assert_eq!(buffer.buffer, vec![b'a']);
    }

    #[test]
    fn write_bytes_appends_slice_to_buffer() {
        let mut buffer = Buffer::new();
        buffer.write_bytes(b"hello");
        assert_eq!(buffer.buffer, b"hello".to_vec());
    }

    #[test]
    fn clear_empties_the_buffer() {
        let mut buffer = Buffer::new();
        buffer.write_bytes(b"hello");
        buffer.clear();
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn to_string_converts_buffer_to_utf8() {
        let mut buffer = Buffer::new();
        buffer.write_bytes(b"hello");
        assert_eq!(buffer.to_string(), "hello");
    }

    #[test]
    fn last_byte_returns_last_byte() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.last_byte(), None);
        buffer.write_byte(b'x');
        assert_eq!(buffer.last_byte(), Some(b'x'));
        buffer.write_byte(b'y');
        assert_eq!(buffer.last_byte(), Some(b'y'));
    }
}
