//! Lightweight error types for embedded systems
//!
//! This module provides error types that don't require heap allocation,
//! making them suitable for no_std environments with limited memory.

use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Lightweight error type for bencode parsing in embedded systems.
/// Uses no heap allocation - all error information is in the enum variant itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BencodeError {
    /// Input was empty or exhausted unexpectedly
    EmptyInput,

    /// Invalid integer format (e.g., non-numeric characters, -0, etc.)
    InvalidInteger,

    /// Integer not properly terminated with 'e'
    UnterminatedInteger,

    /// String length prefix is invalid or not a valid number
    InvalidStringLength,

    /// String content is shorter than declared length
    StringTooShort,

    /// List not properly terminated with 'e'
    UnterminatedList,

    /// Dictionary not properly terminated with 'e'
    UnterminatedDictionary,

    /// Dictionary keys must be in lexicographic order
    DictKeysOutOfOrder,

    /// Dictionary key must be a string type
    DictKeyMustBeString,

    /// Encountered an unexpected character (stores the character)
    UnexpectedCharacter(char),

    /// File not found (for std environments)
    FileNotFound,

    /// Generic IO error
    IoError,
}

impl BencodeError {
    /// Returns a static string describing the error.
    /// This avoids allocation and is suitable for no_std environments.
    pub const fn as_str(&self) -> &'static str {
        match self {
            BencodeError::EmptyInput => "Empty input",
            BencodeError::InvalidInteger => "Invalid integer",
            BencodeError::UnterminatedInteger => "Unterminated integer",
            BencodeError::InvalidStringLength => "Invalid string length",
            BencodeError::StringTooShort => "String too short",
            BencodeError::UnterminatedList => "Unterminated list",
            BencodeError::UnterminatedDictionary => "Unterminated dictionary",
            BencodeError::DictKeysOutOfOrder => "Dictionary keys must be in order",
            BencodeError::DictKeyMustBeString => "Dictionary key must be string",
            BencodeError::UnexpectedCharacter(_) => "Unexpected character",
            BencodeError::FileNotFound => "File not found",
            BencodeError::IoError => "IO error",
        }
    }

    /// Returns the error code as a u8 for compact error reporting
    pub const fn code(&self) -> u8 {
        match self {
            BencodeError::EmptyInput => 1,
            BencodeError::InvalidInteger => 2,
            BencodeError::UnterminatedInteger => 3,
            BencodeError::InvalidStringLength => 4,
            BencodeError::StringTooShort => 5,
            BencodeError::UnterminatedList => 6,
            BencodeError::UnterminatedDictionary => 7,
            BencodeError::DictKeysOutOfOrder => 8,
            BencodeError::DictKeyMustBeString => 9,
            BencodeError::UnexpectedCharacter(_) => 10,
            BencodeError::FileNotFound => 11,
            BencodeError::IoError => 12,
        }
    }
}

impl fmt::Display for BencodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BencodeError::UnexpectedCharacter(c) => {
                write!(f, "Unexpected character: {}", c)
            }
            _ => f.write_str(self.as_str()),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BencodeError {}

/// Conversion from String errors (for backward compatibility)
#[cfg(feature = "std")]
impl From<BencodeError> for String {
    fn from(err: BencodeError) -> String {
        err.to_string()
    }
}

/// Conversion to String errors (for backward compatibility)
impl From<String> for BencodeError {
    fn from(s: String) -> BencodeError {
        // Try to map common error strings back to enum variants
        match s.as_str() {
            "Empty input" => BencodeError::EmptyInput,
            "Invalid integer" => BencodeError::InvalidInteger,
            "Unterminated integer" => BencodeError::UnterminatedInteger,
            "Invalid string length" => BencodeError::InvalidStringLength,
            "String too short" => BencodeError::StringTooShort,
            "Unterminated list" => BencodeError::UnterminatedList,
            "Unterminated dictionary" => BencodeError::UnterminatedDictionary,
            "Dictionary keys must be in order" => BencodeError::DictKeysOutOfOrder,
            "Dictionary key must be string" => BencodeError::DictKeyMustBeString,
            "File not found" => BencodeError::FileNotFound,
            _ => BencodeError::IoError,
        }
    }
}

impl From<&str> for BencodeError {
    fn from(s: &str) -> BencodeError {
        match s {
            "Empty input" => BencodeError::EmptyInput,
            "Invalid integer" => BencodeError::InvalidInteger,
            "Unterminated integer" => BencodeError::UnterminatedInteger,
            "Invalid string length" => BencodeError::InvalidStringLength,
            "String too short" => BencodeError::StringTooShort,
            "Unterminated list" => BencodeError::UnterminatedList,
            "Unterminated dictionary" => BencodeError::UnterminatedDictionary,
            "Dictionary keys must be in order" => BencodeError::DictKeysOutOfOrder,
            "Dictionary key must be string" => BencodeError::DictKeyMustBeString,
            "File not found" => BencodeError::FileNotFound,
            _ => BencodeError::IoError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_as_str_works() {
        assert_eq!(BencodeError::EmptyInput.as_str(), "Empty input");
        assert_eq!(BencodeError::InvalidInteger.as_str(), "Invalid integer");
    }

    #[test]
    fn error_as_str_all_variants() {
        assert_eq!(BencodeError::EmptyInput.as_str(), "Empty input");
        assert_eq!(BencodeError::InvalidInteger.as_str(), "Invalid integer");
        assert_eq!(
            BencodeError::UnterminatedInteger.as_str(),
            "Unterminated integer"
        );
        assert_eq!(
            BencodeError::InvalidStringLength.as_str(),
            "Invalid string length"
        );
        assert_eq!(BencodeError::StringTooShort.as_str(), "String too short");
        assert_eq!(BencodeError::UnterminatedList.as_str(), "Unterminated list");
        assert_eq!(
            BencodeError::UnterminatedDictionary.as_str(),
            "Unterminated dictionary"
        );
        assert_eq!(
            BencodeError::DictKeysOutOfOrder.as_str(),
            "Dictionary keys must be in order"
        );
        assert_eq!(
            BencodeError::DictKeyMustBeString.as_str(),
            "Dictionary key must be string"
        );
        assert_eq!(
            BencodeError::UnexpectedCharacter('x').as_str(),
            "Unexpected character"
        );
        assert_eq!(BencodeError::FileNotFound.as_str(), "File not found");
        assert_eq!(BencodeError::IoError.as_str(), "IO error");
    }

    #[test]
    fn error_code_is_unique() {
        let errors = [
            BencodeError::EmptyInput,
            BencodeError::InvalidInteger,
            BencodeError::UnterminatedInteger,
            BencodeError::InvalidStringLength,
            BencodeError::StringTooShort,
            BencodeError::UnterminatedList,
            BencodeError::UnterminatedDictionary,
            BencodeError::DictKeysOutOfOrder,
            BencodeError::DictKeyMustBeString,
            BencodeError::UnexpectedCharacter('x'),
            BencodeError::FileNotFound,
            BencodeError::IoError,
        ];

        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                assert_ne!(errors[i].code(), errors[j].code());
            }
        }
    }

    #[test]
    fn error_code_exact_values() {
        assert_eq!(BencodeError::EmptyInput.code(), 1);
        assert_eq!(BencodeError::InvalidInteger.code(), 2);
        assert_eq!(BencodeError::UnterminatedInteger.code(), 3);
        assert_eq!(BencodeError::InvalidStringLength.code(), 4);
        assert_eq!(BencodeError::StringTooShort.code(), 5);
        assert_eq!(BencodeError::UnterminatedList.code(), 6);
        assert_eq!(BencodeError::UnterminatedDictionary.code(), 7);
        assert_eq!(BencodeError::DictKeysOutOfOrder.code(), 8);
        assert_eq!(BencodeError::DictKeyMustBeString.code(), 9);
        assert_eq!(BencodeError::UnexpectedCharacter('!').code(), 10);
        assert_eq!(BencodeError::FileNotFound.code(), 11);
        assert_eq!(BencodeError::IoError.code(), 12);
    }

    #[test]
    fn error_display_works() {
        assert_eq!(format!("{}", BencodeError::EmptyInput), "Empty input");
        assert_eq!(
            format!("{}", BencodeError::UnexpectedCharacter('z')),
            "Unexpected character: z"
        );
    }

    #[test]
    fn error_display_all_variants() {
        assert_eq!(format!("{}", BencodeError::EmptyInput), "Empty input");
        assert_eq!(
            format!("{}", BencodeError::InvalidInteger),
            "Invalid integer"
        );
        assert_eq!(
            format!("{}", BencodeError::UnterminatedInteger),
            "Unterminated integer"
        );
        assert_eq!(
            format!("{}", BencodeError::InvalidStringLength),
            "Invalid string length"
        );
        assert_eq!(
            format!("{}", BencodeError::StringTooShort),
            "String too short"
        );
        assert_eq!(
            format!("{}", BencodeError::UnterminatedList),
            "Unterminated list"
        );
        assert_eq!(
            format!("{}", BencodeError::UnterminatedDictionary),
            "Unterminated dictionary"
        );
        assert_eq!(
            format!("{}", BencodeError::DictKeysOutOfOrder),
            "Dictionary keys must be in order"
        );
        assert_eq!(
            format!("{}", BencodeError::DictKeyMustBeString),
            "Dictionary key must be string"
        );
        assert_eq!(format!("{}", BencodeError::FileNotFound), "File not found");
        assert_eq!(format!("{}", BencodeError::IoError), "IO error");
    }

    #[test]
    fn unexpected_character_display_reflects_stored_char() {
        assert_eq!(
            format!("{}", BencodeError::UnexpectedCharacter('a')),
            "Unexpected character: a"
        );
        assert_eq!(
            format!("{}", BencodeError::UnexpectedCharacter('0')),
            "Unexpected character: 0"
        );
        assert_eq!(
            format!("{}", BencodeError::UnexpectedCharacter('!')),
            "Unexpected character: !"
        );
        assert_eq!(
            format!("{}", BencodeError::UnexpectedCharacter('\n')),
            "Unexpected character: \n"
        );
    }

    #[test]
    fn unexpected_character_as_str_ignores_stored_char() {
        // as_str() always returns the same static string regardless of the stored char
        assert_eq!(
            BencodeError::UnexpectedCharacter('a').as_str(),
            "Unexpected character"
        );
        assert_eq!(
            BencodeError::UnexpectedCharacter('Z').as_str(),
            "Unexpected character"
        );
    }

    #[test]
    fn error_from_string_works() {
        let err: BencodeError = "Invalid integer".into();
        assert_eq!(err, BencodeError::InvalidInteger);
    }

    #[test]
    fn error_from_str_all_known_variants() {
        assert_eq!(BencodeError::from("Empty input"), BencodeError::EmptyInput);
        assert_eq!(
            BencodeError::from("Invalid integer"),
            BencodeError::InvalidInteger
        );
        assert_eq!(
            BencodeError::from("Unterminated integer"),
            BencodeError::UnterminatedInteger
        );
        assert_eq!(
            BencodeError::from("Invalid string length"),
            BencodeError::InvalidStringLength
        );
        assert_eq!(
            BencodeError::from("String too short"),
            BencodeError::StringTooShort
        );
        assert_eq!(
            BencodeError::from("Unterminated list"),
            BencodeError::UnterminatedList
        );
        assert_eq!(
            BencodeError::from("Unterminated dictionary"),
            BencodeError::UnterminatedDictionary
        );
        assert_eq!(
            BencodeError::from("Dictionary keys must be in order"),
            BencodeError::DictKeysOutOfOrder
        );
        assert_eq!(
            BencodeError::from("Dictionary key must be string"),
            BencodeError::DictKeyMustBeString
        );
        assert_eq!(
            BencodeError::from("File not found"),
            BencodeError::FileNotFound
        );
    }

    #[test]
    fn error_from_str_unknown_falls_back_to_io_error() {
        assert_eq!(
            BencodeError::from("some unknown error"),
            BencodeError::IoError
        );
        assert_eq!(BencodeError::from(""), BencodeError::IoError);
    }

    #[test]
    fn error_from_string_all_known_variants() {
        assert_eq!(
            BencodeError::from(String::from("Empty input")),
            BencodeError::EmptyInput
        );
        assert_eq!(
            BencodeError::from(String::from("Invalid integer")),
            BencodeError::InvalidInteger
        );
        assert_eq!(
            BencodeError::from(String::from("Unterminated integer")),
            BencodeError::UnterminatedInteger
        );
        assert_eq!(
            BencodeError::from(String::from("Invalid string length")),
            BencodeError::InvalidStringLength
        );
        assert_eq!(
            BencodeError::from(String::from("String too short")),
            BencodeError::StringTooShort
        );
        assert_eq!(
            BencodeError::from(String::from("Unterminated list")),
            BencodeError::UnterminatedList
        );
        assert_eq!(
            BencodeError::from(String::from("Unterminated dictionary")),
            BencodeError::UnterminatedDictionary
        );
        assert_eq!(
            BencodeError::from(String::from("Dictionary keys must be in order")),
            BencodeError::DictKeysOutOfOrder
        );
        assert_eq!(
            BencodeError::from(String::from("Dictionary key must be string")),
            BencodeError::DictKeyMustBeString
        );
        assert_eq!(
            BencodeError::from(String::from("File not found")),
            BencodeError::FileNotFound
        );
    }

    #[test]
    fn error_from_string_unknown_falls_back_to_io_error() {
        assert_eq!(
            BencodeError::from(String::from("something random")),
            BencodeError::IoError
        );
        assert_eq!(BencodeError::from(String::from("")), BencodeError::IoError);
    }

    #[test]
    fn error_equality() {
        assert_eq!(BencodeError::EmptyInput, BencodeError::EmptyInput);
        assert_ne!(BencodeError::EmptyInput, BencodeError::IoError);
        assert_eq!(
            BencodeError::UnexpectedCharacter('a'),
            BencodeError::UnexpectedCharacter('a')
        );
        assert_ne!(
            BencodeError::UnexpectedCharacter('a'),
            BencodeError::UnexpectedCharacter('b')
        );
    }

    #[test]
    fn error_is_copy() {
        let original = BencodeError::InvalidInteger;
        let copied = original; // Copy semantics – original still usable
        assert_eq!(original, copied);
    }

    #[test]
    fn error_is_clone() {
        let original = BencodeError::UnexpectedCharacter('q');
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn error_debug_contains_variant_name() {
        let s = format!("{:?}", BencodeError::EmptyInput);
        assert!(s.contains("EmptyInput"), "Debug output was: {}", s);

        let s = format!("{:?}", BencodeError::UnexpectedCharacter('x'));
        assert!(s.contains("UnexpectedCharacter"), "Debug output was: {}", s);
        assert!(s.contains('x'), "Debug output was: {}", s);
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_to_string_works() {
        let s: String = BencodeError::InvalidInteger.into();
        assert_eq!(s, "Invalid integer");
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_to_string_all_variants() {
        let s: String = BencodeError::EmptyInput.into();
        assert_eq!(s, "Empty input");

        let s: String = BencodeError::FileNotFound.into();
        assert_eq!(s, "File not found");

        let s: String = BencodeError::IoError.into();
        assert_eq!(s, "IO error");
    }

    #[cfg(feature = "std")]
    #[test]
    fn error_implements_std_error() {
        use std::error::Error;
        let err = BencodeError::EmptyInput;
        // source() should return None as there is no cause chain
        assert!(err.source().is_none());
    }
}
