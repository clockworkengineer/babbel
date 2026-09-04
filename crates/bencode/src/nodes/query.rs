//! Extension trait for schema validation and typed field querying on `Node` structures.
//! Segregates schema validation responsibilities from the core AST data representation (SRP & ISP compliant).

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::String,
    vec::Vec,
    collections::BTreeMap as HashMap,
};

#[cfg(feature = "std")]
use std::collections::HashMap;

use crate::nodes::node::Node;

/// Trait providing dictionary schema validation and field extraction operations.
pub trait NodeQueryExt {
    /// Get a required field from a dictionary, returning an error if not found.
    fn get_required(&self, key: &str) -> Result<&Node, String>;
    /// Get a required integer field from a dictionary.
    fn get_int_required(&self, key: &str) -> Result<i64, String>;
    /// Get a required string field from a dictionary.
    fn get_string_required(&self, key: &str) -> Result<&str, String>;
    /// Get a required list field from a dictionary.
    fn get_list_required(&self, key: &str) -> Result<&Vec<Node>, String>;
    /// Get a required dictionary field from a dictionary.
    fn get_dict_required(&self, key: &str) -> Result<&HashMap<String, Node>, String>;

    /// Get an optional integer field, returning None if not found or not an integer.
    fn get_int_optional(&self, key: &str) -> Option<i64>;
    /// Get an optional string field, returning None if not found or not a string.
    fn get_string_optional(&self, key: &str) -> Option<&str>;
    /// Get an optional list field, returning None if not found or not a list.
    fn get_list_optional(&self, key: &str) -> Option<&Vec<Node>>;
    /// Get an optional dictionary field, returning None if not found or not a dictionary.
    fn get_dict_optional(&self, key: &str) -> Option<&HashMap<String, Node>>;
}

impl NodeQueryExt for Node {
    fn get_required(&self, key: &str) -> Result<&Node, String> {
        self.get(key)
            .ok_or_else(|| format!("Missing required field: '{}'", key))
    }

    fn get_int_required(&self, key: &str) -> Result<i64, String> {
        self.get_required(key)?
            .as_integer()
            .copied()
            .ok_or_else(|| format!("Field '{}' must be an integer", key))
    }

    fn get_string_required(&self, key: &str) -> Result<&str, String> {
        self.get_required(key)?
            .as_string()
            .ok_or_else(|| format!("Field '{}' must be a string", key))
    }

    fn get_list_required(&self, key: &str) -> Result<&Vec<Node>, String> {
        self.get_required(key)?
            .as_list()
            .ok_or_else(|| format!("Field '{}' must be a list", key))
    }

    fn get_dict_required(&self, key: &str) -> Result<&HashMap<String, Node>, String> {
        self.get_required(key)?
            .as_dictionary()
            .ok_or_else(|| format!("Field '{}' must be a dictionary", key))
    }

    fn get_int_optional(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|n| n.as_integer()).copied()
    }

    fn get_string_optional(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|n| n.as_string())
    }

    fn get_list_optional(&self, key: &str) -> Option<&Vec<Node>> {
        self.get(key).and_then(|n| n.as_list())
    }

    fn get_dict_optional(&self, key: &str) -> Option<&HashMap<String, Node>> {
        self.get(key).and_then(|n| n.as_dictionary())
    }
}
