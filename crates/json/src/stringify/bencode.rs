//! Implementation of bencode serialization format.
//! Bencode is the encoding used by the peer-to-peer file sharing system BitTorrent
//! for storing and transmitting loosely structured data.

use crate::io::traits::IDestination;
use crate::nodes::node::*;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// Helper function to write a bencode string directly
#[inline]
fn write_bencode_string(s: &str, destination: &mut dyn IDestination) {
    // Use stack-allocated buffer for length
    let mut buf = itoa::Buffer::new();
    destination.add_bytes(buf.format(s.len()));
    destination.add_bytes(":");
    destination.add_bytes(s);
}

/// Serializes a `Node` into Bencode and writes it to the given destination.
///
/// # Arguments
///
/// * `node` - The Bencode node to serialize.
/// * `destination` - The destination to write the Bencode string to.

pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => destination.add_bytes(""),
        Node::Boolean(value) => destination.add_bytes(if *value { "i1e" } else { "i0e" }),
        Node::Number(value) => match value {
            Numeric::Integer(n) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes("i");
                destination.add_bytes(buf.format(*n));
                destination.add_bytes("e");
            }
            Numeric::UInteger(n) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes("i");
                destination.add_bytes(buf.format(*n));
                destination.add_bytes("e");
            }
            Numeric::Float(f) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes("i");
                destination.add_bytes(buf.format(f.round() as i64));
                destination.add_bytes("e");
            }
            Numeric::Byte(b) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes("i");
                destination.add_bytes(buf.format(*b));
                destination.add_bytes("e");
            }
            Numeric::Int32(i) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes("i");
                destination.add_bytes(buf.format(*i));
                destination.add_bytes("e");
            }
            Numeric::UInt32(u) => {
                let mut buf = itoa::Buffer::new();
                destination.add_bytes("i");
                destination.add_bytes(buf.format(*u));
                destination.add_bytes("e");
            }
            #[allow(unreachable_patterns)]
            _ => {
                destination.add_bytes("i");
                destination.add_bytes(&format!("{:?}", value));
                destination.add_bytes("e");
            }
        },
        Node::Str(value) => write_bencode_string(value, destination),
        Node::Array(items) => {
            destination.add_bytes("l");
            for item in items {
                stringify(item, destination)?;
            }
            destination.add_bytes("e");
        }
        Node::Object(entries) => {
            destination.add_bytes("d");
            let mut sorted_entries: Vec<_> = entries.iter().collect();
            sorted_entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
            for (key, value) in sorted_entries {
                write_bencode_string(key, destination);
                stringify(value, destination)?;
            }
            destination.add_bytes("e");
        }
    }
    Ok(())
}

#[cfg(test)]
/// Test module for verifying bencode serialization functionality.
/// Contains tests for all Node types and their bencode representations.
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;
    use std::collections::HashMap;

    #[test]
    fn test_stringify_none() {
        let mut dest = Buffer::new();
        stringify(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "");
    }

    #[test]
    fn test_stringify_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i1e");

        let mut dest = Buffer::new();
        stringify(&Node::Boolean(false), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i0e");
    }

    #[test]
    fn test_stringify_numbers() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(-42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i-42e");

        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInteger(42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i42e");

        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(42.7)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i43e");

        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Byte(255)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i255e");

        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Int32(-2147483648)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i-2147483648e");

        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInt32(4294967295)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i4294967295e");
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("test".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "4:test");
    }

    #[test]
    fn test_stringify_array() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Str("test".to_string()),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "li1e4:teste");

        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "le");
    }

    #[test]
    fn test_stringify_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("b".to_string(), Node::Str("value1".to_string()));
        map.insert("a".to_string(), Node::Str("value2".to_string()));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "d1:a6:value21:b6:value1e");

        let mut dest = Buffer::new();
        stringify(&Node::Object(HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "de");
    }

    #[test]
    fn test_stringify_empty_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0:");
    }

    #[test]
    fn test_stringify_nested_empty() {
        let mut dest = Buffer::new();
        let inner_map = HashMap::new();
        let mut outer_map = HashMap::new();
        outer_map.insert("empty_object".to_string(), Node::Object(inner_map));
        outer_map.insert("empty_array".to_string(), Node::Array(vec![]));
        outer_map.insert("empty_string".to_string(), Node::Str("".to_string()));
        stringify(&Node::Object(outer_map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "d11:empty_arrayle12:empty_objectde12:empty_string0:e"
        );
    }
    #[test]
    fn test_stringify_nested_object() {
        let mut inner_map = HashMap::new();
        inner_map.insert("a".to_string(), Node::Str("value1".to_string()));
        inner_map.insert("b".to_string(), Node::Str("value2".to_string()));
        let mut outer_map = HashMap::new();
        outer_map.insert("inner".to_string(), Node::Object(inner_map));
    }

    // String encoding: length prefix
    #[test]
    fn test_stringify_string_length_prefix() {
        // Length is the byte count, not char count
        let mut dest = Buffer::new();
        stringify(&Node::Str("hello".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "5:hello");
    }

    #[test]
    fn test_stringify_long_string() {
        let s = "x".repeat(100);
        let mut dest = Buffer::new();
        stringify(&Node::Str(s.clone()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("100:{}", s));
    }

    #[test]
    fn test_stringify_string_with_spaces() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("hello world".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "11:hello world");
    }

    // Integer edge cases
    #[test]
    fn test_stringify_integer_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i0e");
    }

    #[test]
    fn test_stringify_integer_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(i64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("i{}e", i64::MAX));
    }

    #[test]
    fn test_stringify_integer_min() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(i64::MIN)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("i{}e", i64::MIN));
    }

    #[test]
    fn test_stringify_uinteger_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInteger(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i0e");
    }

    #[test]
    fn test_stringify_float_rounds_half_up() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(2.5)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i3e"); // 2.5 rounds to 3
    }

    #[test]
    fn test_stringify_float_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(-1.9)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i-2e");
    }

    #[test]
    fn test_stringify_byte_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Byte(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i0e");
    }

    #[test]
    fn test_stringify_int32_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Int32(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i0e");
    }

    #[test]
    fn test_stringify_int32_positive() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Int32(1234)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "i1234e");
    }

    #[test]
    fn test_stringify_uint32_large() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInt32(u32::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("i{}e", u32::MAX));
    }

    // Array cases
    #[test]
    fn test_stringify_array_of_booleans() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::Boolean(true), Node::Boolean(false)]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "li1ei0ee");
    }

    #[test]
    fn test_stringify_array_of_strings() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Str("ab".to_string()),
                Node::Str("cde".to_string()),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "l2:ab3:cdee");
    }

    #[test]
    fn test_stringify_array_nested() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::Array(vec![Node::Number(Numeric::Integer(1))])]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "lli1eee");
    }

    #[test]
    fn test_stringify_array_mixed_types() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![
                Node::Number(Numeric::Integer(99)),
                Node::Str("hi".to_string()),
                Node::Boolean(false),
                Node::None,
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "li99e2:hii0ee");
    }

    // Object key sorting
    #[test]
    fn test_stringify_object_keys_sorted_lexicographically() {
        let mut map = HashMap::new();
        map.insert("z".to_string(), Node::Number(Numeric::Integer(1)));
        map.insert("a".to_string(), Node::Number(Numeric::Integer(2)));
        map.insert("m".to_string(), Node::Number(Numeric::Integer(3)));
        let mut dest = Buffer::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "d1:ai2e1:mi3e1:zi1ee");
    }

    #[test]
    fn test_stringify_object_single_entry() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::Str("val".to_string()));
        let mut dest = Buffer::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "d3:key3:vale");
    }

    #[test]
    fn test_stringify_object_integer_values() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), Node::Number(Numeric::Integer(42)));
        let mut dest = Buffer::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "d1:xi42ee");
    }

    #[test]
    fn test_stringify_object_with_array_value() {
        let mut map = HashMap::new();
        map.insert(
            "items".to_string(),
            Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
            ]),
        );
        let mut dest = Buffer::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "d5:itemsli1ei2eee");
    }

    // Deeply nested structure
    #[test]
    fn test_stringify_deeply_nested() {
        // {"a": {"b": {"c": 1}}}
        let mut inner = HashMap::new();
        inner.insert("c".to_string(), Node::Number(Numeric::Integer(1)));
        let mut mid = HashMap::new();
        mid.insert("b".to_string(), Node::Object(inner));
        let mut outer = HashMap::new();
        outer.insert("a".to_string(), Node::Object(mid));
        let mut dest = Buffer::new();
        stringify(&Node::Object(outer), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "d1:ad1:bd1:ci1eeee");
    }

    // None inside containers
    #[test]
    fn test_stringify_none_in_array() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::None, Node::None]), &mut dest).unwrap();
        // Node::None writes nothing, so two Nones inside an array produce just "le"
        assert_eq!(dest.to_string(), "le");
    }

    #[test]
    fn test_stringify_result_is_ok_for_all_types() {
        let nodes = vec![
            Node::None,
            Node::Boolean(true),
            Node::Number(Numeric::Integer(1)),
            Node::Str("s".to_string()),
            Node::Array(vec![]),
            Node::Object(HashMap::new()),
        ];
        for node in &nodes {
            let mut dest = Buffer::new();
            assert!(stringify(node, &mut dest).is_ok());
        }
    }
}
