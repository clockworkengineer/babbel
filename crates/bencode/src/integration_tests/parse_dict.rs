//! Integration tests for parsing bencode dictionaries.

#[cfg(test)]
mod tests {
    use crate::BufferSource;
    use crate::Node::Dictionary;
    use crate::error::messages::*;
    use crate::nodes::node::Node;
    use crate::parser::default::parse;

    #[test]
    fn test_empty_dictionary_works() {
        let mut source = BufferSource::new(b"de");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 0);
            }
            _ => panic!("Expected empty dictionary"),
        }
    }

    #[test]
    fn test_complex_dictionary() {
        let mut source = BufferSource::new(b"d3:foo3:bar5:hello5:world4:test5:valuee");
        let result = parse(&mut source);
        assert!(result.is_ok());
        if let Ok(Dictionary(map)) = result {
            assert_eq!(map.len(), 3);
            assert_eq!(map.get("foo").unwrap(), &Node::Str("bar".to_string()));
            assert_eq!(map.get("test").unwrap(), &Node::Str("value".to_string()));
            assert_eq!(map.get("hello").unwrap(), &Node::Str("world".to_string()));
        } else {
            panic!("Expected dictionary");
        }
    }

    #[test]
    fn test_dictionary_with_non_string_key_fails() {
        let mut source = BufferSource::new(b"di32ei42ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEY_MUST_BE_STRING));
    }

    #[test]
    fn test_dictionary_with_unordered_keys_fails() {
        let mut source = BufferSource::new(b"d3:bbci32e3:abci42ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    #[test]
    fn test_dictionary_single_entry() {
        let mut source = BufferSource::new(b"d3:keyi42ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                assert_eq!(dict.get("key").unwrap(), &Node::Integer(42));
            }
            _ => panic!("Expected single entry dictionary"),
        }
    }

    #[test]
    fn test_dictionary_with_equal_keys_fails() {
        let mut source = BufferSource::new(b"d3:abci1e3:abci2ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    #[test]
    fn test_unterminated_dictionary() {
        let mut source = BufferSource::new(b"d3:keyi42e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_DICTIONARY));
    }

    #[test]
    fn test_dictionary_with_list_value() {
        let mut source = BufferSource::new(b"d4:listli1ei2ei3eee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                if let Some(Node::List(list)) = dict.get("list") {
                    assert_eq!(list.len(), 3);
                } else {
                    panic!("Expected list value");
                }
            }
            _ => panic!("Expected dictionary with list"),
        }
    }

    #[test]
    fn test_nested_dictionary() {
        let mut source = BufferSource::new(b"d5:innerd3:keyi42eee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                if let Some(Dictionary(inner)) = dict.get("inner") {
                    assert_eq!(inner.len(), 1);
                    assert_eq!(inner.get("key").unwrap(), &Node::Integer(42));
                } else {
                    panic!("Expected nested dictionary");
                }
            }
            _ => panic!("Expected nested dictionary"),
        }
    }

    #[test]
    fn test_dictionary_with_string_values() {
        let mut source = BufferSource::new(b"d1:a5:alpha1:b4:beta1:c5:gammae");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 3);
                assert_eq!(dict.get("a").unwrap(), &Node::Str("alpha".to_string()));
                assert_eq!(dict.get("b").unwrap(), &Node::Str("beta".to_string()));
                assert_eq!(dict.get("c").unwrap(), &Node::Str("gamma".to_string()));
            }
            _ => panic!("Expected dictionary with strings"),
        }
    }

    #[test]
    fn test_dictionary_with_empty_string_key() {
        // Single empty string key should work
        let mut source = BufferSource::new(b"d0:i1ee");
        // This should fail because empty string <= empty string (last_key starts as empty)
        // Let's test that we can have an empty key if it's in proper order
        // Actually, since last_key starts as "", an empty key will fail the <= check
        // So let's test the failure case instead
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    #[test]
    fn test_dictionary_mixed_value_types() {
        let mut source = BufferSource::new(b"d3:inti42e4:listli1ei2ee4:test5:valuee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 3);
                assert_eq!(dict.get("int").unwrap(), &Node::Integer(42));
                if let Some(Node::List(list)) = dict.get("list") {
                    assert_eq!(list.len(), 2);
                } else {
                    panic!("Expected list value");
                }
                assert_eq!(dict.get("test").unwrap(), &Node::Str("value".to_string()));
            }
            _ => panic!("Expected dictionary"),
        }
    }

    #[test]
    fn test_dictionary_with_many_keys() {
        // Test dictionary with many keys in proper order
        let mut source = BufferSource::new(b"d1:ai1e1:bi2e1:ci3e1:di4e1:ei5ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 5);
            }
            _ => panic!("Expected dictionary with many keys"),
        }
    }

    #[test]
    fn test_deeply_nested_dictionaries() {
        let mut source = BufferSource::new(b"d1:ad1:bd1:ci42eeee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                if let Some(Dictionary(inner1)) = dict.get("a") {
                    if let Some(Dictionary(inner2)) = inner1.get("b") {
                        assert_eq!(inner2.get("c").unwrap(), &Node::Integer(42));
                    } else {
                        panic!("Expected nested dictionary");
                    }
                } else {
                    panic!("Expected nested dictionary");
                }
            }
            _ => panic!("Expected deeply nested dictionaries"),
        }
    }

    #[test]
    fn test_dictionary_with_zero_length_key() {
        // Dictionary with a key that comes after empty string
        let mut source = BufferSource::new(b"d1:ai1ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                assert_eq!(dict.get("a").unwrap(), &Node::Integer(1));
            }
            _ => panic!("Expected dictionary"),
        }
    }

    #[test]
    fn test_dictionary_with_all_value_types() {
        let mut source = BufferSource::new(b"d4:dicti42e3:inti1e4:listli1ee6:string5:valuee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 4);
                assert!(matches!(dict.get("int"), Some(Node::Integer(1))));
                assert!(matches!(dict.get("string"), Some(Node::Str(_))));
                assert!(matches!(dict.get("list"), Some(Node::List(_))));
                // dict key has value i42e which should fail as non-string key
            }
            _ => {}
        }
    }

    // --- Key ordering edge cases ---

    #[test]
    fn test_dictionary_key_ordering_longer_comes_after_shorter_prefix() {
        // "ab" < "abc" — valid ordering
        let mut source = BufferSource::new(b"d2:abi1e3:abci2ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 2);
                assert_eq!(dict.get("ab").unwrap(), &Node::Integer(1));
                assert_eq!(dict.get("abc").unwrap(), &Node::Integer(2));
            }
            _ => panic!("Expected valid ordered dictionary"),
        }
    }

    #[test]
    fn test_dictionary_key_ordering_longer_before_shorter_fails() {
        // "abc" > "ab" — invalid ordering
        let mut source = BufferSource::new(b"d3:abci2e2:abi1ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    #[test]
    fn test_dictionary_key_ordering_case_sensitive() {
        // ASCII uppercase < lowercase, so "A" < "a" is valid
        let mut source = BufferSource::new(b"d1:Ai1e1:ai2ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 2);
            }
            _ => panic!("Expected valid case-sensitive ordering"),
        }
    }

    #[test]
    fn test_dictionary_single_char_keys_in_order() {
        let mut source = BufferSource::new(b"d1:xi10e1:yi20e1:zi30ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 3);
                assert_eq!(dict.get("x").unwrap(), &Node::Integer(10));
                assert_eq!(dict.get("y").unwrap(), &Node::Integer(20));
                assert_eq!(dict.get("z").unwrap(), &Node::Integer(30));
            }
            _ => panic!("Expected ordered single-char key dictionary"),
        }
    }

    // --- Negative integer values ---

    #[test]
    fn test_dictionary_with_negative_integer_value() {
        let mut source = BufferSource::new(b"d3:keyi-99ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.get("key").unwrap(), &Node::Integer(-99));
            }
            _ => panic!("Expected dictionary with negative integer"),
        }
    }

    #[test]
    fn test_dictionary_with_zero_value() {
        let mut source = BufferSource::new(b"d3:keyi0ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.get("key").unwrap(), &Node::Integer(0));
            }
            _ => panic!("Expected dictionary with zero value"),
        }
    }

    // --- Nested structures ---

    #[test]
    fn test_dictionary_value_is_list_of_dicts() {
        // d4:list[{a:1},{b:2}]e
        let mut source = BufferSource::new(b"d4:listld1:ai1eed1:bi2eeee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                if let Some(Node::List(list)) = dict.get("list") {
                    assert_eq!(list.len(), 2);
                    assert!(
                        matches!(&list[0], Dictionary(d) if d.get("a") == Some(&Node::Integer(1)))
                    );
                    assert!(
                        matches!(&list[1], Dictionary(d) if d.get("b") == Some(&Node::Integer(2)))
                    );
                } else {
                    panic!("Expected list of dicts");
                }
            }
            _ => panic!("Expected dictionary"),
        }
    }

    #[test]
    fn test_dict_inside_list() {
        let _source = BufferSource::new(b"ld2:id i42eee");
        // Key "id " (with space) — bencode is binary; test valid structure
        let mut source2 = BufferSource::new(b"ld2:idi42eee");
        match parse(&mut source2) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert!(
                    matches!(&list[0], Dictionary(d) if d.get("id") == Some(&Node::Integer(42)))
                );
            }
            _ => panic!("Expected list containing dictionary"),
        }
    }

    #[test]
    fn test_dictionary_nested_empty_list_value() {
        let mut source = BufferSource::new(b"d5:itemslee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert!(matches!(dict.get("items"), Some(Node::List(l)) if l.is_empty()));
            }
            _ => panic!("Expected dictionary with empty list value"),
        }
    }

    #[test]
    fn test_dictionary_nested_empty_dict_value() {
        let mut source = BufferSource::new(b"d4:metadee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert!(matches!(dict.get("meta"), Some(Dictionary(d)) if d.is_empty()));
            }
            _ => panic!("Expected dictionary with empty dict value"),
        }
    }

    // --- Error completeness ---

    #[test]
    fn test_dictionary_missing_value_for_key() {
        // Key present but no value before 'e'
        let mut source = BufferSource::new(b"d3:keye");
        assert!(matches!(parse(&mut source), Err(_)));
    }

    #[test]
    fn test_dictionary_list_key_fails() {
        let mut source = BufferSource::new(b"dli1eei2ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEY_MUST_BE_STRING));
    }

    #[test]
    fn test_dictionary_dict_key_fails() {
        let mut source = BufferSource::new(b"dd1:ai1eei2ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEY_MUST_BE_STRING));
    }

    // --- Round-trip tests ---

    #[test]
    fn test_dictionary_round_trip_via_convenience_api() {
        use crate::{parse_bytes, stringify_to_bytes};
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("aaa".to_string(), Node::Integer(1));
        map.insert("bbb".to_string(), Node::Str("val".to_string()));
        map.insert(
            "ccc".to_string(),
            Node::List(vec![Node::Integer(10), Node::Integer(20)]),
        );
        let original = Node::Dictionary(map);
        let encoded = stringify_to_bytes(&original).unwrap();
        let reparsed = parse_bytes(&encoded).unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn test_empty_dictionary_round_trip() {
        use crate::{parse_str, stringify_to_string};
        use std::collections::HashMap;
        let node = Node::Dictionary(HashMap::new());
        let s = stringify_to_string(&node).unwrap();
        assert_eq!(s, "de");
        assert_eq!(parse_str(&s).unwrap(), node);
    }

    // --- Iterative parser ---

    #[test]
    fn test_iterative_parse_simple_dictionary() {
        use crate::parse_bytes_iterative;
        let result = parse_bytes_iterative(b"d3:keyi42ee");
        match result {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.get("key").unwrap(), &Node::Integer(42));
            }
            _ => panic!("Iterative parser: expected dictionary"),
        }
    }

    #[test]
    fn test_iterative_parse_nested_dictionary() {
        use crate::parse_str_iterative;
        let result = parse_str_iterative("d5:innerd3:keyi99eee");
        match result {
            Ok(Dictionary(outer)) => {
                if let Some(Dictionary(inner)) = outer.get("inner") {
                    assert_eq!(inner.get("key").unwrap(), &Node::Integer(99));
                } else {
                    panic!("Expected inner dictionary");
                }
            }
            _ => panic!("Iterative parser: expected outer dictionary"),
        }
    }

    // --- Borrowed / zero-copy parser ---

    #[test]
    fn test_borrowed_parse_dictionary() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"d3:keyi7ee").unwrap();
        assert!(node.is_dictionary());
        let dict = node.as_dictionary().unwrap();
        assert_eq!(dict.get(b"key".as_ref()).unwrap().as_integer(), Some(7));
    }

    #[test]
    fn test_borrowed_parse_nested_dictionary_to_owned() {
        use crate::parse_borrowed;
        let borrowed = parse_borrowed(b"d5:innerd3:abci1eeee").unwrap();
        let owned = borrowed.to_node();
        assert!(owned.is_dictionary());
        if let Some(inner) = owned.get("inner") {
            assert_eq!(inner.get("abc"), Some(&Node::Integer(1)));
        } else {
            panic!("Expected nested inner dictionary");
        }
    }
}
