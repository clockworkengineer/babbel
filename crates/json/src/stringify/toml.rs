//! TOML stringification module for Node structures
//!
//! Powered by universal `babbel_core::model::Value` and `TomlEmitter`.

use crate::io::traits::IDestination;
use crate::Node;
use alloc::string::{String, ToString};

/// Converts a Node structure to a TOML formatted string
///
/// # Arguments
/// * `node` - The root Node to convert
/// * `destination` - The destination to write the TOML string to
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if the root node is not an Object
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Object(_) => {
            let val = babbel_core::model::Value::from(node);
            babbel_core::emitters::TomlEmitter.emit_to_dest(&val, destination);
            Ok(())
        }
        _ => Err("TOML format requires a Object at the root level".to_string()),
    }
}
