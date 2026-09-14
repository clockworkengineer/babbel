//! Error types for BSON encoding and decoding.

#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use core::fmt;
use babbel_core::{BabbelError, ErrorCode};

/// Detailed error encountered during BSON serialization or deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BsonError {
    /// Unexpected end of input while reading bytes.
    UnexpectedEof { expected: usize, available: usize },
    /// Invalid type tag marker encountered.
    InvalidTypeMarker(u8),
    /// UTF-8 validation error in string payload.
    InvalidUtf8,
    /// Null-terminated CString was missing trailing null byte.
    InvalidCString,
    /// Document length header is invalid or exceeds available buffer.
    InvalidDocumentLength { length: i32, available: usize },
    /// Recursion depth limit exceeded.
    RecursionLimitExceeded(usize),
    /// Container or document size exceeded maximum allowed limit.
    SizeLimitExceeded { size: usize, limit: usize },
    /// Custom syntax or semantic error.
    Custom(&'static str),
}

impl fmt::Display for BsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof { expected, available } => {
                write!(f, "unexpected end of input: expected {} bytes, but only {} available", expected, available)
            }
            Self::InvalidTypeMarker(byte) => {
                write!(f, "invalid BSON element type marker: 0x{:02x}", byte)
            }
            Self::InvalidUtf8 => {
                write!(f, "invalid UTF-8 sequence in BSON string")
            }
            Self::InvalidCString => {
                write!(f, "invalid null-terminated CString in BSON element key")
            }
            Self::InvalidDocumentLength { length, available } => {
                write!(f, "invalid BSON document length: header specified {} bytes, but only {} available", length, available)
            }
            Self::RecursionLimitExceeded(depth) => {
                write!(f, "recursion depth limit exceeded: {}", depth)
            }
            Self::SizeLimitExceeded { size, limit } => {
                write!(f, "payload size {} exceeds limit of {} bytes", size, limit)
            }
            Self::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BsonError {}

impl From<BsonError> for BabbelError {
    fn from(err: BsonError) -> Self {
        match err {
            BsonError::InvalidUtf8 => {
                BabbelError::encoding(err.to_string()).with_format("bson")
            }
            BsonError::UnexpectedEof { .. } => {
                BabbelError::eof(err.to_string()).with_format("bson")
            }
            BsonError::RecursionLimitExceeded(_) | BsonError::SizeLimitExceeded { .. } => {
                BabbelError::new(ErrorCode::Custom, err.to_string()).with_format("bson")
            }
            _ => {
                BabbelError::syntax(err.to_string()).with_format("bson")
            }
        }
    }
}
