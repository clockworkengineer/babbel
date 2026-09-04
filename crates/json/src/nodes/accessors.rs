#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

use core::mem;
use super::types::{Node, Numeric};

impl Node {
    /// Safely gets a value from an object by key without panicking
    pub fn get(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Safely gets a mutable reference to a value from an object by key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Node> {
        match self {
            Node::Object(map) => map.get_mut(key),
            _ => None,
        }
    }

    /// Safely gets a value from an array by index
    pub fn at(&self, index: usize) -> Option<&Node> {
        match self {
            Node::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// Safely gets a mutable reference to a value from an array by index
    pub fn at_mut(&mut self, index: usize) -> Option<&mut Node> {
        match self {
            Node::Array(arr) => arr.get_mut(index),
            _ => None,
        }
    }

    /// Returns true if this node is an object
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, Node::Object(_))
    }

    /// Returns true if this node is an array
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, Node::Array(_))
    }

    /// Returns true if this node is a string
    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Node::Str(_))
    }

    /// Returns true if this node is a number
    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Node::Number(_))
    }

    /// Returns true if this node is a boolean
    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Node::Boolean(_))
    }

    /// Returns true if this node is None/null
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Node::None)
    }

    /// Returns an iterator over the keys of an object
    #[inline]
    pub fn keys(&self) -> Option<impl Iterator<Item = &str>> {
        match self {
            Node::Object(map) => Some(map.keys().map(|s| s.as_str())),
            _ => None,
        }
    }

    /// Returns an iterator over the values if this is an object
    #[inline]
    pub fn object_values(&self) -> Option<impl Iterator<Item = &Node>> {
        match self {
            Node::Object(map) => Some(map.values()),
            _ => None,
        }
    }

    /// Returns a mutable iterator over object values
    #[inline]
    pub fn object_values_mut(&mut self) -> Option<impl Iterator<Item = &mut Node>> {
        match self {
            Node::Object(map) => Some(map.values_mut()),
            _ => None,
        }
    }

    /// Returns an iterator over array elements
    #[inline]
    pub fn array_iter(&self) -> Option<impl Iterator<Item = &Node>> {
        match self {
            Node::Array(vec) => Some(vec.iter()),
            _ => None,
        }
    }

    /// Returns a mutable iterator over array elements
    #[inline]
    pub fn array_iter_mut(&mut self) -> Option<impl Iterator<Item = &mut Node>> {
        match self {
            Node::Array(vec) => Some(vec.iter_mut()),
            _ => None,
        }
    }

    /// Returns the string value if this node is a Str, None otherwise
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the boolean value if this node is a Boolean, None otherwise
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the number value if this node is a Number, None otherwise
    #[inline]
    pub fn as_number(&self) -> Option<&Numeric> {
        match self {
            Node::Number(n) => Some(n),
            _ => None,
        }
    }

    /// Converts the node to i64 if it's a numeric type
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Node::Number(Numeric::Integer(n)) => Some(*n),
            Node::Number(Numeric::Int32(n)) => Some(*n as i64),
            Node::Number(Numeric::Int16(n)) => Some(*n as i64),
            Node::Number(Numeric::Int8(n)) => Some(*n as i64),
            Node::Number(Numeric::UInteger(n)) => Some(*n as i64),
            Node::Number(Numeric::UInt32(n)) => Some(*n as i64),
            Node::Number(Numeric::UInt16(n)) => Some(*n as i64),
            Node::Number(Numeric::Byte(n)) => Some(*n as i64),
            Node::Number(Numeric::Float(f)) => Some(*f as i64),
            _ => None,
        }
    }

    /// Converts the node to f64 if it's a numeric type
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Node::Number(Numeric::Float(f)) => Some(*f),
            Node::Number(Numeric::Integer(n)) => Some(*n as f64),
            Node::Number(Numeric::Int32(n)) => Some(*n as f64),
            Node::Number(Numeric::Int16(n)) => Some(*n as f64),
            Node::Number(Numeric::Int8(n)) => Some(*n as f64),
            Node::Number(Numeric::UInteger(n)) => Some(*n as f64),
            Node::Number(Numeric::UInt32(n)) => Some(*n as f64),
            Node::Number(Numeric::UInt16(n)) => Some(*n as f64),
            Node::Number(Numeric::Byte(n)) => Some(*n as f64),
            _ => None,
        }
    }

    /// Converts the node to u64 if it's a numeric type
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Node::Number(Numeric::UInteger(n)) => Some(*n),
            Node::Number(Numeric::UInt32(n)) => Some(*n as u64),
            Node::Number(Numeric::UInt16(n)) => Some(*n as u64),
            Node::Number(Numeric::Byte(n)) => Some(*n as u64),
            Node::Number(Numeric::Integer(n)) if *n >= 0 => Some(*n as u64),
            Node::Number(Numeric::Int32(n)) if *n >= 0 => Some(*n as u64),
            Node::Number(Numeric::Int16(n)) if *n >= 0 => Some(*n as u64),
            Node::Number(Numeric::Int8(n)) if *n >= 0 => Some(*n as u64),
            Node::Number(Numeric::Float(f)) if *f >= 0.0 => Some(*f as u64),
            _ => None,
        }
    }

    /// Returns the array reference if this node is an Array, None otherwise
    #[inline]
    pub fn as_array(&self) -> Option<&Vec<Node>> {
        match self {
            Node::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Returns a mutable array reference if this node is an Array, None otherwise
    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Node::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Returns the object reference if this node is an Object, None otherwise
    #[inline]
    pub fn as_object(&self) -> Option<&HashMap<String, Node>> {
        match self {
            Node::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a mutable object reference if this node is an Object, None otherwise
    #[inline]
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Node>> {
        match self {
            Node::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Consumes the node and returns the string if this is a Str variant
    #[inline]
    pub fn into_string(self) -> Option<String> {
        match self {
            Node::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Consumes the node and returns the array if this is an Array variant
    #[inline]
    pub fn into_array(self) -> Option<Vec<Node>> {
        match self {
            Node::Array(vec) => Some(vec),
            _ => None,
        }
    }

    /// Consumes the node and returns the object if this is an Object variant
    #[inline]
    pub fn into_object(self) -> Option<HashMap<String, Node>> {
        match self {
            Node::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Consumes the node and returns the number if this is a Number variant
    #[inline]
    pub fn into_number(self) -> Option<Numeric> {
        match self {
            Node::Number(n) => Some(n),
            _ => None,
        }
    }

    /// Returns the length of an array or object, None for other types
    #[inline]
    pub fn len(&self) -> Option<usize> {
        match self {
            Node::Array(arr) => Some(arr.len()),
            Node::Object(map) => Some(map.len()),
            _ => None,
        }
    }

    /// Returns true if this node is an empty array or object
    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            Node::Array(arr) => arr.is_empty(),
            Node::Object(map) => map.is_empty(),
            _ => false,
        }
    }

    /// Creates a new empty object Node
    pub fn object() -> Self {
        Node::Object(HashMap::new())
    }

    /// Creates a new empty array Node
    pub fn array() -> Self {
        Node::Array(Vec::new())
    }

    /// Creates a new null Node
    pub fn null() -> Self {
        Node::None
    }

    /// Takes the value out of the Node, leaving Node::None in its place
    pub fn take(&mut self) -> Node {
        mem::replace(self, Node::None)
    }

    /// Inserts a key-value pair into an object
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Node>) -> Option<Node> {
        match self {
            Node::Object(map) => map.insert(key.into(), value.into()),
            _ => None,
        }
    }
}
