//! String constants and shared escape utilities for JSON serialization
//!
//! Centralizes all string literals and escape sequences used across the JSON
//! stringify modules so they are defined in one place and reused consistently.

use crate::io::traits::IDestination;

/// JSON keyword strings
pub const JSON_NULL: &str = "null";
pub const JSON_TRUE: &str = "true";
pub const JSON_FALSE: &str = "false";

/// JSON structural strings
pub const STR_QUOTE: &str = "\"";
pub const STR_COMMA: &str = ",";
pub const STR_COLON: &str = ":";
pub const STR_ARRAY_START: &str = "[";
pub const STR_ARRAY_END: &str = "]";
pub const STR_OBJECT_START: &str = "{";
pub const STR_OBJECT_END: &str = "}";

/// JSON string escape sequences
pub const ESC_QUOTE: &str = "\\\"";
pub const ESC_BACKSLASH: &str = "\\\\";
pub const ESC_NEWLINE: &str = "\\n";
pub const ESC_CARRIAGE_RETURN: &str = "\\r";
pub const ESC_TAB: &str = "\\t";

/// Byte values that require escaping inside a JSON string
pub const BYTE_QUOTE: u8 = b'"';
pub const BYTE_BACKSLASH: u8 = b'\\';
pub const BYTE_NEWLINE: u8 = b'\n';
pub const BYTE_CARRIAGE_RETURN: u8 = b'\r';
pub const BYTE_TAB: u8 = b'\t';
/// Control characters below this value (exclusive) must be escaped
pub const CONTROL_CHAR_LIMIT: u8 = 32;

/// Returns `true` when the string contains characters that must be escaped in JSON.
#[inline]
pub fn needs_escaping(s: &str) -> bool {
    for &b in s.as_bytes() {
        match b {
            BYTE_QUOTE | BYTE_BACKSLASH | BYTE_NEWLINE | BYTE_CARRIAGE_RETURN | BYTE_TAB => {
                return true;
            }
            b if b < CONTROL_CHAR_LIMIT => return true,
            _ => {}
        }
    }
    false
}

/// Write a quoted JSON string to `destination`, escaping characters as required.
///
/// Batches runs of unescaped bytes into a single `add_bytes` call for efficiency.
#[inline]
pub fn write_escaped_string(s: &str, destination: &mut dyn IDestination) {
    destination.add_bytes(STR_QUOTE);

    let bytes = s.as_bytes();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        let needs_escape = match bytes[i] {
            BYTE_QUOTE | BYTE_BACKSLASH | BYTE_NEWLINE | BYTE_CARRIAGE_RETURN | BYTE_TAB => true,
            b if b < CONTROL_CHAR_LIMIT => true,
            _ => false,
        };

        if needs_escape {
            // Flush accumulated unescaped bytes
            if i > start {
                destination.add_bytes(core::str::from_utf8(&bytes[start..i]).unwrap());
            }

            // Write the escape sequence
            match bytes[i] {
                BYTE_QUOTE => destination.add_bytes(ESC_QUOTE),
                BYTE_BACKSLASH => destination.add_bytes(ESC_BACKSLASH),
                BYTE_NEWLINE => destination.add_bytes(ESC_NEWLINE),
                BYTE_CARRIAGE_RETURN => destination.add_bytes(ESC_CARRIAGE_RETURN),
                BYTE_TAB => destination.add_bytes(ESC_TAB),
                b => {
                    // \uXXXX encoding for other control characters (no heap allocation)
                    let b = b as u32;
                    let mut buf = [b'\\', b'u', b'0', b'0', b'0', b'0'];
                    for j in (2..6).rev() {
                        let digit = (b >> (4 * (5 - j))) & 0xF;
                        buf[j] = match digit {
                            0..=9 => b'0' + digit as u8,
                            _ => b'a' + (digit as u8 - 10),
                        };
                    }
                    destination.add_bytes(core::str::from_utf8(&buf).unwrap());
                }
            }

            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    // Flush any remaining unescaped bytes
    if start < bytes.len() {
        destination.add_bytes(core::str::from_utf8(&bytes[start..]).unwrap());
    }

    destination.add_bytes(STR_QUOTE);
}
