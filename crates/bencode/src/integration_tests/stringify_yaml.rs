//! Integration tests for the bencode YAML stringify functionality.
//! These tests validate the YAML stringify behavior from an external perspective,
//! testing the public API against various node structures.

#[cfg(test)]
mod tests {
    use crate::BufferDestination;
    use crate::nodes::node::Node;
    use crate::stringify::yaml::stringify;
    use std::collections::HashMap;

    #[test]
    fn test_nested_list() {
        let mut destination = BufferDestination::new();
        stringify(
            &Node::List(vec![Node::List(vec![Node::Integer(1)])]),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "\n- \n  - 1\n\n");
    }

    #[test]
    fn test_nested_dictionary() {
        let mut destination = BufferDestination::new();
        let mut inner_dict = HashMap::new();
        inner_dict.insert("inner".to_string(), Node::Integer(1));
        let mut outer_dict = HashMap::new();
        outer_dict.insert("outer".to_string(), Node::Dictionary(inner_dict));
        stringify(&Node::Dictionary(outer_dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\nouter: \n  inner: 1\n\n");
    }

    #[test]
    fn test_multi_entry_dictionary() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("a".to_string(), Node::Integer(1));
        dict.insert("b".to_string(), Node::Integer(2));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\na: 1\nb: 2\n");
    }

    #[test]
    fn test_mixed_list() {
        let mut destination = BufferDestination::new();
        stringify(
            &Node::List(vec![
                Node::Integer(1),
                Node::Str("test".to_string()),
                Node::List(vec![]),
            ]),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "\n- 1\n- \"test\"\n- []\n");
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut destination = BufferDestination::new();
        let mut level3 = HashMap::new();
        level3.insert("deep".to_string(), Node::Integer(42));

        let mut level2 = HashMap::new();
        level2.insert("level3".to_string(), Node::Dictionary(level3));

        let mut level1 = HashMap::new();
        level1.insert("level2".to_string(), Node::Dictionary(level2));

        stringify(&Node::Dictionary(level1), &mut destination).unwrap();
        let output = destination.to_string();

        assert!(output.contains("level2:"));
        assert!(output.contains("level3:"));
        assert!(output.contains("deep: 42"));
    }

    #[test]
    fn test_complex_mixed_structure() {
        let mut destination = BufferDestination::new();

        let mut dict = HashMap::new();
        dict.insert("number".to_string(), Node::Integer(100));
        dict.insert("text".to_string(), Node::Str("hello".to_string()));
        dict.insert(
            "list".to_string(),
            Node::List(vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)]),
        );

        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        let output = destination.to_string();

        assert!(output.contains("number: 100"));
        assert!(output.contains("text: \"hello\""));
        assert!(output.contains("list:"));
        assert!(output.contains("- 1"));
        assert!(output.contains("- 2"));
        assert!(output.contains("- 3"));
    }

    #[test]
    fn test_dictionary_key_sorting() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("zebra".to_string(), Node::Integer(3));
        dict.insert("apple".to_string(), Node::Integer(1));
        dict.insert("middle".to_string(), Node::Integer(2));

        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        let output = destination.to_string();

        // Check that keys appear in sorted order
        let apple_pos = output.find("apple:").unwrap();
        let middle_pos = output.find("middle:").unwrap();
        let zebra_pos = output.find("zebra:").unwrap();

        assert!(apple_pos < middle_pos);
        assert!(middle_pos < zebra_pos);
    }

    #[test]
    fn test_list_of_dictionaries() {
        let mut destination = BufferDestination::new();

        let mut dict1 = HashMap::new();
        dict1.insert("id".to_string(), Node::Integer(1));

        let mut dict2 = HashMap::new();
        dict2.insert("id".to_string(), Node::Integer(2));

        let list = Node::List(vec![Node::Dictionary(dict1), Node::Dictionary(dict2)]);

        stringify(&list, &mut destination).unwrap();
        let output = destination.to_string();

        assert!(output.contains("- "));
        assert!(output.contains("id: 1"));
        assert!(output.contains("id: 2"));
    }

    #[test]
    fn test_empty_string() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"\"");
    }

    #[test]
    fn test_negative_integer() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(-42), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "-42");
    }

    #[test]
    fn test_deeply_nested_lists() {
        let mut destination = BufferDestination::new();
        let inner = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        let middle = Node::List(vec![inner]);
        let outer = Node::List(vec![middle]);

        stringify(&outer, &mut destination).unwrap();
        let output = destination.to_string();

        // Check that we have proper nesting with indentation
        assert!(output.contains("- "));
        assert!(output.contains("  - "));
    }

    #[test]
    fn test_dictionary_with_empty_structures() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("empty_list".to_string(), Node::List(vec![]));
        dict.insert("empty_dict".to_string(), Node::Dictionary(HashMap::new()));
        dict.insert("value".to_string(), Node::Integer(42));

        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        let output = destination.to_string();

        assert!(output.contains("empty_list: []"));
        assert!(output.contains("empty_dict: {}"));
        assert!(output.contains("value: 42"));
    }

    #[test]
    fn test_none_node() {
        let mut destination = BufferDestination::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "unknown");
    }

    #[test]
    fn test_string_with_special_chars() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("test\nline".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"test\\u000aline\"");
    }

    // --- Scalar edge cases ---

    #[test]
    fn test_integer_zero() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(0), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "0");
    }

    #[test]
    fn test_integer_one() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(1), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "1");
    }

    #[test]
    fn test_i64_max() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(i64::MAX), &mut destination).unwrap();
        assert_eq!(destination.to_string(), i64::MAX.to_string());
    }

    #[test]
    fn test_i64_min() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Integer(i64::MIN), &mut destination).unwrap();
        assert_eq!(destination.to_string(), i64::MIN.to_string());
    }

    #[test]
    fn test_plain_string() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("hello".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"hello\"");
    }

    // --- String escaping ---

    #[test]
    fn test_string_with_tab() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\tb".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u0009b\"");
    }

    #[test]
    fn test_string_with_carriage_return() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\rb".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u000db\"");
    }

    #[test]
    fn test_string_with_null_byte() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("a\x00b".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"a\\u0000b\"");
    }

    #[test]
    fn test_string_with_backslash() {
        let mut destination = BufferDestination::new();
        // one backslash → escape_string writes "\\" → output is "\\"
        stringify(&Node::Str("\\".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"\\\\\"");
    }

    #[test]
    fn test_string_with_double_quote() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Str("\"".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"\\\"\"");
    }

    // --- None in collections ---

    #[test]
    fn test_none_in_list() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![Node::None, Node::None]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\n- unknown\n- unknown\n");
    }

    #[test]
    fn test_none_as_dict_value() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("k".to_string(), Node::None);
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\nk: unknown\n");
    }

    // --- Exact output for common shapes ---

    #[test]
    fn test_single_element_list_integer() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![Node::Integer(7)]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\n- 7\n");
    }

    #[test]
    fn test_list_of_strings() {
        let mut destination = BufferDestination::new();
        stringify(
            &Node::List(vec![
                Node::Str("foo".to_string()),
                Node::Str("bar".to_string()),
            ]),
            &mut destination,
        )
        .unwrap();
        assert_eq!(destination.to_string(), "\n- \"foo\"\n- \"bar\"\n");
    }

    #[test]
    fn test_single_entry_dict_string_value() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("name".to_string(), Node::Str("alice".to_string()));
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\nname: \"alice\"\n");
    }

    #[test]
    fn test_dict_value_is_list() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            "nums".to_string(),
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
        );
        stringify(&Node::Dictionary(dict), &mut destination).unwrap();
        // top-level dict entry, then list items indented one level
        assert_eq!(destination.to_string(), "\nnums: \n  - 1\n  - 2\n\n");
    }

    #[test]
    fn test_list_containing_dict() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("k".to_string(), Node::Integer(99));
        stringify(&Node::List(vec![Node::Dictionary(dict)]), &mut destination).unwrap();
        // list item is a dict: "- \n  k: 99\n"
        assert_eq!(destination.to_string(), "\n- \n  k: 99\n\n");
    }

    #[test]
    fn test_empty_dict_standalone() {
        let mut destination = BufferDestination::new();
        stringify(&Node::Dictionary(HashMap::new()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{}");
    }

    #[test]
    fn test_empty_list_standalone() {
        let mut destination = BufferDestination::new();
        stringify(&Node::List(vec![]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "[]");
    }

    // --- Indentation at nesting levels ---

    #[test]
    fn test_dict_inside_dict_indentation() {
        let mut destination = BufferDestination::new();
        let mut inner = HashMap::new();
        inner.insert("x".to_string(), Node::Integer(5));
        let mut outer = HashMap::new();
        outer.insert("inner".to_string(), Node::Dictionary(inner));
        stringify(&Node::Dictionary(outer), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\ninner: \n  x: 5\n\n");
    }

    #[test]
    fn test_list_inside_list_indentation() {
        let mut destination = BufferDestination::new();
        let inner = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
        stringify(&Node::List(vec![inner]), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\n- \n  - 1\n  - 2\n\n");
    }

    // --- Public to_yaml API ---

    #[test]
    fn test_to_yaml_integer() {
        use crate::to_yaml;
        let mut destination = BufferDestination::new();
        to_yaml(&Node::Integer(42), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "42");
    }

    #[test]
    fn test_to_yaml_string() {
        use crate::to_yaml;
        let mut destination = BufferDestination::new();
        to_yaml(&Node::Str("hi".to_string()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "\"hi\"");
    }

    #[test]
    fn test_to_yaml_empty_dict() {
        use crate::to_yaml;
        let mut destination = BufferDestination::new();
        to_yaml(&Node::Dictionary(HashMap::new()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "{}");
    }

    #[test]
    fn test_to_yaml_nested_structure() {
        use crate::to_yaml;
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), Node::Integer(7));
        dict.insert("s".to_string(), Node::Str("ok".to_string()));
        to_yaml(&Node::Dictionary(dict), &mut destination).unwrap();
        let output = destination.to_string();
        assert!(output.contains("n: 7"));
        assert!(output.contains("s: \"ok\""));
    }
}
