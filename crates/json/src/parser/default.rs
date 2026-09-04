//! JSON parser implementation that converts JSON text into Node structures
//! Provides functions for parsing different JSON data types including objects,
//! arrays, strings, numbers, boolean and null values.

use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;
use crate::nodes::node::Numeric;
use crate::parser::config::ParserConfig;
use crate::parser::constants::*;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

// Use smallvec for small arrays to reduce heap allocations
use smallvec::SmallVec;

/// Parses JSON input from a source and returns a Node representation
///
/// # Arguments
/// * `source` - Source implementing ISource trait that provides JSON input
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message if parsing fails
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    parse_with_config(source, &ParserConfig::unlimited())
}

/// Convenience function to parse JSON from a string slice
///
/// # Arguments
/// * `s` - JSON string to parse
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
///
/// # Examples
/// ```
/// use json_lib::parser::default::from_str;
///
/// let node = from_str(r#"{"name": "Alice", "age": 30}"#).unwrap();
/// assert!(node.is_object());
/// ```
pub fn from_str(s: &str) -> Result<Node, String> {
    use crate::io::sources::buffer::Buffer as BufferSource;
    let mut source = BufferSource::new(s.as_bytes());
    parse(&mut source)
}

/// Convenience function to parse JSON from a byte slice
///
/// # Arguments
/// * `bytes` - JSON bytes to parse
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
///
/// # Examples
/// ```
/// use json_lib::parser::default::from_bytes;
///
/// let bytes = br#"{"name": "Alice"}"#;
/// let node = from_bytes(bytes).unwrap();
/// assert!(node.is_object());
/// ```
pub fn from_bytes(bytes: &[u8]) -> Result<Node, String> {
    use crate::io::sources::buffer::Buffer as BufferSource;
    let mut source = BufferSource::new(bytes);
    parse(&mut source)
}

/// Parses JSON input with custom configuration for resource limits
///
/// # Arguments
/// * `source` - Source implementing ISource trait that provides JSON input
/// * `config` - Parser configuration with resource limits
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message if parsing fails
pub fn parse_with_config(source: &mut dyn ISource, config: &ParserConfig) -> Result<Node, String> {
    parse_value(source, config, 0)
}

/// Internal parsing function with depth tracking
fn parse_value(
    source: &mut dyn ISource,
    config: &ParserConfig,
    depth: usize,
) -> Result<Node, String> {
    // Check depth limit
    if let Some(max_depth) = config.max_depth {
        if depth >= max_depth {
            use arrayvec::ArrayString;
            use core::fmt::Write;
            let mut msg: ArrayString<64> = ArrayString::new();
            let _ = write!(&mut msg, "Maximum nesting depth of {} exceeded", max_depth);
            return Err(msg.to_string());
        }
    }

    skip_whitespace(source);

    match source.current() {
        Some(OBJECT_START) => parse_object_with_config(source, config, depth),
        Some(ARRAY_START) => parse_array_with_config(source, config, depth),
        Some(QUOTE) => parse_string_with_config(source, config),
        Some(TRUE_START) => parse_true(source),
        Some(FALSE_START) => parse_false(source),
        Some(NULL_START) => parse_null(source),
        Some(c) if c.is_digit(10) || c == MINUS => parse_number(source),
        Some(c) => Err(format!("{}{}", ERR_UNEXPECTED_CHAR, c)),
        None => Err(ERR_EMPTY_INPUT.to_string()),
    }
}

/// Advances the source past any whitespace characters
///
/// # Arguments
/// * `source` - Source to read characters from
#[inline]
fn skip_whitespace(source: &mut dyn ISource) {
    while let Some(c) = source.current() {
        if !is_json_whitespace(c) {
            break;
        }
        source.next();
    }
}

/// Parses a JSON object starting with '{' and returns Node::Object
/// Handles nested key-value pairs separated by commas
///
/// # Arguments
/// * `source` - Source to read characters from
///
/// # Returns
/// * `Result<Node, String>` - Object Node or error message
#[allow(dead_code)]
fn parse_object(source: &mut dyn ISource) -> Result<Node, String> {
    parse_object_with_config(source, &ParserConfig::unlimited(), 0)
}

/// Parses a JSON object with configuration and depth tracking
fn parse_object_with_config(
    source: &mut dyn ISource,
    config: &ParserConfig,
    depth: usize,
) -> Result<Node, String> {
    // Use SmallVec for small objects to reduce heap allocations
    let mut pairs: SmallVec<[(String, Node); 8]> = SmallVec::new();
    source.next(); // Skip '{'

    skip_whitespace(source);

    if let Some(OBJECT_END) = source.current() {
        source.next();
        // Build HashMap from pairs (empty)
        return Ok(Node::Object(HashMap::new()));
    }

    loop {
        // Check object size limit
        if let Some(max_size) = config.max_object_size {
            if pairs.len() >= max_size {
                use arrayvec::ArrayString;
                use core::fmt::Write;
                let mut msg: ArrayString<64> = ArrayString::new();
                let _ = write!(&mut msg, "Maximum object size of {} exceeded", max_size);
                return Err(msg.to_string());
            }
        }

        skip_whitespace(source);

        // Parse key
        let key = match parse_string_with_config(source, config)? {
            Node::Str(s) => s,
            _ => return Err(ERR_OBJECT_KEY.to_string()),
        };

        skip_whitespace(source);

        // Check for colon
        match source.current() {
            Some(COLON) => source.next(),
            _ => return Err(ERR_EXPECT_COLON.to_string()),
        }

        skip_whitespace(source);

        // Parse value with incremented depth
        let value = parse_value(source, config, depth + 1)?;
        pairs.push((key, value));

        skip_whitespace(source);

        match source.current() {
            Some(COMMA) => {
                source.next();
                continue;
            }
            Some(OBJECT_END) => {
                source.next();
                break;
            }
            _ => return Err(ERR_EXPECT_OBJECT_END.to_string()),
        }
    }

    // Build HashMap from pairs
    // Use a higher initial capacity to reduce rehashing (Rust's default load factor is 0.75)
    let mut map = HashMap::with_capacity((pairs.len() * 4) / 3 + 1);
    for (k, v) in pairs {
        map.insert(k, v);
    }
    Ok(Node::Object(map))
}

/// Parses a JSON array starting with '[' and returns Node::Array
/// Handles comma-separated values of any valid JSON type
///
/// # Arguments
/// * `source` - Source to read characters from
///
/// # Returns
/// * `Result<Node, String>` - Array Node or error message
#[allow(dead_code)]
fn parse_array(source: &mut dyn ISource) -> Result<Node, String> {
    parse_array_with_config(source, &ParserConfig::unlimited(), 0)
}

/// Parses a JSON array with configuration and depth tracking
fn parse_array_with_config(
    source: &mut dyn ISource,
    config: &ParserConfig,
    depth: usize,
) -> Result<Node, String> {
    // Use SmallVec for small arrays to reduce heap allocations
    let mut vec: SmallVec<[Node; 8]> = SmallVec::new();
    source.next(); // Skip '['

    skip_whitespace(source);

    if let Some(ARRAY_END) = source.current() {
        source.next();
        return Ok(Node::Array(vec.into_vec()));
    }

    loop {
        // Check array size limit
        if let Some(max_size) = config.max_array_size {
            if vec.len() >= max_size {
                use arrayvec::ArrayString;
                use core::fmt::Write;
                let mut msg: ArrayString<64> = ArrayString::new();
                let _ = write!(&mut msg, "Maximum array size of {} exceeded", max_size);
                return Err(msg.to_string());
            }
        }

        // Parse value with incremented depth
        vec.push(parse_value(source, config, depth + 1)?);

        skip_whitespace(source);

        match source.current() {
            Some(COMMA) => {
                source.next();
                continue;
            }
            Some(ARRAY_END) => {
                source.next();
                break;
            }
            _ => return Err(ERR_EXPECT_ARRAY_END.to_string()),
        }
    }

    // Convert SmallVec to Vec for Node::Array
    Ok(Node::Array(vec.into_vec()))
}

/// Parses a JSON string with support for escape sequences
/// Handles standard escapes and Unicode escape sequences
///
/// # Arguments
/// * `source` - Source to read characters from
///
/// # Returns
/// * `Result<Node, String>` - String Node or error message
#[allow(dead_code)]
fn parse_string(source: &mut dyn ISource) -> Result<Node, String> {
    parse_string_with_config(source, &ParserConfig::unlimited())
}

/// Parses a JSON string with configuration for length limits
fn parse_string_with_config(
    source: &mut dyn ISource,
    config: &ParserConfig,
) -> Result<Node, String> {
    // Use ArrayVec for short strings to reduce heap allocations
    use arrayvec::ArrayVec;
    let mut buf: ArrayVec<u8, 64> = ArrayVec::new();
    let mut heap_buf: Option<Vec<u8>> = None;
    source.next(); // Skip opening quote

    while let Some(c) = source.current() {
        // Check string length limit
        let cur_len = if let Some(ref v) = heap_buf {
            v.len()
        } else {
            buf.len()
        };
        if let Some(max_len) = config.max_string_length {
            if cur_len >= max_len {
                use arrayvec::ArrayString;
                use core::fmt::Write;
                let mut msg: ArrayString<64> = ArrayString::new();
                let _ = write!(
                    &mut msg,
                    "Maximum string length of {} bytes exceeded",
                    max_len
                );
                return Err(msg.to_string());
            }
        }

        match c {
            QUOTE => {
                source.next();
                // Convert buffer to String
                if let Some(v) = heap_buf {
                    return Ok(Node::Str(unsafe { String::from_utf8_unchecked(v) }));
                } else {
                    return Ok(Node::Str(unsafe {
                        String::from_utf8_unchecked(buf.into_iter().collect())
                    }));
                }
            }
            BACKSLASH => {
                source.next();
                let push_byte =
                    |b: u8, buf: &mut ArrayVec<u8, 64>, heap_buf: &mut Option<Vec<u8>>| {
                        if let Some(v) = heap_buf {
                            v.push(b);
                        } else if buf.try_push(b).is_err() {
                            let mut v = buf.clone().into_iter().collect::<Vec<u8>>();
                            v.push(b);
                            *heap_buf = Some(v);
                        }
                    };
                match source.current() {
                    Some('"') => push_byte(b'"', &mut buf, &mut heap_buf),
                    Some('\\') => push_byte(b'\\', &mut buf, &mut heap_buf),
                    Some('/') => push_byte(b'/', &mut buf, &mut heap_buf),
                    Some('b') => push_byte(b'\x08', &mut buf, &mut heap_buf),
                    Some('f') => push_byte(b'\x0c', &mut buf, &mut heap_buf),
                    Some('n') => push_byte(b'\n', &mut buf, &mut heap_buf),
                    Some('r') => push_byte(b'\r', &mut buf, &mut heap_buf),
                    Some('t') => push_byte(b'\t', &mut buf, &mut heap_buf),
                    Some('u') => {
                        source.next();
                        // Use ArrayVec for hex buffer
                        let mut hex: ArrayVec<u8, 4> = ArrayVec::new();
                        for _ in 0..4 {
                            match source.current() {
                                Some(d) if d.is_ascii_hexdigit() => {
                                    let _ = hex.push(d as u8);
                                    source.next();
                                }
                                _ => return Err(ERR_INVALID_ESCAPE.to_string()),
                            }
                        }
                        if let Ok(hex_str) = core::str::from_utf8(&hex) {
                            if let Ok(code) = u32::from_str_radix(hex_str, 16) {
                                if let Some(ch) = char::from_u32(code) {
                                    let mut utf8_buf = [0u8; 4];
                                    let encoded = ch.encode_utf8(&mut utf8_buf);
                                    for b in encoded.as_bytes() {
                                        if let Some(ref mut v) = heap_buf {
                                            v.push(*b);
                                        } else if buf.try_push(*b).is_err() {
                                            let mut v =
                                                buf.clone().into_iter().collect::<Vec<u8>>();
                                            v.push(*b);
                                            heap_buf = Some(v);
                                        }
                                    }
                                } else {
                                    return Err(ERR_INVALID_ESCAPE.to_string());
                                }
                            } else {
                                return Err(ERR_INVALID_ESCAPE.to_string());
                            }
                        } else {
                            return Err(ERR_INVALID_ESCAPE.to_string());
                        }
                        continue;
                    }
                    _ => return Err(ERR_INVALID_ESCAPE.to_string()),
                }
                source.next();
            }
            _ => {
                // Push UTF-8 bytes for char
                let mut utf8_buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut utf8_buf);
                for b in encoded.as_bytes() {
                    if let Some(ref mut v) = heap_buf {
                        v.push(*b);
                    } else if buf.try_push(*b).is_err() {
                        let mut v = buf.clone().into_iter().collect::<Vec<u8>>();
                        v.push(*b);
                        heap_buf = Some(v);
                    }
                }
                source.next();
            }
        }
    }

    Err(ERR_UNTERMINATED_STRING.to_string())
}

/// Parses JSON numbers including integers, floats and scientific notation
/// Supports negative numbers and exponential notation
///
/// # Arguments
/// * `source` - Source to read characters from
///
/// # Returns
/// * `Result<Node, String>` - Number Node or error message
fn parse_number(source: &mut dyn ISource) -> Result<Node, String> {
    use arrayvec::ArrayString;
    let mut num_buf: ArrayString<32> = ArrayString::new();
    let mut is_float = false;

    // Handle negative numbers
    if source.current() == Some(MINUS) {
        let _ = num_buf.push(MINUS);
        source.next();
    }

    while let Some(c) = source.current() {
        match c {
            '0'..='9' => {
                let _ = num_buf.push(c);
                source.next();
            }
            DECIMAL_POINT => {
                if is_float {
                    return Err(ERR_MULTIPLE_DECIMAL.to_string());
                }
                is_float = true;
                let _ = num_buf.push(c);
                source.next();
            }
            EXPONENT_LOWER | EXPONENT_UPPER => {
                is_float = true;
                let _ = num_buf.push(c);
                source.next();

                if let Some(sign) = source.current() {
                    if sign == PLUS || sign == MINUS {
                        let _ = num_buf.push(sign);
                        source.next();
                    }
                }
            }
            _ => break,
        }
    }

    let num_str: &str = num_buf.as_str();
    if is_float {
        match num_str.parse::<f64>() {
            Ok(n) => Ok(Node::Number(Numeric::Float(n))),
            Err(_) => Err(ERR_INVALID_FLOAT.to_string()),
        }
    } else {
        match num_str.parse::<i64>() {
            Ok(n) => Ok(Node::Number(Numeric::Integer(n))),
            Err(_) => Err(ERR_INVALID_INTEGER.to_string()),
        }
    }
}

fn parse_true(source: &mut dyn ISource) -> Result<Node, String> {
    source.next(); // Skip 't'
    for c in ['r', 'u', 'e'] {
        if source.current() != Some(c) {
            return Err(ERR_EXPECT_TRUE.to_string());
        }
        source.next();
    }
    Ok(Node::Boolean(true))
}

fn parse_false(source: &mut dyn ISource) -> Result<Node, String> {
    source.next(); // Skip 'f'
    for c in ['a', 'l', 's', 'e'] {
        if source.current() != Some(c) {
            return Err(ERR_EXPECT_FALSE.to_string());
        }
        source.next();
    }
    Ok(Node::Boolean(false))
}

fn parse_null(source: &mut dyn ISource) -> Result<Node, String> {
    source.next(); // Skip 'n'
    for c in ['u', 'l', 'l'] {
        if source.current() != Some(c) {
            return Err(ERR_EXPECT_NULL.to_string());
        }
        source.next();
    }
    Ok(Node::None)
}

#[cfg(test)]
/// Test module for JSON parser functionality
/// Includes tests for all JSON data types and error conditions
mod tests {
    use super::*;
    use crate::io::sources::buffer::Buffer;

    #[test]
    fn test_parse_null() {
        let mut source = Buffer::new(b"null");
        assert!(matches!(parse(&mut source), Ok(Node::None)));
    }

    #[test]
    fn test_parse_true() {
        let mut source = Buffer::new(b"true");
        assert!(matches!(parse(&mut source), Ok(Node::Boolean(true))));
    }

    #[test]
    fn test_parse_false() {
        let mut source = Buffer::new(b"false");
        assert!(matches!(parse(&mut source), Ok(Node::Boolean(false))));
    }

    #[test]
    fn test_parse_string() {
        let mut source = Buffer::new(b"\"hello\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "hello"));
    }

    #[test]
    fn test_parse_escaped_string() {
        let mut source = Buffer::new(b"\"hello\\\"world\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "hello\"world"));
    }

    #[test]
    fn test_parse_number_integer() {
        let mut source = Buffer::new(b"123");
        assert!(matches!(
            parse(&mut source),
            Ok(Node::Number(Numeric::Integer(123)))
        ));
    }

    #[test]
    fn test_parse_number_float() {
        let mut source = Buffer::new(b"123.45");
        assert!(
            matches!(parse(&mut source), Ok(Node::Number(Numeric::Float(n))) if (n - 123.45).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_parse_array() {
        let mut source = Buffer::new(b"[1,2,3]");
        match parse(&mut source) {
            Ok(Node::Array(arr)) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_object() {
        let mut source = Buffer::new(b"{\"key\":\"value\"}");
        match parse(&mut source) {
            Ok(Node::Object(obj)) => assert_eq!(obj.len(), 1),
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let mut source = Buffer::new(b"[]");
        match parse(&mut source) {
            Ok(Node::Array(arr)) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected empty array"),
        }
    }

    #[test]
    fn test_parse_empty_object() {
        let mut source = Buffer::new(b"{}");
        match parse(&mut source) {
            Ok(Node::Object(obj)) => assert_eq!(obj.len(), 0),
            _ => panic!("Expected empty object"),
        }
    }

    #[test]
    fn test_invalid_number() {
        let mut source = Buffer::new(b"12.34.56");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_invalid_escape() {
        let mut source = Buffer::new(b"\"\\x\"");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_unterminated_string() {
        let mut source = Buffer::new(b"\"unterminated");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_whitespace() {
        let mut source = Buffer::new(b" \t\n\r{} ");
        match parse(&mut source) {
            Ok(Node::Object(obj)) => assert_eq!(obj.len(), 0),
            _ => panic!("Expected empty object"),
        }
    }

    #[test]
    fn test_unicode_escape() {
        let mut source = Buffer::new(b"\"\\u0048\\u0065\\u006c\\u006c\\u006f\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "Hello"));
    }

    #[test]
    fn test_mixed_unicode_escape() {
        let mut source = Buffer::new(b"\"Hello, \\u0057\\u006f\\u0072\\u006c\\u0064!\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "Hello, World!"));
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let mut source = Buffer::new(b"\"\\u00\"");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_invalid_unicode_hex() {
        let mut source = Buffer::new(b"\"\\u00zz\"");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_parse_negative_number() {
        let mut source = Buffer::new(b"-123");
        assert!(matches!(
            parse(&mut source),
            Ok(Node::Number(Numeric::Integer(-123)))
        ));

        let mut source = Buffer::new(b"-123.45");
        assert!(
            matches!(parse(&mut source), Ok(Node::Number(Numeric::Float(n))) if (n - -123.45).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_parse_scientific_notation() {
        let mut source = Buffer::new(b"1.23e+2");
        assert!(
            matches!(parse(&mut source), Ok(Node::Number(Numeric::Float(n))) if (n - 123.0).abs() < f64::EPSILON)
        );

        let mut source = Buffer::new(b"1.23E-2");
        assert!(
            matches!(parse(&mut source), Ok(Node::Number(Numeric::Float(n))) if (n - 0.0123).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn test_parse_complex_object() {
        let mut source =
            Buffer::new(b"{\"array\":[1,{\"nested\":true},null],\"string\":\"value\"}");
        match parse(&mut source) {
            Ok(Node::Object(obj)) => {
                assert_eq!(obj.len(), 2);
                assert!(obj.contains_key("array"));
                assert!(obj.contains_key("string"));
            }
            _ => panic!("Expected complex object"),
        }
    }

    #[test]
    fn test_invalid_syntax() {
        let mut source = Buffer::new(b"{\"key\": value}");
        assert!(parse(&mut source).is_err());

        let mut source = Buffer::new(b"[1,2,]");
        assert!(parse(&mut source).is_err());

        let mut source = Buffer::new(b"{,}");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_string_escapes() {
        let mut source = Buffer::new(b"\"\\\"\\\\\\/\\b\\f\\n\\r\\t\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "\"\\/\x08\x0c\n\r\t"));
    }
    #[test]
    fn test_string_unicode_escapes() {
        let mut source = Buffer::new(b"\"\\u0048\\u0065\\u006c\\u006c\\u006f\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "Hello"));
    }
    #[test]
    fn test_string_unicode_escapes_mixed() {
        let mut source = Buffer::new(b"\"Hello, \\u0057\\u006f\\u0072\\u006c\\u0064!\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "Hello, World!"));
    }
    #[test]
    fn test_string_unicode_escapes_invalid() {
        let mut source = Buffer::new(b"\"\\u00\"");
        assert!(parse(&mut source).is_err());
    }
    #[test]
    fn test_string_unicode_escapes_invalid_hex() {
        let mut source = Buffer::new(b"\"\\u00zz\"");
        assert!(parse(&mut source).is_err());
    }

    // Configuration tests
    #[test]
    fn test_max_depth_exceeded() {
        let config = ParserConfig::new().with_max_depth(Some(2));
        let mut source = Buffer::new(b"{\"a\":{\"b\":{\"c\":1}}}");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("depth"));
    }

    #[test]
    fn test_max_depth_within_limit() {
        let config = ParserConfig::new().with_max_depth(Some(4));
        let mut source = Buffer::new(b"{\"a\":{\"b\":{\"c\":1}}}");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_string_length_exceeded() {
        let config = ParserConfig::new().with_max_string_length(Some(5));
        let mut source = Buffer::new(b"\"verylongstring\"");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("string length"));
    }

    #[test]
    fn test_max_string_length_within_limit() {
        let config = ParserConfig::new().with_max_string_length(Some(10));
        let mut source = Buffer::new(b"\"short\"");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_array_size_exceeded() {
        let config = ParserConfig::new().with_max_array_size(Some(3));
        let mut source = Buffer::new(b"[1,2,3,4,5]");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("array size"));
    }

    #[test]
    fn test_max_array_size_within_limit() {
        let config = ParserConfig::new().with_max_array_size(Some(5));
        let mut source = Buffer::new(b"[1,2,3]");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_object_size_exceeded() {
        let config = ParserConfig::new().with_max_object_size(Some(2));
        let mut source = Buffer::new(b"{\"a\":1,\"b\":2,\"c\":3}");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("object size"));
    }

    #[test]
    fn test_max_object_size_within_limit() {
        let config = ParserConfig::new().with_max_object_size(Some(5));
        let mut source = Buffer::new(b"{\"a\":1,\"b\":2}");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strict_config() {
        let config = ParserConfig::strict();
        // Should fail with deeply nested structure
        let mut source = Buffer::new(b"[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_unlimited_config() {
        let config = ParserConfig::unlimited();
        // Should succeed even with deeply nested structure
        let mut source = Buffer::new(b"[[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]");
        let result = parse_with_config(&mut source, &config);
        assert!(result.is_ok());
    }

    // from_str tests
    #[test]
    fn test_from_str_null() {
        assert!(matches!(from_str("null"), Ok(Node::None)));
    }

    #[test]
    fn test_from_str_boolean() {
        assert!(matches!(from_str("true"), Ok(Node::Boolean(true))));
        assert!(matches!(from_str("false"), Ok(Node::Boolean(false))));
    }

    #[test]
    fn test_from_str_integer() {
        assert!(matches!(
            from_str("42"),
            Ok(Node::Number(Numeric::Integer(42)))
        ));
    }

    #[test]
    fn test_from_str_negative_float() {
        let result = from_str("-3.14").unwrap();
        match result {
            Node::Number(Numeric::Float(n)) => assert!((n - (-3.14)).abs() < f64::EPSILON),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_from_str_string() {
        assert!(matches!(from_str("\"hello\""), Ok(Node::Str(s)) if s == "hello"));
    }

    #[test]
    fn test_from_str_array() {
        let node = from_str("[1,2,3]").unwrap();
        match node {
            Node::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_from_str_object() {
        let node = from_str("{\"x\":1}").unwrap();
        match node {
            Node::Object(obj) => assert!(obj.contains_key("x")),
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_from_str_error() {
        assert!(from_str("").is_err());
        assert!(from_str("{bad}").is_err());
    }

    // from_bytes tests
    #[test]
    fn test_from_bytes_null() {
        assert!(matches!(from_bytes(b"null"), Ok(Node::None)));
    }

    #[test]
    fn test_from_bytes_boolean() {
        assert!(matches!(from_bytes(b"true"), Ok(Node::Boolean(true))));
        assert!(matches!(from_bytes(b"false"), Ok(Node::Boolean(false))));
    }

    #[test]
    fn test_from_bytes_integer() {
        assert!(matches!(
            from_bytes(b"0"),
            Ok(Node::Number(Numeric::Integer(0)))
        ));
    }

    #[test]
    fn test_from_bytes_string() {
        assert!(matches!(from_bytes(b"\"world\""), Ok(Node::Str(s)) if s == "world"));
    }

    #[test]
    fn test_from_bytes_array() {
        let node = from_bytes(b"[true,false,null]").unwrap();
        match node {
            Node::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_from_bytes_object() {
        let node = from_bytes(b"{\"k\":\"v\"}").unwrap();
        match node {
            Node::Object(obj) => {
                assert_eq!(obj.len(), 1);
                assert!(matches!(obj.get("k"), Some(Node::Str(s)) if s == "v"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_from_bytes_error() {
        assert!(from_bytes(b"").is_err());
        assert!(from_bytes(b"[1,2,").is_err());
    }

    // Number edge cases
    #[test]
    fn test_parse_zero() {
        let mut source = Buffer::new(b"0");
        assert!(matches!(
            parse(&mut source),
            Ok(Node::Number(Numeric::Integer(0)))
        ));
    }

    #[test]
    fn test_parse_large_integer() {
        let mut source = Buffer::new(b"9007199254740991");
        assert!(matches!(
            parse(&mut source),
            Ok(Node::Number(Numeric::Integer(9007199254740991)))
        ));
    }

    #[test]
    fn test_parse_scientific_no_decimal() {
        let mut source = Buffer::new(b"5e2");
        match parse(&mut source) {
            Ok(Node::Number(Numeric::Float(n))) => assert!((n - 500.0).abs() < f64::EPSILON),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_parse_float_zero() {
        let mut source = Buffer::new(b"0.0");
        match parse(&mut source) {
            Ok(Node::Number(Numeric::Float(n))) => assert!(n == 0.0),
            _ => panic!("Expected float"),
        }
    }

    // Object and array content tests
    #[test]
    fn test_array_mixed_types() {
        let node = from_bytes(b"[1,\"two\",true,null,3.14]").unwrap();
        match node {
            Node::Array(arr) => {
                assert_eq!(arr.len(), 5);
                assert!(matches!(arr[0], Node::Number(Numeric::Integer(1))));
                assert!(matches!(&arr[1], Node::Str(s) if s == "two"));
                assert!(matches!(arr[2], Node::Boolean(true)));
                assert!(matches!(arr[3], Node::None));
                match arr[4] {
                    Node::Number(Numeric::Float(n)) => assert!((n - 3.14).abs() < 1e-10),
                    _ => panic!("Expected float"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_object_multiple_keys() {
        let node = from_str(r#"{"a":1,"b":"two","c":true,"d":null}"#).unwrap();
        match node {
            Node::Object(obj) => {
                assert_eq!(obj.len(), 4);
                assert!(matches!(
                    obj.get("a"),
                    Some(Node::Number(Numeric::Integer(1)))
                ));
                assert!(matches!(obj.get("b"), Some(Node::Str(s)) if s == "two"));
                assert!(matches!(obj.get("c"), Some(Node::Boolean(true))));
                assert!(matches!(obj.get("d"), Some(Node::None)));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_object_with_whitespace() {
        let node = from_str("{ \"key\" : \"value\" }").unwrap();
        match node {
            Node::Object(obj) => {
                assert_eq!(obj.len(), 1);
                assert!(matches!(obj.get("key"), Some(Node::Str(s)) if s == "value"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_array_with_whitespace() {
        let node = from_str("[ 1 , 2 , 3 ]").unwrap();
        match node {
            Node::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }

    // Error message tests
    #[test]
    fn test_empty_input_error() {
        let mut source = Buffer::new(b"");
        let err = parse(&mut source).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn test_missing_colon_error() {
        let mut source = Buffer::new(b"{\"key\" \"value\"}");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_missing_object_end_error() {
        let mut source = Buffer::new(b"{\"k\":1");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_missing_array_end_error() {
        let mut source = Buffer::new(b"[1,2,3");
        assert!(parse(&mut source).is_err());
    }

    #[test]
    fn test_unexpected_char_error() {
        let mut source = Buffer::new(b"@");
        assert!(parse(&mut source).is_err());
    }

    // Boundary condition tests for config limits
    #[test]
    fn test_depth_exactly_at_limit() {
        // depth limit of 3: top-level (depth 0) → object (depth 1) → value (depth 1 incremented to 2)
        // nesting {"a":{"b":1}} means parsing outer object at depth 0, inner at depth 1, value at depth 2
        let config = ParserConfig::new().with_max_depth(Some(3));
        let mut source = Buffer::new(b"{\"a\":{\"b\":1}}");
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_array_size_exactly_at_limit() {
        let config = ParserConfig::new().with_max_array_size(Some(3));
        let mut source = Buffer::new(b"[1,2,3]");
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_array_size_one_over_limit() {
        let config = ParserConfig::new().with_max_array_size(Some(3));
        let mut source = Buffer::new(b"[1,2,3,4]");
        let err = parse_with_config(&mut source, &config).unwrap_err();
        assert!(err.contains("array size"));
    }

    #[test]
    fn test_object_size_exactly_at_limit() {
        let config = ParserConfig::new().with_max_object_size(Some(2));
        let mut source = Buffer::new(b"{\"a\":1,\"b\":2}");
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_string_length_within_limit() {
        // Limit is checked before the closing quote, so a 4-char string with limit 5 succeeds
        let config = ParserConfig::new().with_max_string_length(Some(5));
        let mut source = Buffer::new(b"\"abcd\"");
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_string_length_one_over_limit() {
        let config = ParserConfig::new().with_max_string_length(Some(5));
        let mut source = Buffer::new(b"\"toolong\"");
        let err = parse_with_config(&mut source, &config).unwrap_err();
        assert!(err.contains("string length"));
    }

    // UTF-8 string content via Unicode escapes
    #[test]
    fn test_multibyte_utf8_string() {
        // Parser uses \uXXXX escapes for non-ASCII; verify CJK characters round-trip correctly
        let node = from_str("\"\\u4e2d\\u6587\"").unwrap();
        assert!(matches!(node, Node::Str(s) if s == "中文"));
    }

    #[test]
    fn test_unicode_escape_emoji_plane() {
        // U+0041 = 'A'
        let mut source = Buffer::new(b"\"\\u0041\"");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "A"));
    }

    // Nested structure tests
    #[test]
    fn test_nested_array_in_object() {
        let node = from_str(r#"{"nums":[10,20,30]}"#).unwrap();
        match node {
            Node::Object(obj) => match obj.get("nums") {
                Some(Node::Array(arr)) => assert_eq!(arr.len(), 3),
                _ => panic!("Expected array value"),
            },
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_nested_object_in_array() {
        let node = from_bytes(b"[{\"x\":1},{\"y\":2}]").unwrap();
        match node {
            Node::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert!(matches!(&arr[0], Node::Object(_)));
                assert!(matches!(&arr[1], Node::Object(_)));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_with_config_no_limits() {
        let config = ParserConfig::new();
        let mut source = Buffer::new(b"{\"deep\":[[[1,2,3]]]}");
        assert!(parse_with_config(&mut source, &config).is_ok());
    }
}
