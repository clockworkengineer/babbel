//! Error types for the RON parser and serializer.

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use core::fmt;

use babbel_core::{BabbelError, ErrorCode};

/// Error variants encountered during RON parsing or serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RonError {
    /// Unexpected end of file / stream.
    UnexpectedEof,
    /// Unexpected character at a specific position.
    UnexpectedChar { ch: char, line: usize, col: usize },
    /// Expected a specific token or syntax element.
    Expected {
        expected: &'static str,
        found: String,
        line: usize,
        col: usize,
    },
    /// Syntax error with custom description.
    Syntax {
        message: String,
        line: usize,
        col: usize,
    },
    /// Numeric parsing overflow or invalid syntax.
    InvalidNumber {
        literal: String,
        line: usize,
        col: usize,
    },
    /// String literal escape sequence was invalid.
    InvalidEscape {
        sequence: String,
        line: usize,
        col: usize,
    },
    /// Recursion depth limit exceeded.
    RecursionLimitExceeded { depth: usize, max: usize },
    /// Serialization error.
    Serialization(String),
    /// UTF-8 encoding error.
    InvalidUtf8,
}

impl fmt::Display for RonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RonError::UnexpectedEof => write!(f, "Unexpected end of input in RON document"),
            RonError::UnexpectedChar { ch, line, col } => {
                write!(
                    f,
                    "Unexpected character '{}' at line {}, column {}",
                    ch, line, col
                )
            }
            RonError::Expected {
                expected,
                found,
                line,
                col,
            } => {
                write!(
                    f,
                    "Expected {} but found '{}' at line {}, column {}",
                    expected, found, line, col
                )
            }
            RonError::Syntax { message, line, col } => {
                write!(
                    f,
                    "Syntax error at line {}, column {}: {}",
                    line, col, message
                )
            }
            RonError::InvalidNumber { literal, line, col } => {
                write!(
                    f,
                    "Invalid number '{}' at line {}, column {}",
                    literal, line, col
                )
            }
            RonError::InvalidEscape {
                sequence,
                line,
                col,
            } => {
                write!(
                    f,
                    "Invalid escape sequence '\\{}' at line {}, column {}",
                    sequence, line, col
                )
            }
            RonError::RecursionLimitExceeded { depth, max } => {
                write!(
                    f,
                    "Recursion depth {} exceeded maximum limit of {}",
                    depth, max
                )
            }
            RonError::Serialization(msg) => write!(f, "RON serialization error: {}", msg),
            RonError::InvalidUtf8 => write!(f, "Input is not valid UTF-8"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RonError {}

impl From<RonError> for BabbelError {
    fn from(err: RonError) -> Self {
        match &err {
            RonError::UnexpectedEof => BabbelError::eof(err.to_string()).with_format("ron"),
            RonError::InvalidUtf8 => BabbelError::encoding(err.to_string()).with_format("ron"),
            RonError::RecursionLimitExceeded { .. } => {
                BabbelError::new(ErrorCode::Custom, err.to_string()).with_format("ron")
            }
            _ => BabbelError::syntax(err.to_string()).with_format("ron"),
        }
    }
}
