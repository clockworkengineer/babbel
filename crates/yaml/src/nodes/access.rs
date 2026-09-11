//! YAML Node Accessors & Inspection
//!
//! Safe, panic-free query, access, and introspection methods for `Node`.
//!
//! Copyright (c) 2026 YAML Library Developers

#[allow(unused_imports)]
use super::types::{Node, Numeric};

#[cfg(feature = "alloc")]
use crate::nodes::builders::{ArrayBuilder, MappingBuilder, SetBuilder};

#[cfg(feature = "alloc")]
impl Node {
    /// Returns true if the node is considered blank (None, empty array, empty string, comment, or recursively blank document/anchored node)
    #[inline]
    pub fn is_blank(&self) -> bool {
        match self {
            Node::None => true,
            Node::Array(items) => items.is_empty(),
            Node::Mapping(_pairs) => false,
            Node::Document(nodes) => nodes.iter().all(|n| n.is_blank()),
            Node::Str(s, _, _) => s.is_empty(),
            Node::Comment(_) => true,
            Node::Anchored(inner, _name) => (**inner).is_blank(),
            Node::Alias(_name) => false,
            _ => false,
        }
    }

    /// Safely get an array element by index without panicking
    ///
    /// Returns None if the index is out of bounds or if the node is not an array/set.
    /// This is the recommended method to avoid panics in production code.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// assert!(array.get(0).is_some());
    /// assert!(array.get(5).is_none());
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Node> {
        match self {
            Node::Array(arr) => arr.get(index),
            Node::Set(set) => set.get(index),
            _ => None,
        }
    }

    /// Safely get a mapping value by key without panicking
    ///
    /// Returns None if the key doesn't exist or if the node is not a mapping.
    /// This is the recommended method to avoid panics in production code.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// assert!(mapping.get_key("key").is_some());
    /// assert!(mapping.get_key("nonexistent").is_none());
    /// ```
    #[inline]
    pub fn get_key(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return Some(v);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Safely get a mutable array element by index without panicking
    ///
    /// Returns None if the index is out of bounds or if the node is not an array/set.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mut array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// if let Some(node) = array.get_mut(0) {
    ///     *node = Node::from(10);
    /// }
    /// ```
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Node> {
        match self {
            Node::Array(arr) => arr.get_mut(index),
            Node::Set(set) => set.get_mut(index),
            _ => None,
        }
    }

    /// Safely get a mutable mapping value by key without panicking
    ///
    /// Returns None if the key doesn't exist or if the node is not a mapping.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mut mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// if let Some(node) = mapping.get_key_mut("key") {
    ///     *node = Node::from("new_value");
    /// }
    /// ```
    #[inline]
    pub fn get_key_mut(&mut self, key: &str) -> Option<&mut Node> {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs.iter_mut() {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return Some(v);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Check if this node is an array or set
    ///
    /// Returns true for Node::Array and Node::Set variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![]);
    /// assert!(array.is_sequence());
    /// let mapping = Node::Mapping(vec![]);
    /// assert!(!mapping.is_sequence());
    /// ```
    #[inline]
    pub fn is_sequence(&self) -> bool {
        matches!(self, Node::Array(_) | Node::Set(_))
    }

    /// Check if this node is a mapping
    ///
    /// Returns true for Node::Mapping variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mapping = Node::Mapping(vec![]);
    /// assert!(mapping.is_mapping());
    /// let array = Node::Array(vec![]);
    /// assert!(!array.is_mapping());
    /// ```
    #[inline]
    pub fn is_mapping(&self) -> bool {
        matches!(self, Node::Mapping(_))
    }

    /// Get the length of an array, set, or mapping
    ///
    /// Returns None if the node is not a collection type.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// assert_eq!(array.len(), Some(2));
    /// let scalar = Node::from(42);
    /// assert_eq!(scalar.len(), None);
    /// ```
    #[inline]
    pub fn len(&self) -> Option<usize> {
        match self {
            Node::Array(arr) => Some(arr.len()),
            Node::Set(set) => Some(set.len()),
            Node::Mapping(pairs) => Some(pairs.len()),
            _ => None,
        }
    }

    /// Check if a collection is empty
    ///
    /// Returns true if the node is a collection and is empty, false otherwise.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![]);
    /// assert!(array.is_empty());
    /// let array = Node::Array(vec![Node::from(1)]);
    /// assert!(!array.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len().map_or(false, |l| l == 0)
    }

    /// Safely convert a numeric node to i32
    ///
    /// Returns None if the node is not numeric or if conversion fails.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let num = Node::from(42);
    /// assert_eq!(num.as_i32(), Some(42));
    /// let string = Node::from("text");
    /// assert_eq!(string.as_i32(), None);
    /// ```
    #[inline]
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            #[cfg(feature = "embedded")]
            Node::Number(num) => num.to_i32(),
            #[cfg(not(feature = "embedded"))]
            Node::Number(num) => match num {
                Numeric::Integer(v) => i32::try_from(*v).ok(),
                Numeric::Float(v) => {
                    if v.is_finite() && *v >= i32::MIN as f64 && *v <= i32::MAX as f64 {
                        Some(*v as i32)
                    } else {
                        None
                    }
                }
                Numeric::UInteger(v) => i32::try_from(*v).ok(),
                Numeric::Byte(v) => Some(*v as i32),
                Numeric::Int32(v) => Some(*v),
                Numeric::UInt32(v) => i32::try_from(*v).ok(),
                Numeric::Int16(v) => Some(*v as i32),
                Numeric::UInt16(v) => Some(*v as i32),
                Numeric::Int8(v) => Some(*v as i32),
                Numeric::UInt8(v) => Some(*v as i32),
            },
            _ => None,
        }
    }

    /// Safely convert a numeric node to f32
    ///
    /// Returns None if the node is not numeric.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let num = Node::from(3.14);
    /// assert_eq!(num.as_f32(), Some(3.14_f32));
    /// let string = Node::from("text");
    /// assert_eq!(string.as_f32(), None);
    /// ```
    #[inline]
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            #[cfg(feature = "embedded")]
            Node::Number(num) => Some(num.to_f32()),
            #[cfg(not(feature = "embedded"))]
            Node::Number(num) => Some(match num {
                Numeric::Integer(v) => *v as f32,
                Numeric::Float(v) => *v as f32,
                Numeric::UInteger(v) => *v as f32,
                Numeric::Byte(v) => *v as f32,
                Numeric::Int32(v) => *v as f32,
                Numeric::UInt32(v) => *v as f32,
                Numeric::Int16(v) => *v as f32,
                Numeric::UInt16(v) => *v as f32,
                Numeric::Int8(v) => *v as f32,
                Numeric::UInt8(v) => *v as f32,
            }),
            _ => None,
        }
    }

    /// Safely get a string value from a string node
    ///
    /// Returns None if the node is not a string.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let string = Node::from("hello");
    /// assert_eq!(string.as_str(), Some("hello"));
    /// let number = Node::from(42);
    /// assert_eq!(number.as_str(), None);
    /// ```
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s, _, _) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Safely get a boolean value
    ///
    /// Returns None if the node is not a boolean.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let bool_node = Node::from(true);
    /// assert_eq!(bool_node.as_bool(), Some(true));
    /// let number = Node::from(42);
    /// assert_eq!(number.as_bool(), None);
    /// ```
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if this node is a string
    ///
    /// Returns true for Node::Str variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let string = Node::from("hello");
    /// assert!(string.is_string());
    /// let number = Node::from(42);
    /// assert!(!number.is_string());
    /// ```
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Node::Str(_, _, _))
    }

    /// Check if this node is a number
    ///
    /// Returns true for Node::Number variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let number = Node::from(42);
    /// assert!(number.is_number());
    /// let string = Node::from("text");
    /// assert!(!string.is_number());
    /// ```
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Node::Number(_))
    }

    /// Check if this node is a boolean
    ///
    /// Returns true for Node::Boolean variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let bool_node = Node::from(true);
    /// assert!(bool_node.is_boolean());
    /// let number = Node::from(42);
    /// assert!(!number.is_boolean());
    /// ```
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Node::Boolean(_))
    }

    /// Alias for is_string() for consistency with as_str()
    ///
    /// Returns true for Node::Str variants.
    #[inline]
    pub fn is_str(&self) -> bool {
        matches!(self, Node::Str(_, _, _))
    }

    /// Check if this node is an array
    ///
    /// Returns true for Node::Array variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![Node::from(1)]);
    /// assert!(array.is_array());
    /// let number = Node::from(42);
    /// assert!(!number.is_array());
    /// ```
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Node::Array(_))
    }

    /// Check if this node is a set
    ///
    /// Returns true for Node::Set variants.
    #[inline]
    pub fn is_set(&self) -> bool {
        matches!(self, Node::Set(_))
    }

    /// Check if this node is None (null)
    ///
    /// Returns true for Node::None variants.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let none = Node::None;
    /// assert!(none.is_none());
    /// let number = Node::from(42);
    /// assert!(!number.is_none());
    /// ```
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, Node::None)
    }

    /// Try to get an array/set as a slice
    ///
    /// Returns None if the node is not an array or set.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// assert_eq!(array.as_slice().map(|s| s.len()), Some(2));
    /// let mapping = Node::Mapping(vec![]);
    /// assert!(mapping.as_slice().is_none());
    /// ```
    #[inline]
    pub fn as_slice(&self) -> Option<&[Node]> {
        match self {
            Node::Array(arr) => Some(arr.as_slice()),
            Node::Set(set) => Some(set.as_slice()),
            _ => None,
        }
    }

    /// Try to get a mapping as a slice of key-value pairs
    ///
    /// Returns None if the node is not a mapping.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// assert_eq!(mapping.as_mapping().map(|m| m.len()), Some(1));
    /// let array = Node::Array(vec![]);
    /// assert!(array.as_mapping().is_none());
    /// ```
    #[inline]
    pub fn as_mapping(&self) -> Option<&[(Node, Node)]> {
        match self {
            Node::Mapping(pairs) => Some(pairs.as_slice()),
            _ => None,
        }
    }

    /// Check if a mapping contains a specific key
    ///
    /// Returns false if the node is not a mapping.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key"), Node::from("value"))
    /// ]);
    /// assert!(mapping.contains_key("key"));
    /// assert!(!mapping.contains_key("nonexistent"));
    /// ```
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.get_key(key).is_some()
    }

    /// Get all keys from a mapping as strings
    ///
    /// Returns an empty vector if the node is not a mapping or if keys are not strings.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let mapping = Node::Mapping(vec![
    ///     (Node::from("key1"), Node::from("value1")),
    ///     (Node::from("key2"), Node::from("value2"))
    /// ]);
    /// let keys = mapping.keys();
    /// assert_eq!(keys.len(), 2);
    /// assert!(keys.contains(&"key1"));
    /// ```
    pub fn keys(&self) -> alloc::vec::Vec<&str> {
        match self {
            Node::Mapping(pairs) => {
                let mut keys = alloc::vec::Vec::with_capacity(pairs.len());
                for (k, _) in pairs {
                    if let Node::Str(s, _, _) = k {
                        keys.push(s.as_str());
                    }
                }
                keys
            }
            _ => alloc::vec::Vec::new(),
        }
    }

    /// Create a new ArrayBuilder for fluent array construction
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::array()
    ///     .push(1)
    ///     .push(2)
    ///     .push(3)
    ///     .build();
    /// ```
    pub fn array() -> ArrayBuilder {
        ArrayBuilder::new()
    }

    /// Create a new MappingBuilder for fluent mapping construction
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let config = Node::mapping()
    ///     .insert("host", "localhost")
    ///     .insert("port", 8080)
    ///     .build();
    /// ```
    pub fn mapping() -> MappingBuilder {
        MappingBuilder::new()
    }

    /// Create a new SetBuilder for fluent set construction
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let set = Node::set()
    ///     .insert(1)
    ///     .insert(2)
    ///     .insert(3)
    ///     .build();
    /// ```
    pub fn set() -> SetBuilder {
        SetBuilder::new()
    }
}
