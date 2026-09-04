//! Integration tests for parsing bencode lists.

#[cfg(test)]
mod tests {
    use crate::BufferSource;
    use crate::error::messages::*;
    use crate::nodes::node::Node;
    use crate::parser::default::parse;

    #[test]
    fn test_empty_list_works() {
        let mut source = BufferSource::new(b"le");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 0);
            }
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_nested_list_works() {
        let mut source = BufferSource::new(b"lli1ei2eeli3ei4eee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::List(l) if l.len() == 2));
                assert!(matches!(&list[1], Node::List(l) if l.len() == 2));
            }
            _ => panic!("Expected nested list"),
        }
    }

    #[test]
    fn test_mixed_list_works() {
        let mut source = BufferSource::new(b"li32e4:testi-42ee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 3);
                assert!(matches!(&list[0], Node::Integer(32)));
                assert!(matches!(&list[1], Node::Str(s) if s == "test"));
                assert!(matches!(&list[2], Node::Integer(-42)));
            }
            _ => panic!("Expected mixed list"),
        }
    }

    #[test]
    fn test_list_single_element() {
        let mut source = BufferSource::new(b"li42ee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert!(matches!(&list[0], Node::Integer(42)));
            }
            _ => panic!("Expected single element list"),
        }
    }

    #[test]
    fn test_unterminated_list() {
        let mut source = BufferSource::new(b"li1ei2e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    #[test]
    fn test_list_of_strings() {
        let mut source = BufferSource::new(b"l3:foo3:bar3:baze");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 3);
                assert!(matches!(&list[0], Node::Str(s) if s == "foo"));
                assert!(matches!(&list[1], Node::Str(s) if s == "bar"));
                assert!(matches!(&list[2], Node::Str(s) if s == "baz"));
            }
            _ => panic!("Expected list of strings"),
        }
    }

    #[test]
    fn test_deeply_nested_lists() {
        let mut source = BufferSource::new(b"llli1eeee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                if let Node::List(inner1) = &list[0] {
                    assert_eq!(inner1.len(), 1);
                    if let Node::List(inner2) = &inner1[0] {
                        assert_eq!(inner2.len(), 1);
                        assert!(matches!(&inner2[0], Node::Integer(1)));
                    } else {
                        panic!("Expected nested list");
                    }
                } else {
                    panic!("Expected nested list");
                }
            }
            _ => panic!("Expected deeply nested list"),
        }
    }

    #[test]
    fn test_list_with_empty_string() {
        let mut source = BufferSource::new(b"li1e0:i2ee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 3);
                assert!(matches!(&list[0], Node::Integer(1)));
                assert!(matches!(&list[1], Node::Str(s) if s.is_empty()));
                assert!(matches!(&list[2], Node::Integer(2)));
            }
            _ => panic!("Expected list with empty string"),
        }
    }

    #[test]
    fn test_list_with_dictionary() {
        let mut source = BufferSource::new(b"ld3:foo3:baree");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert!(matches!(&list[0], Node::Dictionary(_)));
            }
            _ => panic!("Expected list with dictionary"),
        }
    }

    #[test]
    fn test_very_long_list() {
        // Create a list with many elements
        let data = b"li1ei2ei3ei4ei5ei6ei7ei8ei9ei10ee";
        let mut source = BufferSource::new(data);
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 10);
            }
            _ => panic!("Expected long list"),
        }
    }

    // --- Element value correctness ---

    #[test]
    fn test_list_single_string() {
        let mut source = BufferSource::new(b"l5:helloe");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert_eq!(list[0], Node::Str("hello".to_string()));
            }
            _ => panic!("Expected list with one string"),
        }
    }

    #[test]
    fn test_list_negative_integers() {
        let mut source = BufferSource::new(b"li-1ei-2ei-3ee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list[0], Node::Integer(-1));
                assert_eq!(list[1], Node::Integer(-2));
                assert_eq!(list[2], Node::Integer(-3));
            }
            _ => panic!("Expected list of negative integers"),
        }
    }

    #[test]
    fn test_list_element_zero() {
        let mut source = BufferSource::new(b"li0ee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list[0], Node::Integer(0));
            }
            _ => panic!("Expected list with zero"),
        }
    }

    #[test]
    fn test_list_containing_empty_list() {
        let mut source = BufferSource::new(b"llee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert!(matches!(&list[0], Node::List(l) if l.is_empty()));
            }
            _ => panic!("Expected list containing empty list"),
        }
    }

    #[test]
    fn test_list_containing_empty_dict() {
        let mut source = BufferSource::new(b"ldee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert!(matches!(&list[0], Node::Dictionary(d) if d.is_empty()));
            }
            _ => panic!("Expected list containing empty dict"),
        }
    }

    #[test]
    fn test_list_of_dicts() {
        let mut source = BufferSource::new(b"ld1:ai1eed1:bi2eee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(
                    matches!(&list[0], Node::Dictionary(d) if d.get("a") == Some(&Node::Integer(1)))
                );
                assert!(
                    matches!(&list[1], Node::Dictionary(d) if d.get("b") == Some(&Node::Integer(2)))
                );
            }
            _ => panic!("Expected list of dicts"),
        }
    }

    // --- Error cases ---

    #[test]
    fn test_unterminated_empty_list() {
        let mut source = BufferSource::new(b"l");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    #[test]
    fn test_list_invalid_element() {
        let mut source = BufferSource::new(b"lxe");
        assert!(matches!(parse(&mut source), Err(s) if s.contains("Unexpected character")));
    }

    #[test]
    fn test_list_element_unterminated_integer() {
        let mut source = BufferSource::new(b"li42");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_INTEGER));
    }

    #[test]
    fn test_list_element_string_too_short() {
        // Element claims length 10 but only 2 bytes available
        let mut source = BufferSource::new(b"l10:hie");
        assert!(matches!(parse(&mut source), Err(_)));
    }

    // --- Convenience API ---

    #[test]
    fn test_parse_bytes_empty_list() {
        use crate::parse_bytes;
        assert_eq!(parse_bytes(b"le").unwrap(), Node::List(vec![]));
    }

    #[test]
    fn test_parse_str_list_of_integers() {
        use crate::parse_str;
        let node = parse_str("li10ei20ei30ee").unwrap();
        assert_eq!(
            node,
            Node::List(vec![
                Node::Integer(10),
                Node::Integer(20),
                Node::Integer(30)
            ])
        );
    }

    // --- Iterative parser ---

    #[test]
    fn test_iterative_empty_list() {
        use crate::parse_str_iterative;
        assert_eq!(parse_str_iterative("le").unwrap(), Node::List(vec![]));
    }

    #[test]
    fn test_iterative_list_of_strings() {
        use crate::parse_bytes_iterative;
        let node = parse_bytes_iterative(b"l3:one3:twoe").unwrap();
        assert_eq!(
            node,
            Node::List(vec![
                Node::Str("one".to_string()),
                Node::Str("two".to_string())
            ])
        );
    }

    #[test]
    fn test_iterative_nested_list() {
        use crate::parse_str_iterative;
        let node = parse_str_iterative("lli1ei2eeli3ei4eee").unwrap();
        match node {
            Node::List(list) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::List(l) if l.len() == 2));
                assert!(matches!(&list[1], Node::List(l) if l.len() == 2));
            }
            _ => panic!("Expected nested list from iterative parser"),
        }
    }

    #[test]
    fn test_iterative_list_with_dict() {
        use crate::parse_str_iterative;
        let node = parse_str_iterative("ld3:keyi99eee").unwrap();
        match node {
            Node::List(list) => {
                assert_eq!(list.len(), 1);
                assert!(
                    matches!(&list[0], Node::Dictionary(d) if d.get("key") == Some(&Node::Integer(99)))
                );
            }
            _ => panic!("Expected list with dict"),
        }
    }

    #[test]
    fn test_iterative_unterminated_list_fails() {
        use crate::parse_bytes_iterative;
        assert!(parse_bytes_iterative(b"li1e").is_err());
    }

    // --- Borrowed / zero-copy parser ---

    #[test]
    fn test_borrowed_empty_list() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"le").unwrap();
        assert!(node.is_list());
        assert_eq!(node.as_list().unwrap().len(), 0);
    }

    #[test]
    fn test_borrowed_list_of_integers() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"li5ei10ee").unwrap();
        let list = node.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_integer(), Some(5));
        assert_eq!(list[1].as_integer(), Some(10));
    }

    #[test]
    fn test_borrowed_nested_list() {
        use crate::parse_borrowed;
        let node = parse_borrowed(b"lli1eee").unwrap();
        let outer = node.as_list().unwrap();
        assert_eq!(outer.len(), 1);
        assert!(outer[0].is_list());
    }

    #[test]
    fn test_borrowed_list_to_owned() {
        use crate::parse_borrowed;
        let borrowed = parse_borrowed(b"li7ei8ee").unwrap();
        let owned = borrowed.to_node();
        assert_eq!(owned, Node::List(vec![Node::Integer(7), Node::Integer(8)]));
    }

    // --- Round-trips ---

    #[test]
    fn test_round_trip_empty_list() {
        use crate::{parse_str, stringify_to_string};
        let node = Node::List(vec![]);
        let encoded = stringify_to_string(&node).unwrap();
        assert_eq!(encoded, "le");
        assert_eq!(parse_str(&encoded).unwrap(), node);
    }

    #[test]
    fn test_round_trip_mixed_list() {
        use crate::{parse_bytes, stringify_to_bytes};
        let node = Node::List(vec![
            Node::Integer(1),
            Node::Str("hello".to_string()),
            Node::Integer(-99),
        ]);
        let encoded = stringify_to_bytes(&node).unwrap();
        assert_eq!(parse_bytes(&encoded).unwrap(), node);
    }

    #[test]
    fn test_round_trip_nested_list() {
        use crate::{parse_bytes, stringify_to_bytes};
        let node = Node::List(vec![
            Node::List(vec![Node::Integer(1), Node::Integer(2)]),
            Node::List(vec![Node::Integer(3), Node::Integer(4)]),
        ]);
        let encoded = stringify_to_bytes(&node).unwrap();
        assert_eq!(parse_bytes(&encoded).unwrap(), node);
    }
}
