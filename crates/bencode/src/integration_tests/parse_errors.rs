//! Integration tests for parser error handling.

#[cfg(test)]
mod tests {
    use crate::BufferSource;
    use crate::error::messages::*;
    use crate::parser::default::parse;

    #[test]
    fn test_empty_input() {
        let mut source = BufferSource::new(b"");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_EMPTY_INPUT));
    }

    #[test]
    fn test_invalid_character() {
        let mut source = BufferSource::new(b"x123");
        assert!(matches!(parse(&mut source), Err(s) if s.contains("Unexpected character")));
    }

    #[test]
    fn test_invalid_start_character_z() {
        let mut source = BufferSource::new(b"z");
        assert!(matches!(parse(&mut source), Err(s) if s.contains("Unexpected character")));
    }

    #[test]
    fn test_invalid_start_character_dash() {
        let mut source = BufferSource::new(b"-42");
        assert!(matches!(parse(&mut source), Err(s) if s.contains("Unexpected character")));
    }

    #[test]
    fn test_malformed_nested_structure() {
        let mut source = BufferSource::new(b"ld3:keyi42e");
        assert!(
            matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_DICTIONARY || s == ERR_UNTERMINATED_LIST)
        );
    }

    #[test]
    fn test_dictionary_missing_value() {
        let mut source = BufferSource::new(b"d3:keye");
        assert!(matches!(parse(&mut source), Err(_)));
    }

    #[test]
    fn test_list_with_invalid_element() {
        let mut source = BufferSource::new(b"lxe");
        assert!(matches!(parse(&mut source), Err(s) if s.contains("Unexpected character")));
    }

    #[test]
    fn test_string_with_colon_separator_only() {
        // Test the case where ':' is encountered as the first character
        let mut source = BufferSource::new(b":test");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_STRING_LENGTH));
    }

    // --- Integer errors ---

    #[test]
    fn test_unterminated_integer() {
        let mut source = BufferSource::new(b"i42");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_INTEGER));
    }

    #[test]
    fn test_integer_negative_zero_is_invalid() {
        let mut source = BufferSource::new(b"i-0e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_empty_body_is_invalid() {
        let mut source = BufferSource::new(b"ie");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_non_numeric_body_is_invalid() {
        let mut source = BufferSource::new(b"iabce");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_leading_plus_parses_successfully() {
        // Rust's i64 FromStr accepts a leading '+', so i+42e is valid bencode
        let mut source = BufferSource::new(b"i+42e");
        assert!(matches!(
            parse(&mut source),
            Ok(crate::nodes::node::Node::Integer(42))
        ));
    }

    #[test]
    fn test_integer_double_minus_is_invalid() {
        let mut source = BufferSource::new(b"i--1e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    // --- String errors ---

    #[test]
    fn test_string_truncated_default_parser() {
        // Default parser returns ERR_INVALID_STRING_LENGTH when content is shorter than declared
        let mut source = BufferSource::new(b"10:hi");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_STRING_LENGTH));
    }

    #[test]
    fn test_string_truncated_borrowed_parser() {
        // Borrowed parser returns ERR_STRING_TOO_SHORT for the same case
        use crate::parse_borrowed;
        assert!(matches!(parse_borrowed(b"10:hi"), Err(s) if s == ERR_STRING_TOO_SHORT));
    }

    #[test]
    fn test_string_non_numeric_length() {
        // 'a' is not a digit so the parser reports an unexpected character
        let mut source = BufferSource::new(b"abc:test");
        assert!(matches!(parse(&mut source), Err(s) if s.contains("Unexpected character")));
    }

    // --- List errors ---

    #[test]
    fn test_unterminated_list() {
        let mut source = BufferSource::new(b"li1e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    #[test]
    fn test_unterminated_empty_list() {
        let mut source = BufferSource::new(b"l");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    #[test]
    fn test_nested_list_inner_unterminated() {
        let mut source = BufferSource::new(b"lli1ee");
        // inner list "li1e" is missing 'e', so outer consumes the outer 'e' as inner end
        // leaving outer unterminated
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    // --- Dictionary errors ---

    #[test]
    fn test_unterminated_dictionary_empty() {
        let mut source = BufferSource::new(b"d");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_DICTIONARY));
    }

    #[test]
    fn test_dictionary_integer_key_fails() {
        let mut source = BufferSource::new(b"di1ei2ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEY_MUST_BE_STRING));
    }

    #[test]
    fn test_dictionary_list_key_fails() {
        let mut source = BufferSource::new(b"dlee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEY_MUST_BE_STRING));
    }

    #[test]
    fn test_dictionary_out_of_order_keys() {
        let mut source = BufferSource::new(b"d1:zi1e1:ai2ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    #[test]
    fn test_dictionary_duplicate_keys_fail() {
        let mut source = BufferSource::new(b"d3:keyi1e3:keyi2ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    // --- Convenience API error propagation ---

    #[test]
    fn test_parse_bytes_propagates_error() {
        use crate::parse_bytes;
        assert!(parse_bytes(b"").is_err());
        assert!(parse_bytes(b"i42").is_err());
        assert!(parse_bytes(b"10:hi").is_err());
    }

    #[test]
    fn test_parse_str_propagates_error() {
        use crate::parse_str;
        assert!(parse_str("").is_err());
        assert!(parse_str("i-0e").is_err());
        assert!(parse_str("d1:zi1e1:ai2ee").is_err());
    }

    // --- Iterative parser errors ---

    #[test]
    fn test_iterative_empty_input_error() {
        use crate::parse_str_iterative;
        assert!(parse_str_iterative("").is_err());
    }

    #[test]
    fn test_iterative_unterminated_integer_error() {
        use crate::parse_bytes_iterative;
        assert!(parse_bytes_iterative(b"i99").is_err());
    }

    #[test]
    fn test_iterative_invalid_integer_negative_zero() {
        use crate::parse_str_iterative;
        assert!(parse_str_iterative("i-0e").is_err());
    }

    #[test]
    fn test_iterative_unterminated_list_error() {
        use crate::parse_bytes_iterative;
        assert!(parse_bytes_iterative(b"li1e").is_err());
    }

    #[test]
    fn test_iterative_unterminated_dictionary_error() {
        use crate::parse_bytes_iterative;
        assert!(parse_bytes_iterative(b"d3:keyi1e").is_err());
    }

    #[test]
    fn test_iterative_invalid_character_error() {
        use crate::parse_str_iterative;
        assert!(parse_str_iterative("X").is_err());
    }

    // --- Borrowed/zero-copy parser errors ---

    #[test]
    fn test_borrowed_empty_input_error() {
        use crate::parse_borrowed;
        assert!(parse_borrowed(b"").is_err());
    }

    #[test]
    fn test_borrowed_unterminated_integer_error() {
        use crate::parse_borrowed;
        assert!(parse_borrowed(b"i42").is_err());
    }

    #[test]
    fn test_borrowed_string_too_short_error() {
        use crate::parse_borrowed;
        assert!(parse_borrowed(b"10:hi").is_err());
    }

    #[test]
    fn test_borrowed_invalid_character_error() {
        use crate::parse_borrowed;
        assert!(parse_borrowed(b"X").is_err());
    }

    // --- validate_bencode errors ---

    #[test]
    fn test_validate_empty_is_err() {
        use crate::validate_bencode;
        assert!(validate_bencode(b"").is_err());
    }

    #[test]
    fn test_validate_unterminated_integer_is_err() {
        use crate::validate_bencode;
        assert!(validate_bencode(b"i42").is_err());
    }

    #[test]
    fn test_validate_truncated_string_is_err() {
        use crate::validate_bencode;
        assert!(validate_bencode(b"5:hi").is_err());
    }

    #[test]
    fn test_validate_unterminated_list_is_err() {
        use crate::validate_bencode;
        assert!(validate_bencode(b"li1e").is_err());
    }
}
