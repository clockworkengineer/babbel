#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::io::traits::{BencodeWrite, IDestination};
use crate::nodes::node::*;
use crate::stringify::common::escape_string;
use crate::stringify::visitor::{BencodeVisitable, BencodeVisitor};

struct JsonContext {
    first_item: bool,
}

/// JSON format serializer implementing the `BencodeVisitor` pattern.
pub struct JsonSerializer<'a, W: BencodeWrite + ?Sized> {
    writer: &'a mut W,
    stack: Vec<JsonContext>,
    after_key: bool,
}

impl<'a, W: BencodeWrite + ?Sized> JsonSerializer<'a, W> {
    /// Creates a new JsonSerializer writing to the given writer.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            stack: Vec::new(),
            after_key: false,
        }
    }

    fn prepare_value(&mut self) {
        if self.after_key {
            self.after_key = false;
            return;
        }
        if let Some(ctx) = self.stack.last_mut() {
            if !ctx.first_item {
                self.writer.write_byte(b',');
            } else {
                ctx.first_item = false;
            }
        }
    }
}

impl<'a, W: BencodeWrite + ?Sized> BencodeVisitor for JsonSerializer<'a, W> {
    type Error = String;

    fn visit_integer(&mut self, value: i64) -> Result<(), Self::Error> {
        self.prepare_value();
        self.writer.write_bytes(value.to_string().as_bytes());
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Result<(), Self::Error> {
        self.prepare_value();
        self.writer.write_byte(b'"');
        escape_string(value, self.writer);
        self.writer.write_byte(b'"');
        Ok(())
    }

    fn visit_list_start(&mut self) -> Result<(), Self::Error> {
        self.prepare_value();
        self.writer.write_byte(b'[');
        self.stack.push(JsonContext { first_item: true });
        Ok(())
    }

    fn visit_list_end(&mut self) -> Result<(), Self::Error> {
        self.stack.pop();
        self.writer.write_byte(b']');
        Ok(())
    }

    fn visit_dict_start(&mut self) -> Result<(), Self::Error> {
        self.prepare_value();
        self.writer.write_byte(b'{');
        self.stack.push(JsonContext { first_item: true });
        Ok(())
    }

    fn visit_dict_key(&mut self, key: &str) -> Result<(), Self::Error> {
        if let Some(ctx) = self.stack.last_mut() {
            if !ctx.first_item {
                self.writer.write_byte(b',');
            } else {
                ctx.first_item = false;
            }
        }
        self.writer.write_byte(b'"');
        self.writer.write_bytes(key.as_bytes());
        self.writer.write_bytes(b"\":");
        self.after_key = true;
        Ok(())
    }

    fn visit_dict_end(&mut self) -> Result<(), Self::Error> {
        self.stack.pop();
        self.writer.write_byte(b'}');
        Ok(())
    }

    fn visit_none(&mut self) -> Result<(), Self::Error> {
        self.prepare_value();
        self.writer.write_bytes(b"null");
        Ok(())
    }
}

/// Converts a Node structure into a JSON string representation and writes it to the given destination.
/// Handles different node types (Integer, String, List, Dictionary) according to JSON format rules.
///
/// # Arguments
/// * `node` - The Node structure to convert
/// * `destination` - The destination to write the JSON output to
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    let mut serializer = JsonSerializer::new(destination);
    node.accept(&mut serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[test]
    fn stringify_integer_works() {
        let mut destination = Buffer::new();
        stringify(&Node::Integer(42), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "42");
    }

    #[test]
    fn stringify_string_works() {
        let mut destination = Buffer::new();
        stringify(&Node::Str("hello".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"hello\"");
    }

    #[test]
    fn stringify_list_works() {
        let mut destination = Buffer::new();
        stringify(
            &Node::List(vec![
                Node::Integer(1),
                Node::Integer(2),
                Node::Str("three".to_string()),
            ]),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "[1,2,\"three\"]");
    }

    #[test]
    fn stringify_dictionary_works() {
        let mut destination = Buffer::new();
        let mut dict = std::collections::HashMap::new();
        dict.insert("key1".to_string(), Node::Integer(1));
        dict.insert("key2".to_string(), Node::Str("value".to_string()));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"key1\":1,\"key2\":\"value\"}");
    }

    #[test]
    fn stringify_unknown_node_works() {
        let mut destination = Buffer::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "null");
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
    fn stringify_string_with_double_quote() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("say \"hi\"".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn stringify_string_with_backslash() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\\b".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"a\\\\b\"");
    }

    #[test]
    fn stringify_string_with_newline() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("line1\nline2".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"line1\\u000aline2\"");
    }

    #[test]
    fn stringify_string_single_char() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("x".to_string()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "\"x\"");
    }

    // --- List edge cases ---

    #[test]
    fn stringify_empty_list() {
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[]");
    }

    #[test]
    fn stringify_list_single_integer() {
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![Node::Integer(7)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[7]");
    }

    #[test]
    fn stringify_list_of_strings() {
        let mut dest = Buffer::new();
        stringify(
            &Node::List(vec![
                Node::Str("foo".to_string()),
                Node::Str("bar".to_string()),
            ]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[\"foo\",\"bar\"]");
    }

    #[test]
    fn stringify_nested_list() {
        let mut dest = Buffer::new();
        let inner = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        stringify(&Node::List(vec![inner, Node::Integer(3)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[[1,2],3]");
    }

    #[test]
    fn stringify_list_with_null() {
        let mut dest = Buffer::new();
        stringify(
            &Node::List(vec![Node::Integer(1), Node::None, Node::Integer(3)]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "[1,null,3]");
    }

    #[test]
    fn stringify_list_commas_correct_count() {
        // N items should have N-1 commas
        let list = Node::List(vec![
            Node::Integer(1),
            Node::Integer(2),
            Node::Integer(3),
            Node::Integer(4),
        ]);
        let mut dest = Buffer::new();
        stringify(&list, &mut dest).unwrap();
        let s = dest.to_string();
        let comma_count = s.chars().filter(|&c| c == ',').count();
        assert_eq!(comma_count, 3);
    }

    // --- Dictionary edge cases ---

    #[test]
    fn stringify_empty_dict() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Dictionary(std::collections::HashMap::new()),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "{}");
    }

    #[test]
    fn stringify_dict_sorts_keys() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("z".to_string(), Node::Integer(1));
        dict.insert("a".to_string(), Node::Integer(2));
        dict.insert("m".to_string(), Node::Integer(3));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"a\":2,\"m\":3,\"z\":1}");
    }

    #[test]
    fn stringify_dict_string_value() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("key".to_string(), Node::Str("val".to_string()));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"key\":\"val\"}");
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
        assert_eq!(dest.to_string(), "{\"nums\":[1,2]}");
    }

    #[test]
    fn stringify_nested_dict() {
        let mut inner = std::collections::HashMap::new();
        inner.insert("x".to_string(), Node::Integer(9));
        let mut outer = std::collections::HashMap::new();
        outer.insert("inner".to_string(), Node::Dictionary(inner));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(outer), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"inner\":{\"x\":9}}");
    }

    #[test]
    fn stringify_dict_null_value() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("nothing".to_string(), Node::None);
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "{\"nothing\":null}");
    }

    #[test]
    fn stringify_dict_commas_correct_count() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("a".to_string(), Node::Integer(1));
        dict.insert("b".to_string(), Node::Integer(2));
        dict.insert("c".to_string(), Node::Integer(3));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let s = dest.to_string();
        let comma_count = s.chars().filter(|&c| c == ',').count();
        assert_eq!(comma_count, 2);
    }

    // --- make_node convenience helpers ---

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

    // --- Round-trip with bencode parser ---

    #[test]
    fn stringify_json_roundtrip_consistent() {
        // Encode a node to JSON twice and ensure results are identical
        let mut dict = std::collections::HashMap::new();
        dict.insert("age".to_string(), Node::Integer(30));
        dict.insert("name".to_string(), Node::Str("Alice".to_string()));
        let node = Node::Dictionary(dict);
        let mut dest1 = Buffer::new();
        let mut dest2 = Buffer::new();
        stringify(&node, &mut dest1).unwrap();
        stringify(&node, &mut dest2).unwrap();
        assert_eq!(dest1.to_string(), dest2.to_string());
    }
}
