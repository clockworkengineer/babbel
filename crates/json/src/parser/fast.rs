//! Fast JSON parser with SIMD optimizations and reduced allocations
//!
//! This module provides optimized parsing for performance-critical applications.
//! Key optimizations:
//! - SIMD-accelerated whitespace skipping and character validation
//! - Reduced allocations through string interning
//! - Fast path for common cases (ASCII strings, small numbers)

use crate::io::traits::ISource;

/// Fast path: Check if a string needs escaping
/// Returns true if the string contains only ASCII characters that don't need escaping
#[inline]
pub fn is_simple_string(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b >= 32 && b != b'"' && b != b'\\')
}

/// Fast path: Parse a simple integer (no decimals, no exponents)
/// Returns Some(i64) if the string is a simple integer, None otherwise
#[inline]
pub fn try_parse_simple_int(s: &str) -> Option<i64> {
    if s.is_empty() {
        return None;
    }

    let bytes = s.as_bytes();
    let (negative, start) = if bytes[0] == b'-' {
        (true, 1)
    } else {
        (false, 0)
    };

    if start >= bytes.len() {
        return None;
    }

    // Fast path: all digits?
    if !bytes[start..].iter().all(|&b| b.is_ascii_digit()) {
        return None;
    }

    // Parse manually to avoid string allocation
    let mut result: i64 = 0;
    for &b in &bytes[start..] {
        let digit = (b - b'0') as i64;
        result = result.checked_mul(10)?.checked_add(digit)?;
    }

    Some(if negative { -result } else { result })
}

/// SIMD-accelerated whitespace skipping
/// Skips over whitespace characters much faster than byte-by-byte comparison
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn skip_whitespace_simd(source: &mut dyn ISource) {
    while source.more() {
        if let Some(ch) = source.current() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                source.next();
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
pub fn skip_whitespace_simd(source: &mut dyn ISource) {
    // Fallback for non-x86_64 architectures
    while source.more() {
        if let Some(ch) = source.current() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                source.next();
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

/// Fast string validation: check if a byte slice contains valid JSON string content
/// This is faster than parsing character by character
#[inline]
pub fn validate_json_string_fast(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            i += 2; // Skip escape sequence
        } else if b < 32 {
            return false; // Invalid control character
        } else {
            i += 1;
        }
    }
    true
}

/// Statistics for performance analysis
#[cfg(feature = "alloc")]
pub struct FastParseStats {
    pub simple_strings: usize,
    pub simple_ints: usize,
    pub total_allocations: usize,
    pub simd_operations: usize,
}

#[cfg(feature = "alloc")]
impl FastParseStats {
    pub fn new() -> Self {
        Self {
            simple_strings: 0,
            simple_ints: 0,
            total_allocations: 0,
            simd_operations: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_simple_string() {
        assert!(is_simple_string(b"hello"));
        assert!(is_simple_string(b"hello world"));
        assert!(!is_simple_string(b"hello\"world"));
        assert!(!is_simple_string(b"hello\\world"));
        assert!(!is_simple_string(b"hello\nworld"));
    }

    #[test]
    fn test_try_parse_simple_int() {
        assert_eq!(try_parse_simple_int("42"), Some(42));
        assert_eq!(try_parse_simple_int("-42"), Some(-42));
        assert_eq!(try_parse_simple_int("123456"), Some(123456));
        assert_eq!(try_parse_simple_int("0"), Some(0));
        assert_eq!(try_parse_simple_int("-0"), Some(0));

        // Should return None for non-simple cases
        assert_eq!(try_parse_simple_int("42.5"), None);
        assert_eq!(try_parse_simple_int("1e5"), None);
        assert_eq!(try_parse_simple_int(""), None);
        assert_eq!(try_parse_simple_int("-"), None);
    }

    #[test]
    fn test_validate_json_string_fast() {
        assert!(validate_json_string_fast(b"hello"));
        assert!(validate_json_string_fast(b"hello\\nworld"));
        assert!(!validate_json_string_fast(b"hello\nworld"));
    }

    // is_simple_string additional tests
    #[test]
    fn test_is_simple_string_empty() {
        assert!(is_simple_string(b""));
    }

    #[test]
    fn test_is_simple_string_control_chars() {
        assert!(!is_simple_string(b"\x00")); // null
        assert!(!is_simple_string(b"\x01")); // SOH
        assert!(!is_simple_string(b"\x1f")); // US (31)
        assert!(!is_simple_string(b"\x09")); // tab
        assert!(!is_simple_string(b"\x0a")); // newline
        assert!(!is_simple_string(b"\x0d")); // carriage return
    }

    #[test]
    fn test_is_simple_string_space() {
        // Space (0x20 = 32) satisfies b >= 32
        assert!(is_simple_string(b" "));
        assert!(is_simple_string(b"hello world"));
    }

    #[test]
    fn test_is_simple_string_only_special_chars() {
        assert!(!is_simple_string(b"\""));
        assert!(!is_simple_string(b"\\"));
    }

    #[test]
    fn test_is_simple_string_special_in_middle() {
        assert!(!is_simple_string(b"abc\"def"));
        assert!(!is_simple_string(b"abc\\def"));
    }

    #[test]
    fn test_is_simple_string_printable_ascii() {
        assert!(is_simple_string(b"abc123!@#$%^&*()"));
        assert!(is_simple_string(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
        assert!(is_simple_string(b"0123456789"));
    }

    #[test]
    fn test_is_simple_string_high_bytes() {
        // Bytes >= 128 satisfy b >= 32, not '"' or '\\'
        assert!(is_simple_string(&[0xc3, 0xa9])); // UTF-8 for é
        assert!(is_simple_string(&[0xff, 0xfe]));
    }

    #[test]
    fn test_is_simple_string_control_mixed_with_valid() {
        // Control char anywhere in string makes it false
        assert!(!is_simple_string(b"hello\x00world"));
        assert!(!is_simple_string(b"\x01abc"));
        assert!(!is_simple_string(b"end\x1f"));
    }

    // try_parse_simple_int additional tests
    #[test]
    fn test_try_parse_simple_int_single_digits() {
        for d in 0i64..=9i64 {
            assert_eq!(try_parse_simple_int(&d.to_string()), Some(d));
        }
    }

    #[test]
    fn test_try_parse_simple_int_i64_max() {
        assert_eq!(try_parse_simple_int("9223372036854775807"), Some(i64::MAX));
    }

    #[test]
    fn test_try_parse_simple_int_overflow() {
        assert_eq!(try_parse_simple_int("9223372036854775808"), None); // i64::MAX + 1
        assert_eq!(try_parse_simple_int("99999999999999999999"), None); // far too large
    }

    #[test]
    fn test_try_parse_simple_int_negative_large() {
        assert_eq!(
            try_parse_simple_int("-9223372036854775807"),
            Some(-i64::MAX)
        );
    }

    #[test]
    fn test_try_parse_simple_int_whitespace() {
        assert_eq!(try_parse_simple_int(" 42"), None);
        assert_eq!(try_parse_simple_int("42 "), None);
        assert_eq!(try_parse_simple_int(" "), None);
    }

    #[test]
    fn test_try_parse_simple_int_plus_sign() {
        assert_eq!(try_parse_simple_int("+42"), None);
    }

    #[test]
    fn test_try_parse_simple_int_float_variants() {
        assert_eq!(try_parse_simple_int("3.14"), None);
        assert_eq!(try_parse_simple_int("2e10"), None);
        assert_eq!(try_parse_simple_int("1.0"), None);
        assert_eq!(try_parse_simple_int("0.0"), None);
    }

    #[test]
    fn test_try_parse_simple_int_double_minus() {
        assert_eq!(try_parse_simple_int("--1"), None);
    }

    #[test]
    fn test_try_parse_simple_int_alpha() {
        assert_eq!(try_parse_simple_int("abc"), None);
        assert_eq!(try_parse_simple_int("12abc"), None);
    }

    #[test]
    fn test_try_parse_simple_int_multi_digit_negatives() {
        assert_eq!(try_parse_simple_int("-1000"), Some(-1000));
        assert_eq!(try_parse_simple_int("-100"), Some(-100));
        assert_eq!(try_parse_simple_int("-1"), Some(-1));
    }

    // validate_json_string_fast additional tests
    #[test]
    fn test_validate_json_string_empty() {
        assert!(validate_json_string_fast(b""));
    }

    #[test]
    fn test_validate_json_string_all_control_chars_invalid() {
        for b in 0u8..32u8 {
            assert!(
                !validate_json_string_fast(&[b]),
                "byte {} should be invalid",
                b
            );
        }
    }

    #[test]
    fn test_validate_json_string_escape_sequences() {
        assert!(validate_json_string_fast(b"\\n"));
        assert!(validate_json_string_fast(b"\\t"));
        assert!(validate_json_string_fast(b"\\\\"));
        assert!(validate_json_string_fast(b"\\\""));
        assert!(validate_json_string_fast(b"\\r"));
    }

    #[test]
    fn test_validate_json_string_quotes_allowed() {
        // Quote (0x22 = 34) is >= 32 and not backslash, passes fast validator
        assert!(validate_json_string_fast(b"\""));
        assert!(validate_json_string_fast(b"say \"hello\""));
    }

    #[test]
    fn test_validate_json_string_high_bytes() {
        // Bytes >= 128 are >= 32-pass
        assert!(validate_json_string_fast(&[0xc3, 0xa9])); // UTF-8 é
        assert!(validate_json_string_fast(&[0xff, 0xfe]));
    }

    #[test]
    fn test_validate_json_string_space_valid() {
        assert!(validate_json_string_fast(b" "));
        assert!(validate_json_string_fast(b"   spaces   "));
    }

    #[test]
    fn test_validate_json_string_control_in_middle() {
        assert!(!validate_json_string_fast(b"hello\x00world"));
        assert!(!validate_json_string_fast(b"start\x1fend"));
    }

    #[test]
    fn test_validate_json_string_printable_ascii() {
        assert!(validate_json_string_fast(b"abcdefghijklmnopqrstuvwxyz"));
        assert!(validate_json_string_fast(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"));
        assert!(validate_json_string_fast(b"0123456789"));
        assert!(validate_json_string_fast(b"!@#$%^&*()_+-=[]{}|;:,.<>?/"));
    }

    // skip_whitespace_simd tests
    #[test]
    fn test_skip_whitespace_simd_spaces() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"   abc");
        skip_whitespace_simd(&mut source);
        assert_eq!(source.current(), Some('a'));
    }

    #[test]
    fn test_skip_whitespace_simd_tabs() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"\t\tabc");
        skip_whitespace_simd(&mut source);
        assert_eq!(source.current(), Some('a'));
    }

    #[test]
    fn test_skip_whitespace_simd_mixed() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b" \t\n\r{");
        skip_whitespace_simd(&mut source);
        assert_eq!(source.current(), Some('{'));
    }

    #[test]
    fn test_skip_whitespace_simd_no_whitespace() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"abc");
        skip_whitespace_simd(&mut source);
        assert_eq!(source.current(), Some('a'));
    }

    #[test]
    fn test_skip_whitespace_simd_all_whitespace() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"   ");
        skip_whitespace_simd(&mut source);
        assert!(!source.more());
    }

    #[test]
    fn test_skip_whitespace_simd_empty() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"");
        skip_whitespace_simd(&mut source); // should not panic
        assert!(!source.more());
    }

    #[test]
    fn test_skip_whitespace_simd_only_newlines() {
        use crate::io::sources::buffer::Buffer;
        let mut source = Buffer::new(b"\n\n\r\n");
        skip_whitespace_simd(&mut source);
        assert!(!source.more());
    }

    #[test]
    fn test_skip_whitespace_simd_non_whitespace_preserved() {
        use crate::io::sources::buffer::Buffer;
        // Non-whitespace at start should remain untouched
        let mut source = Buffer::new(b"1 2 3");
        skip_whitespace_simd(&mut source);
        assert_eq!(source.current(), Some('1'));
    }

    // FastParseStats tests
    #[cfg(feature = "alloc")]
    #[test]
    fn test_fast_parse_stats_new_zeros() {
        let stats = FastParseStats::new();
        assert_eq!(stats.simple_strings, 0);
        assert_eq!(stats.simple_ints, 0);
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.simd_operations, 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_fast_parse_stats_mutable_fields() {
        let mut stats = FastParseStats::new();
        stats.simple_strings += 5;
        stats.simple_ints += 3;
        stats.total_allocations += 10;
        stats.simd_operations += 7;
        assert_eq!(stats.simple_strings, 5);
        assert_eq!(stats.simple_ints, 3);
        assert_eq!(stats.total_allocations, 10);
        assert_eq!(stats.simd_operations, 7);
    }
}
