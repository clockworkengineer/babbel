//! Buffer Destination for Encoded Output
//!
//! Re-exports the unified `Buffer` destination from `babbel_core::io`.
//!
//! Copyright (c) 2026 YAML Library Developers

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
    fn add_byte_to_destination_buffer_works() {
        let mut destination = Buffer::new();
        destination.add_byte(b'i');
        destination.add_byte(b'3');
        destination.add_byte(b'2');
        destination.add_byte(b'e');
        assert_eq!(destination.to_string(), "i32e");
    }
    #[test]
    fn add_bytes_to_destination_buffer_works() {
        let mut destination = Buffer::new();
        destination.add_bytes("i3");
        assert_eq!(destination.to_string(), "i3");
        destination.add_bytes("2e");
        assert_eq!(destination.to_string(), "i32e");
    }
    #[test]
    fn clear_destination_buffer_works() {
        let mut destination = Buffer::new();
        destination.add_bytes("i32e");
        assert_eq!(destination.to_string(), "i32e");
        destination.clear();
        assert_eq!(destination.to_string(), "");
    }
    #[test]
    fn last_works() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.last(), None);
        buffer.add_byte(b'1');
        assert_eq!(buffer.last(), Some(b'1'));
        buffer.add_byte(b'2');
        assert_eq!(buffer.last(), Some(b'2'));
        buffer.clear();
        assert_eq!(buffer.last(), None);
    }
    #[test]
    fn to_string_handles_non_utf8() {
        let mut buffer = Buffer::new();
        buffer.add_byte(0xFF);
        assert_eq!(buffer.to_string(), "\u{FFFD}");
    }

    #[test]
    fn add_bytes_empty_string() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("");
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn clear_on_empty_buffer() {
        let mut buffer = Buffer::new();
        buffer.clear();
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn add_byte_and_clear_and_add_again() {
        let mut buffer = Buffer::new();
        buffer.add_byte(b'a');
        buffer.clear();
        buffer.add_byte(b'b');
        assert_eq!(buffer.to_string(), "b");
    }

    #[test]
    fn last_after_multiple_adds_and_clear() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("abc");
        assert_eq!(buffer.last(), Some(b'c'));
        buffer.clear();
        assert_eq!(buffer.last(), None);
        buffer.add_bytes("xyz");
        assert_eq!(buffer.last(), Some(b'z'));
    }

    #[test]
    fn to_string_with_mixed_ascii_and_non_utf8() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("abc");
        buffer.add_byte(0xFF);
        assert_eq!(buffer.to_string(), "abc\u{FFFD}");
    }
}
