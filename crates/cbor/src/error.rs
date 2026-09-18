//! Error types for CBOR (RFC 8949) encoding and decoding.

#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use babbel_core::{BabbelError, ErrorCode};
use core::fmt;

/// Detailed error encountered during CBOR serialization or deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CborError {
    /// Unexpected end of input while reading bytes.
    UnexpectedEof { expected: usize, available: usize },
    /// Invalid initial byte or reserved additional information.
    InvalidInitialByte(u8),
    /// UTF-8 validation error in string payload.
    InvalidUtf8,
    /// Recursion depth limit exceeded.
    RecursionLimitExceeded(usize),
    /// Container size exceeded maximum allowed limit.
    SizeLimitExceeded { size: usize, limit: usize },
    /// Unsupported or invalid simple value.
    UnsupportedSimpleValue(u8),
    /// Unexpected break marker outside indefinite container.
    UnexpectedBreak,
    /// Map key was not a string or convertible scalar.
    InvalidMapKey,
    /// Custom syntax or semantic error.
    Custom(&'static str),
}

impl fmt::Display for CborError {
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
            Self::InvalidInitialByte(byte) => {
                write!(f, "invalid CBOR initial byte: 0x{:02x}", byte)
            }
            Self::InvalidUtf8 => {
                write!(f, "invalid UTF-8 sequence in CBOR text string")
            }
            Self::RecursionLimitExceeded(depth) => {
                write!(f, "recursion depth limit exceeded: {}", depth)
            }
            Self::SizeLimitExceeded { size, limit } => {
                write!(f, "payload size {} exceeds limit of {} bytes", size, limit)
            }
            Self::UnsupportedSimpleValue(v) => {
                write!(f, "unsupported CBOR simple value: {}", v)
            }
            Self::UnexpectedBreak => {
                write!(f, "unexpected CBOR break stop code")
            }
            Self::InvalidMapKey => {
                write!(f, "map key must be a string or convertible scalar")
            }
            Self::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CborError {}

impl From<CborError> for BabbelError {
    fn from(err: CborError) -> Self {
        match err {
            CborError::InvalidUtf8 => BabbelError::encoding(err.to_string()).with_format("cbor"),
            CborError::UnexpectedEof { .. } => {
                BabbelError::eof(err.to_string()).with_format("cbor")
            }
            CborError::RecursionLimitExceeded(_) | CborError::SizeLimitExceeded { .. } => {
                BabbelError::new(ErrorCode::Custom, err.to_string()).with_format("cbor")
            }
            _ => BabbelError::syntax(err.to_string()).with_format("cbor"),
        }
    }
}
