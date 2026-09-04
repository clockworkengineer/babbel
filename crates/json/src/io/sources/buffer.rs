use crate::io::traits::ISource;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// A memory buffer implementation for reading JSON data from bytes.
/// Provides functionality to traverse and read byte content from memory.
pub struct Buffer {
    /// Internal vector storing the raw bytes
    buffer: Vec<u8>,
    /// Current reading position in the buffer
    position: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            position: 0,
        }
    }
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
}

impl ISource for Buffer {
    /// Moves to the next character in the buffer
    fn next(&mut self) {
        self.position += 1;
    }
    /// Returns the current character at the buffer position
    fn current(&mut self) -> Option<char> {
        if self.more() {
            Some(self.buffer[self.position] as char)
        } else {
            None
        }
    }
    /// Checks if there are more characters to read
    fn more(&mut self) -> bool {
        self.position < self.buffer.len()
    }
    /// Resets the buffer position to the start
    fn reset(&mut self) {
        self.position = 0;
    }
}
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
    fn default_creates_empty_buffer() {
        let mut source = Buffer::default();
        assert_eq!(source.to_string(), "");
        assert!(!source.more());
        assert_eq!(source.current(), None);
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
    fn next_past_end_does_not_panic() {
        let mut source = Buffer::new(b"a");
        source.next(); // move past the only byte
        source.next(); // past end – should not panic
        assert!(!source.more());
        assert_eq!(source.current(), None);
    }

    #[test]
    fn reset_after_partial_read_restarts_from_beginning() {
        let mut source = Buffer::new(b"hello");
        source.next();
        source.next();
        source.reset();
        assert_eq!(source.current(), Some('h'));
    }

    #[test]
    fn reset_on_empty_buffer_does_not_panic() {
        let mut source = Buffer::new(&[]);
        source.reset();
        assert_eq!(source.current(), None);
    }

    #[test]
    fn can_read_every_character_in_sequence() {
        let input = b"json";
        let expected = ['j', 's', 'o', 'n'];
        let mut source = Buffer::new(input);
        for &ch in &expected {
            assert_eq!(source.current(), Some(ch));
            source.next();
        }
        assert_eq!(source.current(), None);
    }

    #[test]
    fn more_is_false_immediately_after_last_character() {
        let mut source = Buffer::new(b"xy");
        source.next(); // at 'y'
        assert!(source.more());
        source.next(); // past end
        assert!(!source.more());
    }

    #[test]
    fn single_byte_buffer_reads_then_exhausts() {
        let mut source = Buffer::new(b"Z");
        assert!(source.more());
        assert_eq!(source.current(), Some('Z'));
        source.next();
        assert!(!source.more());
        assert_eq!(source.current(), None);
    }

    #[test]
    fn to_string_returns_full_content_regardless_of_position() {
        let mut source = Buffer::new(b"abcde");
        source.next();
        source.next();
        // to_string should reflect the whole underlying buffer, not the read position
        assert_eq!(source.to_string(), "abcde");
    }

    #[test]
    fn reset_multiple_times_always_returns_to_start() {
        let mut source = Buffer::new(b"xyz");
        source.next();
        source.reset();
        assert_eq!(source.current(), Some('x'));
        source.next();
        source.next();
        source.reset();
        assert_eq!(source.current(), Some('x'));
    }

    #[test]
    fn buffer_with_whitespace_characters() {
        let mut source = Buffer::new(b" \t\n");
        assert_eq!(source.current(), Some(' '));
        source.next();
        assert_eq!(source.current(), Some('\t'));
        source.next();
        assert_eq!(source.current(), Some('\n'));
        source.next();
        assert_eq!(source.current(), None);
    }

    #[test]
    fn buffer_with_json_object_string() {
        let json = br#"{"key":"value"}"#;
        let mut source = Buffer::new(json);
        assert_eq!(source.current(), Some('{'));
        // advance to the end
        while source.more() {
            source.next();
        }
        assert_eq!(source.current(), None);
        assert_eq!(source.to_string(), r#"{"key":"value"}"#);
    }

    #[test]
    fn buffer_with_ascii_digits() {
        let mut source = Buffer::new(b"0123456789");
        for digit in '0'..='9' {
            assert_eq!(source.current(), Some(digit));
            source.next();
        }
        assert!(!source.more());
    }
}
