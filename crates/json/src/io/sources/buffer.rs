//! In-Memory Buffer Source for JSON
//!
//! Re-exports the unified `BufferSource` cursor from `babbel_core::io`.

pub use babbel_core::io::BufferSource as Buffer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::traits::ISource;

    #[test]
    fn create_source_buffer_works() {
        let source = Buffer::new(String::from("i32e").as_bytes());
        assert_eq!(source.to_string(), "i32e");
    }

    #[test]
    fn read_character_from_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        assert_eq!(source.current(), Some('i'));
    }

    #[test]
    fn move_to_next_character_in_source_buffer_works() {
        let mut source = Buffer::new(String::from("i32e").as_bytes());
        source.next();
        assert_eq!(source.current(), Some('3'));
    }

    #[test]
    fn test_more_and_reset() {
        let mut source = Buffer::new(b"ab");
        assert!(source.more());
        assert_eq!(source.current(), Some('a'));
        source.next();
        assert_eq!(source.current(), Some('b'));
        source.next();
        assert_eq!(source.current(), None);
        assert!(!source.more());
        source.reset();
        assert!(source.more());
        assert_eq!(source.current(), Some('a'));
    }
}
