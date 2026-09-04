use crate::io::traits::{BencodeRead, ISource, RewindableRead};
use std::fs::File as StdFile;
use std::io::{Read, Seek, SeekFrom};

/// A file-based implementation for reading bencode data from disk.
/// Provides functionality to read and traverse file content byte by byte.
pub struct File {
    /// Internal file handle for reading operations
    file: StdFile,
    /// Current byte being read from the file
    current_byte: Option<u8>,
}

impl File {
    /// Creates a new File instance from the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read from
    ///
    /// # Returns
    /// A Result containing either the new File instance or an IO error
    pub fn new(path: &str) -> std::io::Result<Self> {
        let mut file = StdFile::open(path)?;
        let mut current_byte = [0u8; 1];
        let has_byte = file.read(&mut current_byte)? == 1;

        Ok(Self {
            file,
            current_byte: if has_byte {
                Some(current_byte[0])
            } else {
                None
            },
        })
    }
}

impl BencodeRead for File {
    fn peek_byte(&mut self) -> Option<u8> {
        self.current_byte
    }

    fn read_byte(&mut self) -> Option<u8> {
        let b = self.current_byte;
        self.advance();
        b
    }

    fn advance(&mut self) {
        let mut byte = [0u8; 1];
        self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
            Some(byte[0])
        } else {
            None
        };
    }

    fn has_more(&mut self) -> bool {
        self.current_byte.is_some()
    }
}

impl RewindableRead for File {
    fn reset(&mut self) {
        if let Ok(_) = self.file.seek(SeekFrom::Start(0)) {
            let mut byte = [0u8; 1];
            self.current_byte = if self.file.read(&mut byte).unwrap_or(0) == 1 {
                Some(byte[0])
            } else {
                None
            };
        }
    }
}

impl ISource for File {}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn create_test_file(content: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = format!("test_{}.txt", timestamp);
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
    fn create_source_non_existent_file_fails() {
        let result = File::new("non_existent_file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn create_source_empty_file_works() {
        let path = create_test_file("");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), None);
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn read_character_from_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), Some('i'));
        cleanup_file(&path);
    }

    #[test]
    fn move_to_next_character_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        source.next();
        assert_eq!(source.current(), Some('3'));
        cleanup_file(&path);
    }

    #[test]
    fn move_to_last_character_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        assert_eq!(source.current(), None);
        cleanup_file(&path);
    }

    #[test]
    fn reset_in_source_file_works() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        source.reset();
        assert_eq!(source.current(), Some('i'));
        cleanup_file(&path);
    }

    #[test]
    fn reset_with_seek_error_maintains_state() {
        let path = create_test_file("i32e");
        let mut source = File::new(&path).unwrap();
        cleanup_file(&path); // Remove the file to cause seek error
        source.reset();
        assert_eq!(source.current(), Some('i'));
    }

    #[test]
    fn read_complete_file_content_matches() {
        let test_content = "i32e";
        let path = create_test_file(test_content);
        let mut source = File::new(&path).unwrap();
        let mut content = String::new();
        while source.more() {
            content.push(source.current().unwrap());
            source.next();
        }
        assert_eq!(content, test_content);
        cleanup_file(&path);
    }

    #[test]
    fn current_does_not_advance_position() {
        let path = create_test_file("abc");
        let mut source = File::new(&path).unwrap();
        assert_eq!(source.current(), Some('a'));
        assert_eq!(source.current(), Some('a'));
        assert_eq!(source.current(), Some('a'));
        cleanup_file(&path);
    }

    #[test]
    fn more_returns_true_at_start_of_non_empty_file() {
        let path = create_test_file("x");
        let mut source = File::new(&path).unwrap();
        assert!(source.more());
        cleanup_file(&path);
    }

    #[test]
    fn more_returns_false_after_last_byte() {
        let path = create_test_file("x");
        let mut source = File::new(&path).unwrap();
        source.next(); // consume the only byte
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn next_past_end_does_not_panic() {
        let path = create_test_file("a");
        let mut source = File::new(&path).unwrap();
        source.next(); // at end
        source.next(); // past end – must not panic
        assert_eq!(source.current(), None);
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn single_byte_file() {
        let path = create_test_file("z");
        let mut source = File::new(&path).unwrap();
        assert!(source.more());
        assert_eq!(source.current(), Some('z'));
        source.next();
        assert!(!source.more());
        assert_eq!(source.current(), None);
        cleanup_file(&path);
    }

    #[test]
    fn traverse_all_chars_in_order() {
        let path = create_test_file("i42e");
        let mut source = File::new(&path).unwrap();
        let expected = ['i', '4', '2', 'e'];
        for &ch in &expected {
            assert_eq!(source.current(), Some(ch));
            source.next();
        }
        assert_eq!(source.current(), None);
        cleanup_file(&path);
    }

    #[test]
    fn more_tracks_position_correctly() {
        let path = create_test_file("ab");
        let mut source = File::new(&path).unwrap();
        assert!(source.more());
        source.next();
        assert!(source.more());
        source.next();
        assert!(!source.more());
        cleanup_file(&path);
    }

    #[test]
    fn reset_on_fresh_file_stays_at_first_char() {
        let path = create_test_file("hello");
        let mut source = File::new(&path).unwrap();
        source.reset();
        assert_eq!(source.current(), Some('h'));
        cleanup_file(&path);
    }

    #[test]
    fn reset_mid_stream_returns_to_first_char() {
        let path = create_test_file("abcd");
        let mut source = File::new(&path).unwrap();
        source.next();
        source.next();
        assert_eq!(source.current(), Some('c'));
        source.reset();
        assert_eq!(source.current(), Some('a'));
        cleanup_file(&path);
    }

    #[test]
    fn reset_then_traverse_again() {
        let path = create_test_file("abc");
        let mut source = File::new(&path).unwrap();
        while source.more() {
            source.next();
        }
        source.reset();
        assert!(source.more());
        assert_eq!(source.current(), Some('a'));
        cleanup_file(&path);
    }

    #[test]
    fn bencode_list_traversal() {
        let input = "l4:spami42ee";
        let path = create_test_file(input);
        let mut source = File::new(&path).unwrap();
        let mut collected = String::new();
        while source.more() {
            collected.push(source.current().unwrap());
            source.next();
        }
        assert_eq!(collected, input);
        cleanup_file(&path);
    }

    #[test]
    fn bencode_dict_traversal() {
        let input = "d3:cow3:moo4:spam4:eggse";
        let path = create_test_file(input);
        let mut source = File::new(&path).unwrap();
        let mut collected = String::new();
        while source.more() {
            collected.push(source.current().unwrap());
            source.next();
        }
        assert_eq!(collected, input);
        cleanup_file(&path);
    }

    #[test]
    fn bencode_negative_integer_traversal() {
        let input = "i-7e";
        let path = create_test_file(input);
        let mut source = File::new(&path).unwrap();
        let mut collected = String::new();
        while source.more() {
            collected.push(source.current().unwrap());
            source.next();
        }
        assert_eq!(collected, input);
        cleanup_file(&path);
    }

    #[test]
    fn multiple_resets_are_idempotent() {
        let path = create_test_file("xyz");
        let mut source = File::new(&path).unwrap();
        source.next();
        source.reset();
        source.reset();
        assert_eq!(source.current(), Some('x'));
        assert!(source.more());
        cleanup_file(&path);
    }
}
