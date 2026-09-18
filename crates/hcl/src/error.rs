//! Error types for HCL (HashiCorp Configuration Language) operations.

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use babbel_core::BabbelError;
use core::fmt;

/// Errors encountered when parsing or serializing HCL documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HclError {
    /// Unexpected end of input
    UnexpectedEof,
    /// Invalid syntax at specific location
    InvalidSyntax {
        line: usize,
        col: usize,
        msg: String,
    },
    /// UTF-8 validation error
    InvalidUtf8,
    /// Custom error message
    Custom(&'static str),
}

impl fmt::Display for HclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of HCL input"),
            Self::InvalidSyntax { line, col, msg } => {
                write!(f, "HCL syntax error at {}:{}: {}", line, col, msg)
            }
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in HCL input"),
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HclError {}

impl From<HclError> for BabbelError {
    fn from(err: HclError) -> Self {
        match &err {
            HclError::InvalidSyntax { msg, .. } => {
                BabbelError::syntax(msg.clone()).with_format("hcl")
            }
            HclError::UnexpectedEof => {
                BabbelError::eof("unexpected end of HCL input").with_format("hcl")
            }
            HclError::InvalidUtf8 => {
                BabbelError::encoding("invalid UTF-8 in HCL input").with_format("hcl")
            }
            HclError::Custom(msg) => BabbelError::custom(*msg).with_format("hcl"),
        }
    }
}
