//! YAML Node Definitions
//!
//! Re-exports node types, accessors, conversions, and operations.
//!
//! Copyright (c) 2026 YAML Library Developers

#[allow(unused_imports)]
pub use super::access::*;
#[allow(unused_imports)]
pub use super::convert::*;
#[allow(unused_imports)]
pub use super::ops::*;
#[allow(unused_imports)]
pub use super::types::*;

#[allow(unused_imports)]
pub use super::search::NodeChildIterator;

#[cfg(feature = "alloc")]
pub use crate::nodes::builders::{ArrayBuilder, MappingBuilder, SetBuilder};

#[cfg(test)]
use crate::nodes::util::{make_node, make_set};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_conversions() {
        assert_eq!(Numeric::from(42i64), Numeric::Integer(42));
        assert_eq!(Numeric::from(3.14f64), Numeric::Float(3.14));
        assert_eq!(Numeric::from(42u64), Numeric::UInteger(42));
        assert_eq!(Numeric::from(42u8), Numeric::Byte(42));
        assert_eq!(Numeric::from(42i32), Numeric::Int32(42));
        assert_eq!(Numeric::from(42u32), Numeric::UInt32(42));
        assert_eq!(Numeric::from(42i16), Numeric::Int16(42));
        assert_eq!(Numeric::from(42u16), Numeric::UInt16(42));
        assert_eq!(Numeric::from(42i8), Numeric::Int8(42));
    }

    #[test]
    fn test_node_numeric_conversions() {
        assert_eq!(Node::from(42i64), Node::Number(Numeric::Integer(42)));
        assert_eq!(Node::from(3.14f64), Node::Number(Numeric::Float(3.14)));
        assert_eq!(Node::from(42u64), Node::Number(Numeric::UInteger(42)));
        assert_eq!(Node::from(42u8), Node::Number(Numeric::Byte(42)));
        assert_eq!(Node::from(42i32), Node::Number(Numeric::Int32(42)));
        assert_eq!(Node::from(42u32), Node::Number(Numeric::UInt32(42)));
        assert_eq!(Node::from(42i16), Node::Number(Numeric::Int16(42)));
        assert_eq!(Node::from(42u16), Node::Number(Numeric::UInt16(42)));
        assert_eq!(Node::from(42i8), Node::Number(Numeric::Int8(42)));
    }

    #[test]
    fn test_node_string_conversions() {
        assert_eq!(
            Node::from("test"),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
        assert_eq!(
            Node::from("test".to_string()),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
    }

    #[test]
    fn test_node_bool_conversion() {
        assert_eq!(Node::from(true), Node::Boolean(true));
        assert_eq!(Node::from(false), Node::Boolean(false));
    }

    #[test]
    fn test_node_vec_conversion() {
        let vec = vec![1, 2, 3];
        let node = Node::from(vec);
        match node {
            Node::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(arr[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(arr[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Array node"),
        }
    }

    #[test]
    fn test_array_indexing() {
        let arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(arr[0], Node::Number(Numeric::Int32(1)));
        assert_eq!(arr[1], Node::Number(Numeric::Int32(2)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array/set node with integer")]
    fn test_invalid_array_indexing() {
        let node = Node::Boolean(true);
        let _value = &node[0];
    }

    #[test]
    fn test_mapping_indexing() {
        let obj = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(42),
        )]);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-mapping node with string")]
    fn test_invalid_mapping_indexing() {
        let node = Node::Boolean(true);
        let _value = &node["key"];
    }

    #[test]
    fn test_array_mut_indexing() {
        let mut arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        arr[0] = Node::from(42);
        assert_eq!(arr[0], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array/set node with integer")]
    fn test_invalid_array_mut_indexing() {
        let mut node = Node::Boolean(true);
        node[0] = Node::from(42);
    }

    #[test]
    fn test_mapping_mut_indexing() {
        let mut obj = Node::Mapping(vec![(
            Node::Str("key".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::from(42),
        )]);
        obj["key"] = Node::from(100);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(100)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-mapping node with string")]
    fn test_invalid_mapping_mut_indexing() {
        let mut node = Node::Boolean(true);
        node["key"] = Node::from(42);
    }

    #[test]
    #[should_panic(expected = "No such key exists")]
    fn test_mapping_mut_indexing_nonexistent_key() {
        let mut obj = Node::Mapping(Vec::new());
        obj["nonexistent"] = Node::from(42);
    }

    #[test]
    fn test_make_node() {
        assert_eq!(make_node(42), Node::Number(Numeric::Int32(42)));
        assert_eq!(
            make_node("test"),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
        );
        assert_eq!(make_node(true), Node::Boolean(true));
    }
    #[test]
    fn test_make_node_vec() {
        let vec = vec![1, 2, 3];
        assert_eq!(
            make_node(vec),
            Node::Array(vec![
                Node::Number(Numeric::Int32(1)),
                Node::Number(Numeric::Int32(2)),
                Node::Number(Numeric::Int32(3))
            ])
        );
    }

    #[test]
    fn test_document_node() {
        let doc = Node::Documents(vec![Node::from(1), Node::from("test")]);
        match doc {
            Node::Documents(nodes) => {
                assert_eq!(nodes.len(), 2);
                assert_eq!(nodes[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(
                    nodes[1],
                    Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None)
                );
            }
            _ => panic!("Expected Document node"),
        }
    }

    #[test]
    fn test_comment_node() {
        let comment = Node::Comment("Test comment".to_string());
        match comment {
            Node::Comment(text) => assert_eq!(text, "Test comment"),
            _ => panic!("Expected Comment node"),
        }
    }

    #[test]
    fn test_none_node() {
        assert_eq!(Node::None, Node::None);
        let none = make_node(Node::None);
        assert_eq!(none, Node::None);
    }

    #[test]
    #[should_panic(expected = "No such key exists")]
    fn test_mapping_indexing_nonexistent_key() {
        let obj = Node::Mapping(Vec::new());
        let _ = &obj["nonexistent"];
    }

    #[test]
    fn test_node_from_vec_of_strings() {
        let vec = vec!["a", "b"];
        let node = Node::from(vec);
        assert_eq!(
            node,
            Node::Array(vec![
                Node::Str("a".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::Str("b".to_string(), QuoteType::Unquoted, BlockStyle::None),
            ])
        );
    }

    #[test]
    fn test_nested_mapping_indexing_and_mutation() {
        let mut obj = Node::Mapping(vec![(
            Node::Str("outer".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Mapping(vec![(
                Node::Str("inner".to_string(), QuoteType::Unquoted, BlockStyle::None),
                Node::from(5),
            )]),
        )]);

        assert_eq!(obj["outer"]["inner"], Node::from(5));

        obj["outer"]["inner"] = Node::from(10);
        assert_eq!(obj["outer"]["inner"], Node::from(10));
    }

    #[test]
    fn test_set_node_creation() {
        let set = Node::Set(vec![Node::from(1), Node::from(2), Node::from(3)]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(items[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(items[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_set_indexing() {
        let set = Node::Set(vec![Node::from(1), Node::from(2)]);
        assert_eq!(set[0], Node::Number(Numeric::Int32(1)));
        assert_eq!(set[1], Node::Number(Numeric::Int32(2)));
    }

    #[test]
    fn test_set_mut_indexing() {
        let mut set = Node::Set(vec![Node::from(1), Node::from(2)]);
        set[0] = Node::from(42);
        assert_eq!(set[0], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array/set node with integer")]
    fn test_invalid_set_indexing() {
        let node = Node::Boolean(true);
        let _value = &node[0];
    }

    #[test]
    fn test_make_set_function() {
        let set = make_set(vec![1, 2, 3]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(items[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(items[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_make_set_with_duplicates() {
        let set = make_set(vec![1, 2, 2, 3, 1]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 3); // Duplicates should be removed
                assert_eq!(items[0], Node::Number(Numeric::Int32(1)));
                assert_eq!(items[1], Node::Number(Numeric::Int32(2)));
                assert_eq!(items[2], Node::Number(Numeric::Int32(3)));
            }
            _ => panic!("Expected Set node"),
        }
    }

    #[test]
    fn test_make_set_with_strings() {
        let set = make_set(vec!["apple", "banana", "apple"]);
        match set {
            Node::Set(items) => {
                assert_eq!(items.len(), 2); // Duplicate "apple" removed
                assert_eq!(
                    items[0],
                    Node::Str("apple".to_string(), QuoteType::Unquoted, BlockStyle::None)
                );
                assert_eq!(
                    items[1],
                    Node::Str("banana".to_string(), QuoteType::Unquoted, BlockStyle::None)
                );
            }
            _ => panic!("Expected Set node"),
        }
    }

    // Embedded feature tests
    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_to_i32() {
        let num_i32 = Numeric::Int32(42);
        assert_eq!(num_i32.to_i32(), Some(42));

        let num_i64 = Numeric::Integer(1000);
        assert_eq!(num_i64.to_i32(), Some(1000));

        let num_large = Numeric::Integer(i64::MAX);
        assert_eq!(num_large.to_i32(), None);

        let num_float = Numeric::Float(42.7);
        assert_eq!(num_float.to_i32(), Some(42));

        let num_byte = Numeric::Byte(255);
        assert_eq!(num_byte.to_i32(), Some(255));
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_to_f32() {
        let num_i32 = Numeric::Int32(42);
        assert_eq!(num_i32.to_f32(), 42.0f32);

        let num_float = Numeric::Float(3.14159);
        assert!((num_float.to_f32() - 3.14159f32).abs() < 0.0001);

        let num_i64 = Numeric::Integer(1000);
        assert_eq!(num_i64.to_f32(), 1000.0f32);
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_fits_in_i32() {
        assert!(Numeric::Int32(42).fits_in_i32());
        assert!(Numeric::Integer(1000).fits_in_i32());
        assert!(!Numeric::Integer(i64::MAX).fits_in_i32());
        assert!(Numeric::Float(100.5).fits_in_i32());
        assert!(Numeric::Byte(255).fits_in_i32());
        assert!(Numeric::Int16(1000).fits_in_i32());
    }

    #[test]
    #[cfg(feature = "embedded")]
    fn test_numeric_size_bytes() {
        assert_eq!(Numeric::Integer(0).size_bytes(), 8);
        assert_eq!(Numeric::Float(0.0).size_bytes(), 8);
        assert_eq!(Numeric::UInteger(0).size_bytes(), 8);
        assert_eq!(Numeric::Int32(0).size_bytes(), 4);
        assert_eq!(Numeric::UInt32(0).size_bytes(), 4);
        assert_eq!(Numeric::Int16(0).size_bytes(), 2);
        assert_eq!(Numeric::UInt16(0).size_bytes(), 2);
        assert_eq!(Numeric::Byte(0).size_bytes(), 1);
        assert_eq!(Numeric::Int8(0).size_bytes(), 1);
    }

    #[test]
    fn test_node_get_safe() {
        let arr = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
        ]);

        assert_eq!(arr.get(0), Some(&Node::Number(Numeric::Int32(1))));
        assert_eq!(arr.get(1), Some(&Node::Number(Numeric::Int32(2))));
        assert_eq!(arr.get(2), Some(&Node::Number(Numeric::Int32(3))));
        assert_eq!(arr.get(3), None);

        let not_array = Node::Boolean(true);
        assert_eq!(not_array.get(0), None);
    }

    #[test]
    fn test_node_get_key_safe() {
        let mut pairs = alloc::vec::Vec::new();
        pairs.push((
            Node::Str("name".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None),
        ));
        pairs.push((
            Node::Str("age".to_string(), QuoteType::Unquoted, BlockStyle::None),
            Node::Number(Numeric::Int32(42)),
        ));
        let mapping = Node::Mapping(pairs);

        assert_eq!(
            mapping.get_key("name"),
            Some(&Node::Str(
                "test".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None
            ))
        );
        assert_eq!(
            mapping.get_key("age"),
            Some(&Node::Number(Numeric::Int32(42)))
        );
        assert_eq!(mapping.get_key("nonexistent"), None);

        let not_mapping = Node::Boolean(true);
        assert_eq!(not_mapping.get_key("key"), None);
    }

    #[test]
    fn test_node_get_mut_safe() {
        let mut arr = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
        ]);

        if let Some(node) = arr.get_mut(0) {
            *node = Node::Number(Numeric::Int32(99));
        }

        assert_eq!(arr.get(0), Some(&Node::Number(Numeric::Int32(99))));
        assert_eq!(arr.get_mut(10), None);
    }

    #[test]
    fn test_node_is_sequence() {
        let arr = Node::Array(vec![Node::None]);
        let set = Node::Set(vec![Node::None]);
        let mapping = Node::Mapping(vec![]);
        let boolean = Node::Boolean(true);

        assert!(arr.is_sequence());
        assert!(set.is_sequence());
        assert!(!mapping.is_sequence());
        assert!(!boolean.is_sequence());
    }

    #[test]
    fn test_node_is_mapping() {
        let mapping = Node::Mapping(vec![]);
        let arr = Node::Array(vec![]);
        let boolean = Node::Boolean(true);

        assert!(mapping.is_mapping());
        assert!(!arr.is_mapping());
        assert!(!boolean.is_mapping());
    }

    #[test]
    fn test_node_len() {
        let arr = Node::Array(vec![Node::None, Node::None, Node::None]);
        let set = Node::Set(vec![Node::None, Node::None]);
        let mapping = Node::Mapping(vec![]);
        let boolean = Node::Boolean(true);

        assert_eq!(arr.len(), Some(3));
        assert_eq!(set.len(), Some(2));
        assert_eq!(mapping.len(), Some(0));
        assert_eq!(boolean.len(), None);
    }

    #[test]
    fn test_node_is_empty() {
        let arr_empty = Node::Array(vec![]);
        let arr_full = Node::Array(vec![Node::None]);
        let mapping_empty = Node::Mapping(vec![]);
        let boolean = Node::Boolean(true);

        assert!(arr_empty.is_empty());
        assert!(!arr_full.is_empty());
        assert!(mapping_empty.is_empty());
        assert!(!boolean.is_empty());
    }

    #[test]
    fn test_node_as_i32() {
        let num = Node::Number(Numeric::Int32(42));
        let large = Node::Number(Numeric::Integer(i64::MAX));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert_eq!(num.as_i32(), Some(42));
        assert_eq!(large.as_i32(), None);
        assert_eq!(string.as_i32(), None);
    }

    #[test]
    fn test_node_as_f32() {
        let num = Node::Number(Numeric::Float(3.14));
        let int = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!((num.as_f32().unwrap() - 3.14f32).abs() < 0.01);
        assert_eq!(int.as_f32(), Some(42.0f32));
        assert_eq!(string.as_f32(), None);
    }

    #[test]
    fn test_node_as_str() {
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let number = Node::Number(Numeric::Int32(42));

        assert_eq!(string.as_str(), Some("test"));
        assert_eq!(number.as_str(), None);
    }

    #[test]
    fn test_node_as_bool() {
        let bool_true = Node::Boolean(true);
        let bool_false = Node::Boolean(false);
        let number = Node::Number(Numeric::Int32(42));

        assert_eq!(bool_true.as_bool(), Some(true));
        assert_eq!(bool_false.as_bool(), Some(false));
        assert_eq!(number.as_bool(), None);
    }

    #[test]
    fn test_node_is_string() {
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let number = Node::Number(Numeric::Int32(42));
        let boolean = Node::Boolean(true);

        assert!(string.is_string());
        assert!(!number.is_string());
        assert!(!boolean.is_string());
    }

    #[test]
    fn test_node_is_number() {
        let number = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);
        let boolean = Node::Boolean(true);

        assert!(number.is_number());
        assert!(!string.is_number());
        assert!(!boolean.is_number());
    }

    #[test]
    fn test_node_is_boolean() {
        let boolean = Node::Boolean(true);
        let number = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!(boolean.is_boolean());
        assert!(!number.is_boolean());
        assert!(!string.is_boolean());
    }

    #[test]
    fn test_node_is_none() {
        let none = Node::None;
        let number = Node::Number(Numeric::Int32(42));
        let string = Node::Str("test".to_string(), QuoteType::Unquoted, BlockStyle::None);

        assert!(none.is_none());
        assert!(!number.is_none());
        assert!(!string.is_none());
    }

    #[test]
    fn test_node_as_slice() {
        let array = Node::Array(vec![
            Node::Number(Numeric::Int32(1)),
            Node::Number(Numeric::Int32(2)),
            Node::Number(Numeric::Int32(3)),
        ]);
        let set = Node::Set(vec![Node::Number(Numeric::Int32(1))]);
        let mapping = Node::Mapping(vec![]);

        assert_eq!(array.as_slice().map(|s| s.len()), Some(3));
        assert_eq!(set.as_slice().map(|s| s.len()), Some(1));
        assert!(mapping.as_slice().is_none());
    }

    #[test]
    fn test_node_as_mapping() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
        ]);
        let array = Node::Array(vec![]);

        assert_eq!(mapping.as_mapping().map(|m| m.len()), Some(2));
        assert!(array.as_mapping().is_none());
    }

    #[test]
    fn test_node_contains_key() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
        ]);
        let array = Node::Array(vec![]);

        assert!(mapping.contains_key("key1"));
        assert!(mapping.contains_key("key2"));
        assert!(!mapping.contains_key("key3"));
        assert!(!array.contains_key("key1"));
    }

    #[test]
    fn test_node_keys() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from("value1")),
            (Node::from("key2"), Node::from("value2")),
            (Node::from("key3"), Node::from("value3")),
        ]);
        let array = Node::Array(vec![]);

        let keys = mapping.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1"));
        assert!(keys.contains(&"key2"));
        assert!(keys.contains(&"key3"));

        let empty_keys = array.keys();
        assert_eq!(empty_keys.len(), 0);
    }

    #[test]
    fn test_safe_access_prevents_panics() {
        // Test that safe methods don't panic on invalid access
        let array = Node::Array(vec![Node::from(1), Node::from(2)]);

        // Out of bounds access returns None instead of panicking
        assert!(array.get(100).is_none());

        // Wrong type access returns None instead of panicking
        let mapping = Node::Mapping(vec![]);
        assert!(mapping.get(0).is_none());

        // Nonexistent key returns None instead of panicking
        assert!(mapping.get_key("nonexistent").is_none());

        // Type check before access
        let scalar = Node::from(42);
        if scalar.is_sequence() {
            let _ = scalar.get(0); // Won't execute
        }
        assert!(!scalar.is_sequence());
    }

    #[test]
    fn test_safe_mutable_access() {
        let mut array = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        // Modify existing elements
        if let Some(node) = array.get_mut(1) {
            *node = Node::from(20);
        }
        assert_eq!(array.get(1), Some(&Node::Number(Numeric::Int32(20))));

        // Attempt to modify nonexistent element safely
        assert!(array.get_mut(100).is_none());

        // Mapping mutation
        let mut mapping = Node::Mapping(vec![(Node::from("key"), Node::from("value"))]);

        if let Some(node) = mapping.get_key_mut("key") {
            *node = Node::from("new_value");
        }
        assert_eq!(
            mapping.get_key("key"),
            Some(&Node::Str(
                "new_value".to_string(),
                QuoteType::Unquoted,
                BlockStyle::None
            ))
        );
    }

    // ==================== Tree Traversal Tests ====================

    #[test]
    fn test_children_iterator_array() {
        let array = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
        let children: Vec<_> = array.children().collect();
        assert_eq!(children.len(), 3);
        assert_eq!(children[0], &Node::Number(Numeric::Int32(1)));
        assert_eq!(children[1], &Node::Number(Numeric::Int32(2)));
        assert_eq!(children[2], &Node::Number(Numeric::Int32(3)));
    }

    #[test]
    fn test_children_iterator_mapping() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from(1)),
            (Node::from("key2"), Node::from(2)),
        ]);
        let children: Vec<_> = mapping.children().collect();
        // Should return all keys first, then all values
        assert_eq!(children.len(), 4); // 2 keys + 2 values
    }

    #[test]
    fn test_children_iterator_leaf() {
        let leaf = Node::from(42);
        let children: Vec<_> = leaf.children().collect();
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_visit_simple_tree() {
        let doc = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        let mut visited = Vec::new();
        doc.visit(|node, depth| {
            visited.push((node.clone(), depth));
            true
        });

        assert_eq!(visited.len(), 4); // root + 3 children
        assert_eq!(visited[0].1, 0); // root at depth 0
        assert_eq!(visited[1].1, 1); // children at depth 1
    }

    #[test]
    fn test_visit_nested_tree() {
        let doc = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
        ]);

        let mut count = 0;
        let mut max_depth = 0;
        doc.visit(|_, depth| {
            count += 1;
            if depth > max_depth {
                max_depth = depth;
            }
            true
        });

        assert_eq!(count, 5); // root + 1 + nested array + 2 + 3
        assert_eq!(max_depth, 2);
    }

    #[test]
    fn test_visit_early_termination() {
        let doc = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        let mut count = 0;
        doc.visit(|_, _| {
            count += 1;
            count < 3 // stop after visiting 2 nodes
        });

        assert_eq!(count, 3); // visited 3 nodes before stopping
    }

    #[test]
    fn test_visit_mut_modification() {
        let mut doc = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

        doc.visit_mut(|node, _| {
            if let Node::Number(Numeric::Int32(n)) = node {
                *n *= 2; // double all numbers
            }
            true
        });

        assert_eq!(doc[0], Node::Number(Numeric::Int32(2)));
        assert_eq!(doc[1], Node::Number(Numeric::Int32(4)));
        assert_eq!(doc[2], Node::Number(Numeric::Int32(6)));
    }

    #[test]
    fn test_count_nodes() {
        let leaf = Node::from(42);
        assert_eq!(leaf.count_nodes(), 1);

        let array = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(array.count_nodes(), 3); // array + 2 numbers

        let nested = Node::Array(vec![
            Node::from(1),
            Node::Array(vec![Node::from(2), Node::from(3)]),
        ]);
        assert_eq!(nested.count_nodes(), 5); // outer array + 1 + inner array + 2 + 3
    }

    #[test]
    fn test_max_depth() {
        let leaf = Node::from(42);
        assert_eq!(leaf.max_depth(), 0);

        let array = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(array.max_depth(), 1);

        let nested = Node::Array(vec![Node::Array(vec![Node::Array(vec![Node::from(1)])])]);
        assert_eq!(nested.max_depth(), 3);
    }

    #[test]
    fn test_find_all() {
        let doc = Node::Array(vec![
            Node::from(1),
            Node::from("text"),
            Node::from(2),
            Node::Array(vec![Node::from(3)]),
        ]);

        let numbers = doc.find_all(|node| node.is_number());
        assert_eq!(numbers.len(), 3); // 1, 2, 3

        let strings = doc.find_all(|node| node.is_str());
        assert_eq!(strings.len(), 1); // "text"

        let arrays = doc.find_all(|node| node.is_array());
        assert_eq!(arrays.len(), 2); // outer and inner array
    }

    #[test]
    fn test_find_first() {
        let doc = Node::Array(vec![
            Node::from("first"),
            Node::from(1),
            Node::from("second"),
        ]);

        let first_str = doc.find_first(|node| node.is_str());
        assert!(first_str.is_some());
        if let Some(Node::Str(s, _, _)) = first_str {
            assert_eq!(s, "first"); // should find "first", not "second"
        } else {
            panic!("Expected string node");
        }

        let first_bool = doc.find_first(|node| node.is_boolean());
        assert!(first_bool.is_none());
    }

    #[test]
    fn test_find_all_with_mapping() {
        let doc = Node::Mapping(vec![
            (Node::from("key1"), Node::from(1)),
            (Node::from("key2"), Node::from("value")),
        ]);

        let numbers = doc.find_all(|node| node.is_number());
        assert_eq!(numbers.len(), 1); // just the number value

        let strings = doc.find_all(|node| node.is_str());
        assert_eq!(strings.len(), 3); // 2 keys + 1 value
    }

    #[test]
    fn test_children_with_anchored() {
        let anchored = Node::Anchored(alloc::boxed::Box::new(Node::from(42)), "anchor".to_string());
        let children: Vec<_> = anchored.children().collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], &Node::Number(Numeric::Int32(42)));
    }

    #[test]
    fn test_visit_with_tagged() {
        let tagged = Node::Tagged(
            alloc::boxed::Box::new(Node::Array(vec![Node::from(1), Node::from(2)])),
            "!custom".to_string(),
        );

        let mut count = 0;
        tagged.visit(|_, _| {
            count += 1;
            true
        });

        assert_eq!(count, 4); // tagged + array + 2 numbers
    }

    #[test]
    fn test_find_deeply_nested() {
        let doc = Node::Mapping(vec![(
            Node::from("level1"),
            Node::Mapping(vec![(
                Node::from("level2"),
                Node::Array(vec![
                    Node::from(1),
                    Node::from(42), // target
                    Node::from(3),
                ]),
            )]),
        )]);

        let target = doc.find_first(|node| {
            if let Node::Number(Numeric::Int32(n)) = node {
                *n == 42
            } else {
                false
            }
        });

        assert!(target.is_some());
    }

    #[test]
    fn test_visit_mapping_order() {
        let mapping = Node::Mapping(vec![
            (Node::from("key1"), Node::from(1)),
            (Node::from("key2"), Node::from(2)),
        ]);

        let mut visited_order = Vec::new();
        mapping.visit(|node, _| {
            if let Node::Number(Numeric::Int32(n)) = node {
                visited_order.push(*n);
            }
            true
        });

        // Numbers should be visited in mapping value order
        assert_eq!(visited_order, vec![1, 2]);
    }

    // ==================== Fluent API Builder Tests ====================

    #[test]
    fn test_array_builder_basic() {
        let array = Node::array().push(1).push(2).push(3).build();

        assert!(array.is_array());
        assert_eq!(array.len(), Some(3));
        assert_eq!(array[0], Node::from(1));
        assert_eq!(array[1], Node::from(2));
        assert_eq!(array[2], Node::from(3));
    }

    #[test]
    fn test_array_builder_mixed_types() {
        let array = Node::array()
            .push(42)
            .push("text")
            .push(true)
            .push(3.14)
            .build();

        assert_eq!(array.len(), Some(4));
        assert!(array[0].is_number());
        assert!(array[1].is_string());
        assert!(array[2].is_boolean());
        assert!(array[3].is_number());
    }

    #[test]
    fn test_array_builder_extend() {
        let array = Node::array().push(1).extend(vec![2, 3, 4]).push(5).build();

        assert_eq!(array.len(), Some(5));
        assert_eq!(array[0], Node::from(1));
        assert_eq!(array[4], Node::from(5));
    }

    #[test]
    fn test_array_builder_conditional() {
        let include_optional = true;
        let array = Node::array()
            .push(1)
            .push_if(include_optional, 2)
            .push_if(false, 999) // should not be added
            .push(3)
            .build();

        assert_eq!(array.len(), Some(3));
        assert_eq!(array[1], Node::from(2));
        assert_eq!(array[2], Node::from(3));
    }

    #[test]
    fn test_array_builder_optional() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<i32> = None;

        let array = Node::array()
            .push(1)
            .push_opt(some_value)
            .push_opt(none_value)
            .push(2)
            .build();

        assert_eq!(array.len(), Some(3));
        assert_eq!(array[1], Node::from(42));
        assert_eq!(array[2], Node::from(2));
    }

    #[test]
    fn test_array_builder_nested() {
        let nested = Node::array()
            .push(1)
            .push(Node::array().push(2).push(3).build())
            .push(4)
            .build();

        assert_eq!(nested.len(), Some(3));
        assert!(nested[1].is_array());
        assert_eq!(nested[1].len(), Some(2));
    }

    #[test]
    fn test_array_builder_empty() {
        let array = Node::array().build();
        assert!(array.is_empty());
        assert_eq!(array.len(), Some(0));
    }

    #[test]
    fn test_mapping_builder_basic() {
        let mapping = Node::mapping()
            .insert("name", "Alice")
            .insert("age", 30)
            .insert("active", true)
            .build();

        assert!(mapping.is_mapping());
        assert_eq!(mapping.len(), Some(3));
        assert_eq!(mapping["name"], Node::from("Alice"));
        assert_eq!(mapping["age"], Node::from(30));
        assert_eq!(mapping["active"], Node::from(true));
    }

    #[test]
    fn test_mapping_builder_nested() {
        let config = Node::mapping()
            .insert(
                "database",
                Node::mapping()
                    .insert("host", "localhost")
                    .insert("port", 5432)
                    .build(),
            )
            .insert("debug", false)
            .build();

        assert_eq!(config.len(), Some(2));
        assert!(config["database"].is_mapping());
        assert_eq!(config["database"]["host"], Node::from("localhost"));
    }

    #[test]
    fn test_mapping_builder_conditional() {
        let include_debug = true;
        let mapping = Node::mapping()
            .insert("name", "app")
            .insert_if(include_debug, "debug", true)
            .insert_if(false, "should_not_exist", 999)
            .build();

        assert_eq!(mapping.len(), Some(2));
        assert!(mapping.contains_key("debug"));
        assert!(!mapping.contains_key("should_not_exist"));
    }

    #[test]
    fn test_mapping_builder_optional() {
        let some_value: Option<i32> = Some(42);
        let none_value: Option<&str> = None;

        let mapping = Node::mapping()
            .insert("required", "value")
            .insert_opt("optional1", some_value)
            .insert_opt("optional2", none_value)
            .build();

        assert_eq!(mapping.len(), Some(2));
        assert!(mapping.contains_key("optional1"));
        assert!(!mapping.contains_key("optional2"));
    }

    #[test]
    fn test_mapping_builder_upsert() {
        let mapping = Node::mapping()
            .insert("key", "original")
            .insert("other", "value")
            .upsert("key", "updated") // should replace
            .upsert("new", "added") // should insert
            .build();

        assert_eq!(mapping.len(), Some(3));
        assert_eq!(mapping["key"], Node::from("updated"));
        assert_eq!(mapping["new"], Node::from("added"));
    }

    #[test]
    fn test_mapping_builder_complex() {
        let config = Node::mapping()
            .insert("name", "MyApp")
            .insert("version", "1.0.0")
            .insert(
                "servers",
                Node::array().push("web1").push("web2").push("web3").build(),
            )
            .insert(
                "database",
                Node::mapping()
                    .insert("host", "localhost")
                    .insert("port", 5432)
                    .insert("ssl", true)
                    .build(),
            )
            .build();

        assert_eq!(config.len(), Some(4));
        assert!(config["servers"].is_array());
        assert_eq!(config["servers"].len(), Some(3));
        assert!(config["database"].is_mapping());
        assert_eq!(config["database"]["port"], Node::from(5432));
    }

    #[test]
    fn test_set_builder_basic() {
        let set = Node::set().insert(1).insert(2).insert(3).build();

        assert!(set.is_set());
        assert_eq!(set.len(), Some(3));
    }

    #[test]
    fn test_set_builder_duplicates() {
        let set = Node::set()
            .insert(1)
            .insert(2)
            .insert(1) // duplicate
            .insert(3)
            .insert(2) // duplicate
            .build();

        assert_eq!(set.len(), Some(3)); // should only have 3 unique items
    }

    #[test]
    fn test_set_builder_extend() {
        let set = Node::set()
            .insert(1)
            .extend(vec![2, 3, 2, 4]) // includes duplicate 2
            .insert(5)
            .build();

        assert_eq!(set.len(), Some(5)); // 1, 2, 3, 4, 5
    }

    #[test]
    fn test_set_builder_mixed_types() {
        let set = Node::set().insert(1).insert("text").insert(true).build();

        assert_eq!(set.len(), Some(3));
    }

    #[test]
    fn test_builder_chaining_realistic_config() {
        // Realistic configuration example
        let config = Node::mapping()
            .insert(
                "application",
                Node::mapping()
                    .insert("name", "WebAPI")
                    .insert("version", "2.1.0")
                    .insert("environment", "production")
                    .build(),
            )
            .insert(
                "server",
                Node::mapping()
                    .insert("host", "0.0.0.0")
                    .insert("port", 8080)
                    .insert("workers", 4)
                    .build(),
            )
            .insert(
                "features",
                Node::array()
                    .push("auth")
                    .push("logging")
                    .push("metrics")
                    .build(),
            )
            .insert(
                "allowed_origins",
                Node::set()
                    .insert("https://example.com")
                    .insert("https://api.example.com")
                    .build(),
            )
            .build();

        // Verify structure
        assert_eq!(config.len(), Some(4));
        assert_eq!(config["application"]["name"], Node::from("WebAPI"));
        assert_eq!(config["server"]["port"], Node::from(8080));
        assert_eq!(config["features"].len(), Some(3));
        assert!(config["allowed_origins"].is_set());
    }

    #[test]
    fn test_builder_readability_comparison() {
        // Old way (verbose)
        let _old_way = Node::Mapping(vec![
            (
                Node::from("database"),
                Node::Mapping(vec![
                    (Node::from("host"), Node::from("localhost")),
                    (Node::from("port"), Node::from(5432)),
                ]),
            ),
            (
                Node::from("servers"),
                Node::Array(vec![Node::from("web1"), Node::from("web2")]),
            ),
        ]);

        // New way (fluent)
        let _new_way = Node::mapping()
            .insert(
                "database",
                Node::mapping()
                    .insert("host", "localhost")
                    .insert("port", 5432)
                    .build(),
            )
            .insert("servers", Node::array().push("web1").push("web2").build())
            .build();

        // Both should produce equivalent structures
        assert_eq!(_old_way, _new_way);
    }

    #[test]
    fn test_array_builder_len() {
        let builder = Node::array().push(1).push(2);

        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());

        let empty = Node::array();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_mapping_builder_contains_key() {
        let builder = Node::mapping()
            .insert("key1", "value1")
            .insert("key2", "value2");

        assert!(builder.contains_key("key1"));
        assert!(builder.contains_key("key2"));
        assert!(!builder.contains_key("key3"));
    }

    #[test]
    fn test_set_builder_contains() {
        let node1 = Node::from(1);
        let node2 = Node::from(2);
        let node3 = Node::from(3);

        let builder = Node::set().insert(1).insert(2);

        assert!(builder.contains(&node1));
        assert!(builder.contains(&node2));
        assert!(!builder.contains(&node3));
    }
    // --- New and relevant unit tests ---
    #[test]
    fn test_node_string_convert_trait() {
        let s = Node::from("hello");
        assert_eq!(s.to_string_lossy(), "hello");
        assert_eq!(s.as_str(), Some("hello"));
        assert_eq!(s.clone_as_string(), Some(Node::from("hello")));

        let n = Node::from(42);
        assert_eq!(n.to_string_lossy(), "42");
        assert_eq!(n.as_str(), None);
        assert_eq!(n.clone_as_string(), None);
    }

    #[test]
    fn test_numeric_variants_to_string_lossy() {
        let i = Node::from(-5i64);
        let f = Node::from(3.14f64);
        let u = Node::from(7u64);
        let b = Node::from(255u8);
        let i32v = Node::from(-123i32);
        assert_eq!(i.to_string_lossy(), "-5");
        assert_eq!(f.to_string_lossy(), "3.14");
        assert_eq!(u.to_string_lossy(), "7");
        assert_eq!(b.to_string_lossy(), "255");
        assert_eq!(i32v.to_string_lossy(), "-123");
    }

    #[test]
    fn test_node_equality_and_clone() {
        let n1 = Node::from("abc");
        let n2 = n1.clone();
        assert_eq!(n1, n2);
        let n3 = Node::from(123);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_node_array_and_mapping_index() {
        let arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        assert_eq!(arr[0], Node::from(1));
        assert_eq!(arr[1], Node::from(2));

        let map = Node::mapping().insert("foo", 42).build();
        assert_eq!(map["foo"], Node::from(42));
    }

    #[test]
    fn test_node_set_insert_and_contains() {
        let set = Node::set().insert("a").insert("b");
        assert!(set.contains(&Node::from("a")));
        assert!(set.contains(&Node::from("b")));
        assert!(!set.contains(&Node::from("c")));
    }

    #[test]
    fn test_node_none_and_boolean() {
        let n = Node::None;
        assert_eq!(n.to_string_lossy(), "null");
        let t = Node::from(true);
        let f = Node::from(false);
        assert_eq!(t.to_string_lossy(), "true");
        assert_eq!(f.to_string_lossy(), "false");
    }
}
