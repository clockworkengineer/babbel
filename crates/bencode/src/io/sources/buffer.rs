#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::io::traits::{BencodeRead, ISource, RewindableRead};

/// A memory buffer implementation for reading bencode data from bytes.
/// Provides functionality to traverse and read byte content from memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    buffer: Vec<u8>,
    /// Current reading position in the buffer
    position: usize,
}

impl Buffer {
    /// Creates a new Buffer instance with the specified byte slice.
    ///
    /// # Arguments
    /// * `to_add` - The byte slice to initialize the buffer with
    ///
    /// # Returns
    /// A new Buffer containing the provided bytes
    pub fn new(to_add: &[u8]) -> Self {
        Self {
            buffer: to_add.to_vec(),
            position: 0,
        }
    }
    /// Converts the buffer content to a String.
    ///
    /// # Returns
    /// A String containing UTF-8 interpretation of the buffer bytes.
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.buffer).into_owned()
    }

    /// Resets the buffer position to the start.
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

impl BencodeRead for Buffer {
    fn peek_byte(&mut self) -> Option<u8> {
        if self.position < self.buffer.len() {
            Some(self.buffer[self.position])
        } else {
            None
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.position < self.buffer.len() {
            let byte = self.buffer[self.position];
            self.position += 1;
            Some(byte)
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn has_more(&mut self) -> bool {
        self.position < self.buffer.len()
    }
}

impl RewindableRead for Buffer {
    fn reset(&mut self) {
        self.reset();
    }
}

impl ISource for Buffer {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_source_buffer_works() {
        let source = Buffer::new(String::from("i32e").as_bytes());
        assert_eq!(source.to_string(), "i32e");
    }
    #[test]
    fn read_character_from_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        match source.current() {
            Some('i') => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn move_to_next_character_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        source.next();
        match source.current() {
            Some('3') => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn move_to_last_character_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        while source.more() {
            source.next()
        }
        match source.current() {
            None => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn reset_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        while source.more() {
            source.next()
        }
        source.reset();
        match source.current() {
            Some('i') => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn create_empty_buffer_works() {
        let source = Buffer::new(&[]);
        assert_eq!(source.to_string(), "");
    }
    #[test]
    fn handle_non_utf8_content() {
        let source = Buffer::new(&[0xFF]);
        assert_eq!(source.to_string(), String::from_utf8_lossy(&[0xFF]));
    }
    #[test]
    fn more_returns_correct_at_boundaries() {
        let mut source = Buffer::new(String::from("a").as_bytes());
        assert!(source.more());
        source.next();
        assert!(!source.more());
    }
    #[test]
    fn multiple_next_calls_work() {
        let mut source = Buffer::new(String::from("abc").as_bytes());
        source.next();
        source.next();
        match source.current() {
            Some('c') => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn current_on_empty_buffer_returns_none() {
        let mut source = Buffer::new(&[]);
        assert_eq!(source.current(), None);
    }

    #[test]
    fn more_on_empty_buffer_returns_false() {
        let mut source = Buffer::new(&[]);
        assert!(!source.more());
    }

    #[test]
    fn current_does_not_advance_position() {
        let mut source = Buffer::new(b"xy");
        assert_eq!(source.current(), Some('x'));
        assert_eq!(source.current(), Some('x'));
        assert_eq!(source.current(), Some('x'));
    }

    #[test]
    fn next_past_end_does_not_panic() {
        let mut source = Buffer::new(b"a");
        source.next(); // now at end
        source.next(); // past end – should not panic
        assert_eq!(source.current(), None);
        assert!(!source.more());
    }

    #[test]
    fn reset_on_fresh_buffer_stays_at_start() {
        let mut source = Buffer::new(b"hello");
        source.reset();
        assert_eq!(source.current(), Some('h'));
    }

    #[test]
    fn reset_mid_stream_returns_to_first_char() {
        let mut source = Buffer::new(b"abc");
        source.next();
        source.next();
        assert_eq!(source.current(), Some('c'));
        source.reset();
        assert_eq!(source.current(), Some('a'));
    }

    #[test]
    fn traverse_all_chars_in_order() {
        let mut source = Buffer::new(b"i42e");
        let expected = ['i', '4', '2', 'e'];
        for &ch in &expected {
            assert_eq!(source.current(), Some(ch));
            source.next();
        }
        assert_eq!(source.current(), None);
    }

    #[test]
    fn more_tracks_position_correctly() {
        let mut source = Buffer::new(b"ab");
        assert!(source.more());
        source.next();
        assert!(source.more());
        source.next();
        assert!(!source.more());
    }

    #[test]
    fn bencode_list_traversal() {
        let mut source = Buffer::new(b"l4:spami42ee");
        let expected = "l4:spami42ee";
        let mut collected = String::new();
        while source.more() {
            if let Some(ch) = source.current() {
                collected.push(ch);
            }
            source.next();
        }
        assert_eq!(collected, expected);
    }

    #[test]
    fn bencode_dict_traversal() {
        let input = b"d3:cow3:moo4:spam4:eggse";
        let mut source = Buffer::new(input);
        let mut collected = String::new();
        while source.more() {
            if let Some(ch) = source.current() {
                collected.push(ch);
            }
            source.next();
        }
        assert_eq!(collected, String::from_utf8_lossy(input));
    }

    #[test]
    fn to_string_returns_full_content_regardless_of_position() {
        let mut source = Buffer::new(b"i99e");
        source.next();
        source.next();
        // to_string should still return the full buffer, not just remaining bytes
        assert_eq!(source.to_string(), "i99e");
    }

    #[test]
    fn to_string_empty_buffer_is_empty_string() {
        let source = Buffer::new(&[]);
        assert_eq!(source.to_string(), "");
    }

    #[test]
    fn reset_then_traverse_again() {
        let mut source = Buffer::new(b"abc");
        while source.more() {
            source.next();
        }
        source.reset();
        assert!(source.more());
        assert_eq!(source.current(), Some('a'));
    }

    #[test]
    fn single_byte_buffer() {
        let mut source = Buffer::new(b"z");
        assert!(source.more());
        assert_eq!(source.current(), Some('z'));
        source.next();
        assert!(!source.more());
        assert_eq!(source.current(), None);
    }
}
