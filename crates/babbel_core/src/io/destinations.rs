//! Concrete implementations of output destinations.

use super::traits::IDestination;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// In-memory byte buffer destination (`Vec<u8>`).
#[derive(Debug, Clone, Default)]
pub struct Buffer {
    /// Internal vector storing the raw bytes
    pub buffer: Vec<u8>,
}

impl Buffer {
    /// Creates a new buffer destination.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Creates a new buffer with initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Converts the buffer content to a String.
    pub fn to_string(&self) -> String {
        #[cfg(feature = "std")]
        return String::from_utf8_lossy(&self.buffer).into_owned();
        #[cfg(not(feature = "std"))]
        {
            if let Ok(s) = core::str::from_utf8(&self.buffer) {
                alloc::string::ToString::to_string(s)
            } else {
                alloc::string::ToString::to_string(&alloc::string::String::from_utf8_lossy(&self.buffer))
            }
        }
    }

    /// Returns byte slice of written content.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Consumes and returns the inner byte vector.
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }

    /// Clears all content from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl IDestination for Buffer {
    fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    fn add_bytes(&mut self, bytes: &str) {
        self.buffer.extend_from_slice(bytes.as_bytes());
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn last(&self) -> Option<u8> {
        self.buffer.last().copied()
    }
}

/// Type alias for backward compatibility.
pub type BufferDestination = Buffer;

/// In-memory UTF-8 string destination (`String`).
#[derive(Debug, Clone, Default)]
pub struct StringDestination {
    buffer: String,
}

impl StringDestination {
    /// Creates a new empty string destination.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Creates a string destination with initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Returns a string slice reference.
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Consumes and returns the inner string.
    pub fn into_string(self) -> String {
        self.buffer
    }
}

impl IDestination for StringDestination {
    fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte as char);
    }

    fn add_bytes(&mut self, bytes: &str) {
        self.buffer.push_str(bytes);
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn last(&self) -> Option<u8> {
        self.buffer.as_bytes().last().copied()
    }
}

#[cfg(feature = "file-io")]
/// File output destination writing directly to a file on disk.
pub struct FileDestination {
    writer: std::io::BufWriter<std::fs::File>,
    last_byte: Option<u8>,
}

#[cfg(feature = "file-io")]
impl FileDestination {
    /// Creates or opens a file for writing.
    pub fn create(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = std::fs::File::create(path)?;
        Ok(Self {
            writer: std::io::BufWriter::new(file),
            last_byte: None,
        })
    }

    /// Flushes the inner buffered writer.
    pub fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.writer.flush()
    }
}

#[cfg(feature = "file-io")]
impl IDestination for FileDestination {
    fn add_byte(&mut self, byte: u8) {
        use std::io::Write;
        let _ = self.writer.write_all(&[byte]);
        self.last_byte = Some(byte);
    }

    fn add_bytes(&mut self, bytes: &str) {
        use std::io::Write;
        let _ = self.writer.write_all(bytes.as_bytes());
        self.last_byte = bytes.as_bytes().last().copied();
    }

    fn clear(&mut self) {
        use std::io::{Seek, SeekFrom, Write};
        let _ = self.writer.flush();
        let file = self.writer.get_mut();
        let _ = file.set_len(0);
        let _ = file.seek(SeekFrom::Start(0));
        self.last_byte = None;
    }

    fn last(&self) -> Option<u8> {
        self.last_byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_destination() {
        let mut dest = BufferDestination::new();
        dest.add_byte(b'h');
        dest.add_bytes("ello");
        assert_eq!(dest.as_bytes(), b"hello");
        assert_eq!(dest.last(), Some(b'o'));
        dest.clear();
        assert_eq!(dest.last(), None);
    }

    #[test]
    fn test_string_destination() {
        let mut dest = StringDestination::new();
        dest.add_bytes("testing");
        assert_eq!(dest.as_str(), "testing");
        assert_eq!(dest.into_string(), "testing");
    }
}
