use super::traits::{IByteStream, ICharStream, ILineReader, IPositionAware, IRewindable, ISource};
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

    /// Advances to the next character.
    pub fn next(&mut self) {
        if let Some((idx, ch)) = self.chars.next() {
            self.pos = idx;
            self.current_char = Some(ch);
        } else {
            self.pos = self.data.len();
            self.current_char = None;
        }
    }

    /// Returns the character at the current position.
    pub fn current(&mut self) -> Option<char> {
        self.current_char
    }

    /// Checks if more characters are available.
    pub fn more(&mut self) -> bool {
        self.current_char.is_some()
    }

    /// Resets reading position to the beginning.
    pub fn reset(&mut self) {
        self.chars = self.data.char_indices();
        self.pos = 0;
        self.next();
    }

    /// Returns absolute byte offset in slice.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Reads the next line into a String without trailing newline.
    pub fn read_line(&mut self) -> Option<String> {
        let mut s = String::new();
        if self.read_line_into(&mut s) {
            Some(s)
        } else {
            None
        }
    }

    /// Reads the next line into an existing buffer. Returns false at EOF.
    pub fn read_line_into(&mut self, buf: &mut String) -> bool {
        if !self.more() {
            return false;
        }
        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.next();
                break;
            } else if ch == '\r' {
                self.next();
                if self.current() == Some('\n') {
                    self.next();
                }
                break;
            } else {
                buf.push(ch);
                self.next();
            }
        }
        true
    }

    /// Zero-copy read of the next line as a borrowed string slice without newline characters.
    pub fn read_line_slice(&mut self) -> Option<&'a str> {
        if !self.more() {
            return None;
        }
        let start = self.pos;
        let mut end = self.pos;
        while let Some(ch) = self.current() {
            if ch == '\n' {
                end = self.pos;
                self.next();
                return Some(&self.data[start..end]);
            } else if ch == '\r' {
                end = self.pos;
                self.next();
                if self.current() == Some('\n') {
                    self.next();
                }
                return Some(&self.data[start..end]);
            } else {
                self.next();
                end = self.pos;
            }
        }
        Some(&self.data[start..end])
    }
}

impl<'a> ISource for SliceSource<'a> {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
    fn reset(&mut self) {
        self.reset();
    }
}

impl<'a> ICharStream for SliceSource<'a> {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
}

impl<'a> IRewindable for SliceSource<'a> {
    fn reset(&mut self) {
        self.reset();
    }
}

impl<'a> IPositionAware for SliceSource<'a> {
    fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> ILineReader for SliceSource<'a> {
    fn read_line_into(&mut self, buf: &mut String) -> bool {
        self.read_line_into(buf)
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

    /// Advances to the next character.
    pub fn next(&mut self) {
        if self.pos < self.content.len() {
            if let Some(ch) = self.content[self.pos..].chars().next() {
                self.pos += ch.len_utf8();
            }
        }
    }

    /// Returns the current character.
    pub fn current(&mut self) -> Option<char> {
        if self.pos < self.content.len() {
            self.content[self.pos..].chars().next()
        } else {
            None
        }
    }

    /// Checks if more characters are available.
    pub fn more(&mut self) -> bool {
        self.pos < self.content.len()
    }

    /// Resets reading position.
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Returns current byte offset in source string.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Reads the next line into a String without trailing newline.
    pub fn read_line(&mut self) -> Option<String> {
        let mut s = String::new();
        if self.read_line_into(&mut s) {
            Some(s)
        } else {
            None
        }
    }

    /// Reads the next line into an existing buffer. Returns false at EOF.
    pub fn read_line_into(&mut self, buf: &mut String) -> bool {
        if !self.more() {
            return false;
        }
        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.next();
                break;
            } else if ch == '\r' {
                self.next();
                if self.current() == Some('\n') {
                    self.next();
                }
                break;
            } else {
                buf.push(ch);
                self.next();
            }
        }
        true
    }
}

impl IPositionAware for StringSource {
    fn position(&self) -> usize {
        self.pos
    }
}

impl ISource for StringSource {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
    fn reset(&mut self) {
        self.reset();
    }
}

impl ICharStream for StringSource {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
}

impl IRewindable for StringSource {
    fn reset(&mut self) {
        self.reset();
    }
}

impl ILineReader for StringSource {
    fn read_line_into(&mut self, buf: &mut String) -> bool {
        self.read_line_into(buf)
    }
}

/// Raw byte slice reader implementing `IByteStream` and `IByteReader`.
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

impl<'a> IPositionAware for ByteSliceSource<'a> {
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

impl<'a> IRewindable for ByteSliceSource<'a> {
    fn reset(&mut self) {
        self.pos = 0;
    }
}

/// In-memory byte vector input source supporting both binary byte streaming
/// and Unicode UTF-8 character streaming.
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

    /// Creates a new BufferSource by taking ownership of a byte vector (zero-copy).
    pub fn from_vec(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// Advances to the next character.
    pub fn next(&mut self) {
        if self.position < self.buffer.len() {
            if let Some(ch) = self.current() {
                let len = ch.len_utf8();
                if len > 0 && self.position + len <= self.buffer.len() {
                    self.position += len;
                    return;
                }
            }
            self.position += 1;
        }
    }

    /// Returns the character at the current position.
    pub fn current(&mut self) -> Option<char> {
        if self.position >= self.buffer.len() {
            return None;
        }
        match core::str::from_utf8(&self.buffer[self.position..]) {
            Ok(s) => s.chars().next(),
            Err(e) => {
                if e.valid_up_to() > 0 {
                    let valid = unsafe { core::str::from_utf8_unchecked(&self.buffer[self.position..self.position + e.valid_up_to()]) };
                    valid.chars().next()
                } else {
                    Some(self.buffer[self.position] as char)
                }
            }
        }
    }

    /// Checks if more characters are available.
    pub fn more(&mut self) -> bool {
        self.position < self.buffer.len()
    }

    /// Resets the position to 0.
    pub fn reset(&mut self) {
        self.position = 0;
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

    /// Returns a slice of the underlying buffer.
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns current byte offset.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Reads the next line into a String without trailing newline.
    pub fn read_line(&mut self) -> Option<String> {
        let mut s = String::new();
        if self.read_line_into(&mut s) {
            Some(s)
        } else {
            None
        }
    }

    /// Reads the next line into an existing buffer. Returns false at EOF.
    pub fn read_line_into(&mut self, buf: &mut String) -> bool {
        if !self.more() {
            return false;
        }
        while let Some(ch) = self.current() {
            if ch == '\n' {
                self.next();
                break;
            } else if ch == '\r' {
                self.next();
                if self.current() == Some('\n') {
                    self.next();
                }
                break;
            } else {
                buf.push(ch);
                self.next();
            }
        }
        true
    }
}

impl IPositionAware for BufferSource {
    fn position(&self) -> usize {
        self.position
    }
}

impl ISource for BufferSource {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
    fn reset(&mut self) {
        self.reset();
    }
}

impl ICharStream for BufferSource {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
}

impl IRewindable for BufferSource {
    fn reset(&mut self) {
        self.reset();
    }
}

impl ILineReader for BufferSource {
    fn read_line_into(&mut self, buf: &mut String) -> bool {
        self.read_line_into(buf)
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
/// File input source reading binary or text data from disk.
#[derive(Debug, Clone)]
pub struct FileSource {
    path: std::path::PathBuf,
    inner: BufferSource,
}

#[cfg(feature = "file-io")]
impl FileSource {
    /// Opens and reads an entire file into memory as raw bytes.
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Self::open(path)
    }

    /// Opens and reads an entire file into memory as raw bytes (zero copy).
    pub fn open(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        let bytes = std::fs::read(&p)?;
        Ok(Self {
            path: p,
            inner: BufferSource::from_vec(bytes),
        })
    }

    /// Advances to the next character.
    pub fn next(&mut self) {
        self.inner.next();
    }

    /// Returns the character at the current position.
    pub fn current(&mut self) -> Option<char> {
        self.inner.current()
    }

    /// Checks if more characters are available.
    pub fn more(&mut self) -> bool {
        self.inner.more()
    }

    /// Resets reading position.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Returns the file path name.
    pub fn file_name(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    /// Reads the next line into a String without trailing newline.
    pub fn read_line(&mut self) -> Option<String> {
        self.inner.read_line()
    }

    /// Reads the next line into an existing buffer. Returns false at EOF.
    pub fn read_line_into(&mut self, buf: &mut String) -> bool {
        self.inner.read_line_into(buf)
    }

    /// Returns the current byte position in the file.
    pub fn position(&self) -> usize {
        self.inner.position()
    }

    /// Returns buffer as UTF-8 string.
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Peeks at current byte without advancing.
    pub fn peek_byte(&mut self) -> Option<u8> {
        self.inner.peek_byte()
    }

    /// Reads current byte and advances position by one byte.
    pub fn read_byte(&mut self) -> Option<u8> {
        self.inner.read_byte()
    }

    /// Advances position by one byte.
    pub fn advance(&mut self) {
        self.inner.advance();
    }

    /// Checks if there are more bytes available.
    pub fn has_more(&mut self) -> bool {
        self.inner.has_more()
    }
}

#[cfg(feature = "file-io")]
impl IPositionAware for FileSource {
    fn position(&self) -> usize {
        self.inner.position()
    }
}

#[cfg(feature = "file-io")]
impl ISource for FileSource {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
    fn reset(&mut self) {
        self.reset();
    }
}

#[cfg(feature = "file-io")]
impl ICharStream for FileSource {
    fn next(&mut self) {
        self.next();
    }
    fn current(&mut self) -> Option<char> {
        self.current()
    }
    fn more(&mut self) -> bool {
        self.more()
    }
}

#[cfg(feature = "file-io")]
impl IByteStream for FileSource {
    fn peek_byte(&mut self) -> Option<u8> {
        self.inner.peek_byte()
    }
    fn read_byte(&mut self) -> Option<u8> {
        self.inner.read_byte()
    }
    fn advance(&mut self) {
        self.inner.advance();
    }
    fn has_more(&mut self) -> bool {
        self.inner.has_more()
    }
}

#[cfg(feature = "file-io")]
impl IRewindable for FileSource {
    fn reset(&mut self) {
        self.reset();
    }
}

#[cfg(feature = "file-io")]
impl ILineReader for FileSource {
    fn read_line_into(&mut self, buf: &mut String) -> bool {
        self.inner.read_line_into(buf)
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

    #[test]
    fn test_buffer_source_utf8() {
        let mut src = BufferSource::new("hello 🦀".as_bytes());
        assert_eq!(src.current(), Some('h'));
        src.next();
        assert_eq!(src.current(), Some('e'));
        src.next();
        src.next();
        src.next();
        src.next(); // at space
        assert_eq!(src.current(), Some(' '));
        src.next(); // at crab emoji (4-byte UTF-8)
        assert_eq!(src.current(), Some('🦀'));
        src.next();
        assert_eq!(src.current(), None);
        assert!(!src.more());
    }

    #[test]
    fn test_line_reading_crlf_and_lf() {
        let text = "First line\r\nSecond line\nThird line\rFourth line";
        let mut src = SliceSource::new(text);
        assert_eq!(src.read_line().as_deref(), Some("First line"));
        assert_eq!(src.read_line().as_deref(), Some("Second line"));
        assert_eq!(src.read_line().as_deref(), Some("Third line"));
        assert_eq!(src.read_line().as_deref(), Some("Fourth line"));
        assert_eq!(src.read_line(), None);

        let mut buf_src = BufferSource::new(text.as_bytes());
        let lines: Vec<String> = buf_src.lines().collect();
        assert_eq!(lines, vec!["First line", "Second line", "Third line", "Fourth line"]);

        let mut slice_src = SliceSource::new(text);
        assert_eq!(slice_src.read_line_slice(), Some("First line"));
        assert_eq!(slice_src.read_line_slice(), Some("Second line"));
        assert_eq!(slice_src.read_line_slice(), Some("Third line"));
        assert_eq!(slice_src.read_line_slice(), Some("Fourth line"));
        assert_eq!(slice_src.read_line_slice(), None);
    }
}
