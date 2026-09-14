//! Error types for the KDL parser and serializer.

use core::fmt;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

use babbel_core::{BabbelError, ErrorCode};

/// Error variants encountered during KDL parsing or serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KdlError {
    /// Unexpected end of file / stream.
    UnexpectedEof,
    /// Unexpected character at a specific position.
    UnexpectedChar {
        ch: char,
        line: usize,
        col: usize,
    },
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
    RecursionLimitExceeded {
        depth: usize,
        max: usize,
    },
    /// Serialization error.
    Serialization(String),
    /// UTF-8 encoding error.
    InvalidUtf8,
}

impl fmt::Display for KdlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KdlError::UnexpectedEof => write!(f, "Unexpected end of input in KDL document"),
            KdlError::UnexpectedChar { ch, line, col } => {
                write!(f, "Unexpected character '{}' at line {}, column {}", ch, line, col)
            }
            KdlError::Expected { expected, found, line, col } => {
                write!(f, "Expected {} but found '{}' at line {}, column {}", expected, found, line, col)
            }
            KdlError::Syntax { message, line, col } => {
                write!(f, "Syntax error at line {}, column {}: {}", line, col, message)
            }
            KdlError::InvalidNumber { literal, line, col } => {
                write!(f, "Invalid number '{}' at line {}, column {}", literal, line, col)
            }
            KdlError::InvalidEscape { sequence, line, col } => {
                write!(f, "Invalid escape sequence '\\{}' at line {}, column {}", sequence, line, col)
            }
            KdlError::RecursionLimitExceeded { depth, max } => {
                write!(f, "Recursion depth {} exceeded maximum limit of {}", depth, max)
            }
            KdlError::Serialization(msg) => write!(f, "KDL serialization error: {}", msg),
            KdlError::InvalidUtf8 => write!(f, "Input is not valid UTF-8"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for KdlError {}

impl From<KdlError> for BabbelError {
    fn from(err: KdlError) -> Self {
        match &err {
            KdlError::UnexpectedEof => BabbelError::eof(err.to_string()).with_format("kdl"),
            KdlError::InvalidUtf8 => BabbelError::encoding(err.to_string()).with_format("kdl"),
            KdlError::RecursionLimitExceeded { .. } => {
                BabbelError::new(ErrorCode::Custom, err.to_string()).with_format("kdl")
            }
            _ => BabbelError::syntax(err.to_string()).with_format("kdl"),
        }
    }
}
