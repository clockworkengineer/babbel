//! Bencode Buffer Source
//!
//! Re-exports the unified `BufferSource` cursor from `babbel_core::io`
//! and implements Bencode-specific read traits.

pub use babbel_core::io::BufferSource as Buffer;
use crate::io::traits::{ISource, RewindableRead};

impl RewindableRead for Buffer {
    fn reset(&mut self) {
        babbel_core::io::ISource::reset(self);
    }
}

impl ISource for Buffer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::traits::BencodeRead;

    #[test]
    fn create_source_buffer_works() {
        let source = Buffer::new(b"i32e");
        assert_eq!(source.to_string(), "i32e");
    }

    #[test]
    fn read_byte_from_source_buffer_works() {
        let mut source = Buffer::new(b"i32e");
        assert_eq!(source.read_byte(), Some(b'i'));
    }

    #[test]
    fn peek_byte_does_not_advance() {
        let mut source = Buffer::new(b"i32e");
        assert_eq!(source.peek_byte(), Some(b'i'));
        assert_eq!(source.peek_byte(), Some(b'i'));
        assert_eq!(source.read_byte(), Some(b'i'));
        assert_eq!(source.peek_byte(), Some(b'3'));
    }

    #[test]
    fn advance_works() {
        let mut source = Buffer::new(b"i32e");
        source.advance();
        assert_eq!(source.read_byte(), Some(b'3'));
    }

    #[test]
    fn has_more_works() {
        let mut source = Buffer::new(b"ab");
        assert!(source.has_more());
        source.advance();
        assert!(source.has_more());
        source.advance();
        assert!(!source.has_more());
    }
}
