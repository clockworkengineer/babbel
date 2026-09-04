//! Pretty-printing JSON stringifier module

use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::escape::*;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Serializes a `Node` into pretty-printed JSON
///
/// # Arguments
/// * `node` - The JSON node to serialize
/// * `destination` - The destination to write to
/// * `indent` - The indentation string (e.g., "  " for 2 spaces, "\t" for tabs)
pub fn stringify_pretty(
    node: &Node,
    destination: &mut dyn IDestination,
    indent: &str,
) -> Result<(), String> {
    stringify_pretty_internal(node, destination, indent, 0)
}

fn stringify_pretty_internal(
    node: &Node,
    destination: &mut dyn IDestination,
    indent: &str,
    depth: usize,
) -> Result<(), String> {
    match node {
        Node::None => destination.add_bytes(JSON_NULL),
        Node::Boolean(value) => destination.add_bytes(if *value { JSON_TRUE } else { JSON_FALSE }),
        Node::Number(value) => match value {
            Numeric::Integer(n) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*n));
            }
            Numeric::UInteger(n) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*n));
            }
            Numeric::Float(f) => {
                let mut buf = dtoa::Buffer::new();
                destination.add_bytes(buf.format(*f));
            }
            Numeric::Byte(b) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*b as u64));
            }
            Numeric::Int32(i) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*i));
            }
            Numeric::UInt32(u) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*u));
            }
            Numeric::Int16(i) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*i));
            }
            Numeric::UInt16(u) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*u));
            }
            Numeric::Int8(i) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes(buf.format(*i));
            }
        },
        Node::Str(value) => write_escaped_string(value, destination),
        Node::Array(items) => {
            if items.is_empty() {
                destination.add_bytes("[]");
            } else {
                destination.add_bytes("[\n");
                for (index, item) in items.iter().enumerate() {
                    // Add indentation
                    for _ in 0..=depth {
                        destination.add_bytes(indent);
                    }
                    stringify_pretty_internal(item, destination, indent, depth + 1)?;
                    if index < items.len() - 1 {
                        destination.add_bytes(STR_COMMA);
                    }
                    destination.add_bytes("\n");
                }
                // Closing bracket indentation
                for _ in 0..depth {
                    destination.add_bytes(indent);
                }
                destination.add_bytes(STR_ARRAY_END);
            }
        }
        Node::Object(map) => {
            if map.is_empty() {
                destination.add_bytes("{}");
            } else {
                destination.add_bytes("{\n");
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort(); // Sort keys for consistent output

                for (index, key) in keys.iter().enumerate() {
                    // Add indentation
                    for _ in 0..=depth {
                        destination.add_bytes(indent);
                    }
                    write_escaped_string(key, destination);
                    destination.add_bytes(": ");
                    let value = map.get(*key).unwrap();
                    stringify_pretty_internal(value, destination, indent, depth + 1)?;
                    if index < keys.len() - 1 {
                        destination.add_bytes(STR_COMMA);
                    }
                    destination.add_bytes("\n");
                }
                // Closing brace indentation
                for _ in 0..depth {
                    destination.add_bytes(indent);
                }
                destination.add_bytes(STR_OBJECT_END);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[cfg(feature = "std")]
    use std::collections::HashMap;

    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as HashMap;

    #[test]
    fn test_pretty_simple_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("name".to_string(), Node::Str("Alice".to_string()));
        map.insert("age".to_string(), Node::Number(Numeric::Int32(30)));

        stringify_pretty(&Node::Object(map), &mut dest, "  ").unwrap();
        let result = dest.to_string();

        assert!(result.contains("\"age\": 30"));
        assert!(result.contains("\"name\": \"Alice\""));
    }

    #[test]
    fn test_pretty_nested() {
        let mut dest = Buffer::new();
        let mut inner = HashMap::new();
        inner.insert("city".to_string(), Node::Str("NYC".to_string()));

        let mut outer = HashMap::new();
        outer.insert("address".to_string(), Node::Object(inner));

        stringify_pretty(&Node::Object(outer), &mut dest, "  ").unwrap();
        let result = dest.to_string();

        assert!(result.contains("\"address\": {"));
        assert!(result.contains("    \"city\": \"NYC\""));
    }

    #[test]
    fn test_pretty_array() {
        let mut dest = Buffer::new();
        let array = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
        ]);

        stringify_pretty(&array, &mut dest, "  ").unwrap();
        let result = dest.to_string();

        assert!(result.contains("[\n"));
        assert!(result.contains("  1"));
        assert!(result.contains("]"));
    }

    // Scalars
    #[test]
    fn test_pretty_null() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::None, &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_pretty_boolean_true() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Boolean(true), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_pretty_boolean_false() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Boolean(false), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "false");
    }

    // Empty collections
    #[test]
    fn test_pretty_empty_array() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Array(vec![]), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "[]");
    }

    #[test]
    fn test_pretty_empty_object() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Object(HashMap::new()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "{}");
    }

    // Exact format checks
    #[test]
    fn test_pretty_single_array_element_exact() {
        let mut dest = Buffer::new();
        stringify_pretty(
            &Node::Array(vec![Node::Number(Numeric::Integer(42))]),
            &mut dest,
            "  ",
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[\n  42\n]");
    }

    #[test]
    fn test_pretty_single_key_object_exact() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::Number(Numeric::Integer(1)));
        stringify_pretty(&Node::Object(map), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "{\n  \"k\": 1\n}");
    }

    // All numeric variants
    #[test]
    fn test_pretty_integer() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::Integer(-9999)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "-9999");
    }

    #[test]
    fn test_pretty_integer_zero() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::Integer(0)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    #[test]
    fn test_pretty_uinteger() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::UInteger(u64::MAX)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), u64::MAX.to_string());
    }

    #[test]
    fn test_pretty_float() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::Float(2.718)), &mut dest, "  ").unwrap();
        let s = dest.to_string();
        assert!(s.starts_with("2.718"), "got: {}", s);
    }

    #[test]
    fn test_pretty_byte() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::Byte(200)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "200");
    }

    #[test]
    fn test_pretty_uint32() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::UInt32(u32::MAX)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), u32::MAX.to_string());
    }

    #[test]
    fn test_pretty_int16() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::Int16(-1000)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "-1000");
    }

    #[test]
    fn test_pretty_uint16() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::UInt16(65535)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "65535");
    }

    #[test]
    fn test_pretty_int8() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Number(Numeric::Int8(-128)), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "-128");
    }

    // String escaping
    #[test]
    fn test_pretty_string_plain() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("hello".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"hello\"");
    }

    #[test]
    fn test_pretty_string_empty() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"\"");
    }

    #[test]
    fn test_pretty_string_quote() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("say \"hi\"".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn test_pretty_string_backslash() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("a\\b".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"a\\\\b\"");
    }

    #[test]
    fn test_pretty_string_newline() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("a\nb".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"a\\nb\"");
    }

    #[test]
    fn test_pretty_string_tab() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("a\tb".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"a\\tb\"");
    }

    #[test]
    fn test_pretty_string_cr() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("a\rb".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"a\\rb\"");
    }

    #[test]
    fn test_pretty_string_control_char() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("\u{0001}".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"\\u0001\"");
    }

    #[test]
    fn test_pretty_string_multiple_escapes() {
        let mut dest = Buffer::new();
        stringify_pretty(&Node::Str("\"\\\n".to_string()), &mut dest, "  ").unwrap();
        assert_eq!(dest.to_string(), "\"\\\"\\\\\\n\"");
    }

    // Indent variants
    #[test]
    fn test_pretty_tab_indent() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("x".to_string(), Node::Number(Numeric::Integer(1)));
        stringify_pretty(&Node::Object(map), &mut dest, "\t").unwrap();
        assert_eq!(dest.to_string(), "{\n\t\"x\": 1\n}");
    }

    #[test]
    fn test_pretty_four_space_indent() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("y".to_string(), Node::Boolean(true));
        stringify_pretty(&Node::Object(map), &mut dest, "    ").unwrap();
        assert_eq!(dest.to_string(), "{\n    \"y\": true\n}");
    }

    // Sorted keys
    #[test]
    fn test_pretty_object_keys_sorted() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("z".to_string(), Node::Number(Numeric::Integer(3)));
        map.insert("a".to_string(), Node::Number(Numeric::Integer(1)));
        map.insert("m".to_string(), Node::Number(Numeric::Integer(2)));
        stringify_pretty(&Node::Object(map), &mut dest, "  ").unwrap();
        let result = dest.to_string();
        let a_pos = result.find("\"a\"").unwrap();
        let m_pos = result.find("\"m\"").unwrap();
        let z_pos = result.find("\"z\"").unwrap();
        assert!(
            a_pos < m_pos && m_pos < z_pos,
            "keys not sorted: {}",
            result
        );
    }

    // Multi-level nesting
    #[test]
    fn test_pretty_deep_nesting() {
        let mut dest = Buffer::new();
        let mut level2 = HashMap::new();
        level2.insert("v".to_string(), Node::Boolean(true));
        let mut level1 = HashMap::new();
        level1.insert("b".to_string(), Node::Object(level2));
        stringify_pretty(&Node::Object(level1), &mut dest, "  ").unwrap();
        let result = dest.to_string();
        assert!(
            result.contains("  \"b\": {"),
            "missing outer key: {}",
            result
        );
        assert!(
            result.contains("    \"v\": true"),
            "missing inner key: {}",
            result
        );
    }

    // Array of objects
    #[test]
    fn test_pretty_array_of_objects() {
        let mut dest = Buffer::new();
        let mut m = HashMap::new();
        m.insert("n".to_string(), Node::Number(Numeric::Integer(1)));
        stringify_pretty(&Node::Array(vec![Node::Object(m)]), &mut dest, "  ").unwrap();
        let result = dest.to_string();
        assert!(result.contains("\"n\": 1"), "got: {}", result);
    }

    // Object with array value
    #[test]
    fn test_pretty_object_with_array_value() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("items".to_string(), Node::Array(vec![Node::None]));
        stringify_pretty(&Node::Object(map), &mut dest, "  ").unwrap();
        let result = dest.to_string();
        assert!(result.contains("\"items\": ["), "got: {}", result);
        assert!(result.contains("null"), "got: {}", result);
    }

    // Mixed-type array
    #[test]
    fn test_pretty_mixed_type_array() {
        let mut dest = Buffer::new();
        stringify_pretty(
            &Node::Array(vec![
                Node::None,
                Node::Boolean(true),
                Node::Number(Numeric::Integer(7)),
                Node::Str("hi".to_string()),
            ]),
            &mut dest,
            "  ",
        )
        .unwrap();
        let result = dest.to_string();
        assert!(result.contains("null"));
        assert!(result.contains("true"));
        assert!(result.contains("7"));
        assert!(result.contains("\"hi\""));
    }

    // No trailing comma
    #[test]
    fn test_pretty_no_trailing_comma_array() {
        let mut dest = Buffer::new();
        stringify_pretty(
            &Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
            ]),
            &mut dest,
            "  ",
        )
        .unwrap();
        let result = dest.to_string();
        assert!(
            result.ends_with("2\n]"),
            "trailing comma detected: {}",
            result
        );
    }

    #[test]
    fn test_pretty_no_trailing_comma_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::Boolean(false));
        stringify_pretty(&Node::Object(map), &mut dest, "  ").unwrap();
        let result = dest.to_string();
        assert!(!result.contains(","), "unexpected comma: {}", result);
    }

    // Return value
    #[test]
    fn test_pretty_returns_ok() {
        let nodes: Vec<Node> = vec![
            Node::None,
            Node::Boolean(false),
            Node::Number(Numeric::Integer(0)),
            Node::Str("".to_string()),
            Node::Array(vec![]),
            Node::Object(HashMap::new()),
        ];
        for node in &nodes {
            let mut dest = Buffer::new();
            assert!(stringify_pretty(node, &mut dest, "  ").is_ok());
        }
    }
}
