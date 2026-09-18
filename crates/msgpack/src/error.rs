//! Error types for MessagePack encoding and decoding.

#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use babbel_core::{BabbelError, ErrorCode};
use core::fmt;

/// Detailed error encountered during MessagePack serialization or deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MsgPackError {
    /// Unexpected end of input while reading bytes.
    UnexpectedEof { expected: usize, available: usize },
    /// Invalid format byte marker encountered.
    InvalidMarker(u8),
    /// UTF-8 validation error in string payload.
    InvalidUtf8,
    /// Recursion depth limit exceeded.
    RecursionLimitExceeded(usize),
    /// Container size exceeded maximum allowed limit.
    SizeLimitExceeded { size: usize, limit: usize },
    /// Unsupported or invalid extension type.
    InvalidExtension { ext_type: i8, len: usize },
    /// Map key was not a string or convertible type.
    InvalidMapKey,
    /// Custom syntax or semantic error.
    Custom(&'static str),
}

impl fmt::Display for MsgPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof {
                expected,
                available,
            } => {
                write!(
                    f,
                    "unexpected end of input: expected {} bytes, but only {} available",
                    expected, available
                )
            }
            Self::InvalidMarker(byte) => {
                write!(f, "invalid MessagePack marker: 0x{:02x}", byte)
            }
            Self::InvalidUtf8 => {
                write!(f, "invalid UTF-8 sequence in MessagePack string")
            }
            Self::RecursionLimitExceeded(depth) => {
                write!(f, "recursion depth limit exceeded: {}", depth)
            }
            Self::SizeLimitExceeded { size, limit } => {
                write!(f, "payload size {} exceeds limit of {} bytes", size, limit)
            }
            Self::InvalidExtension { ext_type, len } => {
                write!(f, "invalid extension: type {}, length {}", ext_type, len)
            }
            Self::InvalidMapKey => {
                write!(f, "map key must be a string or valid scalar")
            }
            Self::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for MsgPackError {}

impl From<MsgPackError> for BabbelError {
    fn from(err: MsgPackError) -> Self {
        match err {
            MsgPackError::InvalidUtf8 => {
                BabbelError::encoding(err.to_string()).with_format("msgpack")
            }
            MsgPackError::UnexpectedEof { .. } => {
                BabbelError::eof(err.to_string()).with_format("msgpack")
            }
            MsgPackError::RecursionLimitExceeded(_) | MsgPackError::SizeLimitExceeded { .. } => {
                BabbelError::new(ErrorCode::Custom, err.to_string()).with_format("msgpack")
            }
            _ => BabbelError::syntax(err.to_string()).with_format("msgpack"),
        }
    }
}
