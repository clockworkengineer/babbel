//! Cross-format conversion matrix for the Babbel ecosystem.
//!
//! Provides effortless conversion pipelines between JSON, YAML, XML, and Bencode.

#[cfg(feature = "json")]
use json_lib;
#[cfg(feature = "yaml")]
use yaml_lib;
#[cfg(feature = "bencode")]
use bencode_lib;

/// Supported serialization format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Bencode,
}

/// Convert JSON string to YAML string.
#[cfg(all(feature = "json", feature = "yaml"))]
pub fn json_to_yaml(json: &str) -> Result<String, String> {
    let node = json_lib::from_str(json).map_err(|e| e.to_string())?;
    let mut dest = json_lib::BufferDestination::new();
    json_lib::to_yaml(&node, &mut dest)?;
    Ok(dest.to_string())
}

/// Convert JSON string to XML string.
#[cfg(all(feature = "json", feature = "xml"))]
pub fn json_to_xml(json: &str) -> Result<String, String> {
    let node = json_lib::from_str(json).map_err(|e| e.to_string())?;
    let mut dest = json_lib::BufferDestination::new();
    json_lib::to_xml(&node, &mut dest)?;
    Ok(dest.to_string())
}

/// Convert JSON string to Bencode byte vector.
#[cfg(all(feature = "json", feature = "bencode"))]
pub fn json_to_bencode(json: &str) -> Result<Vec<u8>, String> {
    let node = json_lib::from_str(json).map_err(|e| e.to_string())?;
    let mut dest = json_lib::BufferDestination::new();
    json_lib::to_bencode(&node, &mut dest)?;
    Ok(dest.buffer)
}

/// Convert YAML string to JSON string.
#[cfg(all(feature = "yaml", feature = "json"))]
pub fn yaml_to_json(yaml: &str) -> Result<String, String> {
    let node = yaml_lib::parse_string(yaml).map_err(|e| e.to_string())?;
    let mut dest = yaml_lib::BufferDestination::new();
    yaml_lib::to_json(&node, &mut dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string())
}

/// Convert YAML string to XML string.
#[cfg(all(feature = "yaml", feature = "xml"))]
pub fn yaml_to_xml(yaml: &str) -> Result<String, String> {
    let node = yaml_lib::parse_string(yaml).map_err(|e| e.to_string())?;
    let mut dest = yaml_lib::BufferDestination::new();
    yaml_lib::to_xml(&node, &mut dest).map_err(|e| e.to_string())?;
    Ok(dest.to_string())
}

/// Convert Bencode binary payload to JSON string.
#[cfg(all(feature = "bencode", feature = "json"))]
pub fn bencode_to_json(bencode: &[u8]) -> Result<String, String> {
    let node = bencode_lib::parse_bytes(bencode).map_err(|e| e.to_string())?;
    let mut dest = bencode_lib::BufferDestination::new();
    bencode_lib::to_json(&node, &mut dest)?;
    Ok(dest.to_string())
}

/// Convert Bencode binary payload to YAML string.
#[cfg(all(feature = "bencode", feature = "yaml"))]
pub fn bencode_to_yaml(bencode: &[u8]) -> Result<String, String> {
    let node = bencode_lib::parse_bytes(bencode).map_err(|e| e.to_string())?;
    let mut dest = bencode_lib::BufferDestination::new();
    bencode_lib::to_yaml(&node, &mut dest)?;
    Ok(dest.to_string())
}

/// Convert Bencode binary payload to XML string.
#[cfg(all(feature = "bencode", feature = "xml"))]
pub fn bencode_to_xml(bencode: &[u8]) -> Result<String, String> {
    let node = bencode_lib::parse_bytes(bencode).map_err(|e| e.to_string())?;
    let mut dest = bencode_lib::BufferDestination::new();
    bencode_lib::to_xml(&node, &mut dest)?;
    Ok(dest.to_string())
}
