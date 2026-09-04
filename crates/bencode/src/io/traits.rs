//! Core streaming I/O traits powered by `babbel_core`.

pub use babbel_core::io::traits::IByteStream as BencodeRead;

/// Optional extension trait for sources that support position resetting (LSP compliant).
pub trait RewindableRead {
    /// Resets the reading position to the beginning of the source.
    fn reset(&mut self);
}

/// Core trait defining the interface for writing bencode output (ISP & DIP compliant).
pub trait BencodeWrite {
    /// Writes a single byte to the destination.
    fn write_byte(&mut self, byte: u8);
    /// Writes multiple bytes from a raw byte slice to the destination (binary-safe).
    fn write_bytes(&mut self, bytes: &[u8]);
}

/// Optional extension trait for destinations supporting buffer clearing & tail inspection (LSP compliant).
pub trait BufferedWrite {
    /// Clears all content from the destination.
    fn clear(&mut self);
    /// Returns the last written byte, if any.
    fn last_byte(&self) -> Option<u8>;
}

/// Trait defining the interface for reading and traversing bencode data from a source.
/// (Maintained for backward compatibility, extending BencodeRead and RewindableRead).
pub trait ISource: BencodeRead + RewindableRead {
    /// Advances the reading position to the next character.
    fn next(&mut self) {
        self.advance();
    }
    /// Returns the character at the current reading position.
    fn current(&mut self) -> Option<char> {
        self.peek_byte().map(|b| b as char)
    }
    /// Checks if there are more characters available to read.
    fn more(&mut self) -> bool {
        self.has_more()
    }
}

/// Trait defining the interface for writing bencode data to a destination.
/// (Maintained for backward compatibility, extending BencodeWrite and BufferedWrite).
pub trait IDestination: BencodeWrite + BufferedWrite {
    /// Adds a single byte to the destination.
    fn add_byte(&mut self, byte: u8) {
        self.write_byte(byte);
    }
    /// Adds multiple bytes from a string slice to the destination.
    fn add_bytes(&mut self, bytes: &str) {
        self.write_bytes(bytes.as_bytes());
    }
    /// Returns the last byte in the destination, if any.
    fn last(&self) -> Option<u8> {
        self.last_byte()
    }
}
