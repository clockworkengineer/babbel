//! YAML string conversion module for Node structures
//! Provides functionality to convert Node types into YAML formatted strings

use crate::io::traits::IDestination;
use crate::nodes::node::*;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Converts a Node into a YAML formatted string and writes it to the destination
///
/// # Arguments
/// * `node` - The Node to convert
/// * `destination` - The output destination implementing IDestination
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    stringify_with_indent(node, destination, 0);
    Ok(())
}

/// Converts a Node into a YAML formatted string with proper indentation
///
/// # Arguments
/// * `node` - The Node to convert
/// * `destination` - The output destination implementing IDestination
/// * `indent` - Current indentation level in spaces
fn stringify_with_indent(node: &Node, destination: &mut dyn IDestination, indent: usize) {
    match node {
        // Handle null values
        Node::None => destination.add_bytes("null"),
        // Handle boolean values
        Node::Boolean(value) => destination.add_bytes(if *value { "true" } else { "false" }),
        // Handle different numeric types
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
            #[allow(unreachable_patterns)]
            _ => destination.add_bytes(&format!("{:?}", value)),
        },
        // Handle string values with special treatment for multi-line and quoted strings
        Node::Str(value) => {
            if value.contains('\n') || value.contains('"') {
                destination.add_bytes("|\n");
                for line in value.lines() {
                    destination.add_bytes(&" ".repeat(indent + 2));
                    destination.add_bytes(line);
                    destination.add_bytes("\n");
                }
            } else {
                destination.add_bytes(value);
            }
        }
        // Handle arrays with proper YAML list formatting
        Node::Array(items) => {
            if items.is_empty() {
                destination.add_bytes("[]");
                return;
            }
            destination.add_bytes("\n");
            for item in items {
                destination.add_bytes(&" ".repeat(indent));
                destination.add_bytes("- ");
                stringify_with_indent(item, destination, indent + 2);
                destination.add_bytes("\n");
            }
        }
        // Handle objects/maps with proper YAML mapping formatting
        Node::Object(entries) => {
            if entries.is_empty() {
                destination.add_bytes("{}");
                return;
            }
            destination.add_bytes("\n");
            for (key, value) in entries {
                destination.add_bytes(&" ".repeat(indent));
                destination.add_bytes(key);
                destination.add_bytes(": ");
                stringify_with_indent(value, destination, indent + 2);
                destination.add_bytes("\n");
            }
        }
    }
}

#[cfg(test)]
/// Tests for YAML stringification functionality
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
        dest.clear();
        stringify(&Node::Boolean(false), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "false");
    }

    #[test]
    fn test_stringify_numbers() {
        let mut dest = Buffer::new();
        let test_cases = vec![
            (Node::Number(Numeric::Integer(-42)), "-42"),
            (Node::Number(Numeric::UInteger(42)), "42"),
            (Node::Number(Numeric::Float(42.5)), "42.5"),
            (Node::Number(Numeric::Byte(255)), "255"),
            (Node::Number(Numeric::Int32(-2147483648)), "-2147483648"),
            (Node::Number(Numeric::UInt32(4294967295)), "4294967295"),
        ];
        for (node, expected) in test_cases {
            dest.clear();
            stringify(&node, &mut dest).unwrap();
            assert_eq!(dest.to_string(), expected);
        }
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("simple".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "simple");

        dest.clear();
        stringify(&Node::Str("multi\nline".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "|\n  multi\n  line\n");

        dest.clear();
        stringify(&Node::Str("with \"quotes\"".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "|\n  with \"quotes\"\n");
    }

    #[test]
    fn test_stringify_array() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[]");

        dest.clear();
        stringify(
            &Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Str("test".to_string()),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\n- 1\n- test\n");
    }

    #[test]
    fn test_stringify_object() {
        let mut dest = Buffer::new();
        stringify(&Node::Object(HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{}");

        dest.clear();
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::Str("value".to_string()));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nkey: value\n");
    }

    #[test]
    fn test_stringify_complex() {
        let mut dest = Buffer::new();
        let mut inner_map = HashMap::new();
        inner_map.insert(
            "nested".to_string(),
            Node::Array(vec![
                Node::Boolean(true),
                Node::Str("multi\nline".to_string()),
            ]),
        );
        let mut outer_map = HashMap::new();
        outer_map.insert("test".to_string(), Node::Object(inner_map));
        stringify(&Node::Object(outer_map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "\ntest: \n  nested: \n    - true\n    - |\n        multi\n        line\n\n\n\n"
        );
    }
    #[test]
    fn test_stringify_empty_array() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[]");
    }

    #[test]
    fn test_stringify_nested_arrays() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Array(vec![
                    Node::Number(Numeric::Integer(1)),
                    Node::Number(Numeric::Integer(2)),
                ]),
                Node::Array(vec![
                    Node::Number(Numeric::Integer(3)),
                    Node::Number(Numeric::Integer(4)),
                ]),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(
            dest.to_string(),
            "\n- \n  - 1\n  - 2\n\n- \n  - 3\n  - 4\n\n"
        );
    }

    #[test]
    fn test_stringify_complex_string() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Str("Hello \"world\"\nWith\nMultiple\nLines".to_string()),
            &mut dest,
        )
        .unwrap();
        assert_eq!(
            dest.to_string(),
            "|\n  Hello \"world\"\n  With\n  Multiple\n  Lines\n"
        );
    }

    #[test]
    fn test_stringify_array_with_empty_object() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Object(HashMap::new()),
                Node::Number(Numeric::Integer(1)),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\n- {}\n- 1\n");
    }
    #[test]
    fn test_stringify_empty_object() {
        let mut dest = Buffer::new();
        stringify(&Node::Object(HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{}");
    }
    #[test]
    fn test_stringify_empty_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("".to_string()), &mut dest).unwrap();
    }
    #[test]
    fn test_stringify_empty_number() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }
    #[test]
    fn test_stringify_empty_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(false), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "false");
    }
    #[test]
    fn test_stringify_empty_null() {
        let mut dest = Buffer::new();
        stringify(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "null");
    }
    #[test]
    fn test_stringify_empty_array_with_empty_string() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Str("".to_string()),
                Node::Number(Numeric::Integer(1)),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\n- \n- 1\n");
    }

    // Numeric variants individually
    #[test]
    fn test_stringify_integer_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    #[test]
    fn test_stringify_integer_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(-1)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "-1");
    }

    #[test]
    fn test_stringify_integer_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(i64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i64::MAX.to_string());
    }

    #[test]
    fn test_stringify_uinteger_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInteger(u64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), u64::MAX.to_string());
    }

    #[test]
    fn test_stringify_float_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(-3.14)), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.starts_with("-3.14"), "got: {}", s);
    }

    #[test]
    fn test_stringify_float_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(0.0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0.0");
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
    fn test_stringify_uint32_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInt32(u32::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), u32::MAX.to_string());
    }

    #[test]
    fn test_stringify_int32_min() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Int32(i32::MIN)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i32::MIN.to_string());
    }

    // String variants
    #[test]
    fn test_stringify_string_no_special_chars() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("hello world".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "hello world");
    }

    #[test]
    fn test_stringify_string_only_newline_triggers_block() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\nb".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "|\n  a\n  b\n");
    }

    #[test]
    fn test_stringify_string_only_quote_triggers_block() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("say \"hi\"".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "|\n  say \"hi\"\n");
    }

    #[test]
    fn test_stringify_string_single_line_block() {
        // A string with a trailing newline only — one empty line in block
        let mut dest = Buffer::new();
        stringify(&Node::Str("line\n".to_string()), &mut dest).unwrap();
        // lines() splits but ignores trailing newline — produces one entry
        assert_eq!(dest.to_string(), "|\n  line\n");
    }

    // Array — single element
    #[test]
    fn test_stringify_array_single_null() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::None]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\n- null\n");
    }

    #[test]
    fn test_stringify_array_single_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::Boolean(true)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\n- true\n");
    }

    #[test]
    fn test_stringify_array_of_nulls() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::None, Node::None]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\n- null\n- null\n");
    }

    #[test]
    fn test_stringify_array_of_booleans() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::Boolean(true), Node::Boolean(false)]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\n- true\n- false\n");
    }

    #[test]
    fn test_stringify_array_empty_inside_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("list".to_string(), Node::Array(vec![]));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nlist: []\n");
    }

    // Object variants
    #[test]
    fn test_stringify_object_single_bool_value() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("active".to_string(), Node::Boolean(true));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nactive: true\n");
    }

    #[test]
    fn test_stringify_object_single_null_value() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("nothing".to_string(), Node::None);
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nnothing: null\n");
    }

    #[test]
    fn test_stringify_object_single_number_value() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("n".to_string(), Node::Number(Numeric::Integer(42)));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nn: 42\n");
    }

    #[test]
    fn test_stringify_object_nested_empty_array() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("arr".to_string(), Node::Array(vec![]));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\narr: []\n");
    }

    #[test]
    fn test_stringify_object_nested_empty_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("sub".to_string(), Node::Object(HashMap::new()));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nsub: {}\n");
    }

    // Indentation inside nested array
    #[test]
    fn test_stringify_array_items_indented_at_depth2() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert(
            "nums".to_string(),
            Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
            ]),
        );
        stringify(&Node::Object(map), &mut dest).unwrap();
        let result = dest.to_string();
        // Items inside object's array are indented by 2
        assert!(result.contains("  - 1"), "got: {}", result);
        assert!(result.contains("  - 2"), "got: {}", result);
    }

    // Block literal indented inside nested object
    #[test]
    fn test_stringify_multiline_string_at_depth2() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("msg".to_string(), Node::Str("line1\nline2".to_string()));
        stringify(&Node::Object(map), &mut dest).unwrap();
        let result = dest.to_string();
        // Block literal lines are indent+2 = 4 spaces deep
        assert!(result.contains("    line1"), "got: {}", result);
        assert!(result.contains("    line2"), "got: {}", result);
    }

    // Return value
    #[test]
    fn test_stringify_returns_ok_all_types() {
        let nodes = vec![
            Node::None,
            Node::Boolean(false),
            Node::Number(Numeric::Integer(0)),
            Node::Str("".to_string()),
            Node::Array(vec![]),
            Node::Object(HashMap::new()),
        ];
        for node in &nodes {
            let mut dest = Buffer::new();
            assert!(stringify(node, &mut dest).is_ok());
        }
    }
}
