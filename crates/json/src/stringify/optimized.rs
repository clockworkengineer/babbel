//! Optimized JSON stringifier with lazy string escaping
//!
//! This module provides optimized JSON serialization with several performance improvements:
//! - Lazy string escaping: only escape when necessary
//! - Batch writing: write large chunks of unescaped data at once
//! - Fast paths for common cases (simple strings, small integers)

use crate::io::traits::IDestination;
use crate::nodes::node::*;
use crate::stringify::escape::*;
use dtoa;
use itoa;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Write a string without escaping (fast path when `needs_escaping` returns false)
#[inline]
fn write_simple_string(s: &str, destination: &mut dyn IDestination) {
    destination.add_bytes(STR_QUOTE);
    destination.add_bytes(s);
    destination.add_bytes(STR_QUOTE);
}

/// Optimized stringify with lazy escaping
pub fn stringify_optimized(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::None => destination.add_bytes(JSON_NULL),
        Node::Boolean(value) => destination.add_bytes(if *value { JSON_TRUE } else { JSON_FALSE }),
        Node::Number(value) => {
            let mut ibuf = itoa::Buffer::new();
            let mut fbuf = dtoa::Buffer::new();
            match value {
                Numeric::Integer(n) => destination.add_bytes(ibuf.format(*n)),
                Numeric::UInteger(n) => destination.add_bytes(ibuf.format(*n)),
                Numeric::Float(f) => destination.add_bytes(fbuf.format(*f)),
                Numeric::Byte(b) => destination.add_bytes(ibuf.format(*b)),
                Numeric::Int32(i) => destination.add_bytes(ibuf.format(*i)),
                Numeric::UInt32(u) => destination.add_bytes(ibuf.format(*u)),
                Numeric::Int16(i) => destination.add_bytes(ibuf.format(*i)),
                Numeric::UInt16(u) => destination.add_bytes(ibuf.format(*u)),
                Numeric::Int8(i) => destination.add_bytes(ibuf.format(*i)),
            }
        }
        Node::Str(value) => {
            if needs_escaping(value) {
                write_escaped_string(value, destination);
            } else {
                write_simple_string(value, destination);
            }
        }
        Node::Array(items) => {
            destination.add_bytes(STR_ARRAY_START);
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    destination.add_bytes(STR_COMMA);
                }
                stringify_optimized(item, destination)?;
            }
            destination.add_bytes(STR_ARRAY_END);
        }
        Node::Object(entries) => {
            destination.add_bytes(STR_OBJECT_START);
            for (index, (key, value)) in entries.iter().enumerate() {
                if index > 0 {
                    destination.add_bytes(STR_COMMA);
                }
                write_escaped_string(key, destination);
                destination.add_bytes(STR_COLON);
                stringify_optimized(value, destination)?;
            }
            destination.add_bytes(STR_OBJECT_END);
        }
    }
    Ok(())
}

/// Performance statistics for stringify operations
#[cfg(feature = "alloc")]
pub struct StringifyStats {
    pub simple_strings: usize,
    pub escaped_strings: usize,
    pub total_bytes: usize,
}

#[cfg(feature = "alloc")]
impl StringifyStats {
    pub fn new() -> Self {
        Self {
            simple_strings: 0,
            escaped_strings: 0,
            total_bytes: 0,
        }
    }

    pub fn escape_percentage(&self) -> f64 {
        let total = self.simple_strings + self.escaped_strings;
        if total == 0 {
            0.0
        } else {
            (self.escaped_strings as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[test]
    fn test_needs_escaping() {
        assert!(!needs_escaping("hello"));
        assert!(!needs_escaping("hello world"));
        assert!(needs_escaping("hello\"world"));
        assert!(needs_escaping("hello\\world"));
        assert!(needs_escaping("hello\nworld"));
    }

    #[test]
    fn test_stringify_simple_string() {
        let mut dest = Buffer::new();
        let node = Node::Str("hello".to_string());
        stringify_optimized(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"hello\"");
    }

    #[test]
    fn test_stringify_escaped_string() {
        let mut dest = Buffer::new();
        let node = Node::Str("hello\nworld".to_string());
        stringify_optimized(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"hello\\nworld\"");
    }

    #[test]
    fn test_stringify_number() {
        let mut dest = Buffer::new();
        let node = Node::Number(Numeric::Int32(42));
        stringify_optimized(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "42");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_stringify_object() {
        let mut dest = Buffer::new();
        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), Node::Str("Alice".to_string()));
        let node = Node::Object(map);
        stringify_optimized(&node, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"name\":\"Alice\"}");
    }

    // needs_escaping edge cases
    #[test]
    fn test_needs_escaping_empty() {
        assert!(!needs_escaping(""));
    }

    #[test]
    fn test_needs_escaping_control_chars() {
        assert!(needs_escaping("\u{0000}"));
        assert!(needs_escaping("\u{001F}"));
        assert!(needs_escaping("\u{0008}")); // backspace
    }

    #[test]
    fn test_needs_escaping_tab_cr_lf() {
        assert!(needs_escaping("\t"));
        assert!(needs_escaping("\r"));
        assert!(needs_escaping("\n"));
    }

    #[test]
    fn test_needs_escaping_backslash_only() {
        assert!(needs_escaping("\\"));
    }

    #[test]
    fn test_needs_escaping_quote_only() {
        assert!(needs_escaping("\""));
    }

    #[test]
    fn test_needs_escaping_high_bytes_safe() {
        // Bytes >= 128 are not in the escape set
        assert!(!needs_escaping("café"));
        assert!(!needs_escaping("中文"));
    }

    #[test]
    fn test_needs_escaping_printable_ascii() {
        assert!(!needs_escaping(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()"
        ));
    }

    // null
    #[test]
    fn test_stringify_null() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::None, &mut dest).unwrap();
        assert_eq!(dest.to_string(), "null");
    }

    // Boolean
    #[test]
    fn test_stringify_boolean_true() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "true");
    }

    #[test]
    fn test_stringify_boolean_false() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Boolean(false), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "false");
    }

    // Numbers — all variants
    #[test]
    fn test_stringify_integer() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Integer(-99)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "-99");
    }

    #[test]
    fn test_stringify_integer_zero() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Integer(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "0");
    }

    #[test]
    fn test_stringify_uinteger() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::UInteger(u64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), u64::MAX.to_string());
    }

    #[test]
    fn test_stringify_float() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Float(3.14)), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.starts_with("3.14"), "got: {}", s);
    }

    #[test]
    fn test_stringify_byte() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Byte(255)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "255");
    }

    #[test]
    fn test_stringify_int32_negative() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Int32(i32::MIN)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), i32::MIN.to_string());
    }

    #[test]
    fn test_stringify_uint32() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::UInt32(u32::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), u32::MAX.to_string());
    }

    #[test]
    fn test_stringify_int16() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Int16(-1000)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "-1000");
    }

    #[test]
    fn test_stringify_uint16() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::UInt16(65535)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "65535");
    }

    #[test]
    fn test_stringify_int8() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Number(Numeric::Int8(-128)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "-128");
    }

    // String — fast/slow paths
    #[test]
    fn test_stringify_empty_string() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\"");
    }

    #[test]
    fn test_stringify_string_backslash() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("a\\b".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\\\b\"");
    }

    #[test]
    fn test_stringify_string_quote() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("say \"hi\"".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn test_stringify_string_tab() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("a\tb".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\tb\"");
    }

    #[test]
    fn test_stringify_string_cr() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("a\rb".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\rb\"");
    }

    #[test]
    fn test_stringify_string_control_char() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("\u{0001}".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\\u0001\"");
    }

    #[test]
    fn test_stringify_string_no_escape_passthrough() {
        let s = "Hello, World! 0123456789".to_string();
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str(s.clone()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("\"{}\"", s));
    }

    #[test]
    fn test_stringify_string_escape_in_middle() {
        // Escape character surrounded by normal chars — tests batch write logic
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("abc\ndef".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"abc\\ndef\"");
    }

    #[test]
    fn test_stringify_string_multiple_escapes() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Str("\"\\\n\r\t".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"\\\"\\\\\\n\\r\\t\"");
    }

    // Array edge cases
    #[test]
    fn test_stringify_empty_array() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Array(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[]");
    }

    #[test]
    fn test_stringify_array_single() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Array(vec![Node::Boolean(true)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[true]");
    }

    #[test]
    fn test_stringify_array_commas() {
        let mut dest = Buffer::new();
        stringify_optimized(
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

    #[test]
    fn test_stringify_array_of_nulls() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Array(vec![Node::None, Node::None]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[null,null]");
    }

    #[test]
    fn test_stringify_nested_array() {
        let mut dest = Buffer::new();
        stringify_optimized(
            &Node::Array(vec![Node::Array(vec![Node::Number(Numeric::Integer(1))])]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[[1]]");
    }

    // Object edge cases
    #[test]
    fn test_stringify_empty_object() {
        let mut dest = Buffer::new();
        stringify_optimized(&Node::Object(std::collections::HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{}");
    }

    #[test]
    fn test_stringify_object_empty_key() {
        let mut dest = Buffer::new();
        let mut map = std::collections::HashMap::new();
        map.insert("".to_string(), Node::Number(Numeric::Integer(1)));
        stringify_optimized(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"\":1}");
    }

    #[test]
    fn test_stringify_object_key_with_escape() {
        let mut dest = Buffer::new();
        let mut map = std::collections::HashMap::new();
        map.insert("ke\ny".to_string(), Node::Boolean(false));
        stringify_optimized(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"ke\\ny\":false}");
    }

    // Return value
    #[test]
    fn test_stringify_returns_ok_all_types() {
        let nodes: Vec<Node> = vec![
            Node::None,
            Node::Boolean(true),
            Node::Number(Numeric::Integer(1)),
            Node::Number(Numeric::Float(1.0)),
            Node::Str("".to_string()),
            Node::Array(vec![]),
            Node::Object(std::collections::HashMap::new()),
        ];
        for node in &nodes {
            let mut dest = Buffer::new();
            assert!(stringify_optimized(node, &mut dest).is_ok());
        }
    }

    // StringifyStats
    #[cfg(feature = "alloc")]
    #[test]
    fn test_stringify_stats_new_zeros() {
        let stats = StringifyStats::new();
        assert_eq!(stats.simple_strings, 0);
        assert_eq!(stats.escaped_strings, 0);
        assert_eq!(stats.total_bytes, 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stringify_stats_escape_percentage_empty() {
        let stats = StringifyStats::new();
        assert_eq!(stats.escape_percentage(), 0.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stringify_stats_all_simple() {
        let mut stats = StringifyStats::new();
        stats.simple_strings = 4;
        stats.escaped_strings = 0;
        assert_eq!(stats.escape_percentage(), 0.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stringify_stats_all_escaped() {
        let mut stats = StringifyStats::new();
        stats.simple_strings = 0;
        stats.escaped_strings = 3;
        assert_eq!(stats.escape_percentage(), 100.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stringify_stats_mixed_percentage() {
        let mut stats = StringifyStats::new();
        stats.simple_strings = 3;
        stats.escaped_strings = 1;
        // 1/4 = 25%
        assert!((stats.escape_percentage() - 25.0).abs() < 0.001);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stringify_stats_total_bytes_mutable() {
        let mut stats = StringifyStats::new();
        stats.total_bytes += 512;
        assert_eq!(stats.total_bytes, 512);
    }
}
