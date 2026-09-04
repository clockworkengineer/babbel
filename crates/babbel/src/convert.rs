//! Cross-format conversion matrix for the Babbel ecosystem.
//!
//! Provides effortless conversion pipelines between JSON, YAML, XML, and Bencode.

#[cfg(feature = "json")]
use json_lib;
#[cfg(feature = "yaml")]
use yaml_lib;
#[cfg(feature = "bencode")]
use bencode_lib;
#[cfg(feature = "xml")]
use xml_lib_rust;

use babbel_core::{BabbelError, Buffer, FormatEmitter, FormatParser, Value};

/// Supported serialization format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Bencode,
}

/// Generic, open-ended conversion pipeline from any format parser to any format emitter (OCP & DIP).
pub fn convert_text<P: FormatParser, E: FormatEmitter>(
    input: &str,
    parser: &P,
    emitter: &E,
) -> Result<String, BabbelError> {
    let value = parser.parse_str(input)?;
    let mut dest = Buffer::new();
    emitter.emit(&value, &mut dest)?;
    Ok(dest.to_string())
}

/// Generic, open-ended conversion pipeline from byte inputs to byte outputs (OCP & DIP).
pub fn convert_bytes<P: FormatParser, E: FormatEmitter>(
    input: &[u8],
    parser: &P,
    emitter: &E,
) -> Result<Vec<u8>, BabbelError> {
    let value = parser.parse_bytes(input)?;
    let mut dest = Buffer::new();
    emitter.emit(&value, &mut dest)?;
    Ok(dest.into_vec())
}

/// JSON parser implementing `FormatParser`.
#[cfg(feature = "json")]
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonParser;

#[cfg(feature = "json")]
impl FormatParser for JsonParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let node = json_lib::from_str(input)?;
        Ok(Value::from(&node))
    }
}

/// YAML parser implementing `FormatParser`.
#[cfg(feature = "yaml")]
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlParser;

#[cfg(feature = "yaml")]
impl FormatParser for YamlParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let node = yaml_lib::parse_string(input)?;
        Ok(Value::from(&node))
    }
}

/// Bencode parser implementing `FormatParser`.
#[cfg(feature = "bencode")]
#[derive(Debug, Default, Clone, Copy)]
pub struct BencodeParser;

#[cfg(feature = "bencode")]
impl FormatParser for BencodeParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        self.parse_bytes(input.as_bytes())
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        let node = bencode_lib::parse_bytes(input)?;
        Ok(Value::from(&node))
    }
}

/// XML parser implementing `FormatParser`.
#[cfg(feature = "xml")]
#[derive(Debug, Default, Clone, Copy)]
pub struct XmlParser;

#[cfg(feature = "xml")]
impl FormatParser for XmlParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let doc = xml_lib_rust::parse(input)?;
        let name = doc.get_root_element_name().unwrap_or("xml").to_string();
        Ok(Value::Object(vec![(name, Value::String(input.to_string()))]))
    }
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
