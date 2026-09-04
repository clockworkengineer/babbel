//! Integration tests for the bencode JSON stringify functionality.
//! These tests validate the JSON stringify behavior from an external perspective,
//! testing the public API against various node structures.

#[cfg(test)]
mod tests {
    use crate::BufferDestination;
    use crate::nodes::node::Node;
    use crate::stringify::json::stringify;
    use std::collections::HashMap;

    #[test]
    fn test_stringify_dictionary_sorting() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("z".to_string(), Node::Integer(1));
        dict.insert("a".to_string(), Node::Integer(2));
        dict.insert("m".to_string(), Node::Integer(3));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"a\":2,\"m\":3,\"z\":1}");
    }

    #[test]
    fn test_stringify_complex_nested_structure() {
        let mut destination = BufferDestination::new();
        let mut inner_dict1 = HashMap::new();
        inner_dict1.insert("x".to_string(), Node::Integer(1));
        let mut inner_dict2 = HashMap::new();
        inner_dict2.insert(
            "y".to_string(),
            Node::List(vec![
                Node::Str("a".to_string()),
                Node::Dictionary(inner_dict1),
                Node::Integer(42),
            ]),
        );
        stringify(&Node::Dictionary(inner_dict2), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"y\":[\"a\",{\"x\":1},42]}");
    }

    #[test]
    fn test_stringify_nested_structures() {
        let mut destination = BufferDestination::new();
        let inner_list = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        let mut inner_dict = HashMap::new();
        inner_dict.insert("key".to_string(), Node::Str("value".to_string()));
        let dict = Node::Dictionary(inner_dict);

        stringify(&Node::List(vec![inner_list, dict]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[[1,2],{\"key\":\"value\"}]");
    }

    #[test]
    fn test_stringify_list_with_none() {
        let mut destination = BufferDestination::new();
        let list = vec![Node::Integer(1), Node::None, Node::Integer(2)];
        stringify(&Node::List(list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[1,null,2]");
    }

    #[test]
    fn test_stringify_dictionary_with_list() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            "list".to_string(),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
        );
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"list\":[1,2]}");
    }

    #[test]
    fn test_stringify_empty_structures() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[]");
        destination.clear();
        stringify(&Node::Dictionary(HashMap::new()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{}");
    }

    #[test]
    fn test_stringify_deeply_nested_dictionary() {
        let mut destination = BufferDestination::new();
        let mut level3 = HashMap::new();
        level3.insert("level3".to_string(), Node::Integer(3));

        let mut level2 = HashMap::new();
        level2.insert("level2".to_string(), Node::Dictionary(level3));

        let mut level1 = HashMap::new();
        level1.insert("level1".to_string(), Node::Dictionary(level2));

        stringify(&Node::Dictionary(level1), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "{\"level1\":{\"level2\":{\"level3\":3}}}"
        );
    }

    #[test]
    fn test_stringify_mixed_types_in_list() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Node::Str("value".to_string()));

        let list = vec![
            Node::Integer(42),
            Node::Str("text".to_string()),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
            Node::Dictionary(dict),
            Node::None,
        ];

        stringify(&Node::List(list), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "[42,\"text\",[1,2],{\"key\":\"value\"},null]"
        );
    }

    #[test]
    fn test_stringify_multiple_none_values() {
        let mut destination = BufferDestination::new();
        let list = vec![Node::None, Node::None, Node::None];
        stringify(&Node::List(list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[null,null,null]");
    }

    #[test]
    fn test_stringify_dictionary_with_empty_string_key() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("".to_string(), Node::Integer(1));
        dict.insert("key".to_string(), Node::Integer(2));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"\":1,\"key\":2}");
    }

    #[test]
    fn test_stringify_negative_integers() {
        let mut destination = BufferDestination::new();
        let list = vec![
            Node::Integer(-1),
            Node::Integer(-100),
            Node::Integer(0),
            Node::Integer(100),
        ];
        stringify(&Node::List(list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[-1,-100,0,100]");
    }

    #[test]
    fn test_stringify_string_with_escapes() {
        let mut destination = BufferDestination::new();
        stringify(
            &Node::Str("hello\"world\\test".to_string()),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "\"hello\\\"world\\\\test\"");
    }

    #[test]
    fn test_stringify_string_with_newline_tab() {
        let mut destination = BufferDestination::new();
        stringify(
            &Node::Str("line1\nline2\ttab".to_string()),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "\"line1\\u000aline2\\u0009tab\"");
    }

    // --- Standalone scalar values ---

    #[test]
    fn test_stringify_standalone_integer() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(0), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "0");
    }

    #[test]
    fn test_stringify_i64_max() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(i64::MAX), &mut destination).unwrap();
        assert_eq!(destination.to_string(), i64::MAX.to_string());
    }

    #[test]
    fn test_stringify_i64_min() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(i64::MIN), &mut destination).unwrap();
        assert_eq!(destination.to_string(), i64::MIN.to_string());
    }

    #[test]
    fn test_stringify_standalone_none() {
        let mut destination = BufferDestination::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "null");
    }

    #[test]
    fn test_stringify_standalone_empty_string() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"\"");
    }

    // --- String escape sequences ---

    #[test]
    fn test_stringify_string_with_carriage_return() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\rb".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u000db\"");
    }

    #[test]
    fn test_stringify_string_with_null_byte() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\x00b".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u0000b\"");
    }

    #[test]
    fn test_stringify_string_with_backspace() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\x08b".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u0008b\"");
    }

    #[test]
    fn test_stringify_string_with_formfeed() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\x0Cb".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u000cb\"");
    }

    #[test]
    fn test_stringify_string_only_backslash() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("\\".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"\\\\\"");
    }

    #[test]
    fn test_stringify_string_only_quote() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("\"".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"\\\"\"");
    }

    #[test]
    fn test_stringify_string_mixed_escapes() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("\"\\\n\t\r\x00".to_string()), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "\"\\\"\\\\\\u000a\\u0009\\u000d\\u0000\""
        );
    }

    // --- Single-element collections ---

    #[test]
    fn test_stringify_single_element_list() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![Node::Integer(99)]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[99]");
    }

    #[test]
    fn test_stringify_single_entry_dict() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("k".to_string(), Node::Str("v".to_string()));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"k\":\"v\"}");
    }

    // --- Nested collections ---

    #[test]
    fn test_stringify_list_of_lists() {
        let mut destination = BufferDestination::new();
        let node = Node::List(vec![
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
            Node::List(vec![Node::Integer(3), Node::Integer(4)]),
        ]);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[[1,2],[3,4]]");
    }

    #[test]
    fn test_stringify_dict_of_dicts() {
        let mut destination = BufferDestination::new();
        let mut inner_a = HashMap::new();
        inner_a.insert("x".to_string(), Node::Integer(1));
        let mut inner_b = HashMap::new();
        inner_b.insert("y".to_string(), Node::Integer(2));
        let mut outer = HashMap::new();
        outer.insert("a".to_string(), Node::Dictionary(inner_a));
        outer.insert("b".to_string(), Node::Dictionary(inner_b));
        stringify(&Node::Dictionary(outer), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"a\":{\"x\":1},\"b\":{\"y\":2}}");
    }

    #[test]
    fn test_stringify_list_of_empty_dicts() {
        let mut destination = BufferDestination::new();
        let node = Node::List(vec![
            Node::Dictionary(HashMap::new()),
            Node::Dictionary(HashMap::new()),
        ]);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[{},{}]");
    }

    // --- Dictionary key ordering ---

    #[test]
    fn test_stringify_dict_keys_always_sorted() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("zebra".to_string(), Node::Integer(3));
        dict.insert("apple".to_string(), Node::Integer(1));
        dict.insert("mango".to_string(), Node::Integer(2));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "{\"apple\":1,\"mango\":2,\"zebra\":3}"
        );
    }

    // --- public to_json API ---

    #[test]
    fn test_to_json_integer() {
        use crate::to_json;
        let mut destination = BufferDestination::new();
        to_json(&Node::Integer(7), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "7");
    }

    #[test]
    fn test_to_json_nested_structure() {
        use crate::to_json;
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), Node::Integer(42));
        dict.insert("s".to_string(), Node::Str("hi".to_string()));
        to_json(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{\"n\":42,\"s\":\"hi\"}");
    }
}
