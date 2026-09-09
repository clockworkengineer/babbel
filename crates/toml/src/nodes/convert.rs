//! Bi-directional conversions between `Node` and `babbel_core::Value`.

#[cfg(not(feature = "std"))]
use alloc::{string::String, string::ToString, vec::Vec};

use babbel_core::model::Value;

use super::datetime::TomlDatetime;
use super::node::Node;

impl From<&Node> for Value {
    fn from(node: &Node) -> Self {
        match node {
            Node::String(s) => Value::String(s.clone()),
            Node::Integer(i) => Value::Integer(*i as i128),
            Node::Float(f) => Value::Float(*f),
            Node::Boolean(b) => Value::Bool(*b),
            Node::Datetime(dt) => Value::String(dt.raw.clone()),
            Node::Array(arr) => Value::Array(arr.iter().map(Value::from).collect()),
            Node::Table(entries) => {
                let mapped: Vec<(String, Value)> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::from(v)))
                    .collect();
                Value::Object(mapped)
            }
        }
    }
}

impl From<Node> for Value {
    fn from(node: Node) -> Self {
        Value::from(&node)
    }
}

impl From<&Value> for Node {
    fn from(val: &Value) -> Self {
        match val {
            Value::Null => Node::String(String::new()),
            Value::Bool(b) => Node::Boolean(*b),
            Value::Integer(i) => Node::Integer(*i as i64),
            Value::Float(f) => Node::Float(*f),
            Value::String(s) => {
                if let Some(dt) = TomlDatetime::parse(s) {
                    Node::Datetime(dt)
                } else {
                    Node::String(s.clone())
                }
            }
            Value::Bytes(b) => Node::String(String::from_utf8_lossy(b).into_owned()),
            Value::Array(items) => Node::Array(items.iter().map(Node::from).collect()),
            Value::Object(entries) => {
                let mapped: Vec<(String, Node)> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), Node::from(v)))
                    .collect();
                Node::Table(mapped)
            }
        }
    }
}

impl From<Value> for Node {
    fn from(val: Value) -> Self {
        Node::from(&val)
    }
}

impl From<&str> for Node {
    fn from(s: &str) -> Self {
        Node::String(s.to_string())
    }
}

impl From<String> for Node {
    fn from(s: String) -> Self {
        Node::String(s)
    }
}

impl From<i64> for Node {
    fn from(i: i64) -> Self {
        Node::Integer(i)
    }
}

impl From<f64> for Node {
    fn from(f: f64) -> Self {
        Node::Float(f)
    }
}

impl From<bool> for Node {
    fn from(b: bool) -> Self {
        Node::Boolean(b)
    }
}
