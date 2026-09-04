#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// Centralized error messages for the bencode library.
/// Keep all user-visible error strings and small helpers here to ensure consistency.

/// Generic IO-related errors
pub const FILE_NOT_FOUND: &str = "File not found";

/// Parser errors
pub const ERR_EMPTY_INPUT: &str = "Empty input";
pub const ERR_INVALID_INTEGER: &str = "Invalid integer";
pub const ERR_UNTERMINATED_INTEGER: &str = "Unterminated integer";
pub const ERR_INVALID_STRING_LENGTH: &str = "Invalid string length";
pub const ERR_STRING_TOO_SHORT: &str = "String too short";
pub const ERR_UNTERMINATED_LIST: &str = "Unterminated list";
pub const ERR_UNTERMINATED_DICTIONARY: &str = "Unterminated dictionary";
pub const ERR_DICT_KEYS_ORDER: &str = "Dictionary keys must be in order";
pub const ERR_DICT_KEY_MUST_BE_STRING: &str = "Dictionary key must be string";

/// Helpers for constructing formatted error messages
pub fn unexpected_character(c: char) -> String {
    format!("Unexpected character: {}", c)
}
