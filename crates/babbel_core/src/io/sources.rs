//! Concrete implementations of input sources.

use super::traits::{IByteStream, ISource};
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Zero-copy input source reading from an in-memory byte or UTF-8 slice.
#[derive(Debug, Clone)]
pub struct SliceSource<'a> {
    data: &'a str,
    chars: core::str::CharIndices<'a>,
    current_char: Option<char>,
    pos: usize,
}

impl<'a> SliceSource<'a> {
    /// Creates a new `SliceSource` wrapping a string slice.
    pub fn new(data: &'a str) -> Self {
        let mut s = Self {
            data,
            chars: data.char_indices(),
            current_char: None,
            pos: 0,
        };
        s.next();
        s
    }

    /// Returns absolute byte offset in slice.
    pub fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> ISource for SliceSource<'a> {
    fn next(&mut self) {
        if let Some((idx, ch)) = self.chars.next() {
            self.pos = idx;
            self.current_char = Some(ch);
        } else {
            self.pos = self.data.len();
            self.current_char = None;
        }
    }

    fn current(&mut self) -> Option<char> {
        self.current_char
    }

    fn more(&mut self) -> bool {
        self.current_char.is_some()
    }

    fn reset(&mut self) {
        self.chars = self.data.char_indices();
        self.pos = 0;
        self.next();
    }
}

impl<'a> super::traits::IPositionAware for SliceSource<'a> {
    fn position(&self) -> usize {
        self.pos
    }
}

/// Owned string input source.
#[derive(Debug, Clone)]
pub struct StringSource {
    content: String,
    pos: usize,
}

impl StringSource {
    /// Creates a new `StringSource`.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            pos: 0,
        }
    }

    /// Returns current byte offset in source string.
    pub fn position(&self) -> usize {
        self.pos
    }
}

impl super::traits::IPositionAware for StringSource {
    fn position(&self) -> usize {
        self.pos
    }
}

impl ISource for StringSource {
    fn next(&mut self) {
        if self.pos < self.content.len() {
            if let Some(ch) = self.content[self.pos..].chars().next() {
                self.pos += ch.len_utf8();
            }
        }
    }

    fn current(&mut self) -> Option<char> {
        if self.pos < self.content.len() {
            self.content[self.pos..].chars().next()
        } else {
            None
        }
    }

    fn more(&mut self) -> bool {
        self.pos < self.content.len()
    }

    fn reset(&mut self) {
        self.pos = 0;
    }
}

/// Raw byte slice reader implementing `IByteStream`.
#[derive(Debug, Clone)]
pub struct ByteSliceSource<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> ByteSliceSource<'a> {
    /// Creates a new byte slice source.
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice, pos: 0 }
    }

    /// Returns current byte offset.
    pub fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> super::traits::IPositionAware for ByteSliceSource<'a> {
    fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> IByteStream for ByteSliceSource<'a> {
    fn peek_byte(&mut self) -> Option<u8> {
        self.slice.get(self.pos).copied()
    }

    fn read_byte(&mut self) -> Option<u8> {
        let b = self.slice.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }

    fn advance(&mut self) {
        if self.pos < self.slice.len() {
            self.pos += 1;
        }
    }

    fn has_more(&mut self) -> bool {
        self.pos < self.slice.len()
    }
}

/// In-memory byte vector input source.
#[derive(Debug, Clone, Default)]
pub struct BufferSource {
    buffer: Vec<u8>,
    position: usize,
}

impl BufferSource {
    /// Creates a new BufferSource from a byte slice.
    pub fn new(data: &[u8]) -> Self {
        Self {
            buffer: data.to_vec(),
            position: 0,
        }
    }

    /// Converts the buffer to a UTF-8 string.
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

    /// Returns current byte offset.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Resets the position to 0.
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

impl super::traits::IPositionAware for BufferSource {
    fn position(&self) -> usize {
        self.position
    }
}

impl ISource for BufferSource {
    fn next(&mut self) {
        self.position += 1;
    }

    fn current(&mut self) -> Option<char> {
        if self.more() {
            Some(self.buffer[self.position] as char)
        } else {
            None
        }
    }

    fn more(&mut self) -> bool {
        self.position < self.buffer.len()
    }

    fn reset(&mut self) {
        self.position = 0;
    }
}

impl IByteStream for BufferSource {
    fn peek_byte(&mut self) -> Option<u8> {
        self.buffer.get(self.position).copied()
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.position < self.buffer.len() {
            let b = self.buffer[self.position];
            self.position += 1;
            Some(b)
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

#[cfg(feature = "file-io")]
/// File input source reading from disk.
pub struct FileSource {
    _content: String,
    inner: StringSource,
}

#[cfg(feature = "file-io")]
impl FileSource {
    /// Opens and reads an entire file into memory as UTF-8.
    pub fn open(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let inner = StringSource::new(content.clone());
        Ok(Self {
            _content: content,
            inner,
        })
    }
}

#[cfg(feature = "file-io")]
impl ISource for FileSource {
    fn next(&mut self) {
        self.inner.next();
    }
    fn current(&mut self) -> Option<char> {
        self.inner.current()
    }
    fn more(&mut self) -> bool {
        self.inner.more()
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_source() {
        let mut src = SliceSource::new("abc");
        assert_eq!(src.current(), Some('a'));
        src.next();
        assert_eq!(src.current(), Some('b'));
        src.next();
        assert_eq!(src.current(), Some('c'));
        src.next();
        assert_eq!(src.current(), None);
        assert!(!src.more());
        src.reset();
        assert_eq!(src.current(), Some('a'));
    }

    #[test]
    fn test_byte_slice_source() {
        let mut src = ByteSliceSource::new(b"xyz");
        assert_eq!(src.peek_byte(), Some(b'x'));
        assert_eq!(src.read_byte(), Some(b'x'));
        assert_eq!(src.read_byte(), Some(b'y'));
        assert_eq!(src.read_byte(), Some(b'z'));
        assert_eq!(src.read_byte(), None);
        assert!(!src.has_more());
    }
}
