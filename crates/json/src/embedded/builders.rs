use crate::nodes::types::{Node, Numeric};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec::Vec,
};

/// Builder for constructing JSON objects with a fluent API
pub struct ObjectBuilder {
    map: HashMap<String, Node>,
}

impl ObjectBuilder {
    /// Creates a new empty object builder
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates a builder with pre-allocated capacity (std version)
    #[cfg(feature = "std")]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    /// Creates a builder with pre-allocated capacity (no-std version)
    #[cfg(not(feature = "std"))]
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }

    /// Adds a string field
    pub fn add_str(mut self, key: &str, value: &str) -> Self {
        self.map
            .insert(key.to_string(), Node::Str(value.to_string()));
        self
    }

    /// Adds an i32 field
    pub fn add_i32(mut self, key: &str, value: i32) -> Self {
        self.map
            .insert(key.to_string(), Node::Number(Numeric::Int32(value)));
        self
    }

    /// Adds an i64 field
    pub fn add_i64(mut self, key: &str, value: i64) -> Self {
        self.map
            .insert(key.to_string(), Node::Number(Numeric::Integer(value)));
        self
    }

    /// Adds a u32 field
    pub fn add_u32(mut self, key: &str, value: u32) -> Self {
        self.map
            .insert(key.to_string(), Node::Number(Numeric::UInt32(value)));
        self
    }

    /// Adds a u64 field
    pub fn add_u64(mut self, key: &str, value: u64) -> Self {
        self.map
            .insert(key.to_string(), Node::Number(Numeric::UInteger(value)));
        self
    }

    /// Adds an f64 field
    pub fn add_f64(mut self, key: &str, value: f64) -> Self {
        self.map
            .insert(key.to_string(), Node::Number(Numeric::Float(value)));
        self
    }

    /// Adds a boolean field
    pub fn add_bool(mut self, key: &str, value: bool) -> Self {
        self.map.insert(key.to_string(), Node::Boolean(value));
        self
    }

    /// Adds a null field
    pub fn add_null(mut self, key: &str) -> Self {
        self.map.insert(key.to_string(), Node::None);
        self
    }

    /// Adds a nested Node
    pub fn add_node(mut self, key: &str, node: Node) -> Self {
        self.map.insert(key.to_string(), node);
        self
    }

    /// Adds an object field
    pub fn add_object(mut self, key: &str, map: HashMap<String, Node>) -> Self {
        self.map.insert(key.to_string(), Node::Object(map));
        self
    }

    /// Adds an array field
    pub fn add_array(mut self, key: &str, vec: Vec<Node>) -> Self {
        self.map.insert(key.to_string(), Node::Array(vec));
        self
    }

    /// Builds the final Node::Object
    pub fn build(self) -> Node {
        Node::Object(self.map)
    }
}

impl Default for ObjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing JSON arrays with a fluent API
pub struct ArrayBuilder {
    vec: Vec<Node>,
}

impl ArrayBuilder {
    /// Creates a new empty array builder
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    /// Creates a builder with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
        }
    }

    /// Pushes a string element
    pub fn push_str(mut self, value: &str) -> Self {
        self.vec.push(Node::Str(value.to_string()));
        self
    }

    /// Alias for push_str
    pub fn add_str(self, value: &str) -> Self {
        self.push_str(value)
    }

    /// Pushes an i32 element
    pub fn push_i32(mut self, value: i32) -> Self {
        self.vec.push(Node::Number(Numeric::Int32(value)));
        self
    }

    /// Alias for push_i32
    pub fn add_i32(self, value: i32) -> Self {
        self.push_i32(value)
    }

    /// Pushes an f64 element
    pub fn push_f64(mut self, value: f64) -> Self {
        self.vec.push(Node::Number(Numeric::Float(value)));
        self
    }

    /// Alias for push_f64
    pub fn add_f64(self, value: f64) -> Self {
        self.push_f64(value)
    }

    /// Pushes a boolean element
    pub fn push_bool(mut self, value: bool) -> Self {
        self.vec.push(Node::Boolean(value));
        self
    }

    /// Alias for push_bool
    pub fn add_bool(self, value: bool) -> Self {
        self.push_bool(value)
    }

    /// Pushes a null element
    pub fn push_null(mut self) -> Self {
        self.vec.push(Node::None);
        self
    }

    /// Alias for push_null
    pub fn add_null(self) -> Self {
        self.push_null()
    }

    /// Pushes a Node element
    pub fn push_node(mut self, node: Node) -> Self {
        self.vec.push(node);
        self
    }

    /// Alias for push_node
    pub fn add_node(self, node: Node) -> Self {
        self.push_node(node)
    }

    /// Builds the final Node::Array
    pub fn build(self) -> Node {
        Node::Array(self.vec)
    }
}

impl Default for ArrayBuilder {
    fn default() -> Self {
        Self::new()
    }
}
