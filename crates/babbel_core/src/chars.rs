//! Character utilities for data format lexing and parsing.

/// Check if character is ASCII/JSON whitespace (space, tab, newline, carriage return).
#[inline]
pub const fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// Check if character is a line break character.
#[inline]
pub const fn is_newline(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

/// Check if character is a decimal digit (0-9).
#[inline]
pub const fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Check if character is a hexadecimal digit (0-9, a-f, A-F).
#[inline]
pub const fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        assert!(is_whitespace(' '));
        assert!(is_whitespace('\t'));
        assert!(is_whitespace('\n'));
        assert!(is_whitespace('\r'));
        assert!(!is_whitespace('a'));
        assert!(!is_whitespace('\0'));
    }

    #[test]
    fn test_digits() {
        assert!(is_digit('5'));
        assert!(!is_digit('a'));
        assert!(is_hex_digit('f'));
        assert!(is_hex_digit('F'));
        assert!(!is_hex_digit('g'));
    }
}
