#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

use core::fmt;
use super::types::{Node, Numeric};

/// Converts a vector of values into an array node
impl<T: Into<Node>> From<Vec<T>> for Node {
    fn from(value: Vec<T>) -> Self {
        Node::Array(value.into_iter().map(|x| x.into()).collect())
    }
}

impl From<i64> for Numeric {
    fn from(value: i64) -> Self {
        Numeric::Integer(value)
    }
}

impl From<f64> for Numeric {
    fn from(value: f64) -> Self {
        Numeric::Float(value)
    }
}

impl From<u64> for Numeric {
    fn from(value: u64) -> Self {
        Numeric::UInteger(value)
    }
}

impl From<u8> for Numeric {
    fn from(value: u8) -> Self {
        Numeric::Byte(value)
    }
}

impl From<i32> for Numeric {
    fn from(value: i32) -> Self {
        Numeric::Int32(value)
    }
}

impl From<u32> for Numeric {
    fn from(value: u32) -> Self {
        Numeric::UInt32(value)
    }
}

impl From<i16> for Numeric {
    fn from(value: i16) -> Self {
        Numeric::Int16(value)
    }
}

impl From<u16> for Numeric {
    fn from(value: u16) -> Self {
        Numeric::UInt16(value)
    }
}

impl From<i8> for Numeric {
    fn from(value: i8) -> Self {
        Numeric::Int8(value)
    }
}

impl From<i64> for Node {
    fn from(value: i64) -> Self {
        Node::Number(Numeric::Integer(value))
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::Str(String::from(value))
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

impl From<String> for Node {
    fn from(value: String) -> Self {
        Node::Str(value)
    }
}

/// Convert from Option<T> where T: Into<Node>
impl<T: Into<Node>> From<Option<T>> for Node {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Node::None,
        }
    }
}

/// Convert from fixed-size array [T; N]
impl<T: Into<Node> + Clone, const N: usize> From<[T; N]> for Node {
    fn from(arr: [T; N]) -> Self {
        Node::Array(arr.into_iter().map(|x| x.into()).collect())
    }
}

/// Convert from slice &[T]
impl<T: Into<Node> + Clone> From<&[T]> for Node {
    fn from(slice: &[T]) -> Self {
        Node::Array(slice.iter().cloned().map(|x| x.into()).collect())
    }
}

/// Convert from HashMap<String, T> where T: Into<Node>
impl<T: Into<Node>> From<HashMap<String, T>> for Node {
    fn from(map: HashMap<String, T>) -> Self {
        Node::Object(map.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

// Display implementations for better debugging
impl fmt::Display for Numeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Numeric::Integer(n) => write!(f, "{}", n),
            Numeric::Float(n) => write!(f, "{}", n),
            Numeric::UInteger(n) => write!(f, "{}", n),
            Numeric::Byte(n) => write!(f, "{}", n),
            Numeric::Int32(n) => write!(f, "{}", n),
            Numeric::UInt32(n) => write!(f, "{}", n),
            Numeric::Int16(n) => write!(f, "{}", n),
            Numeric::UInt16(n) => write!(f, "{}", n),
            Numeric::Int8(n) => write!(f, "{}", n),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::None => write!(f, "null"),
            Node::Boolean(b) => write!(f, "{}", b),
            Node::Number(n) => write!(f, "{}", n),
            Node::Str(s) => write!(f, "\"{}\"", s),
            Node::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Node::Object(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            }
        }
    }
}

// TryFrom implementations for extracting values from Node
use core::convert::TryFrom;

impl TryFrom<Node> for String {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        match node {
            Node::Str(s) => Ok(s),
            _ => Err("Node is not a string"),
        }
    }
}

impl TryFrom<Node> for Vec<Node> {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        match node {
            Node::Array(vec) => Ok(vec),
            _ => Err("Node is not an array"),
        }
    }
}

impl TryFrom<Node> for HashMap<String, Node> {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        match node {
            Node::Object(map) => Ok(map),
            _ => Err("Node is not an object"),
        }
    }
}

impl TryFrom<Node> for i64 {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        node.as_i64()
            .ok_or("Node is not a number or cannot be converted to i64")
    }
}

impl TryFrom<&Node> for i64 {
    type Error = &'static str;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        node.as_i64()
            .ok_or("Node is not a number or cannot be converted to i64")
    }
}

impl TryFrom<Node> for f64 {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        node.as_f64()
            .ok_or("Node is not a number or cannot be converted to f64")
    }
}

impl TryFrom<&Node> for f64 {
    type Error = &'static str;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        node.as_f64()
            .ok_or("Node is not a number or cannot be converted to f64")
    }
}

impl TryFrom<Node> for u64 {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        node.as_u64()
            .ok_or("Node is not a number or cannot be converted to u64")
    }
}

impl TryFrom<&Node> for u64 {
    type Error = &'static str;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        node.as_u64()
            .ok_or("Node is not a number or cannot be converted to u64")
    }
}

impl TryFrom<Node> for bool {
    type Error = &'static str;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        match node {
            Node::Boolean(b) => Ok(b),
            _ => Err("Node is not a boolean"),
        }
    }
}

impl TryFrom<&Node> for bool {
    type Error = &'static str;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        node.as_bool().ok_or("Node is not a boolean")
    }
}

