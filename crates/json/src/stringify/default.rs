// Use itoa/dtoa for fast, allocation-free number formatting
/// JSON stringifier module.
///
/// Provides a function to serialize a `Node` (representing a JSON value) into a string
/// and write it to a destination implementing `IDestination`. Handles all JSON value types,
/// including strings (with proper escaping), numbers, booleans, arrays, objects, and nulls.
use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::escape::*;
use dtoa;
use itoa;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Serializes a `Node` into JSON and writes it to the given destination.
///
/// # Arguments
///
/// * `node` - The JSON node to serialize.
/// * `destination` - The destination to write the JSON string to.

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => destination.add_bytes(JSON_NULL),
        Node::Boolean(value) => destination.add_bytes(if *value { JSON_TRUE } else { JSON_FALSE }),
        Node::Number(value) => {
            let mut buf = itoa::Buffer::new();
            let mut fbuf = dtoa::Buffer::new();
            match value {
                Numeric::Integer(n) => destination.add_bytes(buf.format(*n)),
                Numeric::UInteger(n) => destination.add_bytes(buf.format(*n)),
                Numeric::Float(f) => destination.add_bytes(fbuf.format(*f)),
                Numeric::Byte(b) => destination.add_bytes(buf.format(*b)),
                Numeric::Int32(i) => destination.add_bytes(buf.format(*i)),
                Numeric::UInt32(u) => destination.add_bytes(buf.format(*u)),
                #[allow(unreachable_patterns)]
                _ => destination.add_bytes(JSON_NULL), // fallback for unknown numeric type
            }
        }
        Node::Str(value) => write_escaped_string(value, destination),
        Node::Array(items) => {
            destination.add_bytes(STR_ARRAY_START);
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    destination.add_bytes(STR_COMMA);
                }
                stringify(item, destination)?;
            }
            destination.add_bytes(STR_ARRAY_END);
        }
        Node::Object(entries) => {
            destination.add_bytes(STR_OBJECT_START);
            let mut first = true;
            for (key, value) in entries {
                if !first {
                    destination.add_bytes(STR_COMMA);
                }
                first = false;
                write_escaped_string(key, destination);
                destination.add_bytes(STR_COLON);
                stringify(value, destination)?;
            }
            destination.add_bytes(STR_OBJECT_END);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;
    use std::collections::HashMap;

    #[test]
    fn test_stringify_null() {
        let mut dest = Buffer::new();
        stringify(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_stringify_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_stringify_number() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(42.5)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "42.5");
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("Hello\n\"World\"".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"Hello\\n\\\"World\\\"\"");
    }

    #[test]
    fn test_stringify_array() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Number(Numeric::Float(1.0)),
                Node::Str("test".to_string()),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[1.0,\"test\"]");
    }

    #[test]
    fn test_stringify_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::Str("value".to_string()));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"key\":\"value\"}");
    }

    #[test]
    fn test_stringify_nested_objects() {
        let mut dest = Buffer::new();
        let mut inner_map = HashMap::new();
        inner_map.insert("inner_key".to_string(), Node::Number(Numeric::Integer(42)));
        let mut outer_map = HashMap::new();
        outer_map.insert("outer_key".to_string(), Node::Object(inner_map));
        stringify(&Node::Object(outer_map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"outer_key\":{\"inner_key\":42}}");
    }

    #[test]
    fn test_stringify_mixed_array() {
        let mut dest = Buffer::new();
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Node::Boolean(true));
        let array = Node::Array(vec![
            Node::Number(Numeric::Float(1.5)),
            Node::Array(vec![Node::Str("nested".to_string())]),
            Node::Object(obj),
            Node::None,
        ]);
        stringify(&array, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[1.5,[\"nested\"],{\"key\":true},null]");
    }

    #[test]
    fn test_stringify_special_characters() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert(
            "special\t\n".to_string(),
            Node::Str("value\u{0001}".to_string()),
        );
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"special\\t\\n\":\"value\\u0001\"}");
    }

    #[test]
    fn test_stringify_number_formats() {
        let mut dest = Buffer::new();
        let array = Node::Array(vec![
            Node::Number(Numeric::Integer(-42)),
            Node::Number(Numeric::UInteger(42)),
            Node::Number(Numeric::Float(42.42)),
            Node::Number(Numeric::Byte(255)),
            Node::Number(Numeric::Int32(-2147483648)),
            Node::Number(Numeric::UInt32(4294967295)),
        ]);
        stringify(&array, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[-42,42,42.42,255,-2147483648,4294967295]"
        );
    }

    #[test]
    fn test_stringify_empty_array() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[]");
    }

    #[test]
    fn test_stringify_empty_object() {
        let mut dest = Buffer::new();
        stringify(&Node::Object(HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{}");
    }

    #[test]
    fn test_stringify_control_chars() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("\u{0000}\u{001F}".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\\u0000\\u001f\"");
    }

    // Boolean
    #[test]
    fn test_stringify_boolean_false() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(false), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "false");
    }

    // Numbers — all variants and edge cases
    #[test]
    fn test_stringify_integer_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    #[test]
    fn test_stringify_integer_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(i64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i64::MAX.to_string());
    }

    #[test]
    fn test_stringify_integer_min() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(i64::MIN)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i64::MIN.to_string());
    }

    #[test]
    fn test_stringify_uinteger_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInteger(u64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), u64::MAX.to_string());
    }

    #[test]
    fn test_stringify_float_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(0.0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0.0");
    }

    #[test]
    fn test_stringify_float_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(-3.14)), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.starts_with("-3.14"), "got: {}", s);
    }

    #[test]
    fn test_stringify_byte_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Byte(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    #[test]
    fn test_stringify_byte_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Byte(255)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "255");
    }

    #[test]
    fn test_stringify_int32_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Int32(i32::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i32::MAX.to_string());
    }

    #[test]
    fn test_stringify_uint32_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInt32(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    // String escape sequences
    #[test]
    fn test_stringify_empty_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\"");
    }

    #[test]
    fn test_stringify_string_backslash() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\\b".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\\\b\"");
    }

    #[test]
    fn test_stringify_string_tab() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\tb".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\tb\"");
    }

    #[test]
    fn test_stringify_string_carriage_return() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\rb".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\rb\"");
    }

    #[test]
    fn test_stringify_string_newline() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("line1\nline2".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"line1\\nline2\"");
    }

    #[test]
    fn test_stringify_string_all_escapes() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("\"\\\n\r\t".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\\\"\\\\\\n\\r\\t\"");
    }

    #[test]
    fn test_stringify_string_no_escaping_needed() {
        let mut dest = Buffer::new();
        let s = "Hello, World! 12345 !@#$%^&*()".to_string();
        stringify(&Node::Str(s.clone()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("\"{}\"", s));
    }

    #[test]
    fn test_stringify_string_unicode_control_low() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("\u{0008}".to_string()), &mut dest).unwrap(); // backspace
        assert_eq!(dest.to_string(), "\"\\u0008\"");
    }

    #[test]
    fn test_stringify_string_key_with_escape() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("ke\ny".to_string(), Node::Boolean(true));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"ke\\ny\":true}");
    }

    // Array edge cases
    #[test]
    fn test_stringify_array_single_element() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::Boolean(true)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[true]");
    }

    #[test]
    fn test_stringify_array_nested_empty() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::Array(vec![]), Node::Array(vec![])]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[[],[]]");
    }

    #[test]
    fn test_stringify_array_of_nulls() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::None, Node::None, Node::None]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[null,null,null]");
    }

    #[test]
    fn test_stringify_array_commas_correct() {
        // Verify no leading/trailing comma
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
                Node::Number(Numeric::Integer(3)),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[1,2,3]");
    }

    // Object edge cases
    #[test]
    fn test_stringify_object_multiple_values() {
        // Single key to avoid HashMap ordering
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("n".to_string(), Node::None);
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"n\":null}");
    }

    #[test]
    fn test_stringify_object_empty_key() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("".to_string(), Node::Number(Numeric::Integer(1)));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"\":1}");
    }

    #[test]
    fn test_stringify_object_nested_array() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert(
            "arr".to_string(),
            Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
            ]),
        );
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"arr\":[1,2]}");
    }

    // Return value
    #[test]
    fn test_stringify_returns_ok_all_types() {
        let nodes = vec![
            Node::None,
            Node::Boolean(true),
            Node::Boolean(false),
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Float(1.0)),
            Node::Str("".to_string()),
            Node::Array(vec![]),
            Node::Object(HashMap::new()),
        ];
        for node in &nodes {
            let mut dest = Buffer::new();
            assert!(stringify(node, &mut dest).is_ok());
        }
    }

    // Roundtrip: parse then stringify
    #[test]
    fn test_stringify_roundtrip_object() {
        use crate::parser::default::from_str;
        let original = r#"{"a":1,"b":"hello","c":true,"d":null}"#;
        let node = from_str(original).unwrap();
        let mut dest = Buffer::new();
        stringify(&node, &mut dest).unwrap();
        // Re-parse the stringified output to verify it's valid JSON
        let reparsed = from_str(&dest.to_string());
        assert!(reparsed.is_ok());
        let reparsed = reparsed.unwrap();
        assert_eq!(reparsed["a"].as_i64(), Some(1));
        assert_eq!(reparsed["b"].as_str(), Some("hello"));
        assert!(reparsed["c"].is_boolean());
        assert!(reparsed["d"].is_null());
    }

    #[test]
    fn test_stringify_roundtrip_array() {
        use crate::parser::default::from_str;
        let original = "[1,2,3,\"four\",true,null]";
        let node = from_str(original).unwrap();
        let mut dest = Buffer::new();
        stringify(&node, &mut dest).unwrap();
        let reparsed = from_str(&dest.to_string()).unwrap();
        assert_eq!(reparsed.len(), Some(6));
    }
}
