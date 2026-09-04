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
