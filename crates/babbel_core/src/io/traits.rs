//! Core I/O traits for sequential streaming input and output.

/// Trait defining sequential character reading from an input source.
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

/// Minimal character reading interface adhering to ISP (Interface Segregation Principle).
pub trait ICharStream {
    /// Advances reading position to next character.
    fn next(&mut self);
    /// Returns character at current position.
    fn current(&mut self) -> Option<char>;
    /// Checks if more characters are available.
    fn more(&mut self) -> bool;
}

impl<T: ISource + ?Sized> ICharStream for T {
    fn next(&mut self) {
        ISource::next(self);
    }
    fn current(&mut self) -> Option<char> {
        ISource::current(self)
    }
    fn more(&mut self) -> bool {
        ISource::more(self)
    }
}

/// Interface for streams that support rewinding to the beginning.
pub trait IRewindable {
    /// Resets reading position to the beginning.
    fn reset(&mut self);
}

impl<T: ISource + ?Sized> IRewindable for T {
    fn reset(&mut self) {
        ISource::reset(self);
    }
}

/// Interface for sources that report byte position.
pub trait IPositionAware {
    /// Returns current absolute byte offset.
    fn position(&self) -> usize;
}

/// Interface for raw byte reading (essential for Bencode and binary protocol parsing).
pub trait IByteStream {
    /// Peeks at current byte without advancing.
    fn peek_byte(&mut self) -> Option<u8>;
    /// Reads current byte and advances position by one byte.
    fn read_byte(&mut self) -> Option<u8>;
    /// Advances position by one byte.
    fn advance(&mut self);
    /// Checks if there are more bytes available.
    fn has_more(&mut self) -> bool;
}

/// Interface for destinations that can be cleared or truncated.
pub trait IClearable {
    /// Clears all accumulated content from the destination.
    fn clear(&mut self);
}

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

impl<T: IDestination + ?Sized> IClearable for T {
    fn clear(&mut self) {
        IDestination::clear(self);
    }
}

/// Indentation tracking trait for whitespace-sensitive formats (YAML, pretty-printers).
pub trait IIndentationAware {
    /// Returns current indentation level (number of spaces or tab-stops).
    fn get_current_indent_level(&self) -> usize;
    /// Checks if character is considered indentation whitespace.
    fn is_indent_whitespace(&self, c: char) -> bool {
        c == ' ' || c == '\t'
    }
}

