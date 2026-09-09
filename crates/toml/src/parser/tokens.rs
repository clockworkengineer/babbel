//! Token types for the TOML lexical scanner.

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::nodes::TomlDatetime;

/// Lexical token produced by the TOML lexer.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// Identifier / key (bare or unescaped quoted key)
    Key(String),
    /// String scalar value
    String(String),
    /// 64-bit integer
    Integer(i64),
    /// 64-bit float
    Float(f64),
    /// Boolean value (`true` or `false`)
    Boolean(bool),
    /// RFC 3339 Date/Time
    Datetime(TomlDatetime),
    /// `=`
    Equals,
    /// `.`
    Period,
    /// `,`
    Comma,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// Newline separator
    Newline,
    /// End of stream
    Eof,
}

/// Token paired with source location for detailed error reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}
