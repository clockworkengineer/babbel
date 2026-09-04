//! In-Memory Buffer Destination for JSON
//!
//! Re-exports the unified `Buffer` destination from `babbel_core::io`.

pub use babbel_core::io::Buffer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::traits::IDestination;

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
    fn add_byte_appends_to_buffer() {
        let mut buffer = Buffer::new();
        buffer.add_byte(b'a');
        assert_eq!(buffer.buffer, vec![b'a']);
    }

    #[test]
    fn add_bytes_appends_string_to_buffer() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello");
        assert_eq!(buffer.buffer, b"hello".to_vec());
    }

    #[test]
    fn clear_empties_the_buffer() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello");
        buffer.clear();
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn to_string_converts_buffer_to_utf8() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello");
        assert_eq!(buffer.to_string(), "hello");
    }

    #[test]
    fn last_returns_last_byte() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.last(), None);
        buffer.add_byte(b'x');
        assert_eq!(buffer.last(), Some(b'x'));
        buffer.add_byte(b'y');
        assert_eq!(buffer.last(), Some(b'y'));
    }
}
