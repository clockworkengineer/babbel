//! Format converters serializing XML `Document` to other data formats (JSON, YAML, TOML, Bencode).

use alloc::string::String;
use alloc::vec::Vec;
use babbel_core::io::traits::IDestination;
use babbel_core::model::Value;
use crate::document::Document;
use crate::error::XmlError;

/// Converts an XML Document DOM tree to JSON format.
pub fn to_json(doc: &Document, dest: &mut dyn IDestination) -> Result<(), XmlError> {
    let val = Value::from(doc);
    val.serialize_json(dest);
    Ok(())
}

/// Converts an XML Document DOM tree to an owned JSON String.
pub fn to_json_string(doc: &Document) -> Result<String, XmlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_json(doc, &mut dest)?;
    Ok(dest.to_string())
}

/// Converts an XML Document DOM tree to YAML format.
pub fn to_yaml(doc: &Document, dest: &mut dyn IDestination) -> Result<(), XmlError> {
    let val = Value::from(doc);
    val.serialize_yaml(dest, 0);
    Ok(())
}

/// Converts an XML Document DOM tree to an owned YAML String.
pub fn to_yaml_string(doc: &Document) -> Result<String, XmlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_yaml(doc, &mut dest)?;
    Ok(dest.to_string())
}

/// Converts an XML Document DOM tree to TOML format.
pub fn to_toml(doc: &Document, dest: &mut dyn IDestination) -> Result<(), XmlError> {
    let val = Value::from(doc);
    val.serialize_toml(dest);
    Ok(())
}

/// Converts an XML Document DOM tree to an owned TOML String.
pub fn to_toml_string(doc: &Document) -> Result<String, XmlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_toml(doc, &mut dest)?;
    Ok(dest.to_string())
}

/// Converts an XML Document DOM tree to Bencode format.
pub fn to_bencode(doc: &Document, dest: &mut dyn IDestination) -> Result<(), XmlError> {
    let val = Value::from(doc);
    val.serialize_bencode(dest);
    Ok(())
}

/// Converts an XML Document DOM tree to an owned Bencode byte vector.
pub fn to_bencode_bytes(doc: &Document) -> Result<Vec<u8>, XmlError> {
    let mut dest = babbel_core::io::Buffer::new();
    to_bencode(doc, &mut dest)?;
    Ok(dest.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_converters() {
        let xml = "<root attr=\"val\"><child>hello</child></root>";
        let doc = Document::parse_str(xml).expect("parse xml");

        let json = to_json_string(&doc).expect("to_json_string");
        assert!(json.contains("\"child\""));
        assert!(json.contains("\"hello\""));

        let yaml = to_yaml_string(&doc).expect("to_yaml_string");
        assert!(yaml.contains("child: hello"));

        let toml = to_toml_string(&doc).expect("to_toml_string");
        assert!(toml.contains("child = \"hello\""));

        let bencode = to_bencode_bytes(&doc).expect("to_bencode_bytes");
        assert!(!bencode.is_empty());

        // Test method syntax on doc
        let json_from_doc = doc.to_json_string().unwrap();
        assert_eq!(json, json_from_doc);
    }
}

