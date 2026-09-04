//! Integration tests for parsing bencode integers.

#[cfg(test)]
mod tests {
    use crate::BufferSource;
    use crate::error::messages::*;
    use crate::nodes::node::Node;
    use crate::parser::default::parse;

    #[test]
    fn test_invalid_integer_format_fails() {
        let mut source = BufferSource::new(b"i++32e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_valid_positive_integer() {
        let mut source = BufferSource::new(b"i42e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(42))));
    }

    #[test]
    fn test_valid_negative_integer() {
        let mut source = BufferSource::new(b"i-42e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(-42))));
    }

    #[test]
    fn test_integer_zero() {
        let mut source = BufferSource::new(b"i0e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(0))));
    }

    #[test]
    fn test_negative_zero_fails() {
        let mut source = BufferSource::new(b"i-0e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_unterminated_integer() {
        let mut source = BufferSource::new(b"i42");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_INTEGER));
    }

    #[test]
    fn test_large_positive_integer() {
        let mut source = BufferSource::new(b"i9223372036854775807e");
        assert!(matches!(
            parse(&mut source),
            Ok(Node::Integer(9223372036854775807))
        ));
    }

    #[test]
    fn test_large_negative_integer() {
        let mut source = BufferSource::new(b"i-9223372036854775808e");
        assert!(matches!(
            parse(&mut source),
            Ok(Node::Integer(-9223372036854775808))
        ));
    }

    #[test]
    fn test_integer_with_invalid_chars() {
        let mut source = BufferSource::new(b"i42ae");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_empty() {
        let mut source = BufferSource::new(b"ie");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_only_negative_sign() {
        let mut source = BufferSource::new(b"i-e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_with_leading_zeros() {
        // Bencode should accept integers with leading zeros (parser doesn't validate this)
        let mut source = BufferSource::new(b"i00042e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(42))));
    }

    #[test]
    fn test_integer_overflow() {
        // Test a number that's too large for i64
        let mut source = BufferSource::new(b"i99999999999999999999e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_underflow() {
        // One less than i64::MIN
        let mut source = BufferSource::new(b"i-9223372036854775809e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_one() {
        let mut source = BufferSource::new(b"i1e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(1))));
    }

    #[test]
    fn test_integer_negative_one() {
        let mut source = BufferSource::new(b"i-1e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(-1))));
    }

    #[test]
    fn test_integer_float_like_fails() {
        // Decimals are not valid bencode integers
        let mut source = BufferSource::new(b"i3.14e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn test_integer_space_inside_fails() {
        let mut source = BufferSource::new(b"i4 2e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    // --- Convenience API (parse_bytes / parse_str) ---

    #[test]
    fn test_parse_bytes_positive_integer() {
        use crate::parse_bytes;
        assert_eq!(parse_bytes(b"i42e").unwrap(), Node::Integer(42));
    }

    #[test]
    fn test_parse_bytes_negative_integer() {
        use crate::parse_bytes;
        assert_eq!(parse_bytes(b"i-7e").unwrap(), Node::Integer(-7));
    }

    #[test]
    fn test_parse_str_integer() {
        use crate::parse_str;
        assert_eq!(parse_str("i0e").unwrap(), Node::Integer(0));
    }

    #[test]
    fn test_parse_str_i64_max() {
        use crate::parse_str;
        let input = format!("i{}e", i64::MAX);
        assert_eq!(parse_str(&input).unwrap(), Node::Integer(i64::MAX));
    }

    #[test]
    fn test_parse_str_i64_min() {
        use crate::parse_str;
        let input = format!("i{}e", i64::MIN);
        assert_eq!(parse_str(&input).unwrap(), Node::Integer(i64::MIN));
    }

    // --- Iterative parser ---

    #[test]
    fn test_iterative_positive_integer() {
        use crate::parse_str_iterative;
        assert_eq!(parse_str_iterative("i100e").unwrap(), Node::Integer(100));
    }

    #[test]
    fn test_iterative_negative_integer() {
        use crate::parse_bytes_iterative;
        assert_eq!(parse_bytes_iterative(b"i-55e").unwrap(), Node::Integer(-55));
    }

    #[test]
    fn test_iterative_zero() {
        use crate::parse_str_iterative;
        assert_eq!(parse_str_iterative("i0e").unwrap(), Node::Integer(0));
    }

    #[test]
    fn test_iterative_i64_max() {
        use crate::parse_str_iterative;
        let input = format!("i{}e", i64::MAX);
        assert_eq!(
            parse_str_iterative(&input).unwrap(),
            Node::Integer(i64::MAX)
        );
    }

    #[test]
    fn test_iterative_negative_zero_fails() {
        use crate::parse_str_iterative;
        assert!(parse_str_iterative("i-0e").is_err());
    }

    #[test]
    fn test_iterative_empty_integer_fails() {
        use crate::parse_str_iterative;
        assert!(parse_str_iterative("ie").is_err());
    }

    #[test]
    fn test_iterative_unterminated_fails() {
        use crate::parse_str_iterative;
        assert!(parse_str_iterative("i42").is_err());
    }

    // --- Borrowed / zero-copy parser ---

    #[test]
    fn test_borrowed_positive_integer() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"i77e").unwrap();
        assert_eq!(node.as_integer(), Some(77));
    }

    #[test]
    fn test_borrowed_negative_integer() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"i-3e").unwrap();
        assert_eq!(node.as_integer(), Some(-3));
    }

    #[test]
    fn test_borrowed_zero() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"i0e").unwrap();
        assert_eq!(node.as_integer(), Some(0));
    }

    #[test]
    fn test_borrowed_i64_min() {
        use crate::parse_borrowed;
        let input = format!("i{}e", i64::MIN);
        let node = parse_borrowed(input.as_bytes()).unwrap();
        assert_eq!(node.as_integer(), Some(i64::MIN));
    }

    #[test]
    fn test_borrowed_negative_zero_not_rejected() {
        use crate::parse_borrowed;
        // The borrowed parser does not validate -0 (only the default parser does).
        // Parsing succeeds and returns 0.
        let result = parse_borrowed(b"i-0e");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_integer(), Some(0));
    }

    // --- Round-trips ---

    #[test]
    fn test_round_trip_positive() {
        use crate::{parse_str, stringify_to_string};
        let node = Node::Integer(12345);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "i12345e");
        assert_eq!(parse_str(&encoded).unwrap(), node);
    }

    #[test]
    fn test_round_trip_negative() {
        use crate::{parse_str, stringify_to_string};
        let node = Node::Integer(-9876);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "i-9876e");
        assert_eq!(parse_str(&encoded).unwrap(), node);
    }

    #[test]
    fn test_round_trip_zero() {
        use crate::{parse_str, stringify_to_string};
        let node = Node::Integer(0);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "i0e");
        assert_eq!(parse_str(&encoded).unwrap(), node);
    }

    #[test]
    fn test_round_trip_i64_max() {
        use crate::{parse_str, stringify_to_string};
        let node = Node::Integer(i64::MAX);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(parse_str(&encoded).unwrap(), node);
    }

    #[test]
    fn test_round_trip_i64_min() {
        use crate::{parse_str, stringify_to_string};
        let node = Node::Integer(i64::MIN);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(parse_str(&encoded).unwrap(), node);
    }

    // --- Integer inside collections ---

    #[test]
    fn test_integer_inside_list() {
        use crate::parse_str;
        let node = parse_str("li-1ei0ei1ee").unwrap();
        assert_eq!(
            node,
            Node::List(vec![Node::Integer(-1), Node::Integer(0), Node::Integer(1)])
        );
    }

    #[test]
    fn test_integer_inside_dictionary() {
        use crate::parse_str;
        let node = parse_str("d3:keyi42ee").unwrap();
        assert_eq!(node.get("key"), Some(&Node::Integer(42)));
    }
}
