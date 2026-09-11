//! Error location, span, and diagnostic formatting utilities.

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Accurate 1-based source code location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Location {
    /// 1-based line index
    pub line: usize,
    /// 1-based column index
    pub column: usize,
    /// 0-based absolute byte offset
    pub byte_offset: usize,
}

impl Location {
    /// Creates a new location marker.
    pub const fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            column,
            byte_offset,
        }
    }
}

/// Source code span identifying start and end locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start location
    pub start: Location,
    /// End location
    pub end: Location,
}

impl Span {
    /// Creates a new span.
    pub const fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }
}

/// Standardized diagnostic error codes across format parsers, encoders, and validators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Syntax or grammatical parse failure
    SyntaxError,
    /// Unexpected end of input stream
    UnexpectedEof,
    /// Invalid text encoding (e.g. malformed UTF-8/UTF-16)
    InvalidEncoding,
    /// Schema, DTD, or structural validation failure
    SchemaValidation,
    /// Type or data model mismatch
    UnsupportedType,
    /// Low-level I/O failure
    IoError,
    /// Application-level or custom error
    Custom,
}

/// Unified, structured error type for polyglot serialization and parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct BabbelError {
    /// High-level diagnostic category
    pub code: ErrorCode,
    /// Human-readable explanation of error
    pub message: String,
    /// Optional source location or span
    pub span: Option<Span>,
    /// Optional underlying format identifier
    pub format: Option<&'static str>,
}

impl BabbelError {
    /// Creates a new `BabbelError`.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
            format: None,
        }
    }

    /// Sets the source span for this error.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Sets the format identifier for this error.
    pub fn with_format(mut self, format: &'static str) -> Self {
        self.format = Some(format);
        self
    }

    /// Creates a syntax error.
    pub fn syntax(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::SyntaxError, message)
    }

    /// Creates an unexpected EOF error.
    pub fn eof(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::UnexpectedEof, message)
    }

    /// Creates an encoding error.
    pub fn encoding(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidEncoding, message)
    }

    /// Creates a custom or unclassified error.
    pub fn custom(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Custom, message)
    }
}

impl core::fmt::Display for BabbelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(fmt) = self.format {
            write!(f, "[{fmt}] ")?;
        }
        write!(f, "{:?}: {}", self.code, self.message)?;
        if let Some(span) = &self.span {
            write!(f, " at line {}:{}", span.start.line, span.start.column)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BabbelError {}

impl From<String> for BabbelError {
    fn from(s: String) -> Self {
        Self::custom(s)
    }
}

impl From<&str> for BabbelError {
    fn from(s: &str) -> Self {
        Self::custom(s)
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for BabbelError {
    fn from(e: std::io::Error) -> Self {
        Self::new(ErrorCode::IoError, e.to_string())
    }
}

/// Renders a rustc-style error message pointing to the exact line and column in the source text.
pub fn format_error_snippet(source: &str, loc: Location, message: &str) -> String {
    let line_str = source.lines().nth(loc.line.saturating_sub(1)).unwrap_or("");
    let line_num = loc.line;
    let col_num = loc.column;
    let indent = col_num.saturating_sub(1);

    format!(
        "error: {}\n  --> line {}:{}\n   |\n{:4} | {}\n   | {:indent$}^\n",
        message,
        line_num,
        col_num,
        line_num,
        line_str,
        "",
        indent = indent
    )
}

/// Standardized parse error constants and formatting helpers.
pub mod messages {
    #[cfg(not(feature = "std"))]
    use alloc::{format, string::String};

    pub const FILE_NOT_FOUND: &str = "File not found";
    pub const ERR_EMPTY_INPUT: &str = "Empty input";
    pub const ERR_UNEXPECTED_CHAR: &str = "Unexpected character: ";
    pub const ERR_OBJECT_KEY: &str = "Object key must be a string";
    pub const ERR_EXPECT_COLON: &str = "Expected ':' after object key";
    pub const ERR_EXPECT_OBJECT_END: &str = "Expected ',' or '}' in object";
    pub const ERR_EXPECT_ARRAY_END: &str = "Expected ',' or ']' in array";
    pub const ERR_INVALID_ESCAPE: &str = "Invalid escape sequence";
    pub const ERR_UNTERMINATED_STRING: &str = "Unterminated string";
    pub const ERR_MULTIPLE_DECIMAL: &str = "Multiple decimal points in number";
    pub const ERR_INVALID_FLOAT: &str = "Invalid float number";
    pub const ERR_INVALID_INTEGER: &str = "Invalid integer number";
    pub const ERR_EXPECT_TRUE: &str = "Expected 'true'";
    pub const ERR_EXPECT_FALSE: &str = "Expected 'false'";
    pub const ERR_EXPECT_NULL: &str = "Expected 'null'";
    pub const ERR_INVALID_UTF8: &str = "Invalid UTF-8 encoding";
    pub const ERR_INVALID_NUMBER: &str = "Invalid number format";
    pub const ERR_UNESCAPED_CONTROL_CHAR: &str = "Unescaped control character in string";

    /// Helper for constructing formatted unexpected character error messages.
    #[inline]
    pub fn unexpected_character(c: char) -> String {
        format!("Unexpected character: {}", c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_rendering() {
        let src = "{\n  \"key\": \"value\",\n  \"bad\": syntax_error\n}";
        let loc = Location::new(3, 10, 25);
        let snippet = format_error_snippet(src, loc, "unexpected token");
        assert!(snippet.contains("line 3:10"));
        assert!(snippet.contains("3 |   \"bad\": syntax_error"));
        assert!(snippet.contains("^"));
    }
}
