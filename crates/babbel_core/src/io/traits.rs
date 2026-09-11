//! Core I/O traits for sequential streaming input and output adhering to SOLID principles.

use crate::error::ErrorCode;

// ==========================================
// 1. Primitive Byte Streaming (ISP compliant)
// ==========================================

/// Capability to read sequential bytes (forward byte stream, ISP compliant).
pub trait IByteReader {
    /// Reads current byte and advances position by one byte.
    fn read_byte(&mut self) -> Option<u8>;
}

/// Capability to peek at the next byte without consuming it (ISP compliant).
pub trait IPeekable {
    /// Peeks at current byte without advancing.
    fn peek_byte(&mut self) -> Option<u8>;
}

/// Composite interface for raw byte reading (essential for Bencode and binary protocol parsing).
/// Automatically implemented for any type that satisfies [`IByteReader`] and [`IPeekable`].
pub trait IByteStream: IByteReader + IPeekable {
    /// Advances position by one byte.
    fn advance(&mut self) {
        let _ = self.read_byte();
    }
    /// Checks if there are more bytes available.
    fn has_more(&mut self) -> bool {
        self.peek_byte().is_some()
    }
}

impl<T: IByteReader + IPeekable + ?Sized> IByteStream for T {}

/// Capability to write sequential bytes (binary-safe output interface).
pub trait IByteWriter {
    /// Writes a single byte to the destination.
    fn write_byte(&mut self, byte: u8) -> Result<(), ErrorCode>;
    /// Writes multiple bytes from a raw byte slice to the destination.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ErrorCode>;
}

// ==========================================
// 2. Character Streaming (LSP & ISP compliant)
// ==========================================

/// Minimal character reading interface adhering to ISP (Interface Segregation Principle).
pub trait ICharStream {
    /// Advances reading position to next character.
    fn next(&mut self);
    /// Returns character at current position.
    fn current(&mut self) -> Option<char>;
    /// Checks if more characters are available.
    fn more(&mut self) -> bool;
}

/// Capability to read sequential Unicode characters (ISP alias for ICharStream).
pub trait ICharReader: ICharStream {}
impl<T: ICharStream + ?Sized> ICharReader for T {}

/// Standard sequential character reading source trait (LSP & backward compatibility).
pub trait ISource {
    /// Advances reading position to next character.
    fn next(&mut self);
    /// Returns character at current position.
    fn current(&mut self) -> Option<char>;
    /// Checks if more characters are available.
    fn more(&mut self) -> bool;
    /// Resets reading position to the beginning.
    fn reset(&mut self);
}




// ==========================================
// 3. Segregated Capabilities (ISP compliant)
// ==========================================

/// Interface for streams that support rewinding to the beginning.
pub trait IRewindable {
    /// Resets reading position to the beginning.
    fn reset(&mut self);
}

/// Interface for sources that report byte position.
pub trait IPositionAware {
    /// Returns current absolute byte offset.
    fn position(&self) -> usize;
}

/// Interface for sources tracking 1-based line and column metrics.
pub trait ILocationAware {
    /// Returns 1-based line index.
    fn line(&self) -> usize;
    /// Returns 1-based column index.
    fn column(&self) -> usize;
}

/// Unified tracking trait for 1-based source line, column, and byte offset metrics (ISP compliant).
/// Automatically implemented for any type that satisfies [`ILocationAware`] and [`IPositionAware`].
pub trait ITracked: ILocationAware + IPositionAware {
    /// Returns current absolute byte offset (delegates to [`IPositionAware::position`]).
    fn offset(&self) -> usize {
        self.position()
    }
}

impl<T: ILocationAware + IPositionAware + ?Sized> ITracked for T {}



/// Interface for destinations that can be flushed to underlying storage.
pub trait IFlushable {
    /// Flushes any buffered bytes to destination.
    fn flush(&mut self) -> Result<(), ErrorCode>;
}

/// Interface for destinations that can inspect the last written byte.
pub trait ITailInspectable {
    /// Returns the last written byte, if any.
    fn last_byte(&self) -> Option<u8>;
}

/// Interface for destinations that can be cleared or truncated.
pub trait IClearable {
    /// Clears all accumulated content from the destination.
    fn clear(&mut self);
}

// ==========================================
// 4. Output Destination (ISP & DIP compliant)
// ==========================================

/// Interface for writing data to an output destination.
pub trait IDestination {
    /// Writes a single byte to the destination.
    fn add_byte(&mut self, byte: u8);
    /// Writes multiple bytes from a string slice.
    fn add_bytes(&mut self, bytes: &str);
    /// Clears all accumulated content from the destination.
    fn clear(&mut self);
    /// Returns the last written byte, if any.
    fn last(&self) -> Option<u8>;
}

/// Indentation tracking trait for whitespace-sensitive formats (YAML, pretty-printers).
pub trait IIndentationAware {
    /// Returns current indentation level (number of spaces or tab-stops).
    fn get_current_indent_level(&self) -> usize;
    /// Checks if character is considered indentation whitespace.
    fn is_indent_whitespace(&self, c: char) -> bool {
        c == ' ' || c == '\t'
    }
    /// Checks if character is whitespace (space or tab).
    fn is_whitespace(&self, c: char) -> bool {
        c == ' ' || c == '\t'
    }
    /// Checks if character is a tab.
    fn is_tab(&self, c: char) -> bool {
        c == '\t'
    }
}

/// Concrete save/restore snapshot of stream state and location metrics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SaveState {
    /// Absolute byte position in the underlying source.
    pub pos: u64,
    /// The current byte at that position when the snapshot was taken (if any).
    pub current_byte: Option<u8>,
    /// Column (indent) at the snapshot.
    pub column: usize,
    /// Line number at the snapshot.
    pub line: usize,
}

/// Interface for snapshotting and restoring stream read state.
pub trait IStatefulStream {
    /// Snapshots the source read position and metadata.
    fn save_state(&mut self) -> SaveState;
    /// Restores a previously saved state.
    fn restore_state(&mut self, state: SaveState);
}

// ==========================================
// 5. Line-Oriented Text Reading (ISP compliant)
// ==========================================

/// Interface for character streams supporting line-by-line text reading.
pub trait ILineReader: ICharStream {
    /// Reads the next line up to `\n` or `\r\n` (or lone `\r`), returning it without trailing newline.
    /// Returns `None` when end of stream is reached and no characters were read.
    fn read_line(&mut self) -> Option<alloc::string::String> {
        let mut line = alloc::string::String::new();
        if self.read_line_into(&mut line) {
            Some(line)
        } else {
            None
        }
    }

    /// Reads the next line into an existing buffer to avoid heap allocations.
    /// Appends the line content without newline characters to `buf`.
    /// Returns `true` if a line (including an empty line) was read, `false` at EOF.
    fn read_line_into(&mut self, buf: &mut alloc::string::String) -> bool;

    /// Returns an iterator yielding lines from this stream.
    fn lines(&mut self) -> LineIter<'_, Self>
    where
        Self: Sized,
    {
        LineIter::new(self)
    }
}

/// Iterator over lines produced by an [`ILineReader`].
pub struct LineIter<'a, R: ?Sized> {
    reader: &'a mut R,
}

impl<'a, R: ILineReader + ?Sized> LineIter<'a, R> {
    /// Creates a new line iterator borrowing the reader.
    pub fn new(reader: &'a mut R) -> Self {
        Self { reader }
    }
}

impl<'a, R: ILineReader + ?Sized> Iterator for LineIter<'a, R> {
    type Item = alloc::string::String;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.read_line()
    }
}
