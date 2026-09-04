#[cfg(not(feature = "std"))]
use alloc::format;

use crate::io::traits::BencodeWrite;

/// Escapes and writes a string value to the destination, handling special characters
/// and converting unprintable characters to \u escape sequences.
///
/// # Arguments
/// * `value` - The string value to escape and write
/// * `destination` - The destination to write the escaped string to
#[allow(dead_code)]
pub(crate) fn escape_string(value: &str, destination: &mut (impl BencodeWrite + ?Sized)) {
    for &byte in value.as_bytes() {
        if byte == b'"' || byte == b'\\' {
            destination.write_byte(b'\\');
            destination.write_byte(byte);
        } else if byte.is_ascii_graphic() || byte == b' ' {
            destination.write_byte(byte);
        } else {
            // Convert unprintable characters to \u escape sequence
            let escaped = format!("\\u{:04x}", byte);
            destination.write_bytes(escaped.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[test]
    fn test_escape_regular_string() {
        let mut destination = Buffer::new();
        escape_string("hello world", &mut destination);
        assert_eq!(destination.to_string(), "hello world");
    }

    #[test]
    fn test_escape_special_characters() {
        let mut destination = Buffer::new();
        escape_string("hello\"world\\test", &mut destination);
        assert_eq!(destination.to_string(), "hello\\\"world\\\\test");
    }

    #[test]
    fn test_escape_unprintable_characters() {
        let mut destination = Buffer::new();
        escape_string("hello\nworld\t", &mut destination);
        assert_eq!(destination.to_string(), "hello\\u000aworld\\u0009");
    }

    #[test]
    fn test_escape_empty_string() {
        let mut destination = Buffer::new();
        escape_string("", &mut destination);
        assert_eq!(destination.to_string(), "");
    }

    #[test]
    fn test_escape_null_byte() {
        let mut destination = Buffer::new();
        escape_string("hello\x00world", &mut destination);
        assert_eq!(destination.to_string(), "hello\\u0000world");
    }

    #[test]
    fn test_escape_carriage_return() {
        let mut destination = Buffer::new();
        escape_string("line1\rline2", &mut destination);
        assert_eq!(destination.to_string(), "line1\\u000dline2");
    }

    #[test]
    fn test_escape_backspace() {
        let mut destination = Buffer::new();
        escape_string("hello\x08world", &mut destination);
        assert_eq!(destination.to_string(), "hello\\u0008world");
    }

    #[test]
    fn test_escape_form_feed() {
        let mut destination = Buffer::new();
        escape_string("page1\x0Cpage2", &mut destination);
        assert_eq!(destination.to_string(), "page1\\u000cpage2");
    }

    #[test]
    fn test_escape_vertical_tab() {
        let mut destination = Buffer::new();
        escape_string("line1\x0Bline2", &mut destination);
        assert_eq!(destination.to_string(), "line1\\u000bline2");
    }

    #[test]
    fn test_escape_delete_char() {
        let mut destination = Buffer::new();
        escape_string("text\x7Fmore", &mut destination);
        assert_eq!(destination.to_string(), "text\\u007fmore");
    }

    #[test]
    fn test_escape_mixed_special_chars() {
        let mut destination = Buffer::new();
        escape_string("\"test\"\n\t\r\x00", &mut destination);
        assert_eq!(
            destination.to_string(),
            "\\\"test\\\"\\u000a\\u0009\\u000d\\u0000"
        );
    }

    #[test]
    fn test_escape_all_printable_ascii() {
        let mut destination = Buffer::new();
        escape_string("abc123!@#$%^&*()", &mut destination);
        assert_eq!(destination.to_string(), "abc123!@#$%^&*()");
    }

    #[test]
    fn test_escape_space_is_preserved() {
        // Space (0x20) is explicitly handled as printable
        let mut destination = Buffer::new();
        escape_string("a b c", &mut destination);
        assert_eq!(destination.to_string(), "a b c");
    }

    #[test]
    fn test_escape_only_quote() {
        let mut destination = Buffer::new();
        escape_string("\"", &mut destination);
        assert_eq!(destination.to_string(), "\\\"");
    }

    #[test]
    fn test_escape_only_backslash() {
        let mut destination = Buffer::new();
        escape_string("\\", &mut destination);
        assert_eq!(destination.to_string(), "\\\\");
    }

    #[test]
    fn test_escape_consecutive_quotes() {
        let mut destination = Buffer::new();
        escape_string("\"\"\"", &mut destination);
        assert_eq!(destination.to_string(), "\\\"\\\"\\\"");
    }

    #[test]
    fn test_escape_consecutive_backslashes() {
        let mut destination = Buffer::new();
        escape_string("\\\\", &mut destination);
        assert_eq!(destination.to_string(), "\\\\\\\\");
    }

    #[test]
    fn test_escape_quote_and_backslash_adjacent() {
        let mut destination = Buffer::new();
        escape_string("\\\"", &mut destination);
        assert_eq!(destination.to_string(), "\\\\\\\"");
    }

    #[test]
    fn test_escape_high_utf8_bytes() {
        // 'é' (U+00E9) encodes as UTF-8 bytes [0xC3, 0xA9]; both are non-graphic and get \u escaped
        let mut destination = Buffer::new();
        escape_string("é", &mut destination);
        assert_eq!(destination.to_string(), "\\u00c3\\u00a9");
    }

    #[test]
    fn test_escape_latin_y_diaeresis() {
        // 'ÿ' (U+00FF) encodes as UTF-8 bytes [0xC3, 0xBF]
        let mut destination = Buffer::new();
        escape_string("ÿ", &mut destination);
        assert_eq!(destination.to_string(), "\\u00c3\\u00bf");
    }

    #[test]
    fn test_escape_byte_0x1f() {
        // 0x1F is the last non-printable ASCII control character before space
        let mut destination = Buffer::new();
        escape_string("\x1f", &mut destination);
        assert_eq!(destination.to_string(), "\\u001f");
    }

    #[test]
    fn test_escape_leading_and_trailing_special_chars() {
        let mut destination = Buffer::new();
        escape_string("\ntext\n", &mut destination);
        assert_eq!(destination.to_string(), "\\u000atext\\u000a");
    }

    #[test]
    fn test_escape_only_unprintable_sequence() {
        let mut destination = Buffer::new();
        escape_string("\x01\x02\x03", &mut destination);
        assert_eq!(destination.to_string(), "\\u0001\\u0002\\u0003");
    }

    #[test]
    fn test_escape_length_increases_for_escaped_chars() {
        // Quote gets \", so a single '"' expands to 2 chars
        let mut dest_plain = Buffer::new();
        let mut dest_quoted = Buffer::new();
        escape_string("a", &mut dest_plain);
        escape_string("\"", &mut dest_quoted);
        assert_eq!(dest_plain.to_string().len(), 1);
        assert_eq!(dest_quoted.to_string().len(), 2);
    }

    #[test]
    fn test_escape_unprintable_length_is_six() {
        // \u00xx is always 6 characters
        let mut destination = Buffer::new();
        escape_string("\n", &mut destination);
        assert_eq!(destination.to_string().len(), 6);
    }

    #[test]
    fn test_escape_long_string_no_special_chars() {
        let input: String = "a".repeat(1000);
        let mut destination = Buffer::new();
        escape_string(&input, &mut destination);
        assert_eq!(destination.to_string(), input);
    }

    #[test]
    fn test_escape_is_idempotent_for_plain_text() {
        // Calling escape_string on already-escaped output of a plain string gives the same result
        let plain = "hello world";
        let mut dest1 = Buffer::new();
        escape_string(plain, &mut dest1);
        let first = dest1.to_string();
        // first result should equal the original since no special chars
        assert_eq!(first, plain);
    }

    #[test]
    fn test_escape_all_digits_and_letters_preserved() {
        let input = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut destination = Buffer::new();
        escape_string(input, &mut destination);
        assert_eq!(destination.to_string(), input);
    }
}
