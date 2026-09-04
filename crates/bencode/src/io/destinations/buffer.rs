#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

use crate::io::traits::{BencodeWrite, BufferedWrite, IDestination};
/// A memory buffer implementation for storing encoded bencode data as bytes.
/// Provides functionality to write and manipulate byte content in memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    pub buffer: Vec<u8>,
}

impl Buffer {
    /// Creates a new empty Buffer instance.
    ///
    /// # Returns
    /// A new Buffer with an empty internal byte vector.
    pub fn new() -> Self {
        Self { buffer: vec![] }
    }

    /// Converts the buffer content to a String.
    ///
    /// # Returns
    /// A String containing UTF-8 interpretation of the buffer bytes.
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }

    /// Clears all content from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

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

impl IDestination for Buffer {}
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
    fn new_buffer_has_zero_length() {
        let buffer = Buffer::new();
        assert_eq!(buffer.buffer.len(), 0);
    }

    #[test]
    fn add_byte_increments_length() {
        let mut buffer = Buffer::new();
        buffer.add_byte(b'a');
        assert_eq!(buffer.buffer.len(), 1);
        buffer.add_byte(b'b');
        assert_eq!(buffer.buffer.len(), 2);
    }

    #[test]
    fn add_bytes_increments_length_by_str_len() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("hello");
        assert_eq!(buffer.buffer.len(), 5);
        buffer.add_bytes("!!");
        assert_eq!(buffer.buffer.len(), 7);
    }

    #[test]
    fn add_bytes_empty_string_is_noop() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("");
        assert!(buffer.buffer.is_empty());
        assert_eq!(buffer.last(), None);
    }

    #[test]
    fn add_byte_stores_correct_raw_value() {
        let mut buffer = Buffer::new();
        buffer.add_byte(0x00);
        buffer.add_byte(0x7F);
        buffer.add_byte(0xFE);
        assert_eq!(buffer.buffer[0], 0x00);
        assert_eq!(buffer.buffer[1], 0x7F);
        assert_eq!(buffer.buffer[2], 0xFE);
    }

    #[test]
    fn add_bytes_stores_correct_raw_bytes() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("abc");
        assert_eq!(buffer.buffer, b"abc");
    }

    #[test]
    fn add_byte_then_add_bytes_mixed() {
        let mut buffer = Buffer::new();
        buffer.add_byte(b'4');
        buffer.add_bytes(":spam");
        assert_eq!(buffer.to_string(), "4:spam");
    }

    #[test]
    fn clear_resets_length_to_zero() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("some data");
        buffer.clear();
        assert_eq!(buffer.buffer.len(), 0);
        assert!(buffer.buffer.is_empty());
    }

    #[test]
    fn clear_then_reuse_works() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("first");
        buffer.clear();
        buffer.add_bytes("second");
        assert_eq!(buffer.to_string(), "second");
    }

    #[test]
    fn last_after_add_bytes_returns_last_char() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("list");
        assert_eq!(buffer.last(), Some(b't'));
    }

    #[test]
    fn last_single_element() {
        let mut buffer = Buffer::new();
        buffer.add_byte(b'e');
        assert_eq!(buffer.last(), Some(b'e'));
    }

    #[test]
    fn to_string_empty_buffer() {
        let buffer = Buffer::new();
        assert_eq!(buffer.to_string(), "");
    }

    #[test]
    fn to_string_bencode_integer() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("i42e");
        assert_eq!(buffer.to_string(), "i42e");
    }

    #[test]
    fn to_string_bencode_string() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("4:spam");
        assert_eq!(buffer.to_string(), "4:spam");
    }

    #[test]
    fn to_string_bencode_list() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("l4:spami42ee");
        assert_eq!(buffer.to_string(), "l4:spami42ee");
    }

    #[test]
    fn to_string_bencode_dict() {
        let mut buffer = Buffer::new();
        buffer.add_bytes("d3:cow3:moo4:spam4:eggse");
        assert_eq!(buffer.to_string(), "d3:cow3:moo4:spam4:eggse");
    }

    #[test]
    fn buffer_public_field_directly_accessible() {
        let mut buffer = Buffer::new();
        buffer.buffer.push(b'x');
        assert_eq!(buffer.buffer, vec![b'x']);
    }

    #[test]
    fn multiple_clears_are_idempotent() {
        let mut buffer = Buffer::new();
        buffer.clear();
        buffer.clear();
        assert!(buffer.buffer.is_empty());
        assert_eq!(buffer.last(), None);
    }
}
