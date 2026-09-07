#[allow(unused_imports)]
#[cfg(feature = "std")]
use std::collections::HashMap;

#[allow(unused_imports)]
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

use core::str::FromStr;

#[allow(unused_imports)]
pub use super::types::{Node, Numeric};
#[allow(unused_imports)]
pub use super::accessors::*;

impl Node {

    /// Deep merges another node into this node
    ///
    /// For objects, recursively merges keys. If both have the same key:
    /// - If both values are objects, recursively merge them
    /// - Otherwise, the value from `other` overwrites the value in `self`
    ///
    /// For arrays, replaces the entire array with `other`
    /// For all other types, replaces `self` with `other`
    ///
    /// # Arguments
    /// * `other` - The node to merge into this node
    ///
    /// # Examples
    /// ```
    /// use babbel_json::Node;
    /// use std::collections::HashMap;
    ///
    /// let mut obj1 = HashMap::new();
    /// obj1.insert("a".to_string(), Node::Str("old".to_string()));
    /// obj1.insert("b".to_string(), Node::Str("keep".to_string()));
    /// let mut node1 = Node::Object(obj1);
    ///
    /// let mut obj2 = HashMap::new();
    /// obj2.insert("a".to_string(), Node::Str("new".to_string()));
    /// obj2.insert("c".to_string(), Node::Str("added".to_string()));
    /// let node2 = Node::Object(obj2);
    ///
    /// node1.merge(node2);
    /// // node1 now has: {"a": "new", "b": "keep", "c": "added"}
    /// ```
    pub fn merge(&mut self, other: Node) {
        self.merge_ref(&other);
    }

    /// Efficiently merges another node into this node by reference.
    /// For objects, recursively merges keys. If both have the same key:
    /// - If both values are objects, recursively merge them
    /// - Otherwise, replaces the value
    /// For non-objects, replaces self with other.
    pub fn merge_ref(&mut self, other: &Node) {
        match (self, other) {
            (Node::Object(self_map), Node::Object(other_map)) => {
                // Fast path: if self_map is empty, just clone all from other_map
                if self_map.is_empty() {
                    self_map.extend(other_map.iter().map(|(k, v)| (k.clone(), v.clone())));
                    return;
                }
                for (key, other_value) in other_map {
                    match self_map.entry(key.clone()) {
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            if entry.get().is_object() && other_value.is_object() {
                                entry.get_mut().merge_ref(other_value);
                            } else {
                                // Only clone if value is different
                                if entry.get() != other_value {
                                    entry.insert(other_value.clone());
                                }
                            }
                        }
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            entry.insert(other_value.clone());
                        }
                    }
                }
            }
            (this, other) => {
                if this != other {
                    *this = other.clone();
                }
            }
        }
    }

    /// Returns the length of an array or object, None for other types


    /// Gets a value using JSON Pointer notation (RFC 6901)
    ///
    /// # Examples
    /// ```
    /// use babbel_json::{json, Node};
    ///
    /// let data = json!({
    ///     "user": {
    ///         "name": "Alice"
    ///     }
    /// });
    ///
    /// assert_eq!(data.pointer("/user/name").unwrap().as_str(), Some("Alice"));
    /// ```
    #[cfg(feature = "json-pointer")]
    pub fn pointer(&self, path: &str) -> Option<&Node> {
        crate::nodes::json_pointer::get(self, path)
    }

    /// Gets a mutable value using JSON Pointer notation (RFC 6901)
    ///
    /// # Examples
    /// ```
    /// use babbel_json::{json, Node};
    ///
    /// let mut data = json!({"x": 10});
    /// if let Some(x) = data.pointer_mut("/x") {
    ///     *x = Node::from(20);
    /// }
    /// assert_eq!(data["x"].as_i64(), Some(20));
    /// ```
    #[cfg(feature = "json-pointer")]
    pub fn pointer_mut(&mut self, path: &str) -> Option<&mut Node> {
        crate::nodes::json_pointer::get_mut(self, path)
    }

    /// Converts the Node to a pretty-printed JSON string
    ///
    /// # Examples
    /// ```
    /// use babbel_json::{json, Node};
    ///
    /// let data = json!({"name": "Alice", "age": 30});
    /// let pretty = data.to_string_pretty();
    /// assert!(pretty.contains("\n"));
    /// ```
    #[cfg(feature = "alloc")]
    pub fn to_string_pretty(&self) -> String {
        self.to_string_with_indent("  ")
    }

    /// Converts the Node to a pretty-printed JSON string with custom indentation
    ///
    /// # Examples
    /// ```
    /// use babbel_json::{json, Node};
    ///
    /// let data = json!({"x": 1});
    /// let pretty = data.to_string_with_indent("\t");
    /// assert!(pretty.contains("\t"));
    /// ```
    #[cfg(feature = "alloc")]
    pub fn to_string_with_indent(&self, indent: &str) -> String {
        use crate::io::destinations::buffer::Buffer;
        use crate::stringify::pretty::stringify_pretty;

        let mut dest = Buffer::new();
        stringify_pretty(self, &mut dest, indent).unwrap_or_default();
        dest.to_string()
    }
}



#[allow(unused_imports)]
pub use super::indexing::*;
#[allow(unused_imports)]
pub use super::convert::*;

/// Helper functions to create a Node from any value that can be converted into a Node
pub fn make_node<T>(value: T) -> Node
where
    T: Into<Node>,
{
    value.into()
}

/// Implement FromStr to enable `"string".parse::<Node>()`
impl FromStr for Node {
    type Err = String;

    /// Parse a JSON string into a Node
    ///
    /// # Examples
    /// ```
    /// use babbel_json::Node;
    /// use std::str::FromStr;
    ///
    /// let node = Node::from_str(r#"{"name": "Alice"}"#).unwrap();
    /// assert!(node.is_object());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::io::sources::buffer::Buffer as BufferSource;
        use crate::parser::default::parse;

        let mut source = BufferSource::new(s.as_bytes());
        parse(&mut source)
    }
}

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
        assert_eq!(Node::from("test"), Node::Str("test".to_string()));
        assert_eq!(
            Node::from("test".to_string()),
            Node::Str("test".to_string())
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
    fn test_invalid_array_indexing() {
        let node = Node::Boolean(true);
        // Non-panicking - returns &Node::None
        assert!(node[0].is_null());
    }

    #[test]
    fn test_object_indexing() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::from(42));
        let obj = Node::Object(map);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    fn test_invalid_object_indexing() {
        let node = Node::Boolean(true);
        // Non-panicking - returns &Node::None
        assert!(node["key"].is_null());
    }

    #[test]
    fn test_array_mut_indexing() {
        let mut arr = Node::Array(vec![Node::from(1), Node::from(2)]);
        arr[0] = Node::from(42);
        assert_eq!(arr[0], Node::Number(Numeric::Int32(42)));
    }

    #[test]
    #[should_panic(expected = "Cannot index non-array node with integer")]
    fn test_invalid_array_mut_indexing() {
        let mut node = Node::Boolean(true);
        node[0] = Node::from(42);
    }

    #[test]
    fn test_object_mut_indexing() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::from(42));
        let mut obj = Node::Object(map);
        obj["key"] = Node::from(100);
        assert_eq!(obj["key"], Node::Number(Numeric::Int32(100)));
    }

    #[test]
    #[should_panic(expected = "Cannot mutably index non-object node with string")]
    fn test_invalid_object_mut_indexing() {
        let mut node = Node::Boolean(true);
        node["key"] = Node::from(42);
    }

    #[test]
    #[should_panic(expected = "Key does not exist")]
    fn test_object_mut_indexing_nonexistent_key() {
        let mut obj = Node::Object(HashMap::new());
        obj["nonexistent"] = Node::from(42);
    }

    #[test]
    fn test_make_node() {
        assert_eq!(make_node(42), Node::Number(Numeric::Int32(42)));
        assert_eq!(make_node("test"), Node::Str("test".to_string()));
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
    fn test_enum_sizes() {
        use core::mem::size_of;
        println!("Size of Node: {} bytes", size_of::<Node>());
        println!("Size of Numeric: {} bytes", size_of::<Numeric>());
        println!("Size of String: {} bytes", size_of::<String>());
        println!("Size of Vec<Node>: {} bytes", size_of::<Vec<Node>>());
        println!(
            "Size of HashMap<String, Node>: {} bytes",
            size_of::<HashMap<String, Node>>()
        );
    }

    #[test]
    fn test_is_predicates_on_all_variants() {
        let boolean = Node::Boolean(true);
        let number = Node::from(42i64);
        let string = Node::from("hello");
        let array = Node::Array(vec![]);
        let object = Node::Object(HashMap::new());
        let null = Node::None;

        assert!(boolean.is_boolean());
        assert!(!boolean.is_number());
        assert!(!boolean.is_string());
        assert!(!boolean.is_array());
        assert!(!boolean.is_object());
        assert!(!boolean.is_null());

        assert!(!number.is_boolean());
        assert!(number.is_number());
        assert!(!number.is_string());
        assert!(!number.is_array());
        assert!(!number.is_object());
        assert!(!number.is_null());

        assert!(!string.is_boolean());
        assert!(!string.is_number());
        assert!(string.is_string());
        assert!(!string.is_array());
        assert!(!string.is_object());
        assert!(!string.is_null());

        assert!(!array.is_boolean());
        assert!(!array.is_number());
        assert!(!array.is_string());
        assert!(array.is_array());
        assert!(!array.is_object());
        assert!(!array.is_null());

        assert!(!object.is_boolean());
        assert!(!object.is_number());
        assert!(!object.is_string());
        assert!(!object.is_array());
        assert!(object.is_object());
        assert!(!object.is_null());

        assert!(!null.is_boolean());
        assert!(!null.is_number());
        assert!(!null.is_string());
        assert!(!null.is_array());
        assert!(!null.is_object());
        assert!(null.is_null());
    }

    #[test]
    fn test_as_str() {
        let s = Node::from("hello");
        assert_eq!(s.as_str(), Some("hello"));

        let not_str = Node::from(42i64);
        assert_eq!(not_str.as_str(), None);

        let not_str2 = Node::Boolean(true);
        assert_eq!(not_str2.as_str(), None);

        let not_str3 = Node::None;
        assert_eq!(not_str3.as_str(), None);
    }

    #[test]
    fn test_as_bool() {
        let t = Node::Boolean(true);
        assert_eq!(t.as_bool(), Some(true));

        let f = Node::Boolean(false);
        assert_eq!(f.as_bool(), Some(false));

        let not_bool = Node::from(42i64);
        assert_eq!(not_bool.as_bool(), None);

        let not_bool2 = Node::from("true");
        assert_eq!(not_bool2.as_bool(), None);

        let null = Node::None;
        assert_eq!(null.as_bool(), None);
    }

    #[test]
    fn test_as_number() {
        let n = Node::from(42i64);
        assert_eq!(n.as_number(), Some(&Numeric::Integer(42)));

        let not_number = Node::Boolean(true);
        assert_eq!(not_number.as_number(), None);

        let null = Node::None;
        assert_eq!(null.as_number(), None);
    }

    #[test]
    fn test_as_i64_all_numeric_variants() {
        assert_eq!(Node::from(42i64).as_i64(), Some(42));
        assert_eq!(Node::from(42i32).as_i64(), Some(42));
        assert_eq!(Node::from(42i16).as_i64(), Some(42));
        assert_eq!(Node::from(42i8).as_i64(), Some(42));
        assert_eq!(Node::from(42u64).as_i64(), Some(42));
        assert_eq!(Node::from(42u32).as_i64(), Some(42));
        assert_eq!(Node::from(42u16).as_i64(), Some(42));
        assert_eq!(Node::from(42u8).as_i64(), Some(42));
        assert_eq!(Node::from(3.7f64).as_i64(), Some(3)); // truncates
        assert_eq!(Node::Boolean(true).as_i64(), None);
        assert_eq!(Node::from("42").as_i64(), None);
        assert_eq!(Node::None.as_i64(), None);
    }

    #[test]
    fn test_as_f64_all_numeric_variants() {
        assert_eq!(Node::from(42i64).as_f64(), Some(42.0));
        assert_eq!(Node::from(42i32).as_f64(), Some(42.0));
        assert_eq!(Node::from(42i16).as_f64(), Some(42.0));
        assert_eq!(Node::from(42i8).as_f64(), Some(42.0));
        assert_eq!(Node::from(42u64).as_f64(), Some(42.0));
        assert_eq!(Node::from(42u32).as_f64(), Some(42.0));
        assert_eq!(Node::from(42u16).as_f64(), Some(42.0));
        assert_eq!(Node::from(42u8).as_f64(), Some(42.0));
        assert_eq!(Node::from(1.5f64).as_f64(), Some(1.5));
        assert_eq!(Node::Boolean(false).as_f64(), None);
        assert_eq!(Node::None.as_f64(), None);
    }

    #[test]
    fn test_as_u64_positive_and_negative() {
        assert_eq!(Node::from(42u64).as_u64(), Some(42));
        assert_eq!(Node::from(42u32).as_u64(), Some(42));
        assert_eq!(Node::from(42u16).as_u64(), Some(42));
        assert_eq!(Node::from(42u8).as_u64(), Some(42));
        assert_eq!(Node::from(42i64).as_u64(), Some(42));
        assert_eq!(Node::from(42i32).as_u64(), Some(42));
        assert_eq!(Node::from(42i16).as_u64(), Some(42));
        assert_eq!(Node::from(42i8).as_u64(), Some(42));
        assert_eq!(Node::from(3.5f64).as_u64(), Some(3));
        // Negative values return None
        assert_eq!(Node::from(-1i64).as_u64(), None);
        assert_eq!(Node::from(-1i32).as_u64(), None);
        assert_eq!(Node::from(-1i16).as_u64(), None);
        assert_eq!(Node::from(-1i8).as_u64(), None);
        assert_eq!(Node::from(-1.0f64).as_u64(), None);
        assert_eq!(Node::Boolean(true).as_u64(), None);
        assert_eq!(Node::None.as_u64(), None);
    }

    #[test]
    fn test_as_array() {
        let arr = Node::Array(vec![Node::from(1i32), Node::from(2i32)]);
        assert!(arr.as_array().is_some());
        assert_eq!(arr.as_array().unwrap().len(), 2);

        let not_arr = Node::from("hello");
        assert!(not_arr.as_array().is_none());

        let null = Node::None;
        assert!(null.as_array().is_none());
    }

    #[test]
    fn test_as_array_mut() {
        let mut arr = Node::Array(vec![Node::from(1i32)]);
        if let Some(v) = arr.as_array_mut() {
            v.push(Node::from(2i32));
        }
        assert_eq!(arr.as_array().unwrap().len(), 2);

        let mut not_arr = Node::from("hello");
        assert!(not_arr.as_array_mut().is_none());
    }

    #[test]
    fn test_as_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::from(1i32));
        let obj = Node::Object(map);
        assert!(obj.as_object().is_some());
        assert_eq!(obj.as_object().unwrap().len(), 1);

        let not_obj = Node::from(42i64);
        assert!(not_obj.as_object().is_none());

        let null = Node::None;
        assert!(null.as_object().is_none());
    }

    #[test]
    fn test_as_object_mut() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::from(1i32));
        let mut obj = Node::Object(map);
        if let Some(m) = obj.as_object_mut() {
            m.insert("b".to_string(), Node::from(2i32));
        }
        assert_eq!(obj.as_object().unwrap().len(), 2);

        let mut not_obj = Node::from(42i64);
        assert!(not_obj.as_object_mut().is_none());
    }

    #[test]
    fn test_get_and_get_mut() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), Node::from("Alice"));
        let mut obj = Node::Object(map);

        assert_eq!(obj.get("name").and_then(|n| n.as_str()), Some("Alice"));
        assert!(obj.get("missing").is_none());

        // get on non-object returns None
        let num = Node::from(42i64);
        assert!(num.get("key").is_none());

        // get_mut modifies value in place
        if let Some(node) = obj.get_mut("name") {
            *node = Node::from("Bob");
        }
        assert_eq!(obj.get("name").and_then(|n| n.as_str()), Some("Bob"));
        assert!(obj.get_mut("nope").is_none());
    }

    #[test]
    fn test_at_and_at_mut() {
        let mut arr = Node::Array(vec![Node::from(10i32), Node::from(20i32)]);
        assert_eq!(arr.at(0).and_then(|n| n.as_i64()), Some(10));
        assert_eq!(arr.at(1).and_then(|n| n.as_i64()), Some(20));
        assert!(arr.at(5).is_none());

        // at on non-array returns None
        let s = Node::from("hello");
        assert!(s.at(0).is_none());

        // at_mut
        if let Some(n) = arr.at_mut(0) {
            *n = Node::from(99i32);
        }
        assert_eq!(arr.at(0).and_then(|n| n.as_i64()), Some(99));
        assert!(arr.at_mut(99).is_none());

        let mut not_arr = Node::from(42i64);
        assert!(not_arr.at_mut(0).is_none());
    }

    #[test]
    fn test_keys_iterator() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::from(1i32));
        map.insert("b".to_string(), Node::from(2i32));
        let obj = Node::Object(map);
        let keys: Vec<&str> = obj.keys().unwrap().collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));

        // Non-object returns None
        let arr = Node::Array(vec![]);
        assert!(arr.keys().is_none());

        let null = Node::None;
        assert!(null.keys().is_none());
    }

    #[test]
    fn test_object_values_iterator() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), Node::from(1i32));
        let obj = Node::Object(map);
        let values: Vec<&Node> = obj.object_values().unwrap().collect();
        assert_eq!(values.len(), 1);

        let arr = Node::Array(vec![]);
        assert!(arr.object_values().is_none());
    }

    #[test]
    fn test_object_values_mut_iterator() {
        let mut map = HashMap::new();
        map.insert("x".to_string(), Node::from(1i32));
        let mut obj = Node::Object(map);

        for v in obj.object_values_mut().unwrap() {
            *v = Node::from(99i32);
        }
        assert_eq!(obj.get("x").and_then(|n| n.as_i64()), Some(99));

        let mut arr = Node::Array(vec![]);
        assert!(arr.object_values_mut().is_none());
    }

    #[test]
    fn test_array_iter() {
        let arr = Node::Array(vec![Node::from(1i32), Node::from(2i32), Node::from(3i32)]);
        let values: Vec<&Node> = arr.array_iter().unwrap().collect();
        assert_eq!(values.len(), 3);

        let not_arr = Node::from("str");
        assert!(not_arr.array_iter().is_none());
    }

    #[test]
    fn test_array_iter_mut() {
        let mut arr = Node::Array(vec![Node::from(1i32), Node::from(2i32)]);
        for item in arr.array_iter_mut().unwrap() {
            *item = Node::None;
        }
        assert_eq!(arr.at(0), Some(&Node::None));
        assert_eq!(arr.at(1), Some(&Node::None));

        let mut not_arr = Node::from(42i64);
        assert!(not_arr.array_iter_mut().is_none());
    }

    #[test]
    fn test_into_string() {
        let s = Node::from("hello");
        assert_eq!(s.into_string(), Some("hello".to_string()));

        let not_str = Node::from(42i64);
        assert_eq!(not_str.into_string(), None);

        let null = Node::None;
        assert_eq!(null.into_string(), None);
    }

    #[test]
    fn test_into_array() {
        let arr = Node::Array(vec![Node::from(1i32), Node::from(2i32)]);
        let v = arr.into_array().unwrap();
        assert_eq!(v.len(), 2);

        let not_arr = Node::from("hi");
        assert!(not_arr.into_array().is_none());
    }

    #[test]
    fn test_into_object() {
        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::from(1i32));
        let obj = Node::Object(map);
        let m = obj.into_object().unwrap();
        assert_eq!(m.len(), 1);

        let not_obj = Node::from(42i64);
        assert!(not_obj.into_object().is_none());
    }

    #[test]
    fn test_into_number() {
        let n = Node::from(42i64);
        assert_eq!(n.into_number(), Some(Numeric::Integer(42)));

        let not_num = Node::Boolean(false);
        assert_eq!(not_num.into_number(), None);

        let null = Node::None;
        assert_eq!(null.into_number(), None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let empty_arr = Node::Array(vec![]);
        assert_eq!(empty_arr.len(), Some(0));
        assert!(empty_arr.is_empty());

        let arr = Node::Array(vec![Node::from(1i32), Node::from(2i32)]);
        assert_eq!(arr.len(), Some(2));
        assert!(!arr.is_empty());

        let empty_obj = Node::Object(HashMap::new());
        assert_eq!(empty_obj.len(), Some(0));
        assert!(empty_obj.is_empty());

        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::from(1i32));
        let obj = Node::Object(map);
        assert_eq!(obj.len(), Some(1));
        assert!(!obj.is_empty());

        // Other variants return None for len and false for is_empty
        let num = Node::from(42i64);
        assert_eq!(num.len(), None);
        assert!(!num.is_empty());

        let boolean = Node::Boolean(true);
        assert_eq!(boolean.len(), None);
        assert!(!boolean.is_empty());

        let s = Node::from("hello");
        assert_eq!(s.len(), None);
        assert!(!s.is_empty());

        let null = Node::None;
        assert_eq!(null.len(), None);
        assert!(!null.is_empty());
    }

    #[test]
    fn test_take() {
        let mut node = Node::from(42i64);
        let taken = node.take();
        assert_eq!(taken, Node::from(42i64));
        assert_eq!(node, Node::None);

        let mut s = Node::from("hello");
        let taken_s = s.take();
        assert_eq!(taken_s, Node::from("hello"));
        assert!(s.is_null());
    }

    #[test]
    fn test_node_object_constructor() {
        let obj = Node::object();
        assert!(obj.is_object());
        assert_eq!(obj.len(), Some(0));
        assert!(obj.is_empty());
    }

    #[test]
    fn test_node_array_constructor() {
        let arr = Node::array();
        assert!(arr.is_array());
        assert_eq!(arr.len(), Some(0));
        assert!(arr.is_empty());
    }

    #[test]
    fn test_node_null_constructor() {
        let null = Node::null();
        assert!(null.is_null());
        assert_eq!(null, Node::None);
    }

    #[test]
    fn test_insert_into_object() {
        let mut obj = Node::object();
        let old = obj.insert("key", "value");
        assert!(old.is_none()); // no previous value
        assert_eq!(obj.get("key").and_then(|n| n.as_str()), Some("value"));

        // Insert overwrites and returns old value
        let old2 = obj.insert("key", "new_value");
        assert_eq!(
            old2.and_then(|n| n.into_string()),
            Some("value".to_string())
        );

        // Insert into non-object returns None
        let mut num = Node::from(42i64);
        let result = num.insert("x", 1i64);
        assert!(result.is_none());
    }

    #[test]
    fn test_merge_objects() {
        let mut map1 = HashMap::new();
        map1.insert("a".to_string(), Node::from("old"));
        map1.insert("b".to_string(), Node::from("keep"));
        let mut node1 = Node::Object(map1);

        let mut map2 = HashMap::new();
        map2.insert("a".to_string(), Node::from("new"));
        map2.insert("c".to_string(), Node::from("added"));
        let node2 = Node::Object(map2);

        node1.merge(node2);
        assert_eq!(node1.get("a").and_then(|n| n.as_str()), Some("new"));
        assert_eq!(node1.get("b").and_then(|n| n.as_str()), Some("keep"));
        assert_eq!(node1.get("c").and_then(|n| n.as_str()), Some("added"));
    }

    #[test]
    fn test_merge_non_objects_replaces() {
        let mut arr = Node::Array(vec![Node::from(1i32)]);
        let new_arr = Node::Array(vec![Node::from(2i32), Node::from(3i32)]);
        arr.merge(new_arr);
        assert_eq!(arr.as_array().unwrap().len(), 2);

        let mut num = Node::from(1i64);
        num.merge(Node::from(99i64));
        assert_eq!(num.as_i64(), Some(99));
    }

    #[test]
    fn test_merge_ref() {
        let mut obj = Node::object();
        obj.insert("x", 1i32);
        let patch = {
            let mut m = HashMap::new();
            m.insert("x".to_string(), Node::from(2i32));
            m.insert("y".to_string(), Node::from(3i32));
            Node::Object(m)
        };
        obj.merge_ref(&patch);
        assert_eq!(obj.get("x").and_then(|n| n.as_i64()), Some(2));
        assert_eq!(obj.get("y").and_then(|n| n.as_i64()), Some(3));
    }

    #[test]
    fn test_merge_empty_object_fast_path() {
        // merge_ref has a fast path when self_map is empty
        let mut empty = Node::object();
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::from(1i32));
        map.insert("b".to_string(), Node::from(2i32));
        let source = Node::Object(map);
        empty.merge_ref(&source);
        assert_eq!(empty.len(), Some(2));
        assert_eq!(empty.get("a").and_then(|n| n.as_i64()), Some(1));
    }

    #[test]
    fn test_merge_nested_objects() {
        let mut node1: Node = r#"{"a": {"x": 1, "y": 2}}"#.parse().unwrap();
        let node2: Node = r#"{"a": {"y": 99, "z": 3}}"#.parse().unwrap();
        node1.merge(node2);
        let a = node1.get("a").unwrap();
        assert_eq!(a.get("x").and_then(|n| n.as_i64()), Some(1));
        assert_eq!(a.get("y").and_then(|n| n.as_i64()), Some(99));
        assert_eq!(a.get("z").and_then(|n| n.as_i64()), Some(3));
    }

    #[test]
    fn test_default_node_is_none() {
        let node: Node = Default::default();
        assert_eq!(node, Node::None);
        assert!(node.is_null());
    }

    #[test]
    fn test_display_numeric() {
        assert_eq!(format!("{}", Numeric::Integer(42)), "42");
        assert_eq!(format!("{}", Numeric::Float(3.14)), "3.14");
        assert_eq!(format!("{}", Numeric::UInteger(100)), "100");
        assert_eq!(format!("{}", Numeric::Byte(255)), "255");
        assert_eq!(format!("{}", Numeric::Int32(-10)), "-10");
        assert_eq!(format!("{}", Numeric::UInt32(20)), "20");
        assert_eq!(format!("{}", Numeric::Int16(-5)), "-5");
        assert_eq!(format!("{}", Numeric::UInt16(5)), "5");
        assert_eq!(format!("{}", Numeric::Int8(-1)), "-1");
    }

    #[test]
    fn test_display_node() {
        assert_eq!(format!("{}", Node::None), "null");
        assert_eq!(format!("{}", Node::Boolean(true)), "true");
        assert_eq!(format!("{}", Node::Boolean(false)), "false");
        assert_eq!(format!("{}", Node::from(42i64)), "42");
        assert_eq!(format!("{}", Node::from("hello")), "\"hello\"");

        let arr = Node::Array(vec![Node::from(1i32), Node::from(2i32)]);
        assert_eq!(format!("{}", arr), "[1, 2]");

        let empty_arr = Node::Array(vec![]);
        assert_eq!(format!("{}", empty_arr), "[]");

        let empty_obj = Node::Object(HashMap::new());
        assert_eq!(format!("{}", empty_obj), "{}");
    }

    #[test]
    fn test_try_from_node_to_string() {
        use core::convert::TryFrom;
        let s = Node::from("hello");
        let result: Result<String, _> = String::try_from(s);
        assert_eq!(result.unwrap(), "hello");

        let not_str = Node::from(42i64);
        let result2: Result<String, _> = String::try_from(not_str);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_node_to_vec() {
        use core::convert::TryFrom;
        let arr = Node::Array(vec![Node::from(1i32)]);
        let result: Result<Vec<Node>, _> = Vec::try_from(arr);
        assert_eq!(result.unwrap().len(), 1);

        let not_arr = Node::from("hello");
        let result2: Result<Vec<Node>, _> = Vec::try_from(not_arr);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_node_to_hashmap() {
        use core::convert::TryFrom;
        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::from(1i32));
        let obj = Node::Object(map);
        let result: Result<HashMap<String, Node>, _> = HashMap::try_from(obj);
        assert_eq!(result.unwrap().len(), 1);

        let not_obj = Node::from(42i64);
        let result2: Result<HashMap<String, Node>, _> = HashMap::try_from(not_obj);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_node_to_i64() {
        use core::convert::TryFrom;
        let n = Node::from(42i64);
        let result: Result<i64, _> = i64::try_from(n);
        assert_eq!(result.unwrap(), 42);

        let not_num = Node::Boolean(true);
        let result2: Result<i64, _> = i64::try_from(not_num);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_ref_node_to_i64() {
        use core::convert::TryFrom;
        let n = Node::from(42i64);
        let result: Result<i64, _> = i64::try_from(&n);
        assert_eq!(result.unwrap(), 42);

        let not_num = Node::from("hello");
        let result2: Result<i64, _> = i64::try_from(&not_num);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_node_to_f64() {
        use core::convert::TryFrom;
        let n = Node::from(3.14f64);
        let result: Result<f64, _> = f64::try_from(n);
        let v = result.unwrap();
        assert!((v - 3.14).abs() < 1e-10);

        let not_num = Node::None;
        let result2: Result<f64, _> = f64::try_from(not_num);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_ref_node_to_f64() {
        use core::convert::TryFrom;
        let n = Node::from(2.5f64);
        let result: Result<f64, _> = f64::try_from(&n);
        assert_eq!(result.unwrap(), 2.5);
    }

    #[test]
    fn test_try_from_node_to_bool() {
        use core::convert::TryFrom;
        let b = Node::Boolean(true);
        let result: Result<bool, _> = bool::try_from(b);
        assert_eq!(result.unwrap(), true);

        let not_bool = Node::from(1i64);
        let result2: Result<bool, _> = bool::try_from(not_bool);
        assert!(result2.is_err());
    }

    #[test]
    fn test_try_from_ref_node_to_bool() {
        use core::convert::TryFrom;
        let b = Node::Boolean(false);
        let result: Result<bool, _> = bool::try_from(&b);
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_from_str_parses_json() {
        use std::str::FromStr;
        let node = Node::from_str(r#"{"x": 1}"#).unwrap();
        assert!(node.is_object());

        let arr = Node::from_str("[1, 2, 3]").unwrap();
        assert!(arr.is_array());
        assert_eq!(arr.len(), Some(3));

        let null = Node::from_str("null").unwrap();
        assert!(null.is_null());

        let boolean = Node::from_str("true").unwrap();
        assert_eq!(boolean.as_bool(), Some(true));

        let s = Node::from_str("\"hello\"").unwrap();
        assert_eq!(s.as_str(), Some("hello"));

        let err = Node::from_str("{bad json}");
        assert!(err.is_err());
    }

    #[test]
    fn test_from_option() {
        let some: Option<i64> = Some(42);
        let node = Node::from(some);
        assert_eq!(node, Node::from(42i64));

        let none: Option<i64> = None;
        let node2 = Node::from(none);
        assert_eq!(node2, Node::None);
    }

    #[test]
    fn test_from_fixed_array() {
        let node = Node::from([1i32, 2i32, 3i32]);
        assert!(node.is_array());
        assert_eq!(node.len(), Some(3));
        assert_eq!(node.at(0).and_then(|n| n.as_i64()), Some(1));
    }

    #[test]
    fn test_from_slice() {
        let slice: &[i32] = &[10, 20, 30];
        let node = Node::from(slice);
        assert!(node.is_array());
        assert_eq!(node.len(), Some(3));
        assert_eq!(node.at(2).and_then(|n| n.as_i64()), Some(30));
    }

    #[test]
    fn test_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), 1i64);
        map.insert("b".to_string(), 2i64);
        let node = Node::from(map);
        assert!(node.is_object());
        assert_eq!(node.len(), Some(2));
    }

    #[test]
    fn test_from_iter_constructor() {
        let items = vec![Node::from(1i32), Node::from(2i32), Node::from(3i32)];
        let node = Node::from_iter(items);
        assert!(node.is_array());
        assert_eq!(node.len(), Some(3));
    }

    #[test]
    fn test_from_slice_node_constructor() {
        let items = vec![Node::from("a"), Node::from("b")];
        let node = Node::from_slice(&items);
        assert!(node.is_array());
        assert_eq!(node.len(), Some(2));
        assert_eq!(node.at(0).and_then(|n| n.as_str()), Some("a"));
    }

    #[test]
    fn test_from_vec_constructor() {
        let items = vec![Node::from(42i32)];
        let node = Node::from_vec(items);
        assert!(node.is_array());
        assert_eq!(node.len(), Some(1));
    }

    #[test]
    fn test_clone_and_partial_eq() {
        let node = Node::from(42i64);
        let cloned = node.clone();
        assert_eq!(node, cloned);

        let arr = Node::Array(vec![Node::from(1i32), Node::from(2i32)]);
        let arr2 = arr.clone();
        assert_eq!(arr, arr2);

        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::from(1i32));
        let obj = Node::Object(map);
        let obj2 = obj.clone();
        assert_eq!(obj, obj2);
    }

    #[test]
    fn test_partial_eq_different_variants() {
        assert_ne!(Node::None, Node::Boolean(false));
        assert_ne!(Node::from(42i64), Node::from("42"));
        assert_ne!(Node::Array(vec![]), Node::Object(HashMap::new()));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_to_string_pretty() {
        let obj: Node = r#"{"name":"Alice"}"#.parse().unwrap();
        let pretty = obj.to_string_pretty();
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("Alice"));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_to_string_with_indent() {
        let obj: Node = r#"{"x":1}"#.parse().unwrap();
        let indented = obj.to_string_with_indent("\t");
        assert!(indented.contains('\t'));
    }
}
