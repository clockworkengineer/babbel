//! YAML serialization functionality for Bencode nodes.
//! Provides methods to convert Bencode data structures into YAML formatted output.

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::io::traits::{BencodeWrite, IDestination};
use crate::nodes::node::*;
use crate::stringify::common::escape_string;

/// Writes the specified number of indentation spaces to the destination.
///
/// # Arguments
/// * `level` - The indentation level (number of 2-space indents)
/// * `destination` - The output destination to write to
fn write_indent(level: usize, destination: &mut (impl BencodeWrite + ?Sized)) {
    for _ in 1..level {
        destination.write_bytes(b"  ");
    }
}

/// Recursively writes a Bencode node to the destination in YAML format.
///
/// # Arguments
/// * `node` - The Bencode node to serialize
/// * `level` - Current indentation level
/// * `destination` - The output destination to write to
fn write_node(node: &Node, level: usize, destination: &mut (impl BencodeWrite + ?Sized)) {
    match node {
        // Write integer values directly
        Node::Integer(n) => destination.write_bytes(n.to_string().as_bytes()),
        // Write strings with quotes and proper UTF-8 encoding
        Node::Str(s) => {
            destination.write_byte(b'"');
            escape_string(s, destination);
            destination.write_byte(b'"');
        }
        // Write lists with proper YAML array formatting
        Node::List(items) => {
            if items.is_empty() {
                destination.write_bytes(b"[]")
            } else {
                destination.write_bytes(b"\n");
                for item in items {
                    write_indent(level + 1, destination);
                    destination.write_bytes(b"- ");
                    write_node(item, level + 1, destination);
                    destination.write_bytes(b"\n");
                }
            }
        }
        // Write dictionaries with proper YAML mapping format
        Node::Dictionary(dict) => {
            if dict.is_empty() {
                destination.write_bytes(b"{}")
            } else {
                destination.write_bytes(b"\n");
                let mut sorted: Vec<_> = dict.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));
                for (key, value) in sorted {
                    write_indent(level + 1, destination);
                    destination.write_bytes(format!("{}: ", String::from_utf8_lossy(key.as_ref())).as_bytes());
                    write_node(value, level + 1, destination);
                    destination.write_bytes(b"\n");
                }
            }
        }
        // Handle unknown/unsupported node types
        _ => destination.write_bytes(b"unknown"),
    }
}

/// Converts a Bencode node to YAML format and writes it to the destination.
///
/// # Arguments
/// * `node` - The root Bencode node to serialize
/// * `destination` - The output destination to write the YAML to
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    write_node(node, 0, destination);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[test]
    fn stringify_empty_list_works() {
        let mut destination = Buffer::new();
        stringify(&Node::List(vec![]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[]");
    }

    #[test]
    fn stringify_list_works() {
        let mut destination = Buffer::new();
        stringify(
            &Node::List(vec![Node::Integer(1), Node::Integer(2)]),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "\n- 1\n- 2\n");
    }

    #[test]
    fn stringify_empty_dictionary_works() {
        let mut destination = Buffer::new();
        let dict = std::collections::HashMap::new();
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{}");
    }

    #[test]
    fn stringify_dictionary_works() {
        let mut destination = Buffer::new();
        let mut dict = std::collections::HashMap::new();
        dict.insert("key".to_string(), Node::Integer(1));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\nkey: 1\n");
    }

    #[test]
    fn stringify_integer_works() {
        let mut destination = Buffer::new();
        stringify(&Node::Integer(42), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "42");
    }

    #[test]
    fn stringify_string_works() {
        let mut destination = Buffer::new();
        stringify(&Node::Str(String::from("test")), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"test\"");
    }

    #[test]
    fn stringify_unknown_works() {
        let mut destination = Buffer::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "unknown");
    }

    // --- Integer edge cases ---

    #[test]
    fn stringify_integer_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(0), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    #[test]
    fn stringify_integer_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(-99), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "-99");
    }

    #[test]
    fn stringify_integer_max_i64() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(i64::MAX), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i64::MAX.to_string());
    }

    #[test]
    fn stringify_integer_min_i64() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(i64::MIN), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i64::MIN.to_string());
    }

    // --- String edge cases ---

    #[test]
    fn stringify_empty_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str(String::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\"");
    }

    #[test]
    fn stringify_string_single_char() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("z".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"z\"");
    }

    #[test]
    fn stringify_string_with_double_quote_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("say \"hi\"".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn stringify_string_with_backslash_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\\b".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\\\b\"");
    }

    #[test]
    fn stringify_string_with_newline_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("line1\nline2".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"line1\\u000aline2\"");
    }

    // --- List edge cases ---

    #[test]
    fn stringify_list_single_integer() {
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![Node::Integer(7)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\n- 7\n");
    }

    #[test]
    fn stringify_list_of_strings() {
        let mut dest = Buffer::new();
        stringify(
            &Node::List(vec![Node::Str("foo".into()), Node::Str("bar".into())]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\n- \"foo\"\n- \"bar\"\n");
    }

    #[test]
    fn stringify_list_three_integers() {
        let mut dest = Buffer::new();
        stringify(
            &Node::List(vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "\n- 1\n- 2\n- 3\n");
    }

    #[test]
    fn stringify_nested_list() {
        // Inner list is non-empty so it expands to \n, outer wraps it as a list item
        let inner = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![inner]), &mut dest).unwrap();
        let s = dest.to_string();
        // Should contain the outer "- " marker and the inner integers
        assert!(s.contains("- 1"));
        assert!(s.contains("- 2"));
    }

    #[test]
    fn stringify_list_item_count_matches_dash_count() {
        let list = Node::List(vec![
            Node::Integer(10),
            Node::Integer(20),
            Node::Integer(30),
        ]);
        let mut dest = Buffer::new();
        stringify(&list, &mut dest).unwrap();
        let s = dest.to_string();
        let dash_count = s.matches("- ").count();
        assert_eq!(dash_count, 3);
    }

    // --- Dictionary edge cases ---

    #[test]
    fn stringify_dict_string_value() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("name".to_string(), Node::Str("Alice".into()));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\nname: \"Alice\"\n");
    }

    #[test]
    fn stringify_dict_sorts_keys() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("z".to_string(), Node::Integer(1));
        dict.insert("a".to_string(), Node::Integer(2));
        dict.insert("m".to_string(), Node::Integer(3));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let s = dest.to_string();
        let pos_a = s.find("a: ").unwrap();
        let pos_m = s.find("m: ").unwrap();
        let pos_z = s.find("z: ").unwrap();
        assert!(pos_a < pos_m && pos_m < pos_z);
    }

    #[test]
    fn stringify_dict_multiple_entries() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("age".to_string(), Node::Integer(30));
        dict.insert("name".to_string(), Node::Str("Bob".into()));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("age: 30"));
        assert!(s.contains("name: \"Bob\""));
    }

    #[test]
    fn stringify_dict_list_value() {
        let mut dict = std::collections::HashMap::new();
        dict.insert(
            "nums".to_string(),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
        );
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("nums:"));
        assert!(s.contains("- 1"));
        assert!(s.contains("- 2"));
    }

    #[test]
    fn stringify_nested_dict() {
        let mut inner = std::collections::HashMap::new();
        inner.insert("x".to_string(), Node::Integer(9));
        let mut outer = std::collections::HashMap::new();
        outer.insert("inner".to_string(), Node::Dictionary(inner));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(outer), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("inner:"));
        assert!(s.contains("x: 9"));
    }

    // --- write_indent helper via observable output ---

    #[test]
    fn write_indent_level_zero_produces_no_spaces() {
        // level=0: loop runs for _ in 1..0 which never executes
        let mut dest = Buffer::new();
        write_indent(0, &mut dest);
        assert_eq!(dest.to_string(), "");
    }

    #[test]
    fn write_indent_level_one_produces_no_spaces() {
        // level=1: loop runs for _ in 1..1 which never executes
        let mut dest = Buffer::new();
        write_indent(1, &mut dest);
        assert_eq!(dest.to_string(), "");
    }

    #[test]
    fn write_indent_level_two_produces_two_spaces() {
        // level=2: loop runs for _ in 1..2 → one iteration → "  "
        let mut dest = Buffer::new();
        write_indent(2, &mut dest);
        assert_eq!(dest.to_string(), "  ");
    }

    #[test]
    fn write_indent_level_three_produces_four_spaces() {
        // level=3: loop runs 2 iterations → "    "
        let mut dest = Buffer::new();
        write_indent(3, &mut dest);
        assert_eq!(dest.to_string(), "    ");
    }

    // --- Idempotency ---

    #[test]
    fn stringify_twice_gives_same_result() {
        let node = Node::List(vec![Node::Integer(1), Node::Str("a".into())]);
        let mut dest1 = Buffer::new();
        let mut dest2 = Buffer::new();
        stringify(&node, &mut dest1).unwrap();
        stringify(&node, &mut dest2).unwrap();
        assert_eq!(dest1.to_string(), dest2.to_string());
    }

    // --- make_node convenience ---

    #[test]
    fn stringify_via_make_node_integer() {
        let mut dest = Buffer::new();
        stringify(&make_node(5i64), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "5");
    }

    #[test]
    fn stringify_via_make_node_string() {
        let mut dest = Buffer::new();
        stringify(&make_node("hi"), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"hi\"");
    }
}
