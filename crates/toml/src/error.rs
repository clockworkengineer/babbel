//! TOML error diagnostics and error conversion.

#[cfg(not(feature = "std"))]
use alloc::string::String;
use babbel_core::error::{BabbelError, Location, Span};

/// An error occurring during TOML parsing, validation, or stringification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub position: usize,
}

impl TomlError {
    /// Create a new TOML error at a specific position.
    pub fn new(message: impl Into<String>, line: usize, column: usize, position: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
            position,
        }
    }

    /// Create a syntax error with position information.
    pub fn syntax(message: impl Into<String>, line: usize, column: usize, position: usize) -> Self {
        Self::new(message, line, column, position)
    }

    /// Create an error without position details (e.g. root serialization error).
    pub fn custom(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: 1,
            column: 1,
            position: 0,
        }
    }
}

impl core::fmt::Display for TomlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "TOML error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TomlError {}

impl From<TomlError> for BabbelError {
    fn from(err: TomlError) -> Self {
        let loc = Location::new(err.line, err.column, err.position);
        BabbelError::syntax(err.message)
            .with_format("toml")
            .with_span(Span::new(loc, loc))
    }
}
