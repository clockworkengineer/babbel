//! Structured error types for JSON parsing and operations

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "std")]
use std::error::Error as StdError;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Structured error type for JSON parsing operations
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Syntax error in JSON with optional position information
    Syntax {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },
    
    /// Unexpected character encountered
    UnexpectedChar {
        found: char,
        expected: String,
        position: Option<usize>,
    },
    
    /// Unexpected end of input
    UnexpectedEof {
        expected: String,
    },
    
    /// Invalid escape sequence in string
    InvalidEscape {
        sequence: String,
        position: Option<usize>,
    },
    
    /// Invalid Unicode escape sequence
    InvalidUnicode {
        sequence: String,
        position: Option<usize>,
    },
    
    /// Number parsing error
    InvalidNumber {
        value: String,
        reason: String,
    },
    
    /// Configuration limit exceeded
    LimitExceeded {
        limit_type: String,
        limit: usize,
    },
    
    /// Generic string error (for backward compatibility)
    Message(String),
}

impl ParseError {
    /// Create a syntax error with position information
    pub fn syntax(message: impl Into<String>, line: Option<usize>, column: Option<usize>) -> Self {
        ParseError::Syntax {
            message: message.into(),
            line,
            column,
        }
    }
    
    /// Create an unexpected character error
    pub fn unexpected_char(found: char, expected: impl Into<String>) -> Self {
        ParseError::UnexpectedChar {
            found,
            expected: expected.into(),
            position: None,
        }
    }
    
    /// Create an unexpected EOF error
    pub fn unexpected_eof(expected: impl Into<String>) -> Self {
        ParseError::UnexpectedEof {
            expected: expected.into(),
        }
    }
    
    /// Create a limit exceeded error
    pub fn limit_exceeded(limit_type: impl Into<String>, limit: usize) -> Self {
        ParseError::LimitExceeded {
            limit_type: limit_type.into(),
            limit,
        }
    }
    
    /// Create a generic message error
    pub fn message(msg: impl Into<String>) -> Self {
        ParseError::Message(msg.into())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Syntax { message, line, column } => {
                write!(f, "Syntax error: {}", message)?;
                if let (Some(line), Some(col)) = (line, column) {
                    write!(f, " at line {}, column {}", line, col)?;
                } else if let Some(line) = line {
                    write!(f, " at line {}", line)?;
                }
                Ok(())
            }
            ParseError::UnexpectedChar { found, expected, position } => {
                write!(f, "Unexpected character '{}', expected {}", found, expected)?;
                if let Some(pos) = position {
                    write!(f, " at position {}", pos)?;
                }
                Ok(())
            }
            ParseError::UnexpectedEof { expected } => {
                write!(f, "Unexpected end of input, expected {}", expected)
            }
            ParseError::InvalidEscape { sequence, position } => {
                write!(f, "Invalid escape sequence '{}'", sequence)?;
                if let Some(pos) = position {
                    write!(f, " at position {}", pos)?;
                }
                Ok(())
            }
            ParseError::InvalidUnicode { sequence, position } => {
                write!(f, "Invalid Unicode escape sequence '{}'", sequence)?;
                if let Some(pos) = position {
                    write!(f, " at position {}", pos)?;
                }
                Ok(())
            }
            ParseError::InvalidNumber { value, reason } => {
                write!(f, "Invalid number '{}': {}", value, reason)
            }
            ParseError::LimitExceeded { limit_type, limit } => {
                write!(f, "Maximum {} of {} exceeded", limit_type, limit)
            }
            ParseError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl StdError for ParseError {}

/// Convert from String for backward compatibility
impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError::Message(s)
    }
}

/// Convert from &str for convenience
impl From<&str> for ParseError {
    fn from(s: &str) -> Self {
        ParseError::Message(s.into())
    }
}

/// Convert to String for backward compatibility
impl From<ParseError> for String {
    fn from(e: ParseError) -> Self {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error() {
        let err = ParseError::syntax("Invalid JSON", Some(5), Some(10));
        let s = err.to_string();
        assert!(s.contains("Syntax error"));
        assert!(s.contains("line 5"));
        assert!(s.contains("column 10"));
    }

    #[test]
    fn test_unexpected_char() {
        let err = ParseError::unexpected_char('}', "comma or closing bracket");
        let s = err.to_string();
        assert!(s.contains("Unexpected character '}'"));
        assert!(s.contains("expected comma or closing bracket"));
    }

    #[test]
    fn test_limit_exceeded() {
        let err = ParseError::limit_exceeded("nesting depth", 100);
        let s = err.to_string();
        assert!(s.contains("Maximum nesting depth"));
        assert!(s.contains("100"));
    }

    #[test]
    fn test_from_string() {
        let err: ParseError = "Something went wrong".into();
        assert!(matches!(err, ParseError::Message(_)));
    }

    #[test]
    fn test_to_string() {
        let err = ParseError::message("Test error");
        let s: String = err.into();
        assert_eq!(s, "Test error");
    }
}
