#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

use smallvec::SmallVec;

/// Represents different numeric types that can be stored in a JSON node
#[derive(Clone, Debug, PartialEq)]
pub enum Numeric {
    Integer(i64),  // 64-bit signed integer
    Float(f64),    // 64-bit floating point
    UInteger(u64), // 64-bit unsigned integer
    Byte(u8),      // 8-bit unsigned integer
    Int32(i32),    // 32-bit signed integer
    UInt32(u32),   // 32-bit unsigned integer
    Int16(i16),    // 16-bit signed integer
    UInt16(u16),   // 16-bit unsigned integer
    Int8(i8),      // 8-bit signed integer
}

/// A node in the JSON data structure that can represent different types of values.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// Represents a boolean value (true/false)
    Boolean(bool),
    /// Represents a numeric value (various integer and float types)
    Number(Numeric),
    /// Represents a string value
    Str(String),
    /// Represents an array of other nodes
    Array(Vec<Node>),
    /// Represents an object/map of string keys to node values
    Object(HashMap<String, Node>),
    /// Represents a null value or uninitialized node
    None,
}

impl Node {
    /// Creates a Node::Array from an iterator, using SmallVec for small arrays
    pub fn from_iter<I: IntoIterator<Item = Node>>(iter: I) -> Self {
        let mut small: SmallVec<[Node; 8]> = SmallVec::new();
        for item in iter {
            small.push(item);
        }
        Node::Array(small.into_vec())
    }

    /// Creates a Node::Array from a slice
    pub fn from_slice(slice: &[Node]) -> Self {
        Node::Array(slice.to_vec())
    }

    /// Creates a Node::Array from a `Vec<Node>` without cloning (zero-copy).
    pub fn from_vec(vec: Vec<Node>) -> Self {
        Node::Array(vec)
    }

    /// Creates a new empty object Node
    pub fn new_object() -> Self {
        Node::Object(HashMap::new())
    }

    /// Creates a new empty array Node
    pub fn new_array() -> Self {
        Node::Array(Vec::new())
    }
}
