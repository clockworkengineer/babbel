//! Format converters serializing TOML `Node` to other data formats (JSON, YAML, XML, Bencode).

use babbel_core::io::traits::IDestination;
use babbel_core::model::Value;
use crate::error::TomlError;
use crate::nodes::node::Node;

/// Converts a TOML Node tree to JSON format.
pub fn to_json(node: &Node, dest: &mut dyn IDestination) -> Result<(), TomlError> {
    let val = Value::from(node);
    val.serialize_json(dest);
    Ok(())
}

/// Converts a TOML Node tree to an owned JSON String.
pub fn to_json_string(node: &Node) -> Result<alloc::string::String, TomlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_json(node, &mut dest)?;
    Ok(dest.to_string())
}

/// Converts a TOML Node tree to YAML format.
pub fn to_yaml(node: &Node, dest: &mut dyn IDestination) -> Result<(), TomlError> {
    let val = Value::from(node);
    val.serialize_yaml(dest, 0);
    Ok(())
}

/// Converts a TOML Node tree to an owned YAML String.
pub fn to_yaml_string(node: &Node) -> Result<alloc::string::String, TomlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_yaml(node, &mut dest)?;
    Ok(dest.to_string())
}

/// Converts a TOML Node tree to XML format.
pub fn to_xml(node: &Node, dest: &mut dyn IDestination) -> Result<(), TomlError> {
    let val = Value::from(node);
    val.serialize_xml(dest, None);
    Ok(())
}

/// Converts a TOML Node tree to an owned XML String.
pub fn to_xml_string(node: &Node) -> Result<alloc::string::String, TomlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_xml(node, &mut dest)?;
    Ok(dest.to_string())
}

/// Converts a TOML Node tree to Bencode format.
pub fn to_bencode(node: &Node, dest: &mut dyn IDestination) -> Result<(), TomlError> {
    let val = Value::from(node);
    val.serialize_bencode(dest);
    Ok(())
}

/// Converts a TOML Node tree to an owned Bencode byte vector.
pub fn to_bencode_bytes(node: &Node) -> Result<alloc::vec::Vec<u8>, TomlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_bencode(node, &mut dest)?;
    Ok(dest.into_vec())
}
