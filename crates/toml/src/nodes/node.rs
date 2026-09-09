//! Core TOML DOM `Node` enumeration and constructors.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use super::datetime::TomlDatetime;

/// A node in the TOML data structure representing any valid TOML value.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// UTF-8 basic or literal string
    String(String),
    /// 64-bit signed integer
    Integer(i64),
    /// 64-bit floating point number (including inf, -inf, nan)
    Float(f64),
    /// Boolean flag (`true` or `false`)
    Boolean(bool),
    /// RFC 3339 Date/Time
    Datetime(TomlDatetime),
    /// Ordered sequence of TOML values
    Array(Vec<Node>),
    /// Key-value table preserving document order
    Table(Vec<(String, Node)>),
}

impl Node {
    /// Create a new empty Table node.
    pub fn new_table() -> Self {
        Node::Table(Vec::new())
    }

    /// Create a new empty Array node.
    pub fn new_array() -> Self {
        Node::Array(Vec::new())
    }

    /// Returns `true` if node is a Table.
    pub fn is_table(&self) -> bool {
        matches!(self, Node::Table(_))
    }

    /// Returns `true` if node is an Array.
    pub fn is_array(&self) -> bool {
        matches!(self, Node::Array(_))
    }

    /// Returns `true` if node is a String.
    pub fn is_str(&self) -> bool {
        matches!(self, Node::String(_))
    }

    /// Returns `true` if node is an Integer.
    pub fn is_integer(&self) -> bool {
        matches!(self, Node::Integer(_))
    }

    /// Returns `true` if node is a Float.
    pub fn is_float(&self) -> bool {
        matches!(self, Node::Float(_))
    }

    /// Returns `true` if node is a Boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, Node::Boolean(_))
    }

    /// Returns `true` if node is a Datetime.
    pub fn is_datetime(&self) -> bool {
        matches!(self, Node::Datetime(_))
    }

    /// Look up a value by key if node is a Table.
    pub fn get(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Table(entries) => {
                entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Look up a mutable value by key if node is a Table.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Node> {
        match self {
            Node::Table(entries) => {
                entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Insert or overwrite a key-value entry in a Table.
    pub fn insert(&mut self, key: String, val: Node) {
        if let Node::Table(entries) = self {
            if let Some(existing) = entries.iter_mut().find(|(k, _)| k == &key) {
                existing.1 = val;
            } else {
                entries.push((key, val));
            }
        }
    }

    /// Check if a Table contains the given key.
    pub fn contains_key(&self, key: &str) -> bool {
        match self {
            Node::Table(entries) => entries.iter().any(|(k, _)| k == key),
            _ => false,
        }
    }

    /// Push an element to an Array node.
    pub fn push(&mut self, val: Node) {
        if let Node::Array(items) = self {
            items.push(val);
        }
    }

    /// Number of elements in Table or Array, or length of String.
    pub fn len(&self) -> usize {
        match self {
            Node::Table(entries) => entries.len(),
            Node::Array(items) => items.len(),
            Node::String(s) => s.len(),
            _ => 0,
        }
    }

    /// Returns `true` if this container or string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// If this node is an array of tables, returns the active (last) table, otherwise self.
    pub fn get_nested_target_mut(&mut self) -> &mut Node {
        let is_arr_table = matches!(self, Node::Array(items) if !items.is_empty() && items.last().map_or(false, Node::is_table));
        if is_arr_table {
            if let Node::Array(items) = self {
                return items.last_mut().unwrap();
            }
        }
        self
    }
}
