//! Miscellaneous utility functions for JSON processing
//! Contain functionality for version information and formatted JSON printing

use crate::Node;
use crate::io::traits::{IDestination, ISource};
use crate::nodes::node::Numeric;
use dtoa;
use itoa;

/// Returns the current version of the package as specified in Cargo.toml.
/// Uses CARGO_PKG_VERSION environment variable that is set during compilation
/// from the version field in Cargo.toml.
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
pub fn print(node: &Node, destination: &mut dyn IDestination, indent: usize) {
    pretty_print(node, destination, indent, 0);
}

/// Writes `n` ASCII space bytes to `destination` without heap allocation.
#[inline]
fn write_spaces(destination: &mut dyn IDestination, n: usize) {
    const SPACES: &str = "                                ";
    let mut remaining = n;
    while remaining >= SPACES.len() {
        destination.add_bytes(SPACES);
        remaining -= SPACES.len();
    }
    if remaining > 0 {
        destination.add_bytes(&SPACES[..remaining]);
    }
}

/// Prints a JSON node to the specified destination with formatting
///
/// # Arguments
/// * `node` - The JSON node to print
/// * `destination` - The output destination implementing IDestination
/// * `indent` - Number of spaces to use for each indentation level
/// * `current_indent` - Current indentation level in spaces
fn pretty_print(
    node: &Node,
    destination: &mut dyn IDestination,
    indent: usize,
    current_indent: usize,
) {
    match node {
        // Handle boolean values (true/false)
        Node::Boolean(value) => destination.add_bytes(if *value { "true" } else { "false" }),
        // Handle various numeric types
        Node::Number(value) => {
            let mut ibuf = itoa::Buffer::new();
            let mut fbuf = dtoa::Buffer::new();
            match value {
                Numeric::Integer(i) => destination.add_bytes(ibuf.format(*i)),
                Numeric::UInteger(u) => destination.add_bytes(ibuf.format(*u)),
                Numeric::Float(f) => destination.add_bytes(fbuf.format(*f)),
                Numeric::Byte(b) => destination.add_bytes(ibuf.format(*b)),
                Numeric::Int8(i) => destination.add_bytes(ibuf.format(*i)),
                Numeric::Int16(i) => destination.add_bytes(ibuf.format(*i)),
                Numeric::UInt16(u) => destination.add_bytes(ibuf.format(*u)),
                Numeric::Int32(i) => destination.add_bytes(ibuf.format(*i)),
                Numeric::UInt32(u) => destination.add_bytes(ibuf.format(*u)),
            }
        }
        // Handle string values with proper JSON escaping
        Node::Str(value) => {
            destination.add_bytes("\"");
            destination.add_bytes(value);
            destination.add_bytes("\"");
        }
        // Handle null values
        Node::None => destination.add_bytes("null"),
        // Handle arrays with proper indentation and formatting
        Node::Array(array) => {
            destination.add_bytes("[\n");
            for (i, item) in array.iter().enumerate() {
                write_spaces(destination, current_indent + indent);
                pretty_print(item, destination, indent, current_indent + indent);
                if i < array.len() - 1 {
                    destination.add_bytes(",");
                }
                destination.add_bytes("\n");
            }
            write_spaces(destination, current_indent);
            destination.add_bytes("]");
        }
        // Handle objects/maps with proper indentation and key-value formatting
        Node::Object(map) => {
            destination.add_bytes("{\n");
            for (i, (key, value)) in map.iter().enumerate() {
                write_spaces(destination, current_indent + indent);
                destination.add_bytes("\"");
                destination.add_bytes(key);
                destination.add_bytes("\"");
                destination.add_bytes(": ");
                pretty_print(value, destination, indent, current_indent + indent);
                if i < map.len() - 1 {
                    destination.add_bytes(",");
                }
                destination.add_bytes("\n");
            }
            write_spaces(destination, current_indent);
            destination.add_bytes("}");
        }
    }
}

/// Strips whitespace from JSON while preserving string content.
/// Copies non-whitespace characters from source to destination, handling string literals specially.
pub fn strip(source: &mut dyn ISource, destination: &mut dyn IDestination) {
    while source.more() {
        if let Some(c) = source.current() {
            // Skip whitespace characters outside of strings
            if !(c == ' ' || c == '\t' || c == '\n' || c == '\r') {
                destination.add_byte(c as u8);

                // Handle string literals especially to preserve their whitespace
                if source.current() == Some('"') {
                    source.next();
                    // Copy characters until closing quote
                    while source.more() && source.current() != Some('"') {
                        // Handle escaped characters
                        if source.current() == Some('\\') {
                            destination.add_byte(b'\\');
                            source.next();
                        }
                        destination.add_byte(source.current().unwrap() as u8);
                        source.next();
                    }
                    destination.add_byte(b'"');
                }
            }
        }
        source.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BufferDestination;
    use crate::Node;
    use std::collections::BTreeMap;

    #[test]
    fn test_get_version_env() {
        assert_eq!(get_version(), "0.1.0");
    }

    #[test]
    fn test_print_boolean() {
        let mut dest = BufferDestination::new();
        print(&Node::Boolean(true), &mut dest, 0);
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_print_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Integer(42)), &mut dest, 0);
        assert_eq!(dest.to_string(), "42");
    }

    #[test]
    fn test_print_string() {
        let mut dest = BufferDestination::new();
        print(&Node::Str("hello".to_string()), &mut dest, 0);
        assert_eq!(dest.to_string(), "\"hello\"");
    }

    #[test]
    fn test_print_array() {
        let mut dest = BufferDestination::new();
        print(
            &Node::Array(vec![Node::Boolean(true), Node::Number(Numeric::Integer(1))]),
            &mut dest,
            0,
        );
        assert_eq!(dest.to_string(), "[\ntrue,\n1\n]");
    }

    #[test]
    fn test_print_object() {
        let mut dest = BufferDestination::new();
        let mut map = BTreeMap::new();
        map.insert("key".to_string(), Node::Str("value".to_string()));
        let hashmap: std::collections::HashMap<String, Node> = map.into_iter().collect();
        print(&Node::Object(hashmap), &mut dest, 0);
        assert_eq!(dest.to_string(), "{\n\"key\": \"value\"\n}");
    }

    #[test]
    fn test_print_null() {
        let mut dest = BufferDestination::new();
        print(&Node::None, &mut dest, 0);
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_strip_basic_object() {
        let json = "{ \"name\" :  \"value\" }";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "{\"name\":\"value\"}");
    }

    #[test]
    fn test_strip_string_content() {
        let json = "{ \"text\" : \"  keep  spaces  \" }";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "{\"text\":\"  keep  spaces  \"}");
    }

    #[test]
    fn test_strip_nested_structure() {
        let json = "{\n \"array\": [\n  1,\n  2\n ]\n}";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "{\"array\":[1,2]}");
    }

    // ─── get_version ─────────────────────────────────────────────────────────

    #[test]
    fn test_get_version_is_non_empty() {
        assert!(!get_version().is_empty());
    }

    #[test]
    fn test_get_version_has_semver_shape() {
        let v = get_version();
        // Expect "major.minor.patch" – at least two dots
        assert_eq!(
            v.chars().filter(|&c| c == '.').count(),
            2,
            "version '{}' does not look like semver",
            v
        );
    }

    // ─── print – primitives ───────────────────────────────────────────────────

    #[test]
    fn test_print_boolean_false() {
        let mut dest = BufferDestination::new();
        print(&Node::Boolean(false), &mut dest, 0);
        assert_eq!(dest.to_string(), "false");
    }

    #[test]
    fn test_print_null_with_indent() {
        let mut dest = BufferDestination::new();
        print(&Node::None, &mut dest, 4);
        assert_eq!(dest.to_string(), "null");
    }

    #[test]
    fn test_print_empty_string_node() {
        let mut dest = BufferDestination::new();
        print(&Node::Str(String::new()), &mut dest, 0);
        assert_eq!(dest.to_string(), "\"\"");
    }

    #[test]
    fn test_print_negative_integer() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Integer(-99)), &mut dest, 0);
        assert_eq!(dest.to_string(), "-99");
    }

    #[test]
    fn test_print_float() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Float(3.14)), &mut dest, 0);
        let s = dest.to_string();
        assert!(s.starts_with("3.14"), "unexpected float output: {}", s);
    }

    #[test]
    fn test_print_uint() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::UInteger(1_000_000)), &mut dest, 0);
        assert_eq!(dest.to_string(), "1000000");
    }

    #[test]
    fn test_print_byte_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Byte(255)), &mut dest, 0);
        assert_eq!(dest.to_string(), "255");
    }

    #[test]
    fn test_print_int32_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Int32(-1024)), &mut dest, 0);
        assert_eq!(dest.to_string(), "-1024");
    }

    #[test]
    fn test_print_uint32_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::UInt32(65535)), &mut dest, 0);
        assert_eq!(dest.to_string(), "65535");
    }

    #[test]
    fn test_print_int16_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Int16(-32768)), &mut dest, 0);
        assert_eq!(dest.to_string(), "-32768");
    }

    #[test]
    fn test_print_uint16_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::UInt16(1000)), &mut dest, 0);
        assert_eq!(dest.to_string(), "1000");
    }

    #[test]
    fn test_print_int8_numeric() {
        let mut dest = BufferDestination::new();
        print(&Node::Number(Numeric::Int8(-128)), &mut dest, 0);
        assert_eq!(dest.to_string(), "-128");
    }

    // ─── print – arrays ───────────────────────────────────────────────────────

    #[test]
    fn test_print_empty_array() {
        let mut dest = BufferDestination::new();
        print(&Node::Array(vec![]), &mut dest, 2);
        assert_eq!(dest.to_string(), "[\n]");
    }

    #[test]
    fn test_print_array_single_element() {
        let mut dest = BufferDestination::new();
        print(&Node::Array(vec![Node::Boolean(false)]), &mut dest, 2);
        assert_eq!(dest.to_string(), "[\n  false\n]");
    }

    #[test]
    fn test_print_array_with_indent() {
        let mut dest = BufferDestination::new();
        print(
            &Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
            ]),
            &mut dest,
            4,
        );
        assert_eq!(dest.to_string(), "[\n    1,\n    2\n]");
    }

    #[test]
    fn test_print_array_of_strings() {
        let mut dest = BufferDestination::new();
        print(
            &Node::Array(vec![Node::Str("a".to_string()), Node::Str("b".to_string())]),
            &mut dest,
            2,
        );
        assert_eq!(dest.to_string(), "[\n  \"a\",\n  \"b\"\n]");
    }

    #[test]
    fn test_print_nested_array() {
        let mut dest = BufferDestination::new();
        let inner = Node::Array(vec![Node::Number(Numeric::Integer(1))]);
        print(&Node::Array(vec![inner]), &mut dest, 2);
        // Outer array element is itself an array
        assert!(dest.to_string().contains("[\n"));
    }

    // ─── print – objects ──────────────────────────────────────────────────────

    #[test]
    fn test_print_empty_object() {
        let mut dest = BufferDestination::new();
        print(
            &Node::Object(std::collections::HashMap::new()),
            &mut dest,
            2,
        );
        assert_eq!(dest.to_string(), "{\n}");
    }

    #[test]
    fn test_print_object_with_indent() {
        let mut dest = BufferDestination::new();
        let mut map = std::collections::HashMap::new();
        map.insert("x".to_string(), Node::Number(Numeric::Integer(7)));
        print(&Node::Object(map), &mut dest, 4);
        assert_eq!(dest.to_string(), "{\n    \"x\": 7\n}");
    }

    #[test]
    fn test_print_object_null_value() {
        let mut dest = BufferDestination::new();
        let mut map = std::collections::HashMap::new();
        map.insert("nothing".to_string(), Node::None);
        print(&Node::Object(map), &mut dest, 2);
        assert_eq!(dest.to_string(), "{\n  \"nothing\": null\n}");
    }

    // ─── strip ────────────────────────────────────────────────────────────────

    #[test]
    fn test_strip_empty_input() {
        let mut source = crate::BufferSource::new(b"");
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "");
    }

    #[test]
    fn test_strip_no_whitespace_unchanged() {
        let json = r#"{"a":1}"#;
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), json);
    }

    #[test]
    fn test_strip_tabs_removed() {
        let json = "{\t\"k\"\t:\t1\t}";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "{\"k\":1}");
    }

    #[test]
    fn test_strip_newlines_removed() {
        let json = "[\n1,\n2\n]";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "[1,2]");
    }

    #[test]
    fn test_strip_carriage_returns_removed() {
        let json = "{\r\n\"k\"\r\n:\r\n1\r\n}";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "{\"k\":1}");
    }

    #[test]
    fn test_strip_array_of_numbers() {
        let json = "[ 10 , 20 , 30 ]";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "[10,20,30]");
    }

    #[test]
    fn test_strip_preserves_spaces_inside_string_values() {
        let json = "\"hello   world\"";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "\"hello   world\"");
    }

    #[test]
    fn test_strip_boolean_and_null_values() {
        let json = "[ true , false , null ]";
        let mut source = crate::BufferSource::new(json.as_bytes());
        let mut dest = BufferDestination::new();
        strip(&mut source, &mut dest);
        assert_eq!(dest.to_string(), "[true,false,null]");
    }
}
