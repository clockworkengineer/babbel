//! Cross-format conversion matrix for the Babbel ecosystem.
//!
//! Provides effortless conversion pipelines between JSON, YAML, XML, Bencode,
//! CSV, TSV, INI, and JSON Lines.

#[cfg(feature = "json")]
use babbel_json::JsonEngine;
#[cfg(feature = "yaml")]
use babbel_yaml::YamlEngine;
#[cfg(feature = "bencode")]
use babbel_bencode::BencodeEngine;
#[cfg(feature = "xml")]
use babbel_xml::XmlEngine;
#[cfg(feature = "toml")]
use babbel_toml::TomlEngine;

use babbel_core::{
    csv::emit_csv_to, ini::emit_ini_to, parse_csv, parse_ini, BabbelError, Buffer, BufferDestination,
    CsvOptions, FormatEmitter, FormatEngine, FormatOptions, FormatParser, IniOptions, JsonEmitter,
    TomlEmitter, Value, YamlEmitter,
};

/// Supported serialization format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Bencode,
    Toml,
    Csv,
    Tsv,
    Ini,
    JsonLines,
}

/// Configuration options for cross-format conversions.
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Whether to format output with pretty-printing and indentation.
    pub pretty: bool,
    /// Number of spaces for pretty indentation.
    pub indent: usize,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            indent: 2,
        }
    }
}

impl ConversionOptions {
    /// Create options with pretty-printing enabled.
    pub fn pretty() -> Self {
        Self {
            pretty: true,
            indent: 2,
        }
    }

    /// Set custom indentation spaces.
    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }
}

/// Universal cross-format text conversion pipeline adhering to OCP and DIP.
///
/// Parses input text via `from` engine, translates into universal `Value`, and serializes via `to` engine.
pub fn convert_format<F: FormatEngine + ?Sized, T: FormatEngine + ?Sized>(
    input: &str,
    from: &F,
    to: &T,
    options: &ConversionOptions,
) -> Result<String, BabbelError> {
    let value = from.parse_str(input)?;
    let mut dest = BufferDestination::new();
    let format_opts = FormatOptions {
        pretty: options.pretty,
        indent: options.indent,
    };
    to.serialize(&value, &mut dest, &format_opts)?;
    dest.into_string()
        .map_err(|_| BabbelError::encoding("output is not valid UTF-8"))
}

/// Universal cross-format byte conversion pipeline adhering to OCP and DIP.
///
/// Parses input bytes via `from` engine, translates into universal `Value`, and serializes via `to` engine.
pub fn convert_format_bytes<F: FormatEngine + ?Sized, T: FormatEngine + ?Sized>(
    input: &[u8],
    from: &F,
    to: &T,
    options: &ConversionOptions,
) -> Result<Vec<u8>, BabbelError> {
    let value = from.parse_bytes(input)?;
    let mut dest = BufferDestination::new();
    let format_opts = FormatOptions {
        pretty: options.pretty,
        indent: options.indent,
    };
    to.serialize(&value, &mut dest, &format_opts)?;
    Ok(dest.into_vec())
}

/// Universal cross-format byte-to-string conversion pipeline adhering to OCP and DIP.
///
/// Parses input bytes via `from` engine, translates into universal `Value`, and serializes via `to` engine into a UTF-8 string.
pub fn convert_format_bytes_to_str<F: FormatEngine + ?Sized, T: FormatEngine + ?Sized>(
    input: &[u8],
    from: &F,
    to: &T,
    options: &ConversionOptions,
) -> Result<String, BabbelError> {
    let bytes = convert_format_bytes(input, from, to, options)?;
    String::from_utf8(bytes).map_err(|_| BabbelError::encoding("output is not valid UTF-8"))
}

/// Generic, open-ended conversion pipeline from any format parser to any format emitter (OCP & DIP).
pub fn convert_text<P: FormatParser, E: FormatEmitter>(
    input: &str,
    parser: &P,
    emitter: &E,
) -> Result<String, BabbelError> {
    convert_text_with_options(input, parser, emitter, &ConversionOptions::default())
}

/// Generic, open-ended conversion pipeline with custom options (OCP & DIP).
pub fn convert_text_with_options<P: FormatParser, E: FormatEmitter>(
    input: &str,
    parser: &P,
    emitter: &E,
    options: &ConversionOptions,
) -> Result<String, BabbelError> {
    let value = parser.parse_str(input)?;
    let mut dest = Buffer::new();
    if options.pretty {
        emitter.emit_pretty(&value, &mut dest, options.indent)?;
    } else {
        emitter.emit(&value, &mut dest)?;
    }
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
        JsonEngine.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        JsonEngine.parse_bytes(input)
    }
}

/// YAML parser implementing `FormatParser`.
#[cfg(feature = "yaml")]
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlParser;

#[cfg(feature = "yaml")]
impl FormatParser for YamlParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        YamlEngine.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        YamlEngine.parse_bytes(input)
    }
}

/// Bencode parser implementing `FormatParser`.
#[cfg(feature = "bencode")]
#[derive(Debug, Default, Clone, Copy)]
pub struct BencodeParser;

#[cfg(feature = "bencode")]
impl FormatParser for BencodeParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        BencodeEngine.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        BencodeEngine.parse_bytes(input)
    }
}

/// XML parser implementing `FormatParser`.
#[cfg(feature = "xml")]
#[derive(Debug, Default, Clone, Copy)]
pub struct XmlParser;

#[cfg(feature = "xml")]
impl FormatParser for XmlParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        XmlEngine.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        XmlEngine.parse_bytes(input)
    }
}

/// TOML parser implementing `FormatParser`.
#[cfg(feature = "toml")]
#[derive(Debug, Default, Clone, Copy)]
pub struct TomlParser;

#[cfg(feature = "toml")]
impl FormatParser for TomlParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        TomlEngine.parse_str(input)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        TomlEngine.parse_bytes(input)
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
        let nodes = babbel_json::parse_json_lines(input).map_err(BabbelError::syntax)?;
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
pub fn json_to_yaml(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert JSON string to XML string.
#[cfg(all(feature = "json", feature = "xml"))]
pub fn json_to_xml(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert JSON string to Bencode byte vector.
#[cfg(all(feature = "json", feature = "bencode"))]
pub fn json_to_bencode(json: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(json.as_bytes(), &JsonEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert YAML string to JSON string.
#[cfg(all(feature = "yaml", feature = "json"))]
pub fn yaml_to_json(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert YAML string to XML string.
#[cfg(all(feature = "yaml", feature = "xml"))]
pub fn yaml_to_xml(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to JSON string.
#[cfg(all(feature = "bencode", feature = "json"))]
pub fn bencode_to_json(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to YAML string.
#[cfg(all(feature = "bencode", feature = "yaml"))]
pub fn bencode_to_yaml(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to XML string.
#[cfg(all(feature = "bencode", feature = "xml"))]
pub fn bencode_to_xml(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &XmlEngine, &ConversionOptions::default())
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

// =========================================================================
// TOML Cross-Format Conversions
// =========================================================================

/// Convert TOML string to JSON string.
#[cfg(all(feature = "toml", feature = "json"))]
pub fn toml_to_json(toml: &str) -> Result<String, BabbelError> {
    convert_format(toml, &TomlEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert JSON string to TOML string.
#[cfg(all(feature = "json", feature = "toml"))]
pub fn json_to_toml(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &TomlEngine, &ConversionOptions::default())
}

/// Convert TOML string to YAML string.
#[cfg(all(feature = "toml", feature = "yaml"))]
pub fn toml_to_yaml(toml: &str) -> Result<String, BabbelError> {
    convert_format(toml, &TomlEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert YAML string to TOML string.
#[cfg(all(feature = "yaml", feature = "toml"))]
pub fn yaml_to_toml(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &TomlEngine, &ConversionOptions::default())
}

/// Convert TOML string to XML string.
#[cfg(all(feature = "toml", feature = "xml"))]
pub fn toml_to_xml(toml: &str) -> Result<String, BabbelError> {
    convert_format(toml, &TomlEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert XML string to TOML string.
#[cfg(all(feature = "xml", feature = "toml"))]
pub fn xml_to_toml(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &TomlEngine, &ConversionOptions::default())
}

/// Convert TOML string to Bencode byte vector.
#[cfg(all(feature = "toml", feature = "bencode"))]
pub fn toml_to_bencode(toml: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(toml.as_bytes(), &TomlEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to TOML string.
#[cfg(all(feature = "bencode", feature = "toml"))]
pub fn bencode_to_toml(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &TomlEngine, &ConversionOptions::default())
}

/// Convert TOML string to CSV string.
#[cfg(feature = "toml")]
pub fn toml_to_csv(toml: &str) -> Result<String, BabbelError> {
    convert_text(toml, &TomlParser, &CsvEmitter::default())
}

/// Convert CSV string to TOML string.
#[cfg(feature = "toml")]
pub fn csv_to_toml(csv: &str) -> Result<String, BabbelError> {
    convert_text(csv, &CsvParser::default(), &TomlEmitter)
}

/// Convert TOML string to INI string.
#[cfg(feature = "toml")]
pub fn toml_to_ini(toml: &str) -> Result<String, BabbelError> {
    convert_text(toml, &TomlParser, &IniEmitter::default())
}

/// Convert INI string to TOML string.
#[cfg(feature = "toml")]
pub fn ini_to_toml(ini: &str) -> Result<String, BabbelError> {
    convert_text(ini, &IniParser::default(), &TomlEmitter)
}

/// Convert TOML string to JSON Lines string.
#[cfg(all(feature = "toml", feature = "json"))]
pub fn toml_to_jsonlines(toml: &str) -> Result<String, BabbelError> {
    convert_text(toml, &TomlParser, &JsonLinesEmitter)
}

/// Convert JSON Lines string to TOML string.
#[cfg(all(feature = "json", feature = "toml"))]
pub fn jsonlines_to_toml(jsonl: &str) -> Result<String, BabbelError> {
    convert_text(jsonl, &JsonLinesParser, &TomlEmitter)
}

