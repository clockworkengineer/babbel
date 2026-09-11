//! YAML Node Conversions
//!
//! Implements `From` traits for building `Node` instances from primitive types
//! and bidirectional conversions with `babbel_core::model::Value`.
//!
//! Copyright (c) 2026 YAML Library Developers

use super::types::{BlockStyle, Node, NodeStringConvert, Numeric, QuoteType};
use alloc::vec::Vec;

/// Converts a vector of values into an array node
#[cfg(feature = "alloc")]
impl<T: Into<Node>> From<Vec<T>> for Node {
    fn from(value: Vec<T>) -> Self {
        Node::Array(value.into_iter().map(|x| x.into()).collect())
    }
}

#[cfg(feature = "alloc")]
impl From<&Node> for babbel_core::model::Value {
    fn from(node: &Node) -> Self {
        match node {
            Node::None => babbel_core::model::Value::Null,
            Node::Boolean(b) => babbel_core::model::Value::Bool(*b),
            Node::Str(s, _, _) => babbel_core::model::Value::String(s.clone()),
            Node::Number(n) => match n {
                Numeric::Integer(i) => babbel_core::model::Value::Integer(*i as i128),
                Numeric::UInteger(u) => babbel_core::model::Value::Integer(*u as i128),
                Numeric::Byte(b) => babbel_core::model::Value::Integer(*b as i128),
                Numeric::Int32(i) => babbel_core::model::Value::Integer(*i as i128),
                Numeric::UInt32(u) => babbel_core::model::Value::Integer(*u as i128),
                Numeric::Int16(i) => babbel_core::model::Value::Integer(*i as i128),
                Numeric::UInt16(u) => babbel_core::model::Value::Integer(*u as i128),
                Numeric::Int8(i) => babbel_core::model::Value::Integer(*i as i128),
                Numeric::UInt8(u) => babbel_core::model::Value::Integer(*u as i128),
                Numeric::Float(f) => babbel_core::model::Value::Float(*f),
            },
            Node::Array(arr) | Node::Set(arr) => {
                babbel_core::model::Value::Array(arr.iter().map(babbel_core::model::Value::from).collect())
            }
            Node::Document(arr) | Node::Documents(arr) => {
                if arr.len() == 1 {
                    babbel_core::model::Value::from(&arr[0])
                } else {
                    babbel_core::model::Value::Array(arr.iter().map(babbel_core::model::Value::from).collect())
                }
            }
            Node::Mapping(pairs) => {
                let entries: alloc::vec::Vec<(alloc::string::String, babbel_core::model::Value)> = pairs
                    .iter()
                    .map(|(k, v)| (k.to_string_lossy(), babbel_core::model::Value::from(v)))
                    .collect();
                babbel_core::model::Value::Object(entries)
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                babbel_core::model::Value::from(&**inner)
            }
            Node::Alias(s) => babbel_core::model::Value::String(s.clone()),
            Node::Comment(_) => babbel_core::model::Value::Null,
        }
    }
}

#[cfg(feature = "alloc")]
impl From<Node> for babbel_core::model::Value {
    fn from(node: Node) -> Self {
        babbel_core::model::Value::from(&node)
    }
}

impl From<i64> for Node {
    fn from(value: i64) -> Self {
        Node::Number(Numeric::Integer(value))
    }
}

#[cfg(feature = "alloc")]
impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Str(
            alloc::string::String::from(value),
            QuoteType::Unquoted,
            BlockStyle::None,
        )
    }
}

impl From<f64> for Node {
    fn from(value: f64) -> Self {
        Node::Number(Numeric::Float(value))
    }
}

impl From<u64> for Node {
    fn from(value: u64) -> Self {
        Node::Number(Numeric::UInteger(value))
    }
}

impl From<u8> for Node {
    fn from(value: u8) -> Self {
        Node::Number(Numeric::Byte(value))
    }
}

impl From<i32> for Node {
    fn from(value: i32) -> Self {
        Node::Number(Numeric::Int32(value))
    }
}

impl From<u32> for Node {
    fn from(value: u32) -> Self {
        Node::Number(Numeric::UInt32(value))
    }
}

impl From<i16> for Node {
    fn from(value: i16) -> Self {
        Node::Number(Numeric::Int16(value))
    }
}

impl From<u16> for Node {
    fn from(value: u16) -> Self {
        Node::Number(Numeric::UInt16(value))
    }
}

impl From<i8> for Node {
    fn from(value: i8) -> Self {
        Node::Number(Numeric::Int8(value))
    }
}

impl From<bool> for Node {
    fn from(value: bool) -> Self {
        Node::Boolean(value)
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for Node {
    fn from(value: alloc::string::String) -> Self {
        Node::Str(value, QuoteType::Unquoted, BlockStyle::None)
    }
}
