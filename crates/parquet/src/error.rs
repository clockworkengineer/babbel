//! Error types for Parquet columnar decoding and encoding.

use core::fmt;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

use babbel_core::{BabbelError, ErrorCode};

/// Error variants encountered during Parquet decoding or encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParquetError {
    /// File does not start or end with valid 'PAR1' magic bytes.
    InvalidMagic,
    /// Unexpected end of file or data page.
    UnexpectedEof,
    /// Thrift compact protocol decoding or encoding error.
    ThriftError(String),
    /// Column page header or compressed payload is corrupted.
    CorruptedPage(String),
    /// Parquet data type is not supported.
    UnsupportedType(String),
    /// Column encoding is not supported.
    UnsupportedEncoding(String),
    /// Row object fields do not match column schema.
    SchemaMismatch(String),
    /// String payload is not valid UTF-8.
    InvalidUtf8,
    /// General serialization/deserialization error.
    General(String),
}

impl fmt::Display for ParquetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParquetError::InvalidMagic => write!(f, "Invalid Parquet file: missing 'PAR1' magic bytes"),
            ParquetError::UnexpectedEof => write!(f, "Unexpected end of input in Parquet stream"),
            ParquetError::ThriftError(msg) => write!(f, "Thrift metadata error: {}", msg),
            ParquetError::CorruptedPage(msg) => write!(f, "Corrupted Parquet page: {}", msg),
            ParquetError::UnsupportedType(msg) => write!(f, "Unsupported Parquet type: {}", msg),
            ParquetError::UnsupportedEncoding(msg) => write!(f, "Unsupported Parquet encoding: {}", msg),
            ParquetError::SchemaMismatch(msg) => write!(f, "Parquet schema mismatch: {}", msg),
            ParquetError::InvalidUtf8 => write!(f, "Parquet string column contains invalid UTF-8"),
            ParquetError::General(msg) => write!(f, "Parquet error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParquetError {}

impl From<ParquetError> for BabbelError {
    fn from(err: ParquetError) -> Self {
        match &err {
            ParquetError::UnexpectedEof => BabbelError::eof(err.to_string()).with_format("parquet"),
            ParquetError::InvalidUtf8 => BabbelError::encoding(err.to_string()).with_format("parquet"),
            ParquetError::InvalidMagic => BabbelError::syntax(err.to_string()).with_format("parquet"),
            _ => BabbelError::new(ErrorCode::Custom, err.to_string()).with_format("parquet"),
        }
    }
}
