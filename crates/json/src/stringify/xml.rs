//! XML serialization module for JSON nodes
//! Provides functionality to convert JSON nodes into XML format

use crate::io::traits::IDestination;
use crate::nodes::node::*;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Escapes special XML characters in a string and writes them to the destination
///
/// # Arguments
/// * `s` - The string to escape
/// * `destination` - The output destination implementing IDestination
fn escape_xml_string(s: &str, destination: &mut dyn IDestination) {
    for c in s.chars() {
        match c {
            '<' => destination.add_bytes("&lt;"),
            '>' => destination.add_bytes("&gt;"),
            '&' => destination.add_bytes("&amp;"),
            '"' => destination.add_bytes("&quot;"),
            '\'' => destination.add_bytes("&apos;"),
            c if c.is_ascii() => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                destination.add_bytes(s);
            }
            c => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                destination.add_bytes(s);
            }
        }
    }
}

/// Converts a JSON node to XML format and writes it to the destination
///
/// # Arguments
/// * `node` - The JSON node to convert
/// * `destination` - The output destination implementing IDestination
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        // Handle null values with a self-closing tag
        Node::None => destination.add_bytes("<null/>"),
        // Convert boolean values to XML with explicit true/false content
        Node::Boolean(value) => {
            destination.add_bytes("<boolean>");
            destination.add_bytes(if *value { "true" } else { "false" });
            destination.add_bytes("</boolean>");
        }
        // Convert numeric values to XML with type-specific handling
        Node::Number(value) => {
            destination.add_bytes("<number>");
            match value {
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
            }
            destination.add_bytes("</number>");
        }
        // Convert string values to XML with proper escaping
        Node::Str(value) => {
            destination.add_bytes("<string>");
            escape_xml_string(value, destination);
            destination.add_bytes("</string>");
        }
        // Convert arrays to XML with item wrapping for each element
        Node::Array(items) => {
            destination.add_bytes("<array>");
            for item in items {
                destination.add_bytes("<item>");
                stringify(item, destination)?;
                destination.add_bytes("</item>");
            }
            destination.add_bytes("</array>");
        }
        // Convert objects to XML with entry elements containing key-value pairs
        Node::Object(entries) => {
            destination.add_bytes("<object>");
            for (key, value) in entries {
                destination.add_bytes("<entry>");
                destination.add_bytes("<key>");
                escape_xml_string(key, destination);
                destination.add_bytes("</key>");
                destination.add_bytes("<value>");
                stringify(value, destination)?;
                destination.add_bytes("</value>");
                destination.add_bytes("</entry>");
            }
            destination.add_bytes("</object>");
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
        assert_eq!(dest.to_string(), "<null/>");
    }

    #[test]
    fn test_stringify_boolean() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(true), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<boolean>true</boolean>");
    }

    #[test]
    fn test_stringify_boolean_false() {
        let mut dest = Buffer::new();
        stringify(&Node::Boolean(false), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<boolean>false</boolean>");
    }

    #[test]
    fn test_stringify_number() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(42.5)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>42.5</number>");
    }

    #[test]
    fn test_stringify_all_numbers() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(-42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>-42</number>");

        dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInteger(42)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>42</number>");

        dest = Buffer::new();
        stringify(&Node::Number(Numeric::Byte(255)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>255</number>");

        dest = Buffer::new();
        stringify(&Node::Number(Numeric::Int32(-2147483648)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>-2147483648</number>");

        dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInt32(4294967295)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>4294967295</number>");
    }

    #[test]
    fn test_stringify_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("Hello & World".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>Hello &amp; World</string>");
    }

    #[test]
    fn test_stringify_special_chars() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("<>&\"'".to_string()), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<string>&lt;&gt;&amp;&quot;&apos;</string>"
        );
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
        assert_eq!(
            dest.to_string(),
            "<array><item><number>1</number></item><item><string>test</string></item></array>"
        );
    }

    #[test]
    fn test_stringify_object() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::Str("value".to_string()));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key>key</key><value><string>value</string></value></entry></object>"
        );
    }

    #[test]
    fn test_stringify_complex_nested() {
        let mut dest = Buffer::new();
        let mut inner_map = HashMap::new();
        inner_map.insert(
            "inner".to_string(),
            Node::Array(vec![
                Node::Boolean(true),
                Node::None,
                Node::Number(Numeric::Float(42.5)),
            ]),
        );
        let mut outer_map = HashMap::new();
        outer_map.insert("outer<".to_string(), Node::Object(inner_map));
        stringify(&Node::Object(outer_map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key>outer&lt;</key><value><object><entry><key>inner</key><value><array><item><boolean>true</boolean></item><item><null/></item><item><number>42.5</number></item></array></value></entry></object></value></entry></object>"
        );
    }

    #[test]
    fn test_stringify_empty_array() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<array></array>");
    }

    #[test]
    fn test_stringify_empty_object() {
        let mut dest = Buffer::new();
        stringify(&Node::Object(HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<object></object>");
    }

    #[test]
    fn test_stringify_unicode_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("Hello 🦀 World".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>Hello 🦀 World</string>");
    }

    #[test]
    fn test_stringify_large_numbers() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(f64::MAX)), &mut dest).unwrap();
        // dtoa outputs scientific notation for large floats
        assert_eq!(dest.to_string(), "<number>1.7976931348623157e308</number>");
    }

    // #[test]
    // fn test_stringify_nested_empty_structures() {
    //     let mut dest = Buffer::new();
    //     let mut map = HashMap::new();
    //     map.insert("empty_array".to_string(), Node::Array(vec![]));
    //     map.insert("empty_object".to_string(), Node::Object(HashMap::new()));
    //     stringify(&Node::Object(map), &mut dest).unwrap();
    //     assert_eq!(dest.to_string(), "<object><entry><key>empty_array</key><value><array></array></value></entry><entry><key>empty_object</key><value><object></object></value></entry></object>");
    // }

    // escape_xml_string individual char cases
    #[test]
    fn test_escape_lt() {
        let mut dest = Buffer::new();
        escape_xml_string("<", &mut dest);
        assert_eq!(dest.to_string(), "&lt;");
    }

    #[test]
    fn test_escape_gt() {
        let mut dest = Buffer::new();
        escape_xml_string(">", &mut dest);
        assert_eq!(dest.to_string(), "&gt;");
    }

    #[test]
    fn test_escape_amp() {
        let mut dest = Buffer::new();
        escape_xml_string("&", &mut dest);
        assert_eq!(dest.to_string(), "&amp;");
    }

    #[test]
    fn test_escape_quot() {
        let mut dest = Buffer::new();
        escape_xml_string("\"", &mut dest);
        assert_eq!(dest.to_string(), "&quot;");
    }

    #[test]
    fn test_escape_apos() {
        let mut dest = Buffer::new();
        escape_xml_string("'", &mut dest);
        assert_eq!(dest.to_string(), "&apos;");
    }

    #[test]
    fn test_escape_plain_text_unchanged() {
        let mut dest = Buffer::new();
        escape_xml_string("hello world", &mut dest);
        assert_eq!(dest.to_string(), "hello world");
    }

    #[test]
    fn test_escape_empty_string() {
        let mut dest = Buffer::new();
        escape_xml_string("", &mut dest);
        assert_eq!(dest.to_string(), "");
    }

    #[test]
    fn test_escape_mixed_special_and_plain() {
        let mut dest = Buffer::new();
        escape_xml_string("a<b&c>d", &mut dest);
        assert_eq!(dest.to_string(), "a&lt;b&amp;c&gt;d");
    }

    // stringify string edge cases
    #[test]
    fn test_stringify_empty_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string></string>");
    }

    #[test]
    fn test_stringify_string_with_only_special_chars() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("<<>>".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>&lt;&lt;&gt;&gt;</string>");
    }

    #[test]
    fn test_stringify_string_apostrophe_in_value() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("it's".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>it&apos;s</string>");
    }

    // Numeric variants individually
    #[test]
    fn test_stringify_integer_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>0</number>");
    }

    #[test]
    fn test_stringify_integer_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Integer(i64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("<number>{}</number>", i64::MAX));
    }

    #[test]
    fn test_stringify_uinteger_max() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::UInteger(u64::MAX)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("<number>{}</number>", u64::MAX));
    }

    #[test]
    fn test_stringify_byte_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Byte(0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>0</number>");
    }

    #[test]
    fn test_stringify_float_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(0.0)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>0.0</number>");
    }

    #[test]
    fn test_stringify_float_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Number(Numeric::Float(-1.5)), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<number>-1.5</number>");
    }

    // Array structure
    #[test]
    fn test_stringify_array_single_null() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::None]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<array><item><null/></item></array>");
    }

    #[test]
    fn test_stringify_array_of_booleans() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::Boolean(true), Node::Boolean(false)]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(
            dest.to_string(),
            "<array><item><boolean>true</boolean></item><item><boolean>false</boolean></item></array>"
        );
    }

    #[test]
    fn test_stringify_array_of_nulls() {
        let mut dest = Buffer::new();
        stringify(&Node::Array(vec![Node::None, Node::None]), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<array><item><null/></item><item><null/></item></array>"
        );
    }

    #[test]
    fn test_stringify_nested_array() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Array(vec![Node::Array(vec![Node::Number(Numeric::Integer(1))])]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(
            dest.to_string(),
            "<array><item><array><item><number>1</number></item></array></item></array>"
        );
    }

    // Object key escaping
    #[test]
    fn test_stringify_object_key_with_amp() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("a&b".to_string(), Node::Boolean(true));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key>a&amp;b</key><value><boolean>true</boolean></value></entry></object>"
        );
    }

    #[test]
    fn test_stringify_object_key_with_lt() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("<key>".to_string(), Node::None);
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key>&lt;key&gt;</key><value><null/></value></entry></object>"
        );
    }

    #[test]
    fn test_stringify_object_empty_key() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("".to_string(), Node::Number(Numeric::Integer(99)));
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key></key><value><number>99</number></value></entry></object>"
        );
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

    // Deeply nested object
    #[test]
    fn test_stringify_object_with_null_value() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert("nothing".to_string(), Node::None);
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key>nothing</key><value><null/></value></entry></object>"
        );
    }

    #[test]
    fn test_stringify_object_with_array_value() {
        let mut dest = Buffer::new();
        let mut map = HashMap::new();
        map.insert(
            "items".to_string(),
            Node::Array(vec![Node::Number(Numeric::Integer(1))]),
        );
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<object><entry><key>items</key><value><array><item><number>1</number></item></array></value></entry></object>"
        );
    }

    // Multibyte / non-ASCII passthrough (not escaped)
    #[test]
    fn test_stringify_string_chinese_chars() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("中文".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>中文</string>");
    }

    #[test]
    fn test_escape_non_ascii_passthrough() {
        let mut dest = Buffer::new();
        escape_xml_string("café", &mut dest);
        assert_eq!(dest.to_string(), "café");
    }
}
