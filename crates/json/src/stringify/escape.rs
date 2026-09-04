//! String constants and shared escape utilities for JSON serialization
//!
//! Re-exports centralized JSON escaping logic from `babbel_core::escape`.

pub use babbel_core::escape::{
    json_needs_escaping as needs_escaping,
    write_json_escaped_string as write_escaped_string,
    BYTE_BACKSLASH, BYTE_CARRIAGE_RETURN, BYTE_NEWLINE, BYTE_QUOTE, BYTE_TAB,
    CONTROL_CHAR_LIMIT, ESC_BACKSLASH, ESC_CARRIAGE_RETURN, ESC_NEWLINE, ESC_QUOTE, ESC_TAB,
    STR_QUOTE,
};

/// JSON keyword strings
pub const JSON_NULL: &str = "null";
pub const JSON_TRUE: &str = "true";
pub const JSON_FALSE: &str = "false";

/// JSON structural strings
pub const STR_COMMA: &str = ",";
pub const STR_COLON: &str = ":";
pub const STR_ARRAY_START: &str = "[";
pub const STR_ARRAY_END: &str = "]";
pub const STR_OBJECT_START: &str = "{";
pub const STR_OBJECT_END: &str = "}";
