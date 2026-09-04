//! Integration tests for the bencode stringify functionality.
//! These tests validate the bencode (binary encoding) stringify behaviour,
//! testing the public API against various node structures.

#[cfg(test)]
mod tests {
    use crate::BufferDestination;
    use crate::nodes::node::{Node, make_node};
    use crate::stringify::default::stringify;
    use std::collections::HashMap;

    #[test]
    fn test_stringify_complex_dictionary() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(String::from("b"), make_node(1));
        dict.insert(String::from("a"), make_node(2));
        dict.insert(String::from("c"), make_node("test"));
        stringify(&make_node(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d1:ai2e1:bi1e1:c4:teste");
    }

    #[test]
    fn test_stringify_nested_dictionary() {
        let mut destination = BufferDestination::new();
        let mut inner_dict = HashMap::new();
        inner_dict.insert(String::from("key2"), make_node("value"));
        let mut outer_dict = HashMap::new();
        outer_dict.insert(String::from("key1"), make_node(inner_dict));
        stringify(&make_node(outer_dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d4:key1d4:key25:valueee");
    }

    #[test]
    fn test_stringify_list_with_none() {
        let mut destination = BufferDestination::new();
        let list = vec![make_node(32), Node::None, make_node("test")];
        stringify(&make_node(list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "li32e4:teste");
    }

    #[test]
    fn test_stringify_dictionary_with_list() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            String::from("list"),
            make_node(vec![make_node(1), make_node(2)]),
        );
        stringify(&make_node(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d4:listli1ei2eee");
    }

    #[test]
    fn test_stringify_deeply_nested_structure() {
        let mut destination = BufferDestination::new();

        // Create a complex nested structure
        let mut inner_dict = HashMap::new();
        inner_dict.insert(String::from("nested"), make_node("value"));

        let list = vec![make_node(1), make_node("text"), make_node(inner_dict)];

        let mut outer_dict = HashMap::new();
        outer_dict.insert(String::from("data"), make_node(list));
        outer_dict.insert(String::from("count"), make_node(42));

        stringify(&make_node(outer_dict), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "d5:counti42e4:datali1e4:textd6:nested5:valueeee"
        );
    }

    #[test]
    fn test_stringify_multiple_nested_lists() {
        let mut destination = BufferDestination::new();
        let inner_list1 = vec![make_node(1), make_node(2)];
        let inner_list2 = vec![make_node(3), make_node(4)];
        let outer_list = vec![make_node(inner_list1), make_node(inner_list2)];
        stringify(&make_node(outer_list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "lli1ei2eeli3ei4eee");
    }

    #[test]
    fn test_stringify_dictionary_key_ordering() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        // Insert in non-alphabetical order
        dict.insert(String::from("zebra"), make_node(1));
        dict.insert(String::from("apple"), make_node(2));
        dict.insert(String::from("middle"), make_node(3));
        stringify(&make_node(dict), &mut destination).unwrap();
        // Should be ordered alphabetically
        assert_eq!(destination.to_string(), "d5:applei2e6:middlei3e5:zebrai1ee");
    }

    #[test]
    fn test_stringify_empty_string() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(""), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "0:");
    }

    #[test]
    fn test_stringify_negative_integer() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(-42), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "i-42e");
    }

    #[test]
    fn test_stringify_mixed_dictionary() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(String::from("integer"), make_node(100));
        dict.insert(String::from("string"), make_node("hello"));
        dict.insert(
            String::from("list"),
            make_node(vec![make_node(1), make_node(2), make_node(3)]),
        );

        let mut nested = HashMap::new();
        nested.insert(String::from("x"), make_node(10));
        dict.insert(String::from("dict"), make_node(nested));

        stringify(&make_node(dict), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "d4:dictd1:xi10ee7:integeri100e4:listli1ei2ei3ee6:string5:helloe"
        );
    }

    #[test]
    fn test_stringify_zero_integer() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(0), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "i0e");
    }

    #[test]
    fn test_stringify_large_integer() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(9223372036854775807i64), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "i9223372036854775807e");
    }

    #[test]
    fn test_stringify_list_of_empty_strings() {
        let mut destination = BufferDestination::new();
        let list = vec![make_node(""), make_node(""), make_node("")];
        stringify(&make_node(list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "l0:0:0:e");
    }

    #[test]
    fn test_stringify_dictionary_with_numeric_string_keys() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(String::from("1"), make_node("one"));
        dict.insert(String::from("2"), make_node("two"));
        dict.insert(String::from("10"), make_node("ten"));

        stringify(&make_node(dict), &mut destination).unwrap();
        let output = destination.to_string();

        // Keys should be sorted lexicographically: "1", "10", "2"
        assert_eq!(output, "d1:13:one2:103:ten1:23:twoe");
    }

    #[test]
    fn test_stringify_very_nested_structure() {
        let mut destination = BufferDestination::new();

        let level5 = make_node("deepest");
        let level4 = make_node(vec![level5]);

        let mut dict3 = HashMap::new();
        dict3.insert(String::from("l4"), level4);
        let level3 = make_node(dict3);

        let level2 = make_node(vec![level3]);

        let mut dict1 = HashMap::new();
        dict1.insert(String::from("l2"), level2);
        let level1 = make_node(dict1);

        stringify(&level1, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d2:l2ld2:l4l7:deepesteeee");
    }

    // --- Integer encodings ---

    #[test]
    fn test_stringify_i64_min() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(i64::MIN), &mut destination).unwrap();
        assert_eq!(destination.to_string(), format!("i{}e", i64::MIN));
    }

    #[test]
    fn test_stringify_integer_one() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(1), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "i1e");
    }

    #[test]
    fn test_stringify_integer_minus_one() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(-1), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "i-1e");
    }

    // --- String encodings ---

    #[test]
    fn test_stringify_single_char_string() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("x".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "1:x");
    }

    #[test]
    fn test_stringify_string_with_digits() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("12345".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "5:12345");
    }

    #[test]
    fn test_stringify_string_containing_colon() {
        // Colon in content is valid bencode string data
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a:b".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "3:a:b");
    }

    // --- None handling ---

    #[test]
    fn test_stringify_standalone_none_emits_nothing() {
        let mut destination = BufferDestination::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "");
    }

    #[test]
    fn test_stringify_list_all_none_produces_empty_list_body() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![Node::None, Node::None]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "le");
    }

    // --- Empty collections ---

    #[test]
    fn test_stringify_empty_list() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "le");
    }

    #[test]
    fn test_stringify_empty_dictionary() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Dictionary(HashMap::new()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "de");
    }

    // --- Dictionary key ordering ---

    #[test]
    fn test_stringify_dict_keys_sorted_lexicographically() {
        // "ab" < "abc" lexicographically
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("abc".to_string(), Node::Integer(2));
        dict.insert("ab".to_string(), Node::Integer(1));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d2:abi1e3:abci2ee");
    }

    #[test]
    fn test_stringify_dict_single_entry() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("k".to_string(), Node::Integer(99));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d1:ki99ee");
    }

    // --- Convenience API ---

    #[test]
    fn test_stringify_to_string_integer() {
        use crate::stringify_to_string;
        assert_eq!(stringify_to_string(&Node::Integer(42)).unwrap(), "i42e");
    }

    #[test]
    fn test_stringify_to_string_string_node() {
        use crate::stringify_to_string;
        assert_eq!(
            stringify_to_string(&Node::Str("hi".to_string())).unwrap(),
            "2:hi"
        );
    }

    #[test]
    fn test_stringify_to_string_empty_list() {
        use crate::stringify_to_string;
        assert_eq!(stringify_to_string(&Node::List(vec![])).unwrap(), "le");
    }

    #[test]
    fn test_stringify_to_string_empty_dict() {
        use crate::stringify_to_string;
        assert_eq!(
            stringify_to_string(&Node::Dictionary(HashMap::new())).unwrap(),
            "de"
        );
    }

    #[test]
    fn test_stringify_to_bytes_integer() {
        use crate::stringify_to_bytes;
        assert_eq!(
            stringify_to_bytes(&Node::Integer(0)).unwrap(),
            b"i0e".to_vec()
        );
    }

    #[test]
    fn test_stringify_to_bytes_string() {
        use crate::stringify_to_bytes;
        assert_eq!(
            stringify_to_bytes(&Node::Str("abc".to_string())).unwrap(),
            b"3:abc".to_vec()
        );
    }

    // --- Round-trips with parse_bytes ---

    #[test]
    fn test_round_trip_integer() {
        use crate::{parse_bytes, stringify_to_bytes};
        let node = Node::Integer(-999);
        assert_eq!(
            parse_bytes(&stringify_to_bytes(&node).unwrap()).unwrap(),
            node
        );
    }

    #[test]
    fn test_round_trip_string() {
        use crate::{parse_bytes, stringify_to_bytes};
        let node = Node::Str("bencode".to_string());
        assert_eq!(
            parse_bytes(&stringify_to_bytes(&node).unwrap()).unwrap(),
            node
        );
    }

    #[test]
    fn test_round_trip_list() {
        use crate::{parse_bytes, stringify_to_bytes};
        let node = Node::List(vec![
            Node::Integer(1),
            Node::Str("two".to_string()),
            Node::Integer(3),
        ]);
        assert_eq!(
            parse_bytes(&stringify_to_bytes(&node).unwrap()).unwrap(),
            node
        );
    }

    #[test]
    fn test_round_trip_dict() {
        use crate::{parse_bytes, stringify_to_bytes};
        let mut dict = HashMap::new();
        dict.insert("aaa".to_string(), Node::Integer(1));
        dict.insert("bbb".to_string(), Node::Str("val".to_string()));
        let node = Node::Dictionary(dict);
        let encoded = stringify_to_bytes(&node).unwrap();
        let reparsed = parse_bytes(&encoded).unwrap();
        assert_eq!(node, reparsed);
    }

    #[test]
    fn test_round_trip_nested() {
        use crate::{parse_bytes, stringify_to_bytes};
        let mut inner = HashMap::new();
        inner.insert("x".to_string(), Node::Integer(42));
        let node = Node::List(vec![
            Node::Dictionary(inner),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
        ]);
        assert_eq!(
            parse_bytes(&stringify_to_bytes(&node).unwrap()).unwrap(),
            node
        );
    }
}
