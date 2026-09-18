//! Error types for Apache Avro encoding, decoding, and OCF processing.

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use babbel_core::BabbelError;
use core::fmt;

/// Error encountered during Avro serialization, deserialization, or container parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvroError {
    /// Unexpected end of input
    UnexpectedEof { expected: usize, available: usize },
    /// Invalid magic bytes for OCF container file
    InvalidMagic,
    /// Invalid or unrecognized schema structure
    InvalidSchema(String),
    /// Invalid variable-length zigzag integer
    InvalidVarint,
    /// Invalid UTF-8 sequence
    InvalidUtf8,
    /// Sync marker mismatch in OCF block
    SyncMarkerMismatch,
    /// Custom error message
    Custom(&'static str),
}

impl fmt::Display for AvroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof {
                expected,
                available,
            } => {
                write!(
                    f,
                    "unexpected end of Avro input: expected {} bytes, available {}",
                    expected, available
                )
            }
            Self::InvalidMagic => write!(f, "invalid Avro OCF magic header (expected 'Obj\\x01')"),
            Self::InvalidSchema(s) => write!(f, "invalid Avro schema: {}", s),
            Self::InvalidVarint => write!(f, "invalid variable-length zigzag varint"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in Avro string"),
            Self::SyncMarkerMismatch => {
                write!(f, "Avro OCF 16-byte sync marker mismatch between blocks")
            }
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AvroError {}

impl From<AvroError> for BabbelError {
    fn from(err: AvroError) -> Self {
        match &err {
            AvroError::UnexpectedEof { .. } => {
                BabbelError::eof(err.to_string()).with_format("avro")
            }
            AvroError::InvalidUtf8 => BabbelError::encoding(err.to_string()).with_format("avro"),
            _ => BabbelError::syntax(err.to_string()).with_format("avro"),
        }
    }
}
