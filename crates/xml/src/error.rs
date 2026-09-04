//! # Error Types
//!
//! Custom error types for XML parsing, entity expansion, DTD, XSD, and XPath evaluation.

use crate::alloc_prelude::*;
use core::fmt;

/// Result alias returning [`XmlError`].
pub type Result<T> = core::result::Result<T, XmlError>;

/// Enum representing all XML library error conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlError {
    /// XML syntax or malformed token error.
    SyntaxError {
        message: String,
        line: usize,
        col: usize,
    },

    /// Security limit threshold exceeded.
    SecurityLimitExceeded(String),

    /// Entity resolution or reference expansion error.
    EntityError(String),

    /// DTD constraint validation error.
    DtdError(String),

    /// XSD Schema constraint validation error.
    XsdError(String),

    /// XPath syntax or evaluation error.
    XPathError(String),

    /// Invalid Node operation error.
    NodeError(String),

    /// I/O error.
    Io(String),

    /// Serialization / Deserialization error.
    SerializationError(String),
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SyntaxError { message, line, col } => {
                write!(f, "Syntax error at line {line}, col {col}: {message}")
            }
            Self::SecurityLimitExceeded(msg) => write!(f, "Security limit exceeded: {msg}"),
            Self::EntityError(msg) => write!(f, "Entity error: {msg}"),
            Self::DtdError(msg) => write!(f, "DTD Validation error: {msg}"),
            Self::XsdError(msg) => write!(f, "XSD Validation error: {msg}"),
            Self::XPathError(msg) => write!(f, "XPath error: {msg}"),
            Self::NodeError(msg) => write!(f, "Node error: {msg}"),
            Self::Io(msg) => write!(f, "IO error: {msg}"),
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for XmlError {}
