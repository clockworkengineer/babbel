#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::io::traits::{BencodeWrite, IDestination};
use crate::nodes::node::*;
use crate::stringify::common::escape_string;
use crate::stringify::visitor::{BencodeVisitable, BencodeVisitor};

/// XML format serializer implementing the `BencodeVisitor` pattern.
pub struct XmlSerializer<'a, W: BencodeWrite + ?Sized> {
    writer: &'a mut W,
    dict_item_active: Vec<bool>,
}

impl<'a, W: BencodeWrite + ?Sized> XmlSerializer<'a, W> {
    /// Creates a new XmlSerializer writing to the given writer.
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            dict_item_active: Vec::new(),
        }
    }
}

impl<'a, W: BencodeWrite + ?Sized> BencodeVisitor for XmlSerializer<'a, W> {
    type Error = String;

    fn visit_integer(&mut self, value: i64) -> Result<(), Self::Error> {
        self.writer.write_bytes(b"<integer>");
        self.writer.write_bytes(value.to_string().as_bytes());
        self.writer.write_bytes(b"</integer>");
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Result<(), Self::Error> {
        self.writer.write_bytes(b"<string>");
        escape_string(value, self.writer);
        self.writer.write_bytes(b"</string>");
        Ok(())
    }

    fn visit_list_start(&mut self) -> Result<(), Self::Error> {
        self.writer.write_bytes(b"<list>");
        Ok(())
    }

    fn visit_list_end(&mut self) -> Result<(), Self::Error> {
        self.writer.write_bytes(b"</list>");
        Ok(())
    }

    fn visit_dict_start(&mut self) -> Result<(), Self::Error> {
        self.writer.write_bytes(b"<dictionary>");
        self.dict_item_active.push(false);
        Ok(())
    }

    fn visit_dict_key(&mut self, key: &str) -> Result<(), Self::Error> {
        if let Some(active) = self.dict_item_active.last_mut() {
            if *active {
                self.writer.write_bytes(b"</value></item>");
            } else {
                *active = true;
            }
        }
        self.writer.write_bytes(b"<item><key>");
        self.writer.write_bytes(key.as_bytes());
        self.writer.write_bytes(b"</key><value>");
        Ok(())
    }

    fn visit_dict_end(&mut self) -> Result<(), Self::Error> {
        if let Some(active) = self.dict_item_active.pop() {
            if active {
                self.writer.write_bytes(b"</value></item>");
            }
        }
        self.writer.write_bytes(b"</dictionary>");
        Ok(())
    }

    fn visit_none(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Converts a bencode Node into XML format and writes it to the given destination.
/// Each node type is wrapped in appropriate XML tags based on its type.
///
/// # Arguments
/// * `node` - The bencode Node to convert
/// * `destination` - The destination to write the XML output to
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    let mut serializer = XmlSerializer::new(destination);
    node.accept(&mut serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::destinations::buffer::Buffer;

    #[test]
    fn test_string_node() {
        let mut destination = Buffer::new();
        stringify(&Node::Str("test".into()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "<string>test</string>");
    }

    #[test]
    fn test_integer_node() {
        let mut destination = Buffer::new();
        stringify(&Node::Integer(42), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "<integer>42</integer>");
    }

    #[test]
    fn test_list_node() {
        let mut destination = Buffer::new();
        stringify(
            &Node::List(vec![Node::Integer(1), Node::Str("test".into())]),
            &mut destination,
        )
        .unwrap();
        assert_eq!(
            destination.to_string(),
            "<list><integer>1</integer><string>test</string></list>"
        );
    }

    #[test]
    fn test_dictionary_node() {
        let mut destination = Buffer::new();
        let mut dict = std::collections::HashMap::new();
        dict.insert("key".into(), Node::Str("value".into()));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "<dictionary><item><key>key</key><value><string>value</string></value></item></dictionary>"
        );
    }

    #[test]
    fn test_none_node() {
        let mut destination = Buffer::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "");
    }

    // --- Integer edge cases ---

    #[test]
    fn test_integer_zero() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(0), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<integer>0</integer>");
    }

    #[test]
    fn test_integer_negative() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(-7), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<integer>-7</integer>");
    }

    #[test]
    fn test_integer_max_i64() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(i64::MAX), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("<integer>{}</integer>", i64::MAX));
    }

    #[test]
    fn test_integer_min_i64() {
        let mut dest = Buffer::new();
        stringify(&Node::Integer(i64::MIN), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("<integer>{}</integer>", i64::MIN));
    }

    // --- String edge cases ---

    #[test]
    fn test_empty_string() {
        let mut dest = Buffer::new();
        stringify(&Node::Str(String::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string></string>");
    }

    #[test]
    fn test_string_single_char() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("z".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>z</string>");
    }

    #[test]
    fn test_string_with_double_quote_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("say \"hi\"".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>say \\\"hi\\\"</string>");
    }

    #[test]
    fn test_string_with_backslash_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("a\\b".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>a\\\\b</string>");
    }

    #[test]
    fn test_string_with_newline_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("line1\nline2".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>line1\\u000aline2</string>");
    }

    #[test]
    fn test_string_with_tab_escaped() {
        let mut dest = Buffer::new();
        stringify(&Node::Str("col1\tcol2".into()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>col1\\u0009col2</string>");
    }

    // --- List edge cases ---

    #[test]
    fn test_empty_list() {
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<list></list>");
    }

    #[test]
    fn test_list_single_integer() {
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![Node::Integer(5)]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<list><integer>5</integer></list>");
    }

    #[test]
    fn test_list_of_strings() {
        let mut dest = Buffer::new();
        stringify(
            &Node::List(vec![Node::Str("foo".into()), Node::Str("bar".into())]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(
            dest.to_string(),
            "<list><string>foo</string><string>bar</string></list>"
        );
    }

    #[test]
    fn test_nested_list() {
        let mut dest = Buffer::new();
        let inner = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        stringify(&Node::List(vec![inner, Node::Integer(3)]), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<list><list><integer>1</integer><integer>2</integer></list><integer>3</integer></list>"
        );
    }

    #[test]
    fn test_list_with_none_produces_no_content() {
        // None nodes emit nothing, so a list of [None, None] is just <list></list>
        let mut dest = Buffer::new();
        stringify(&Node::List(vec![Node::None, Node::None]), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<list></list>");
    }

    #[test]
    fn test_list_mixed_integer_string() {
        let mut dest = Buffer::new();
        stringify(
            &Node::List(vec![Node::Integer(10), Node::Str("x".into())]),
            &mut dest,
        )
        .unwrap();
        assert_eq!(
            dest.to_string(),
            "<list><integer>10</integer><string>x</string></list>"
        );
    }

    // --- Dictionary edge cases ---

    #[test]
    fn test_empty_dictionary() {
        let mut dest = Buffer::new();
        stringify(
            &Node::Dictionary(std::collections::HashMap::new()),
            &mut dest,
        )
        .unwrap();
        assert_eq!(dest.to_string(), "<dictionary></dictionary>");
    }

    #[test]
    fn test_dictionary_integer_value() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("count".into(), Node::Integer(7));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<dictionary><item><key>count</key><value><integer>7</integer></value></item></dictionary>"
        );
    }

    #[test]
    fn test_dictionary_list_value() {
        let mut dict = std::collections::HashMap::new();
        dict.insert(
            "nums".into(),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
        );
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("<key>nums</key>"));
        assert!(s.contains("<list><integer>1</integer><integer>2</integer></list>"));
    }

    #[test]
    fn test_nested_dictionary() {
        let mut inner = std::collections::HashMap::new();
        inner.insert("x".into(), Node::Integer(9));
        let mut outer = std::collections::HashMap::new();
        outer.insert("inner".into(), Node::Dictionary(inner));
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(outer), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("<key>inner</key>"));
        assert!(s.contains("<dictionary>"));
        assert!(s.contains("<integer>9</integer>"));
    }

    #[test]
    fn test_dictionary_none_value_produces_empty_value_tag() {
        let mut dict = std::collections::HashMap::new();
        dict.insert("k".into(), Node::None);
        let mut dest = Buffer::new();
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "<dictionary><item><key>k</key><value></value></item></dictionary>"
        );
    }

    // --- make_node convenience ---

    #[test]
    fn test_make_node_integer_xml() {
        let mut dest = Buffer::new();
        stringify(&make_node(99i64), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<integer>99</integer>");
    }

    #[test]
    fn test_make_node_string_xml() {
        let mut dest = Buffer::new();
        stringify(&make_node("hello"), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "<string>hello</string>");
    }

    // --- Idempotency / consistency ---

    #[test]
    fn test_stringify_twice_gives_same_result() {
        let node = Node::List(vec![Node::Integer(1), Node::Str("a".into())]);
        let mut dest1 = Buffer::new();
        let mut dest2 = Buffer::new();
        stringify(&node, &mut dest1).unwrap();
        stringify(&node, &mut dest2).unwrap();
        assert_eq!(dest1.to_string(), dest2.to_string());
    }
}
