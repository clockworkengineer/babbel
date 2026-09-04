use crate::nodes::node::Node;
/// Borrowed/zero-copy node implementation for embedded systems.
/// This module provides a Node variant that holds references to the input buffer
/// instead of allocating and copying data, reducing memory usage.

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use core::fmt;

/// A borrowed node that references data from the input buffer without allocation.
/// This is useful for embedded systems where memory is limited.
///
/// The lifetime parameter 'a represents the lifetime of the borrowed input data.
#[derive(Clone, Debug, PartialEq)]
pub enum BorrowedNode<'a> {
    /// Represents a 64-bit signed integer value
    Integer(i64),
    /// Represents a string value as a borrowed byte slice
    /// Note: bencode strings are byte strings, not necessarily UTF-8
    Bytes(&'a [u8]),
    /// Represents a list of other borrowed nodes
    List(Vec<BorrowedNode<'a>>),
    /// Represents a dictionary/map of byte string keys to borrowed node values
    Dictionary(HashMap<&'a [u8], BorrowedNode<'a>>),
}

impl<'a> BorrowedNode<'a> {
    /// Recursively convert a BorrowedNode to an owned Node
    pub fn to_node(&self) -> Node {
        match self {
            BorrowedNode::Integer(i) => Node::Integer(*i),
            BorrowedNode::Bytes(b) => Node::Str(String::from_utf8_lossy(b).into_owned()),
            BorrowedNode::List(list) => {
                Node::List(list.iter().map(|item| item.to_node()).collect())
            }
            BorrowedNode::Dictionary(dict) => Node::Dictionary(
                dict.iter()
                    .map(|(k, v)| {
                        let key = String::from_utf8_lossy(k).into_owned();
                        (key, v.to_node())
                    })
                    .collect(),
            ),
        }
    }
    /// Returns true if the node is an Integer variant
    pub fn is_integer(&self) -> bool {
        matches!(self, BorrowedNode::Integer(_))
    }

    /// Returns true if the node is a Bytes variant
    pub fn is_bytes(&self) -> bool {
        matches!(self, BorrowedNode::Bytes(_))
    }

    /// Returns true if the node is a List variant
    pub fn is_list(&self) -> bool {
        matches!(self, BorrowedNode::List(_))
    }

    /// Returns true if the node is a Dictionary variant
    pub fn is_dictionary(&self) -> bool {
        matches!(self, BorrowedNode::Dictionary(_))
    }

    /// Returns the integer value if this is an Integer node
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            BorrowedNode::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the byte slice if this is a Bytes node
    pub fn as_bytes(&self) -> Option<&'a [u8]> {
        match self {
            BorrowedNode::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Returns the list reference if this is a List node
    pub fn as_list(&self) -> Option<&Vec<BorrowedNode<'a>>> {
        match self {
            BorrowedNode::List(l) => Some(l),
            _ => None,
        }
    }

    /// Returns the dictionary reference if this is a Dictionary node
    pub fn as_dictionary(&self) -> Option<&HashMap<&'a [u8], BorrowedNode<'a>>> {
        match self {
            BorrowedNode::Dictionary(d) => Some(d),
            _ => None,
        }
    }
}

impl<'a> fmt::Display for BorrowedNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowedNode::Integer(i) => write!(f, "{}", i),
            BorrowedNode::Bytes(b) => {
                // Try to display as UTF-8, fallback to debug format
                match core::str::from_utf8(b) {
                    Ok(s) => write!(f, "\"{}\"", s),
                    Err(_) => write!(f, "{:?}", b),
                }
            }
            BorrowedNode::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            BorrowedNode::Dictionary(dict) => {
                write!(f, "{{")?;
                for (i, (key, value)) in dict.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match core::str::from_utf8(key) {
                        Ok(s) => write!(f, "\"{}\": {}", s, value)?,
                        Err(_) => write!(f, "{:?}: {}", key, value)?,
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn edge_cases_empty_and_non_utf8() {
        use crate::nodes::node::Node;
        use crate::parser::borrowed::parse_borrowed;
        // Empty list
        let b = b"le";
        let n = parse_borrowed(b).unwrap().to_node();
        match n {
            Node::List(l) => assert!(l.is_empty()),
            _ => panic!("Expected Node::List"),
        }

        // Empty dict
        let b = b"de";
        let n = parse_borrowed(b).unwrap().to_node();
        match n {
            Node::Dictionary(m) => assert!(m.is_empty()),
            _ => panic!("Expected Node::Dictionary"),
        }

        // Deeply nested
        let mut b = b"i1e".to_vec();
        for _ in 0..10 {
            let mut tmp = Vec::with_capacity(2 + b.len());
            tmp.extend_from_slice(b"l");
            tmp.extend_from_slice(&b);
            tmp.extend_from_slice(b"e");
            b = tmp;
        }
        let n = parse_borrowed(&b).unwrap().to_node();
        let mut cur = &n;
        for _ in 0..10 {
            match cur {
                Node::List(l) => {
                    assert_eq!(l.len(), 1);
                    cur = &l[0];
                }
                _ => panic!("Expected Node::List"),
            }
        }
        assert!(matches!(cur, Node::Integer(1)));

        // Non-UTF8 bytes
        let b = b"3:\xFF\x00\xFE";
        let n = parse_borrowed(b).unwrap();
        assert_eq!(n.as_bytes(), Some(&b"\xFF\x00\xFE"[..]));
        let node = n.to_node();
        match node {
            Node::Str(ref s) => {
                // Should not panic, but will contain replacement chars
                assert!(s.contains("\u{FFFD}") || s.contains("\x00") || s.contains("\u{FFFD}"));
            }
            _ => panic!("Expected Node::Str"),
        }
    }
    #[test]
    fn round_trip_parse_borrowed_to_node() {
        use crate::nodes::node::Node;
        use crate::parser::borrowed::parse_borrowed;
        // Integer
        let b = b"i42e";
        let n = parse_borrowed(b).unwrap().to_node();
        assert!(matches!(n, Node::Integer(42)));

        // String
        let b = b"3:abc";
        let n = parse_borrowed(b).unwrap().to_node();
        assert!(matches!(n, Node::Str(ref s) if s == "abc"));

        // List
        let b = b"li1e3:xyze";
        let n = parse_borrowed(b).unwrap().to_node();
        match n {
            Node::List(l) => {
                assert_eq!(l.len(), 2);
                assert!(matches!(&l[0], Node::Integer(1)));
                assert!(matches!(&l[1], Node::Str(s) if s == "xyz"));
            }
            _ => panic!("Expected Node::List"),
        }

        // Dictionary
        let b = b"d3:foo3:bare";
        let n = parse_borrowed(b).unwrap().to_node();
        match n {
            Node::Dictionary(m) => {
                assert_eq!(m["foo"], Node::Str("bar".to_string()));
            }
            _ => panic!("Expected Node::Dictionary"),
        }
    }
    use super::*;

    #[test]
    fn borrowed_node_type_checks() {
        let int_node = BorrowedNode::Integer(42);
        assert!(int_node.is_integer());
        assert!(!int_node.is_bytes());
        assert!(!int_node.is_list());
        assert!(!int_node.is_dictionary());

        let bytes_node = BorrowedNode::Bytes(b"hello");
        assert!(!bytes_node.is_integer());
        assert!(bytes_node.is_bytes());
        assert!(!bytes_node.is_list());
        assert!(!bytes_node.is_dictionary());
    }

    #[test]
    fn borrowed_node_as_methods() {
        let int_node = BorrowedNode::Integer(42);
        assert_eq!(int_node.as_integer(), Some(42));
        assert_eq!(int_node.as_bytes(), None);

        let bytes_node = BorrowedNode::Bytes(b"test");
        assert_eq!(bytes_node.as_bytes(), Some(&b"test"[..]));
        assert_eq!(bytes_node.as_integer(), None);
    }

    #[test]
    fn borrowed_node_display() {
        let int_node = BorrowedNode::Integer(42);
        assert_eq!(format!("{}", int_node), "42");

        let bytes_node = BorrowedNode::Bytes(b"hello");
        assert_eq!(format!("{}", bytes_node), "\"hello\"");
    }

    #[test]
    fn to_node_conversion_all_variants() {
        use crate::nodes::node::Node;
        // Integer
        let b_int = BorrowedNode::Integer(123);
        let n = b_int.to_node();
        assert!(matches!(n, Node::Integer(123)));

        // Bytes/Str
        let b_bytes = BorrowedNode::Bytes(b"abc");
        let n = b_bytes.to_node();
        assert!(matches!(n, Node::Str(ref s) if s == "abc"));

        // List
        let b_list = BorrowedNode::List(vec![BorrowedNode::Integer(1), BorrowedNode::Bytes(b"x")]);
        let n = b_list.to_node();
        match n {
            Node::List(l) => {
                assert_eq!(l.len(), 2);
                assert!(matches!(&l[0], Node::Integer(1)));
                assert!(matches!(&l[1], Node::Str(s) if s == "x"));
            }
            _ => panic!("Expected Node::List"),
        }

        // Dictionary
        let mut dict = std::collections::HashMap::new();
        dict.insert(b"foo".as_ref(), BorrowedNode::Integer(7));
        dict.insert(b"bar".as_ref(), BorrowedNode::Bytes(b"baz"));
        let b_dict = BorrowedNode::Dictionary(dict);
        let n = b_dict.to_node();
        match n {
            Node::Dictionary(m) => {
                assert_eq!(m["foo"], Node::Integer(7));
                assert_eq!(m["bar"], Node::Str("baz".to_string()));
            }
            _ => panic!("Expected Node::Dictionary"),
        }
    }

    #[test]
    fn to_node_nested_structures() {
        use crate::nodes::node::Node;
        let mut inner_dict = std::collections::HashMap::new();
        inner_dict.insert(b"k".as_ref(), BorrowedNode::Bytes(b"v"));
        let b = BorrowedNode::List(vec![
            BorrowedNode::Dictionary(inner_dict),
            BorrowedNode::Integer(5),
        ]);
        let n = b.to_node();
        match n {
            Node::List(l) => {
                assert_eq!(l.len(), 2);
                match &l[0] {
                    Node::Dictionary(m) => {
                        assert_eq!(m["k"], Node::Str("v".to_string()));
                    }
                    _ => panic!("Expected Node::Dictionary"),
                }
                assert!(matches!(&l[1], Node::Integer(5)));
            }
            _ => panic!("Expected Node::List"),
        }
    }

    // ── is_* checks for all four variants ────────────────────────────────────

    #[test]
    fn is_list_and_is_dictionary_type_checks() {
        let list_node = BorrowedNode::List(vec![]);
        assert!(list_node.is_list());
        assert!(!list_node.is_integer());
        assert!(!list_node.is_bytes());
        assert!(!list_node.is_dictionary());

        let dict_node = BorrowedNode::Dictionary(HashMap::new());
        assert!(dict_node.is_dictionary());
        assert!(!dict_node.is_integer());
        assert!(!dict_node.is_bytes());
        assert!(!dict_node.is_list());
    }

    // ── as_* accessors ────────────────────────────────────────────────────────

    #[test]
    fn as_integer_returns_none_for_non_integer() {
        assert_eq!(BorrowedNode::Bytes(b"x").as_integer(), None);
        assert_eq!(BorrowedNode::List(vec![]).as_integer(), None);
        assert_eq!(BorrowedNode::Dictionary(HashMap::new()).as_integer(), None);
    }

    #[test]
    fn as_bytes_returns_none_for_non_bytes() {
        assert_eq!(BorrowedNode::Integer(1).as_bytes(), None);
        assert_eq!(BorrowedNode::List(vec![]).as_bytes(), None);
        assert_eq!(BorrowedNode::Dictionary(HashMap::new()).as_bytes(), None);
    }

    #[test]
    fn as_list_returns_some_for_list() {
        let list = BorrowedNode::List(vec![BorrowedNode::Integer(1)]);
        let inner = list.as_list().unwrap();
        assert_eq!(inner.len(), 1);
    }

    #[test]
    fn as_list_returns_none_for_non_list() {
        assert!(BorrowedNode::Integer(0).as_list().is_none());
        assert!(BorrowedNode::Bytes(b"").as_list().is_none());
        assert!(BorrowedNode::Dictionary(HashMap::new()).as_list().is_none());
    }

    #[test]
    fn as_dictionary_returns_some_for_dict() {
        let mut d = HashMap::new();
        d.insert(b"a".as_ref(), BorrowedNode::Integer(1));
        let dict = BorrowedNode::Dictionary(d);
        let inner = dict.as_dictionary().unwrap();
        assert_eq!(inner.len(), 1);
    }

    #[test]
    fn as_dictionary_returns_none_for_non_dict() {
        assert!(BorrowedNode::Integer(0).as_dictionary().is_none());
        assert!(BorrowedNode::Bytes(b"").as_dictionary().is_none());
        assert!(BorrowedNode::List(vec![]).as_dictionary().is_none());
    }

    #[test]
    fn as_integer_boundary_values() {
        assert_eq!(BorrowedNode::Integer(0).as_integer(), Some(0));
        assert_eq!(BorrowedNode::Integer(i64::MAX).as_integer(), Some(i64::MAX));
        assert_eq!(BorrowedNode::Integer(i64::MIN).as_integer(), Some(i64::MIN));
        assert_eq!(BorrowedNode::Integer(-1).as_integer(), Some(-1));
    }

    #[test]
    fn as_bytes_empty_slice() {
        let node = BorrowedNode::Bytes(b"");
        assert_eq!(node.as_bytes(), Some(&b""[..]));
    }

    // ── Display ───────────────────────────────────────────────────────────────

    #[test]
    fn display_negative_integer() {
        assert_eq!(format!("{}", BorrowedNode::Integer(-99)), "-99");
    }

    #[test]
    fn display_zero_integer() {
        assert_eq!(format!("{}", BorrowedNode::Integer(0)), "0");
    }

    #[test]
    fn display_empty_bytes() {
        assert_eq!(format!("{}", BorrowedNode::Bytes(b"")), "\"\"");
    }

    #[test]
    fn display_non_utf8_bytes_falls_back_to_debug() {
        let node = BorrowedNode::Bytes(b"\xFF\x00");
        let s = format!("{}", node);
        // Should not be a quoted UTF-8 string, should be the debug repr
        assert!(!s.starts_with('"') || s.contains("\\xff") || s.contains("255"));
    }

    #[test]
    fn display_empty_list() {
        assert_eq!(format!("{}", BorrowedNode::List(vec![])), "[]");
    }

    #[test]
    fn display_list_with_items() {
        let list = BorrowedNode::List(vec![BorrowedNode::Integer(1), BorrowedNode::Bytes(b"ab")]);
        assert_eq!(format!("{}", list), "[1, \"ab\"]");
    }

    #[test]
    fn display_empty_dictionary() {
        assert_eq!(
            format!("{}", BorrowedNode::Dictionary(HashMap::new())),
            "{}"
        );
    }

    // ── Clone / PartialEq ─────────────────────────────────────────────────────

    #[test]
    fn borrowed_node_equality_integers() {
        assert_eq!(BorrowedNode::Integer(42), BorrowedNode::Integer(42));
        assert_ne!(BorrowedNode::Integer(1), BorrowedNode::Integer(2));
    }

    #[test]
    fn borrowed_node_equality_bytes() {
        assert_eq!(BorrowedNode::Bytes(b"abc"), BorrowedNode::Bytes(b"abc"));
        assert_ne!(BorrowedNode::Bytes(b"abc"), BorrowedNode::Bytes(b"xyz"));
    }

    #[test]
    fn borrowed_node_clone_integer() {
        let original = BorrowedNode::Integer(7);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn borrowed_node_clone_bytes() {
        let original = BorrowedNode::Bytes(b"hello");
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn borrowed_node_clone_list() {
        let original = BorrowedNode::List(vec![BorrowedNode::Integer(1)]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // ── to_node edge cases ────────────────────────────────────────────────────

    #[test]
    fn to_node_integer_boundary_values() {
        use crate::nodes::node::Node;
        assert!(matches!(
            BorrowedNode::Integer(0).to_node(),
            Node::Integer(0)
        ));
        assert!(matches!(
            BorrowedNode::Integer(i64::MAX).to_node(),
            Node::Integer(i64::MAX)
        ));
        assert!(matches!(
            BorrowedNode::Integer(i64::MIN).to_node(),
            Node::Integer(i64::MIN)
        ));
    }

    #[test]
    fn to_node_empty_bytes() {
        use crate::nodes::node::Node;
        let n = BorrowedNode::Bytes(b"").to_node();
        assert!(matches!(n, Node::Str(ref s) if s.is_empty()));
    }

    #[test]
    fn to_node_empty_list() {
        use crate::nodes::node::Node;
        let n = BorrowedNode::List(vec![]).to_node();
        match n {
            Node::List(l) => assert!(l.is_empty()),
            _ => panic!("Expected Node::List"),
        }
    }

    #[test]
    fn to_node_empty_dictionary() {
        use crate::nodes::node::Node;
        let n = BorrowedNode::Dictionary(HashMap::new()).to_node();
        match n {
            Node::Dictionary(m) => assert!(m.is_empty()),
            _ => panic!("Expected Node::Dictionary"),
        }
    }

    #[test]
    fn to_node_list_of_integers() {
        use crate::nodes::node::Node;
        let list = BorrowedNode::List(vec![
            BorrowedNode::Integer(-1),
            BorrowedNode::Integer(0),
            BorrowedNode::Integer(1),
        ]);
        match list.to_node() {
            Node::List(l) => {
                assert_eq!(l.len(), 3);
                assert!(matches!(l[0], Node::Integer(-1)));
                assert!(matches!(l[1], Node::Integer(0)));
                assert!(matches!(l[2], Node::Integer(1)));
            }
            _ => panic!("Expected Node::List"),
        }
    }
}
