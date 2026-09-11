//! YAML Node Operations & Indexing
//!
//! Implements indexing traits (`Index`, `IndexMut`) for sequence and mapping access.
//!
//! Copyright (c) 2026 YAML Library Developers

use super::types::Node;

#[cfg(feature = "std")]
use std::ops::{Index, IndexMut};

#[cfg(not(feature = "std"))]
use core::ops::{Index, IndexMut};

/// Implements array-style indexing for Node using integer indices
#[cfg(feature = "alloc")]
impl Index<usize> for Node {
    type Output = Node;

    /// Allows accessing array elements using array[index] syntax
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node::Array(arr) => &arr[index],
            Node::Set(set) => &set[index],
            _ => panic!("Cannot index non-array/set node with integer"),
        }
    }
}

/// Implements mapping-style indexing for Node using string keys
#[cfg(feature = "alloc")]
impl Index<&str> for Node {
    type Output = Node;

    /// Allows accessing mapping properties using mapping["key"] syntax
    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return v;
                        }
                    }
                }
                panic!("No such key exists");
            }
            _ => panic!("Cannot index non-mapping node with string"),
        }
    }
}

/// Implements mutable array-style indexing for Node
#[cfg(feature = "alloc")]
impl IndexMut<usize> for Node {
    /// Allows modifying array elements using array[index] = value syntax
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Node::Array(arr) => &mut arr[index],
            Node::Set(set) => &mut set[index],
            _ => panic!("Cannot index non-array/set node with integer"),
        }
    }
}

/// Implements mutable mapping-style indexing for Node
#[cfg(feature = "alloc")]
impl IndexMut<&str> for Node {
    /// Allows modifying mapping properties using mapping["key"] = value syntax
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        match self {
            Node::Mapping(pairs) => {
                for (k, v) in pairs.iter_mut() {
                    if let Node::Str(s, _, _) = k {
                        if s == key {
                            return v;
                        }
                    }
                }
                panic!("No such key exists");
            }
            _ => panic!("Cannot index non-mapping node with string"),
        }
    }
}
