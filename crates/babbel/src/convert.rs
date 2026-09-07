//! Cross-format conversion matrix for the Babbel ecosystem.
//!
//! Provides effortless conversion pipelines between JSON, YAML, XML, Bencode,
//! CSV, TSV, INI, and JSON Lines.

#[cfg(feature = "json")]
use json_lib;
#[cfg(feature = "yaml")]
use yaml_lib;
#[cfg(feature = "bencode")]
use bencode_lib;
#[cfg(feature = "xml")]
use xml_lib;

use babbel_core::{
    csv::emit_csv_to, ini::emit_ini_to, parse_csv, parse_ini, BabbelError, Buffer,
    CsvOptions, FormatEmitter, FormatParser, IniOptions, JsonEmitter, Value, YamlEmitter,
};

/// Supported serialization format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Bencode,
    Csv,
    Tsv,
    Ini,
    JsonLines,
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
        let doc = xml_lib::parse(input)?;
        let name = doc.get_root_element_name().unwrap_or("xml").to_string();
        Ok(Value::Object(vec![(name, Value::String(input.to_string()))]))
    }
}

/// CSV parser implementing `FormatParser`.
#[derive(Debug, Default, Clone)]
pub struct CsvParser {
    pub options: CsvOptions,
}

impl FormatParser for CsvParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        parse_csv(input, &self.options)
    }
}

/// CSV emitter implementing `FormatEmitter`.
#[derive(Debug, Default, Clone)]
pub struct CsvEmitter {
    pub options: CsvOptions,
}

impl FormatEmitter for CsvEmitter {
    fn emit(&self, value: &Value, destination: &mut dyn babbel_core::IDestination) -> Result<(), BabbelError> {
        emit_csv_to(value, &self.options, destination)
    }
}

/// TSV parser implementing `FormatParser`.
#[derive(Debug, Clone)]
pub struct TsvParser {
    pub options: CsvOptions,
}

impl Default for TsvParser {
    fn default() -> Self {
        Self {
            options: CsvOptions::tsv(),
        }
    }
}

impl FormatParser for TsvParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        parse_csv(input, &self.options)
    }
}

/// TSV emitter implementing `FormatEmitter`.
#[derive(Debug, Clone)]
pub struct TsvEmitter {
    pub options: CsvOptions,
}

impl Default for TsvEmitter {
    fn default() -> Self {
        Self {
            options: CsvOptions::tsv(),
        }
    }
}

impl FormatEmitter for TsvEmitter {
    fn emit(&self, value: &Value, destination: &mut dyn babbel_core::IDestination) -> Result<(), BabbelError> {
        emit_csv_to(value, &self.options, destination)
    }
}

/// INI parser implementing `FormatParser`.
#[derive(Debug, Default, Clone)]
pub struct IniParser {
    pub options: IniOptions,
}

impl FormatParser for IniParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        parse_ini(input, &self.options)
    }
}

/// INI emitter implementing `FormatEmitter`.
#[derive(Debug, Default, Clone)]
pub struct IniEmitter {
    pub options: IniOptions,
}

impl FormatEmitter for IniEmitter {
    fn emit(&self, value: &Value, destination: &mut dyn babbel_core::IDestination) -> Result<(), BabbelError> {
        emit_ini_to(value, &self.options, destination)
    }
}

/// JSON Lines parser implementing `FormatParser`.
#[cfg(feature = "json")]
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonLinesParser;

#[cfg(feature = "json")]
impl FormatParser for JsonLinesParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let nodes = json_lib::parse_json_lines(input).map_err(BabbelError::syntax)?;
        let values: Vec<Value> = nodes.iter().map(Value::from).collect();
        Ok(Value::Array(values))
    }
}

/// JSON Lines emitter implementing `FormatEmitter`.
#[cfg(feature = "json")]
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonLinesEmitter;

#[cfg(feature = "json")]
impl FormatEmitter for JsonLinesEmitter {
    fn emit(&self, value: &Value, destination: &mut dyn babbel_core::IDestination) -> Result<(), BabbelError> {
        let records = match value {
            Value::Array(arr) => arr.as_slice(),
            single => core::slice::from_ref(single),
        };
        for r in records {
            let mut buf = babbel_core::BufferDestination::new();
            JsonEmitter.emit(r, &mut buf)?;
            destination.add_bytes(&buf.to_string());
            destination.add_byte(b'\n');
        }
        Ok(())
    }
}

// =========================================================================
// Convenience Cross-Format Conversion Functions
// =========================================================================

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

// =========================================================================
// Text Format Conversions (CSV, TSV, INI, JSON Lines)
// =========================================================================

/// Convert CSV string to JSON array string.
#[cfg(feature = "json")]
pub fn csv_to_json(csv: &str) -> Result<String, BabbelError> {
    convert_text(csv, &CsvParser::default(), &JsonEmitter)
}

/// Convert JSON string (array of objects or rows) to CSV string.
#[cfg(feature = "json")]
pub fn json_to_csv(json: &str) -> Result<String, BabbelError> {
    convert_text(json, &JsonParser, &CsvEmitter::default())
}

/// Convert TSV string to JSON array string.
#[cfg(feature = "json")]
pub fn tsv_to_json(tsv: &str) -> Result<String, BabbelError> {
    convert_text(tsv, &TsvParser::default(), &JsonEmitter)
}

/// Convert JSON string (array of objects or rows) to TSV string.
#[cfg(feature = "json")]
pub fn json_to_tsv(json: &str) -> Result<String, BabbelError> {
    convert_text(json, &JsonParser, &TsvEmitter::default())
}

/// Convert CSV string to YAML string.
#[cfg(feature = "yaml")]
pub fn csv_to_yaml(csv: &str) -> Result<String, BabbelError> {
    convert_text(csv, &CsvParser::default(), &YamlEmitter)
}

/// Convert YAML string to CSV string.
#[cfg(feature = "yaml")]
pub fn yaml_to_csv(yaml: &str) -> Result<String, BabbelError> {
    convert_text(yaml, &YamlParser, &CsvEmitter::default())
}

/// Convert INI string to JSON object string.
#[cfg(feature = "json")]
pub fn ini_to_json(ini: &str) -> Result<String, BabbelError> {
    convert_text(ini, &IniParser::default(), &JsonEmitter)
}

/// Convert JSON object string to INI configuration string.
#[cfg(feature = "json")]
pub fn json_to_ini(json: &str) -> Result<String, BabbelError> {
    convert_text(json, &JsonParser, &IniEmitter::default())
}

/// Convert INI string to YAML string.
#[cfg(feature = "yaml")]
pub fn ini_to_yaml(ini: &str) -> Result<String, BabbelError> {
    convert_text(ini, &IniParser::default(), &YamlEmitter)
}

/// Convert YAML object string to INI configuration string.
#[cfg(feature = "yaml")]
pub fn yaml_to_ini(yaml: &str) -> Result<String, BabbelError> {
    convert_text(yaml, &YamlParser, &IniEmitter::default())
}

/// Convert JSON Lines string to JSON array string.
#[cfg(feature = "json")]
pub fn jsonlines_to_json(jsonl: &str) -> Result<String, BabbelError> {
    convert_text(jsonl, &JsonLinesParser, &JsonEmitter)
}

/// Convert JSON string to JSON Lines string.
#[cfg(feature = "json")]
pub fn json_to_jsonlines(json: &str) -> Result<String, BabbelError> {
    convert_text(json, &JsonParser, &JsonLinesEmitter)
}

/// Convert JSON Lines string to CSV string.
#[cfg(feature = "json")]
pub fn jsonlines_to_csv(jsonl: &str) -> Result<String, BabbelError> {
    convert_text(jsonl, &JsonLinesParser, &CsvEmitter::default())
}

/// Convert CSV string to JSON Lines string.
#[cfg(feature = "json")]
pub fn csv_to_jsonlines(csv: &str) -> Result<String, BabbelError> {
    convert_text(csv, &CsvParser::default(), &JsonLinesEmitter)
}
