//! Integration tests for the bencode TOML stringify functionality.
//! These tests validate the TOML stringify behavior from an external perspective,
//! testing the public API against various node structures.

#[cfg(test)]
mod tests {
    use crate::BufferDestination;
    use crate::BufferSource;
    use crate::nodes::node::{Node, make_node};
    use crate::stringify::toml::stringify;
    use std::collections::HashMap;

    #[test]
    fn test_stringify_nested_dictionary() {
        let mut destination = BufferDestination::new();
        let mut inner_dict = HashMap::new();
        inner_dict.insert("inner_key".to_string(), make_node("inner_value"));
        let mut outer_dict = HashMap::new();
        outer_dict.insert("outer_key".to_string(), make_node(inner_dict));
        let node = make_node(outer_dict);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(
            destination.to_string(),
            "[outer_key]\ninner_key = \"inner_value\"\n"
        );
    }

    #[test]
    fn test_stringify_deeply_nested_dictionary() {
        let mut level3 = HashMap::new();
        level3.insert("deep_key".to_string(), Node::Integer(123));
        let level3 = Node::Dictionary(level3);

        let mut level2 = HashMap::new();
        level2.insert("level3".to_string(), level3);
        let level2 = Node::Dictionary(level2);

        let mut level1 = HashMap::new();
        level1.insert("level2".to_string(), level2);
        let level1 = Node::Dictionary(level1);

        let mut root = HashMap::new();
        root.insert("level1".to_string(), level1);

        let mut dest = BufferDestination::new();
        stringify(&Node::Dictionary(root), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[level1.level2.level3]\ndeep_key = 123\n");
    }

    #[test]
    fn test_stringify_non_dictionary_root() {
        let mut destination = BufferDestination::new();
        let node = make_node("test");
        let result = stringify(&node, &mut destination);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML format requires a dictionary at the root level"
        );
    }

    #[test]
    fn test_heterogeneous_list() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            "key".to_string(),
            make_node(vec![make_node(1), make_node("test")]),
        );
        let result = stringify(&Node::Dictionary(dict), &mut dest);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML lists must contain elements of the same type"
        );
    }

    #[test]
    fn test_array_table() {
        let mut dest = BufferDestination::new();
        let mut inner1 = HashMap::new();
        inner1.insert("name".to_string(), make_node("first"));
        let mut inner2 = HashMap::new();
        inner2.insert("name".to_string(), make_node("second"));

        let mut dict = HashMap::new();
        dict.insert(
            "items".to_string(),
            make_node(vec![make_node(inner1), make_node(inner2)]),
        );

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[items]]\nname = \"first\"\n[[items]]\nname = \"second\"\n"
        );
    }

    #[test]
    fn test_mixed_array_table() {
        let mut dest = BufferDestination::new();
        let mut inner = HashMap::new();
        inner.insert("simple".to_string(), make_node(42));
        let mut nested = HashMap::new();
        nested.insert("value".to_string(), make_node("test"));
        inner.insert("complex".to_string(), make_node(nested));

        let mut dict = HashMap::new();
        dict.insert("items".to_string(), make_node(vec![make_node(inner)]));

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[items]]\nsimple = 42\n[items.complex]\nvalue = \"test\"\n"
        );
    }

    #[test]
    fn test_nested_array_tables() {
        let mut dest = BufferDestination::new();
        let mut deepest = HashMap::new();
        deepest.insert("value".to_string(), make_node(42));

        let mut inner = HashMap::new();
        inner.insert(
            "nested".to_string(),
            make_node(vec![make_node(deepest.clone())]),
        );

        let mut dict = HashMap::new();
        dict.insert("items".to_string(), make_node(vec![make_node(inner)]));

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[[items]]\n[items.nested]\nvalue = 42\n");
    }

    #[test]
    fn test_stringify_nested_object_with_array() {
        let mut source = BufferSource::new(
            b"d4:infod5:filesld6:lengthi351874e4:pathl10:large.jpegeed6:lengthi100e4:pathl1:2eeeee",
        );
        let node = crate::parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[info.files]]\nlength = 351874\npath = [\"large.jpeg\"]\n[[info.files]]\nlength = 100\npath = [\"2\"]\n"
        );
    }

    #[test]
    fn test_stringify_nested_object_with_array_and_object_and_array() {
        let mut source = BufferSource::new(
            b"d4:infod5:filesld6:lengthi351874e4:pathl10:large.jpegeed6:lengthi100e4:pathl1:2eeeee4:filesld6:lengthi351874e4:pathl10:large.jpege",
        );
        let node = crate::parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[info.files]]\nlength = 351874\npath = [\"large.jpeg\"]\n[[info.files]]\nlength = 100\npath = [\"2\"]\n"
        );
    }

    #[test]
    fn test_stringify_empty_dictionary() {
        let mut dest = BufferDestination::new();
        stringify(&Node::Dictionary(HashMap::new()), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "");
    }

    #[test]
    fn test_stringify_complex_mixed_structure() {
        let mut dest = BufferDestination::new();

        // Create a complex structure with multiple nesting levels
        let mut dict = HashMap::new();
        dict.insert("title".to_string(), make_node("Test Document"));
        dict.insert("version".to_string(), make_node(1));

        let mut author = HashMap::new();
        author.insert("name".to_string(), make_node("John Doe"));
        author.insert("email".to_string(), make_node("john@example.com"));
        dict.insert("author".to_string(), make_node(author));

        let mut item1 = HashMap::new();
        item1.insert("id".to_string(), make_node(1));
        item1.insert("name".to_string(), make_node("Item One"));

        let mut item2 = HashMap::new();
        item2.insert("id".to_string(), make_node(2));
        item2.insert("name".to_string(), make_node("Item Two"));

        dict.insert(
            "items".to_string(),
            make_node(vec![make_node(item1), make_node(item2)]),
        );

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let output = dest.to_string();

        // Verify key components are present
        assert!(output.contains("title = \"Test Document\""));
        assert!(output.contains("version = 1"));
        assert!(output.contains("[author]"));
        assert!(output.contains("name = \"John Doe\""));
        assert!(output.contains("[[items]]"));
        assert!(output.contains("id = 1"));
        assert!(output.contains("name = \"Item One\""));
    }

    #[test]
    fn test_stringify_with_list_of_primitives() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            "numbers".to_string(),
            make_node(vec![make_node(1), make_node(2), make_node(3)]),
        );
        dict.insert(
            "strings".to_string(),
            make_node(vec![make_node("a"), make_node("b"), make_node("c")]),
        );

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let output = dest.to_string();

        assert!(output.contains("numbers = [1, 2, 3]"));
        assert!(output.contains("strings = [\"a\", \"b\", \"c\"]"));
    }

    #[test]
    fn test_stringify_array_of_arrays_fails() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        // Array containing arrays - first element is list, second is integer
        dict.insert(
            "nested".to_string(),
            make_node(vec![make_node(vec![make_node(1)]), make_node(2)]),
        );

        let result = stringify(&Node::Dictionary(dict), &mut dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_stringify_dictionary_with_none_value() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key1".to_string(), make_node("value"));
        dict.insert("key2".to_string(), Node::None);

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let output = dest.to_string();

        assert!(output.contains("key1 = \"value\""));
        assert!(output.contains("key2 = null"));
    }

    #[test]
    fn test_stringify_nested_with_prefix() {
        let mut dest = BufferDestination::new();
        let mut inner = HashMap::new();
        inner.insert("value".to_string(), make_node(42));

        let mut middle = HashMap::new();
        middle.insert("inner".to_string(), make_node(inner));

        let mut outer = HashMap::new();
        outer.insert("middle".to_string(), make_node(middle));

        stringify(&Node::Dictionary(outer), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[middle.inner]\nvalue = 42\n");
    }

    #[test]
    fn test_stringify_array_table_with_nested_dict() {
        let mut dest = BufferDestination::new();

        let mut nested_dict = HashMap::new();
        nested_dict.insert("nested_key".to_string(), make_node("nested_value"));

        let mut item = HashMap::new();
        item.insert("id".to_string(), make_node(1));
        item.insert("details".to_string(), make_node(nested_dict));

        let mut root = HashMap::new();
        root.insert("items".to_string(), make_node(vec![make_node(item)]));

        stringify(&Node::Dictionary(root), &mut dest).unwrap();
        let output = dest.to_string();

        assert!(output.contains("[[items]]"));
        assert!(output.contains("id = 1"));
        assert!(output.contains("[items.details]"));
        assert!(output.contains("nested_key = \"nested_value\""));
    }

    #[test]
    fn test_stringify_list_of_lists_homogeneous() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        // List of lists (homogeneous - all are lists)
        dict.insert(
            "matrix".to_string(),
            make_node(vec![
                make_node(vec![make_node(1), make_node(2)]),
                make_node(vec![make_node(3), make_node(4)]),
            ]),
        );

        let result = stringify(&Node::Dictionary(dict), &mut dest);
        // This should succeed as all elements are lists
        assert!(result.is_ok());
    }

    #[test]
    fn test_stringify_with_special_characters_in_string() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("text".to_string(), make_node("hello\nworld\ttab"));

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let output = dest.to_string();

        // Should have escaped special characters
        assert!(output.contains("text = \"hello\\u000aworld\\u0009tab\""));
    }

    #[test]
    fn test_stringify_negative_numbers() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("negative".to_string(), make_node(-42));
        dict.insert(
            "neg_list".to_string(),
            make_node(vec![make_node(-1), make_node(-2), make_node(-3)]),
        );

        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let output = dest.to_string();

        assert!(output.contains("negative = -42"));
        assert!(output.contains("neg_list = [-1, -2, -3]"));
    }

    // --- Non-dictionary root errors ---

    #[test]
    fn test_non_dict_root_integer_fails() {
        let mut dest = BufferDestination::new();
        assert!(stringify(&Node::Integer(42), &mut dest).is_err());
    }

    #[test]
    fn test_non_dict_root_list_fails() {
        let mut dest = BufferDestination::new();
        assert!(stringify(&Node::List(vec![make_node(1)]), &mut dest).is_err());
    }

    #[test]
    fn test_non_dict_root_none_fails() {
        let mut dest = BufferDestination::new();
        assert!(stringify(&Node::None, &mut dest).is_err());
    }

    // --- Scalar boundary values ---

    #[test]
    fn test_stringify_zero() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), make_node(0i64));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "n = 0\n");
    }

    #[test]
    fn test_stringify_i64_max() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("max".to_string(), Node::Integer(i64::MAX));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert!(dest.to_string().contains(&i64::MAX.to_string()));
    }

    #[test]
    fn test_stringify_i64_min() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("min".to_string(), Node::Integer(i64::MIN));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert!(dest.to_string().contains(&i64::MIN.to_string()));
    }

    // --- String escaping ---

    #[test]
    fn test_stringify_string_with_quotes() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("msg".to_string(), make_node("say \"hi\""));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert!(dest.to_string().contains("msg = \"say \\\"hi\\\"\""));
    }

    #[test]
    fn test_stringify_string_with_backslash() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("path".to_string(), make_node("C:\\\\Users"));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert!(dest.to_string().contains("path = \""));
    }

    #[test]
    fn test_stringify_empty_string_value() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("empty".to_string(), make_node(""));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "empty = \"\"\n");
    }

    // --- List edge cases ---

    #[test]
    fn test_stringify_empty_list_value() {
        // An empty list has no elements, so type check is skipped
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("lst".to_string(), Node::List(vec![]));
        // Should succeed (no elements to type-check)
        let result = stringify(&Node::Dictionary(dict), &mut dest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stringify_single_integer_list() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("lst".to_string(), make_node(vec![make_node(99i64)]));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "lst = [99]\n");
    }

    #[test]
    fn test_stringify_single_string_list() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("lst".to_string(), make_node(vec![make_node("only")]));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "lst = [\"only\"]\n");
    }

    // --- Multiple scalar keys at root ---

    #[test]
    fn test_stringify_multiple_scalar_keys() {
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("aaa".to_string(), make_node(1i64));
        dict.insert("bbb".to_string(), make_node("hello"));
        dict.insert("ccc".to_string(), make_node(0i64));
        stringify(&Node::Dictionary(dict), &mut dest).unwrap();
        let output = dest.to_string();
        assert!(output.contains("aaa = 1"));
        assert!(output.contains("bbb = \"hello\""));
        assert!(output.contains("ccc = 0"));
    }

    // --- Single-entry array table ---

    #[test]
    fn test_array_table_single_entry() {
        let mut dest = BufferDestination::new();
        let mut item = HashMap::new();
        item.insert("val".to_string(), make_node(7i64));
        let mut root = HashMap::new();
        root.insert("rows".to_string(), make_node(vec![make_node(item)]));
        stringify(&Node::Dictionary(root), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[[rows]]\nval = 7\n");
    }

    // --- Public to_toml API ---

    #[test]
    fn test_to_toml_simple_dict() {
        use crate::to_toml;
        let mut dest = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), make_node("value"));
        to_toml(&Node::Dictionary(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "key = \"value\"\n");
    }

    #[test]
    fn test_to_toml_non_dict_fails() {
        use crate::to_toml;
        let mut dest = BufferDestination::new();
        assert!(to_toml(&Node::Integer(1), &mut dest).is_err());
    }
}
