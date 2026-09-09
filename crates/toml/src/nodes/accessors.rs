//! Accessor methods for extracting underlying values from `Node`.

use alloc::string::String;

use super::datetime::TomlDatetime;
use super::node::Node;

impl Node {
    /// Returns string slice if node is `Node::String`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Node::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns 64-bit integer if node is `Node::Integer`.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Node::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns floating-point number if node is `Node::Float`.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Node::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Returns boolean if node is `Node::Boolean`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Node::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns datetime reference if node is `Node::Datetime`.
    pub fn as_datetime(&self) -> Option<&TomlDatetime> {
        match self {
            Node::Datetime(dt) => Some(dt),
            _ => None,
        }
    }

    /// Returns slice of elements if node is `Node::Array`.
    pub fn as_array(&self) -> Option<&[Node]> {
        match self {
            Node::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    /// Returns slice of key-value pairs if node is `Node::Table`.
    pub fn as_table(&self) -> Option<&[(String, Node)]> {
        match self {
            Node::Table(t) => Some(t.as_slice()),
            _ => None,
        }
    }
}
