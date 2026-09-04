//! Integration tests for the Node API and convenience functions

#[cfg(test)]
mod tests {
    use crate::{Node, make_node, parse_bytes, parse_str, stringify_to_bytes, stringify_to_string};
    use std::collections::HashMap;

    #[test]
    fn test_parse_bytes_convenience() {
        let data = b"i42e";
        let result = parse_bytes(data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Node::Integer(42));
    }

    #[test]
    fn test_parse_str_convenience() {
        let data = "4:test";
        let result = parse_str(data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Node::Str("test".to_string()));
    }

    #[test]
    fn test_stringify_to_string_convenience() {
        let node = Node::Integer(42);
        let result = stringify_to_string(&node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "i42e");
    }

    #[test]
    fn test_stringify_to_bytes_convenience() {
        let node = Node::Integer(42);
        let result = stringify_to_bytes(&node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"i42e");
    }

    #[test]
    fn test_node_type_checking() {
        let int_node = Node::Integer(42);
        assert!(int_node.is_integer());
        assert!(!int_node.is_string());
        assert!(!int_node.is_list());
        assert!(!int_node.is_dictionary());
        assert!(!int_node.is_none());

        let str_node = Node::Str("test".to_string());
        assert!(!str_node.is_integer());
        assert!(str_node.is_string());

        let list_node = Node::List(vec![]);
        assert!(list_node.is_list());

        let dict_node = Node::Dictionary(HashMap::new());
        assert!(dict_node.is_dictionary());

        let none_node = Node::None;
        assert!(none_node.is_none());
    }

    #[test]
    fn test_node_as_methods() {
        let int_node = Node::Integer(42);
        assert_eq!(int_node.as_integer(), Some(&42));
        assert_eq!(int_node.as_string(), None);

        let str_node = Node::Str("test".to_string());
        assert_eq!(str_node.as_string(), Some("test"));
        assert_eq!(str_node.as_integer(), None);

        let list_node = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        assert_eq!(list_node.as_list().unwrap().len(), 2);

        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Node::Integer(100));
        let dict_node = Node::Dictionary(dict);
        assert_eq!(dict_node.as_dictionary().unwrap().len(), 1);
    }

    #[test]
    fn test_node_get_methods() {
        let mut dict = HashMap::new();
        dict.insert("a".to_string(), Node::Integer(1));
        dict.insert("b".to_string(), Node::Integer(2));
        let node = Node::Dictionary(dict);

        assert_eq!(node.get("a"), Some(&Node::Integer(1)));
        assert_eq!(node.get("b"), Some(&Node::Integer(2)));
        assert_eq!(node.get("c"), None);
    }

    #[test]
    fn test_node_get_mut() {
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Node::Integer(42));
        let mut node = Node::Dictionary(dict);

        if let Some(value) = node.get_mut("key") {
            *value = Node::Integer(100);
        }

        assert_eq!(node.get("key"), Some(&Node::Integer(100)));
    }

    #[test]
    fn test_node_len_and_is_empty() {
        let empty_list = Node::List(vec![]);
        assert_eq!(empty_list.len(), 0);
        assert!(empty_list.is_empty());

        let list = Node::List(vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)]);
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());

        let empty_dict = Node::Dictionary(HashMap::new());
        assert_eq!(empty_dict.len(), 0);
        assert!(empty_dict.is_empty());

        let mut dict = HashMap::new();
        dict.insert("a".to_string(), Node::Integer(1));
        dict.insert("b".to_string(), Node::Integer(2));
        let dict_node = Node::Dictionary(dict);
        assert_eq!(dict_node.len(), 2);
        assert!(!dict_node.is_empty());

        let str_node = Node::Str("hello".to_string());
        assert_eq!(str_node.len(), 5);
        assert!(!str_node.is_empty());

        let empty_str = Node::Str("".to_string());
        assert_eq!(empty_str.len(), 0);
        assert!(empty_str.is_empty());
    }

    #[test]
    fn test_node_type_name() {
        assert_eq!(Node::Integer(42).type_name(), "integer");
        assert_eq!(Node::Str("test".to_string()).type_name(), "string");
        assert_eq!(Node::List(vec![]).type_name(), "list");
        assert_eq!(Node::Dictionary(HashMap::new()).type_name(), "dictionary");
        assert_eq!(Node::None.type_name(), "none");
    }

    #[test]
    fn test_node_display() {
        let int_node = Node::Integer(42);
        assert_eq!(format!("{}", int_node), "42");

        let str_node = Node::Str("hello".to_string());
        assert_eq!(format!("{}", str_node), "\"hello\"");

        let list_node = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        assert_eq!(format!("{}", list_node), "[1, 2]");

        let mut dict = HashMap::new();
        dict.insert("x".to_string(), Node::Integer(10));
        let dict_node = Node::Dictionary(dict);
        assert_eq!(format!("{}", dict_node), "{\"x\": 10}");

        assert_eq!(format!("{}", Node::None), "null");
    }

    #[test]
    fn test_as_list_mut() {
        let mut node = Node::List(vec![Node::Integer(1)]);

        if let Some(list) = node.as_list_mut() {
            list.push(Node::Integer(2));
            list.push(Node::Integer(3));
        }

        assert_eq!(node.len(), 3);
        assert_eq!(node.as_list().unwrap()[2], Node::Integer(3));
    }

    #[test]
    fn test_as_dictionary_mut() {
        let mut node = Node::Dictionary(HashMap::new());

        if let Some(dict) = node.as_dictionary_mut() {
            dict.insert("a".to_string(), Node::Integer(1));
            dict.insert("b".to_string(), Node::Integer(2));
        }

        assert_eq!(node.len(), 2);
        assert_eq!(node.get("a"), Some(&Node::Integer(1)));
    }

    #[test]
    fn test_complex_nested_display() {
        let mut inner_dict = HashMap::new();
        inner_dict.insert("nested".to_string(), Node::Integer(42));

        let complex = Node::List(vec![
            Node::Integer(1),
            Node::Str("test".to_string()),
            Node::Dictionary(inner_dict),
        ]);

        let display = format!("{}", complex);
        assert!(display.contains("1"));
        assert!(display.contains("\"test\""));
        assert!(display.contains("\"nested\""));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_round_trip_with_convenience_methods() {
        // Create a complex structure
        let mut dict = HashMap::new();
        dict.insert("number".to_string(), Node::Integer(42));
        dict.insert("text".to_string(), Node::Str("hello".to_string()));
        dict.insert(
            "list".to_string(),
            Node::List(vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)]),
        );
        let original = Node::Dictionary(dict);

        // Stringify to string
        let bencode_str = stringify_to_string(&original).unwrap();

        // Parse back
        let parsed = parse_str(&bencode_str).unwrap();

        // Verify they match
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_make_node_with_new_methods() {
        let node = make_node(42);
        assert!(node.is_integer());
        assert_eq!(node.as_integer(), Some(&42));

        let node = make_node("test");
        assert!(node.is_string());
        assert_eq!(node.as_string(), Some("test"));
    }

    // --- Node::From conversions ---

    #[test]
    fn test_node_from_i64() {
        let node: Node = Node::from(99i64);
        assert_eq!(node, Node::Integer(99));
    }

    #[test]
    fn test_node_from_str_slice() {
        let node: Node = Node::from("hello");
        assert_eq!(node, Node::Str("hello".to_string()));
    }

    #[test]
    fn test_node_from_string() {
        let node: Node = Node::from("owned".to_string());
        assert_eq!(node, Node::Str("owned".to_string()));
    }

    #[test]
    fn test_node_from_vec() {
        let node: Node = Node::from(vec![1i64, 2i64, 3i64]);
        assert_eq!(
            node,
            Node::List(vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)])
        );
    }

    #[test]
    fn test_node_from_array() {
        let node: Node = Node::from([1i64, 2i64]);
        assert_eq!(node, Node::List(vec![Node::Integer(1), Node::Integer(2)]));
    }

    #[test]
    fn test_node_from_tuple_array() {
        let node: Node = Node::from([("x", 10i64), ("y", 20i64)]);
        assert!(node.is_dictionary());
        assert_eq!(node.get("x"), Some(&Node::Integer(10)));
        assert_eq!(node.get("y"), Some(&Node::Integer(20)));
    }

    #[test]
    fn test_node_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::Integer(7));
        let node: Node = Node::from(map);
        assert!(node.is_dictionary());
        assert_eq!(node.get("k"), Some(&Node::Integer(7)));
    }

    // --- Node validation helpers ---

    #[test]
    fn test_get_required_present() {
        let node = Node::from([("name", "alice"), ("role", "admin")]);
        assert_eq!(
            node.get_required("name").unwrap(),
            &Node::Str("alice".to_string())
        );
    }

    #[test]
    fn test_get_required_missing_returns_error() {
        let node = Node::from([("a", 1i64)]);
        let result = node.get_required("missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing"));
    }

    #[test]
    fn test_get_int_required_ok() {
        let node = Node::from([("count", 42i64)]);
        assert_eq!(node.get_int_required("count").unwrap(), 42);
    }

    #[test]
    fn test_get_int_required_wrong_type() {
        let node = Node::from([("value", "not-an-int")]);
        assert!(node.get_int_required("value").is_err());
    }

    #[test]
    fn test_get_string_required_ok() {
        let node = Node::from([("title", "hello")]);
        assert_eq!(node.get_string_required("title").unwrap(), "hello");
    }

    #[test]
    fn test_get_string_required_wrong_type() {
        let mut map = HashMap::new();
        map.insert("n".to_string(), Node::Integer(1));
        let node = Node::Dictionary(map);
        assert!(node.get_string_required("n").is_err());
    }

    #[test]
    fn test_get_list_required_ok() {
        let mut map = HashMap::new();
        map.insert(
            "items".to_string(),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
        );
        let node = Node::Dictionary(map);
        assert_eq!(node.get_list_required("items").unwrap().len(), 2);
    }

    #[test]
    fn test_get_dict_required_ok() {
        let mut inner = HashMap::new();
        inner.insert("x".to_string(), Node::Integer(5));
        let mut outer = HashMap::new();
        outer.insert("meta".to_string(), Node::Dictionary(inner));
        let node = Node::Dictionary(outer);
        assert_eq!(node.get_dict_required("meta").unwrap().len(), 1);
    }

    #[test]
    fn test_get_optional_fields() {
        let node = Node::from([("n", 7i64)]);
        assert_eq!(node.get_int_optional("n"), Some(7));
        assert_eq!(node.get_int_optional("missing"), None);
        assert_eq!(node.get_string_optional("n"), None); // wrong type
    }

    #[test]
    fn test_get_string_optional() {
        let node = Node::from([("s", "world")]);
        assert_eq!(node.get_string_optional("s"), Some("world"));
        assert_eq!(node.get_string_optional("absent"), None);
    }

    #[test]
    fn test_get_list_optional() {
        let mut map = HashMap::new();
        map.insert("lst".to_string(), Node::List(vec![Node::Integer(9)]));
        let node = Node::Dictionary(map);
        assert!(node.get_list_optional("lst").is_some());
        assert!(node.get_list_optional("nope").is_none());
    }

    #[test]
    fn test_get_dict_optional() {
        let mut inner = HashMap::new();
        inner.insert("a".to_string(), Node::Integer(1));
        let mut outer = HashMap::new();
        outer.insert("sub".to_string(), Node::Dictionary(inner));
        let node = Node::Dictionary(outer);
        assert!(node.get_dict_optional("sub").is_some());
        assert!(node.get_dict_optional("nope").is_none());
    }

    // --- Zero-copy / borrowed parser ---

    #[test]
    fn test_parse_borrowed_integer() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"i-1e").unwrap();
        assert!(node.is_integer());
        assert_eq!(node.as_integer(), Some(-1));
    }

    #[test]
    fn test_parse_borrowed_bytes() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"5:hello").unwrap();
        assert!(node.is_bytes());
        assert_eq!(node.as_bytes(), Some(b"hello".as_ref()));
    }

    #[test]
    fn test_parse_borrowed_list() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"li1ei2ei3ee").unwrap();
        assert!(node.is_list());
        assert_eq!(node.as_list().unwrap().len(), 3);
    }

    #[test]
    fn test_parse_borrowed_dictionary() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"d3:keyi99ee").unwrap();
        assert!(node.is_dictionary());
    }

    #[test]
    fn test_borrowed_node_to_node_conversion() {
        use crate::parse_borrowed;
        let borrowed = parse_borrowed(b"d4:name5:alicee").unwrap();
        let owned = borrowed.to_node();
        assert!(owned.is_dictionary());
        assert_eq!(owned.get("name"), Some(&Node::Str("alice".to_string())));
    }

    #[test]
    fn test_validate_bencode_valid() {
        use crate::validate_bencode;
        assert!(validate_bencode(b"i42e").is_ok());
        assert!(validate_bencode(b"4:test").is_ok());
        assert!(validate_bencode(b"li1ei2ee").is_ok());
        assert!(validate_bencode(b"d3:keyi0ee").is_ok());
    }

    #[test]
    fn test_validate_bencode_invalid() {
        use crate::validate_bencode;
        assert!(validate_bencode(b"").is_err());
        assert!(validate_bencode(b"i42").is_err()); // unterminated integer
    }

    // --- Iterative parser ---

    #[test]
    fn test_parse_str_iterative_integer() {
        use crate::parse_str_iterative;
        let result = parse_str_iterative("i100e");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Node::Integer(100));
    }

    #[test]
    fn test_parse_bytes_iterative_string() {
        use crate::parse_bytes_iterative;
        let result = parse_bytes_iterative(b"5:world");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Node::Str("world".to_string()));
    }

    #[test]
    fn test_parse_str_iterative_nested_list() {
        use crate::parse_str_iterative;
        let result = parse_str_iterative("lli1ei2eeli3ei4eee");
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_list());
        assert_eq!(node.len(), 2);
    }

    #[test]
    fn test_iterative_round_trip() {
        use crate::{parse_bytes_iterative, stringify_to_bytes};
        let original = b"d4:listli1ei2ee6:numberi42ee";
        let node = parse_bytes_iterative(original).unwrap();
        let encoded = stringify_to_bytes(&node).unwrap();
        // Re-parse and compare nodes rather than raw bytes (dict key order may differ)
        let reparsed = parse_bytes_iterative(&encoded).unwrap();
        assert_eq!(node, reparsed);
    }

    // --- Integer boundary values ---

    #[test]
    fn test_parse_i64_max() {
        let input = format!("i{}e", i64::MAX);
        let node = parse_str(&input).unwrap();
        assert_eq!(node, Node::Integer(i64::MAX));
    }

    #[test]
    fn test_parse_i64_min() {
        let input = format!("i{}e", i64::MIN);
        let node = parse_str(&input).unwrap();
        assert_eq!(node, Node::Integer(i64::MIN));
    }

    #[test]
    fn test_parse_zero() {
        let node = parse_str("i0e").unwrap();
        assert_eq!(node, Node::Integer(0));
    }

    #[test]
    fn test_parse_negative_integer() {
        let node = parse_str("i-999e").unwrap();
        assert_eq!(node, Node::Integer(-999));
    }

    // --- String edge cases ---

    #[test]
    fn test_parse_empty_string() {
        let node = parse_bytes(b"0:").unwrap();
        assert_eq!(node, Node::Str("".to_string()));
    }

    #[test]
    fn test_stringify_empty_string() {
        let node = Node::Str("".to_string());
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "0:");
    }

    #[test]
    fn test_round_trip_empty_list() {
        let node = Node::List(vec![]);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "le");
        let parsed = parse_str(&encoded).unwrap();
        assert_eq!(parsed, node);
    }

    #[test]
    fn test_round_trip_empty_dictionary() {
        let node = Node::Dictionary(HashMap::new());
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "de");
        let parsed = parse_str(&encoded).unwrap();
        assert_eq!(parsed, node);
    }
}
