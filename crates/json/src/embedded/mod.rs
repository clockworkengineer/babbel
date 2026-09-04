//! Embedded utilities module providing common patterns and helpers for embedded systems

pub mod builders;

pub use builders::{ArrayBuilder, ObjectBuilder};

/// Helper functions for creating sensor data JSON structures
pub mod sensor {
    use super::*;
    use crate::nodes::types::{Node, Numeric};

    #[cfg(feature = "std")]
    use std::collections::HashMap;

    #[cfg(not(feature = "std"))]
    use alloc::{
        collections::BTreeMap as HashMap,
        string::{String, ToString},
        vec::Vec,
    };

    /// Creates a simple sensor reading with timestamp
    pub fn simple_reading(device_id: &str, value: f64, timestamp: i64) -> Node {
        ObjectBuilder::new()
            .add_str("device", device_id)
            .add_f64("value", value)
            .add_i64("timestamp", timestamp)
            .build()
    }

    /// Creates a sensor reading with multiple values
    pub fn multi_reading(device_id: &str, values: &[(&str, f64)], timestamp: i64) -> Node {
        let mut builder = ObjectBuilder::new()
            .add_str("device", device_id)
            .add_i64("timestamp", timestamp);

        let mut readings = HashMap::new();
        for (key, value) in values {
            readings.insert(key.to_string(), Node::Number(Numeric::Float(*value)));
        }

        builder = builder.add_node("readings", Node::Object(readings));
        builder.build()
    }

    /// Creates a batch of sensor readings
    pub fn batch_readings(device_id: &str, readings: Vec<Node>) -> Node {
        ObjectBuilder::new()
            .add_str("device", device_id)
            .add_node("readings", Node::Array(readings))
            .build()
    }
}

/// Helper functions for configuration management
pub mod config {
    use super::*;
    use crate::nodes::types::{Node, Numeric};

    /// Creates a simple configuration object
    pub fn simple() -> ObjectBuilder {
        ObjectBuilder::new()
    }

    /// Extracts a string configuration value safely
    pub fn get_string<'a>(node: &'a Node, key: &str) -> Option<&'a str> {
        node.get(key).and_then(|n| n.as_str())
    }

    /// Extracts an integer configuration value safely
    pub fn get_i32(node: &Node, key: &str) -> Option<i32> {
        node.get(key).and_then(|n| {
            if let Node::Number(Numeric::Int32(i)) = n {
                Some(*i)
            } else {
                None
            }
        })
    }

    /// Extracts a boolean configuration value safely
    pub fn get_bool(node: &Node, key: &str) -> Option<bool> {
        node.get(key).and_then(|n| n.as_bool())
    }

    /// Extracts a float configuration value safely
    pub fn get_f64(node: &Node, key: &str) -> Option<f64> {
        node.get(key).and_then(|n| {
            if let Node::Number(Numeric::Float(f)) = n {
                Some(*f)
            } else {
                None
            }
        })
    }
}

/// Memory usage estimation helpers
pub mod memory {
    use crate::nodes::types::{Node, Numeric};
    use core::mem::size_of;

    /// Estimates the memory usage of a Node in bytes
    pub fn estimate_node_size(node: &Node) -> usize {
        match node {
            Node::Boolean(_) => size_of::<Node>(),
            Node::Number(_) => size_of::<Node>(),
            Node::None => size_of::<Node>(),
            Node::Str(s) => size_of::<Node>() + s.capacity(),
            Node::Array(arr) => {
                size_of::<Node>()
                    + size_of::<Node>() * arr.capacity()
                    + arr
                        .iter()
                        .map(|n| estimate_node_size(n) - size_of::<Node>())
                        .sum::<usize>()
            }
            Node::Object(map) => {
                size_of::<Node>()
                    + estimate_map_overhead(map.len())
                    + map
                        .iter()
                        .map(|(k, v)| k.capacity() + estimate_node_size(v) - size_of::<Node>())
                        .sum::<usize>()
            }
        }
    }

    /// Estimates HashMap/BTreeMap overhead
    fn estimate_map_overhead(len: usize) -> usize {
        48 + (len * 80)
    }

    /// Returns the size of the Node enum
    pub fn node_size() -> usize {
        size_of::<Node>()
    }

    /// Returns the size of the Numeric enum
    pub fn numeric_size() -> usize {
        size_of::<Numeric>()
    }
}
