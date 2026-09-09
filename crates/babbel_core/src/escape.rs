//! Centralized string and byte escaping utilities across JSON, YAML, XML, and Bencode.
//!
//! Provides fast, allocation-conscious escaping routines for:
//! - JSON string serialization (standard escapes `\"`, `\\`, `\n`, `\r`, `\t`, and control chars `\u00xx`)
//! - XML text and attribute content (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`)
//! - Bencode printable string escaping

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::io::traits::IDestination;

/// JSON string constants
pub const STR_QUOTE: &str = "\"";
pub const ESC_QUOTE: &str = "\\\"";
pub const ESC_BACKSLASH: &str = "\\\\";
pub const ESC_NEWLINE: &str = "\\n";
pub const ESC_CARRIAGE_RETURN: &str = "\\r";
pub const ESC_TAB: &str = "\\t";

/// Byte constants for JSON escaping
pub const BYTE_QUOTE: u8 = b'"';
pub const BYTE_BACKSLASH: u8 = b'\\';
pub const BYTE_NEWLINE: u8 = b'\n';
pub const BYTE_CARRIAGE_RETURN: u8 = b'\r';
pub const BYTE_TAB: u8 = b'\t';
pub const CONTROL_CHAR_LIMIT: u8 = 32;

/// Checks if string `s` contains characters that require escaping in JSON.
#[inline]
pub fn json_needs_escaping(s: &str) -> bool {
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

/// Escapes a string for JSON and appends the result into `out`.
pub fn escape_for_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

/// Escapes a string for XML text content (`&`, `<`, `>`, `"`, `'`).
pub fn escape_for_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

/// Writes an escaped string (enclosed in double quotes) to `dest` for JSON serialization.
///
/// Batches runs of unescaped bytes into single `add_bytes` calls for efficiency.
pub fn write_json_escaped_string(s: &str, dest: &mut dyn IDestination) {
    dest.add_bytes(STR_QUOTE);

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
            if i > start {
                dest.add_bytes(core::str::from_utf8(&bytes[start..i]).unwrap_or(""));
            }

            match bytes[i] {
                BYTE_QUOTE => dest.add_bytes(ESC_QUOTE),
                BYTE_BACKSLASH => dest.add_bytes(ESC_BACKSLASH),
                BYTE_NEWLINE => dest.add_bytes(ESC_NEWLINE),
                BYTE_CARRIAGE_RETURN => dest.add_bytes(ESC_CARRIAGE_RETURN),
                BYTE_TAB => dest.add_bytes(ESC_TAB),
                b => {
                    let b = b as u32;
                    let mut buf = [b'\\', b'u', b'0', b'0', b'0', b'0'];
                    for j in (2..6).rev() {
                        let digit = (b >> (4 * (5 - j))) & 0xF;
                        buf[j] = match digit {
                            0..=9 => b'0' + digit as u8,
                            _ => b'a' + (digit as u8 - 10),
                        };
                    }
                    if let Ok(seq) = core::str::from_utf8(&buf) {
                        dest.add_bytes(seq);
                    }
                }
            }

            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    if start < bytes.len() {
        dest.add_bytes(core::str::from_utf8(&bytes[start..]).unwrap_or(""));
    }

    dest.add_bytes(STR_QUOTE);
}

/// Writes an escaped XML string directly to `dest` without heap allocations.
pub fn write_xml_escaped_string(s: &str, dest: &mut dyn IDestination) {
    let bytes = s.as_bytes();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        let esc = match bytes[i] {
            b'&' => Some("&amp;"),
            b'<' => Some("&lt;"),
            b'>' => Some("&gt;"),
            b'"' => Some("&quot;"),
            b'\'' => Some("&apos;"),
            _ => None,
        };

        if let Some(replacement) = esc {
            if i > start {
                dest.add_bytes(core::str::from_utf8(&bytes[start..i]).unwrap_or(""));
            }
            dest.add_bytes(replacement);
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    if start < bytes.len() {
        dest.add_bytes(core::str::from_utf8(&bytes[start..]).unwrap_or(""));
    }
}

/// Checks whether an identifier is a valid TOML bare key (`[A-Za-z0-9_-]+`).
#[inline]
pub fn is_valid_toml_bare_key(key: &str) -> bool {
    if key.is_empty() {
        return false;
    }
    key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Escapes a string for a TOML basic string literal (inside double quotes).
pub fn escape_for_toml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\x0C' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            '\x1B' => out.push_str("\\e"),
            c if (c as u32) < 0x20 || (c as u32) == 0x7F => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out
}

/// Writes a quoted, escaped TOML basic string into `dest`.
pub fn write_toml_escaped_string(s: &str, dest: &mut dyn IDestination) {
    dest.add_byte(b'"');
    let bytes = s.as_bytes();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        let esc = match bytes[i] {
            b'"' => Some("\\\""),
            b'\\' => Some("\\\\"),
            0x08 => Some("\\b"),
            b'\t' => Some("\\t"),
            b'\n' => Some("\\n"),
            0x0C => Some("\\f"),
            b'\r' => Some("\\r"),
            0x1B => Some("\\e"),
            _ => None,
        };

        if let Some(replacement) = esc {
            if i > start {
                dest.add_bytes(core::str::from_utf8(&bytes[start..i]).unwrap_or(""));
            }
            dest.add_bytes(replacement);
            i += 1;
            start = i;
        } else if bytes[i] < 0x20 || bytes[i] == 0x7F {
            if i > start {
                dest.add_bytes(core::str::from_utf8(&bytes[start..i]).unwrap_or(""));
            }
            let hex = format!("\\u{:04x}", bytes[i]);
            dest.add_bytes(&hex);
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }

    if start < bytes.len() {
        dest.add_bytes(core::str::from_utf8(&bytes[start..]).unwrap_or(""));
    }
    dest.add_byte(b'"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::BufferDestination;

    #[test]
    fn test_escape_for_json() {
        assert_eq!(escape_for_json("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_for_json("line1\nline2\ttab"), "line1\\nline2\\ttab");
        assert_eq!(escape_for_json("control\x01char"), "control\\u0001char");
    }

    #[test]
    fn test_escape_for_xml() {
        assert_eq!(
            escape_for_xml("<root key=\"val\" & 'item'>"),
            "&lt;root key=&quot;val&quot; &amp; &apos;item&apos;&gt;"
        );
    }

    #[test]
    fn test_write_json_escaped_string() {
        let mut dest = BufferDestination::new();
        write_json_escaped_string("hello\n\"world\"", &mut dest);
        assert_eq!(
            core::str::from_utf8(dest.as_bytes()).unwrap(),
            "\"hello\\n\\\"world\\\"\""
        );
    }

    #[test]
    fn test_escape_for_toml() {
        assert_eq!(escape_for_toml("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_for_toml("path\\to\\file"), "path\\\\to\\\\file");
        assert_eq!(escape_for_toml("\t\n\r"), "\\t\\n\\r");
        assert!(is_valid_toml_bare_key("foo_bar-123"));
        assert!(!is_valid_toml_bare_key("foo.bar"));
        assert!(!is_valid_toml_bare_key(""));
    }
}
