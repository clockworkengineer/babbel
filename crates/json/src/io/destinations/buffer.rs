use crate::io::traits::IDestination;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

/// A memory buffer implementation for storing encoded JSON data as bytes.
/// Provides functionality to write and manipulate byte content in memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    pub buffer: Vec<u8>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self { buffer: vec![] }
    }
}

impl Buffer {
    /// Creates a new empty Buffer instance.
    ///
    /// # Returns
    /// A new Buffer with an empty internal byte vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts the buffer content to a String.
    ///
    /// # Returns
    /// A String containing UTF-8 interpretation of the buffer bytes.
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }
}

impl IDestination for Buffer {
    /// Adds a single byte to the end of the buffer.
    fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    /// Adds multiple bytes from a string slice to the buffer.
    fn add_bytes(&mut self, bytes: &str) {
        self.buffer.extend_from_slice(bytes.as_bytes());
    }

    /// Clears all content from the buffer.
    fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the last byte in the buffer, if any.
    fn last(&self) -> Option<u8> {
        self.buffer.last().copied()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(buffer.to_string(), "�");
    }

    #[test]
    fn default_creates_empty_buffer() {
        let buffer = Buffer::default();
        assert!(buffer.buffer.is_empty());
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn to_string_empty_buffer_returns_empty_string() {
        let buffer = Buffer::new();
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn to_string_returns_correct_utf8_string() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello, world!");
        assert_eq!(buffer.to_string(), "hello, world!");
    }

    #[test]
    fn add_byte_appends_to_existing_content() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("abc");
        buffer.add_byte(b'd');
        assert_eq!(buffer.to_string(), "abcd");
    }

    #[test]
    fn add_bytes_empty_string_does_nothing() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello");
        buffer.add_bytes("");
        assert_eq!(buffer.to_string(), "hello");
    }

    #[test]
    fn add_bytes_multiple_calls_concatenate_in_order() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("foo");
        buffer.add_bytes("bar");
        buffer.add_bytes("baz");
        assert_eq!(buffer.to_string(), "foobarbaz");
    }

    #[test]
    fn clear_on_empty_buffer_is_a_no_op() {
        let mut buffer = Buffer::new();
        buffer.clear();
        assert!(buffer.buffer.is_empty());
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn clear_allows_reuse_of_buffer() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("first");
        buffer.clear();
        buffer.add_bytes("second");
        assert_eq!(buffer.to_string(), "second");
    }

    #[test]
    fn clear_multiple_times_leaves_buffer_empty() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("data");
        buffer.clear();
        buffer.clear();
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn last_after_add_bytes_returns_last_byte() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("abc");
        assert_eq!(buffer.last(), Some(b'c'));
    }

    #[test]
    fn last_reflects_most_recent_byte() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("xyz");
        buffer.add_byte(b'!');
        assert_eq!(buffer.last(), Some(b'!'));
    }

    #[test]
    fn buffer_len_matches_bytes_written() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello");
        assert_eq!(buffer.buffer.len(), 5);
        buffer.add_byte(b'!');
        assert_eq!(buffer.buffer.len(), 6);
    }

    #[test]
    fn add_bytes_unicode_multibyte_chars() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("こんにちは"); // 5 chars, each 3 bytes in UTF-8
        assert_eq!(buffer.to_string(), "こんにちは");
        assert_eq!(buffer.buffer.len(), 15);
    }

    #[test]
    fn add_bytes_json_object_string() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("{\"key\": \"value\"}");
        assert_eq!(buffer.to_string(), "{\"key\": \"value\"}");
    }

    #[test]
    fn add_bytes_newline_and_tab_chars() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("line1\nline2\ttab");
        assert_eq!(buffer.to_string(), "line1\nline2\ttab");
    }

    #[test]
    fn buffer_raw_bytes_match_expected_for_ascii() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("ABC");
        assert_eq!(buffer.buffer, vec![b'A', b'B', b'C']);
    }

    #[test]
    fn last_on_single_byte_buffer_returns_that_byte() {
        let mut buffer = Buffer::new();
        buffer.add_byte(b'Z');
        assert_eq!(buffer.last(), Some(b'Z'));
    }
}
