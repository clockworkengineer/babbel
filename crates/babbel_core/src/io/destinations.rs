//! Concrete implementations of output destinations adhering to SOLID principles.

use super::traits::{IByteWriter, IClearable, IDestination, ITailInspectable};
#[cfg(feature = "file-io")]
use super::traits::IFlushable;
use crate::error::ErrorCode;
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

    /// Appends a single byte.
    pub fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    /// Appends a string of bytes.
    pub fn add_bytes(&mut self, bytes: &str) {
        self.buffer.extend_from_slice(bytes.as_bytes());
    }

    /// Clears all content from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the last written byte.
    pub fn last(&self) -> Option<u8> {
        self.buffer.last().copied()
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

    /// Converts the buffer content into an owned String without reallocating if valid UTF-8.
    pub fn into_string(self) -> Result<String, alloc::string::FromUtf8Error> {
        String::from_utf8(self.buffer)
    }

    /// Returns a string slice of the buffer if it contains valid UTF-8 without allocating.
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.buffer)
    }

    /// Returns byte slice of written content.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Consumes and returns the inner byte vector.
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }
}

impl IByteWriter for Buffer {
    fn write_byte(&mut self, byte: u8) -> Result<(), ErrorCode> {
        self.add_byte(byte);
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }
}

impl ITailInspectable for Buffer {
    fn last_byte(&self) -> Option<u8> {
        self.last()
    }
}

impl IClearable for Buffer {
    fn clear(&mut self) {
        self.clear();
    }
}

impl IDestination for Buffer {
    fn add_byte(&mut self, byte: u8) {
        self.add_byte(byte);
    }
    fn add_bytes(&mut self, bytes: &str) {
        self.add_bytes(bytes);
    }
    fn clear(&mut self) {
        self.clear();
    }
    fn last(&self) -> Option<u8> {
        self.last()
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

    /// Appends a single byte as a character.
    pub fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte as char);
    }

    /// Appends multiple bytes from a string slice.
    pub fn add_bytes(&mut self, bytes: &str) {
        self.buffer.push_str(bytes);
    }

    /// Clears all content from the string.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Returns the last written byte.
    pub fn last(&self) -> Option<u8> {
        self.buffer.as_bytes().last().copied()
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

impl IByteWriter for StringDestination {
    fn write_byte(&mut self, byte: u8) -> Result<(), ErrorCode> {
        self.add_byte(byte);
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        if let Ok(s) = core::str::from_utf8(bytes) {
            self.buffer.push_str(s);
            Ok(())
        } else {
            for &b in bytes {
                self.buffer.push(b as char);
            }
            Ok(())
        }
    }
}

impl ITailInspectable for StringDestination {
    fn last_byte(&self) -> Option<u8> {
        self.last()
    }
}

impl IClearable for StringDestination {
    fn clear(&mut self) {
        self.clear();
    }
}

impl IDestination for StringDestination {
    fn add_byte(&mut self, byte: u8) {
        self.add_byte(byte);
    }
    fn add_bytes(&mut self, bytes: &str) {
        self.add_bytes(bytes);
    }
    fn clear(&mut self) {
        self.clear();
    }
    fn last(&self) -> Option<u8> {
        self.last()
    }
}

#[cfg(feature = "file-io")]
/// File output destination writing to a buffered file on disk (32KB write buffer).
pub struct FileDestination {
    writer: std::io::BufWriter<std::fs::File>,
    path: std::path::PathBuf,
    last_byte: Option<u8>,
    bytes_written: usize,
}

#[cfg(feature = "file-io")]
impl FileDestination {
    /// 32KB buffer for optimal disk write throughput
    const BUFFER_CAPACITY: usize = 32 * 1024;

    /// Creates or opens a file for writing with 32KB buffer.
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Self::create(path)
    }

    /// Creates or opens a file for writing with 32KB buffer.
    pub fn create(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        let file = std::fs::File::create(&p)?;
        let writer = std::io::BufWriter::with_capacity(Self::BUFFER_CAPACITY, file);
        Ok(Self {
            writer,
            path: p,
            last_byte: None,
            bytes_written: 0,
        })
    }

    /// Writes a single byte to the buffered destination.
    pub fn add_byte(&mut self, byte: u8) {
        use std::io::Write;
        let _ = self.writer.write_all(&[byte]);
        self.last_byte = Some(byte);
        self.bytes_written += 1;
    }

    /// Writes multiple bytes from a string slice to the buffered destination.
    pub fn add_bytes(&mut self, bytes: &str) {
        use std::io::Write;
        let _ = self.writer.write_all(bytes.as_bytes());
        self.last_byte = bytes.as_bytes().last().copied();
        self.bytes_written += bytes.len();
    }

    /// Clears all content from the file.
    pub fn clear(&mut self) {
        use std::io::{Seek, SeekFrom, Write};
        let _ = self.writer.flush();
        let file = self.writer.get_mut();
        let _ = file.set_len(0);
        let _ = file.seek(SeekFrom::Start(0));
        self.last_byte = None;
        self.bytes_written = 0;
    }

    /// Returns the last written byte.
    pub fn last(&self) -> Option<u8> {
        self.last_byte
    }

    /// Returns the number of bytes written.
    pub fn file_length(&self) -> usize {
        self.bytes_written
    }

    /// Returns the file path string.
    pub fn file_name(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    /// Flushes the underlying buffered writer and syncs file data.
    pub fn flush(&mut self) -> std::io::Result<()> {
        use std::io::Write;
        self.writer.flush()?;
        self.writer.get_ref().sync_data()
    }

    /// Closes the file handle (auto-flushed on drop).
    pub fn close(&self) -> std::io::Result<()> {
        Ok(())
    }

    /// Writes all bytes to the buffered file.
    pub fn write_all_bytes(&mut self, data: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        self.writer.write_all(data)?;
        self.last_byte = data.last().copied();
        self.bytes_written += data.len();
        Ok(())
    }

    /// Writes a string slice to the buffered file.
    pub fn write_str(&mut self, data: &str) -> std::io::Result<()> {
        self.write_all_bytes(data.as_bytes())
    }
}

#[cfg(feature = "file-io")]
impl Drop for FileDestination {
    fn drop(&mut self) {
        use std::io::Write;
        let _ = self.writer.flush();
    }
}

#[cfg(feature = "file-io")]
impl IByteWriter for FileDestination {
    fn write_byte(&mut self, byte: u8) -> Result<(), ErrorCode> {
        use std::io::Write;
        self.writer.write_all(&[byte]).map_err(|_| ErrorCode::IoError)?;
        self.last_byte = Some(byte);
        self.bytes_written += 1;
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        use std::io::Write;
        self.writer.write_all(bytes).map_err(|_| ErrorCode::IoError)?;
        self.last_byte = bytes.last().copied();
        self.bytes_written += bytes.len();
        Ok(())
    }
}

#[cfg(feature = "file-io")]
impl ITailInspectable for FileDestination {
    fn last_byte(&self) -> Option<u8> {
        self.last()
    }
}

#[cfg(feature = "file-io")]
impl IClearable for FileDestination {
    fn clear(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "file-io")]
impl IFlushable for FileDestination {
    fn flush(&mut self) -> Result<(), ErrorCode> {
        self.flush().map_err(|_| ErrorCode::IoError)
    }
}

#[cfg(feature = "file-io")]
impl IDestination for FileDestination {
    fn add_byte(&mut self, byte: u8) {
        self.add_byte(byte);
    }
    fn add_bytes(&mut self, bytes: &str) {
        self.add_bytes(bytes);
    }
    fn clear(&mut self) {
        self.clear();
    }
    fn last(&self) -> Option<u8> {
        self.last()
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
