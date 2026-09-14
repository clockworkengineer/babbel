//! Cross-format conversion matrix for the Babbel ecosystem.
//!
//! Provides effortless conversion pipelines between JSON, YAML, XML, Bencode,
//! CSV, TSV, INI, and JSON Lines.

#[cfg(feature = "json")]
use babbel_json::{JsonEngine, JsonLinesEngine};
#[cfg(feature = "yaml")]
use babbel_yaml::YamlEngine;
#[cfg(feature = "bencode")]
use babbel_bencode::BencodeEngine;
#[cfg(feature = "xml")]
use babbel_xml::XmlEngine;
#[cfg(feature = "toml")]
use babbel_toml::TomlEngine;
#[cfg(feature = "msgpack")]
use babbel_msgpack::MsgPackEngine;

use babbel_core::{
    csv::emit_csv_to, ini::emit_ini_to, parse_csv, parse_ini, BabbelError, Buffer, BufferDestination,
    CsvEngine, CsvOptions, FormatEmitter, FormatEngine, FormatOptions, FormatParser, IniEngine,
    IniOptions, JsonEmitter, TomlEmitter, TsvEngine, Value,
};

/// Supported serialization format identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Json,
    Yaml,
    Xml,
    Bencode,
    Toml,
    MsgPack,
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

/// MessagePack parser implementing `FormatParser`.
#[cfg(feature = "msgpack")]
#[derive(Debug, Default, Clone, Copy)]
pub struct MsgPackParser;

#[cfg(feature = "msgpack")]
impl FormatParser for MsgPackParser {
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        MsgPackEngine.parse_bytes(input.as_bytes())
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        MsgPackEngine.parse_bytes(input)
    }
}

/// MessagePack emitter implementing `FormatEmitter`.
#[cfg(feature = "msgpack")]
#[derive(Debug, Default, Clone, Copy)]
pub struct MsgPackEmitter;

#[cfg(feature = "msgpack")]
impl FormatEmitter for MsgPackEmitter {
    fn emit(&self, value: &Value, destination: &mut dyn babbel_core::IDestination) -> Result<(), BabbelError> {
        MsgPackEngine.serialize(value, destination, &FormatOptions::compact())
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

/// Convert XML string to JSON string.
#[cfg(all(feature = "xml", feature = "json"))]
pub fn xml_to_json(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert XML string to YAML string.
#[cfg(all(feature = "xml", feature = "yaml"))]
pub fn xml_to_yaml(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert XML string to Bencode byte vector.
#[cfg(all(feature = "xml", feature = "bencode"))]
pub fn xml_to_bencode(xml: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(xml.as_bytes(), &XmlEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert YAML string to Bencode byte vector.
#[cfg(all(feature = "yaml", feature = "bencode"))]
pub fn yaml_to_bencode(yaml: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(yaml.as_bytes(), &YamlEngine, &BencodeEngine, &ConversionOptions::default())
}


// =========================================================================
// Text Format Conversions (CSV, TSV, INI, JSON Lines)
// =========================================================================

/// Convert CSV string to JSON array string.
#[cfg(feature = "json")]
pub fn csv_to_json(csv: &str) -> Result<String, BabbelError> {
    convert_format(csv, &CsvEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert JSON string (array of objects or rows) to CSV string.
#[cfg(feature = "json")]
pub fn json_to_csv(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &CsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to JSON array string.
#[cfg(feature = "json")]
pub fn tsv_to_json(tsv: &str) -> Result<String, BabbelError> {
    convert_format(tsv, &TsvEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert JSON string (array of objects or rows) to TSV string.
#[cfg(feature = "json")]
pub fn json_to_tsv(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert CSV string to YAML string.
#[cfg(feature = "yaml")]
pub fn csv_to_yaml(csv: &str) -> Result<String, BabbelError> {
    convert_format(csv, &CsvEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert YAML string to CSV string.
#[cfg(feature = "yaml")]
pub fn yaml_to_csv(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &CsvEngine, &ConversionOptions::default())
}

/// Convert CSV string to XML string.
#[cfg(feature = "xml")]
pub fn csv_to_xml(csv: &str) -> Result<String, BabbelError> {
    convert_format(csv, &CsvEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert XML string to CSV string.
#[cfg(feature = "xml")]
pub fn xml_to_csv(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &CsvEngine, &ConversionOptions::default())
}

/// Convert CSV string to Bencode byte vector.
#[cfg(feature = "bencode")]
pub fn csv_to_bencode(csv: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(csv.as_bytes(), &CsvEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to CSV string.
#[cfg(feature = "bencode")]
pub fn bencode_to_csv(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &CsvEngine, &ConversionOptions::default())
}

/// Convert CSV string to INI configuration string.
pub fn csv_to_ini(csv: &str) -> Result<String, BabbelError> {
    convert_format(csv, &CsvEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert INI configuration string to CSV string.
pub fn ini_to_csv(ini: &str) -> Result<String, BabbelError> {
    convert_format(ini, &IniEngine, &CsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to YAML string.
#[cfg(feature = "yaml")]
pub fn tsv_to_yaml(tsv: &str) -> Result<String, BabbelError> {
    convert_format(tsv, &TsvEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert YAML string to TSV string.
#[cfg(feature = "yaml")]
pub fn yaml_to_tsv(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to XML string.
#[cfg(feature = "xml")]
pub fn tsv_to_xml(tsv: &str) -> Result<String, BabbelError> {
    convert_format(tsv, &TsvEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert XML string to TSV string.
#[cfg(feature = "xml")]
pub fn xml_to_tsv(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to TOML string.
#[cfg(feature = "toml")]
pub fn tsv_to_toml(tsv: &str) -> Result<String, BabbelError> {
    convert_format(tsv, &TsvEngine, &TomlEngine, &ConversionOptions::default())
}

/// Convert TOML string to TSV string.
#[cfg(feature = "toml")]
pub fn toml_to_tsv(toml: &str) -> Result<String, BabbelError> {
    convert_format(toml, &TomlEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to Bencode byte vector.
#[cfg(feature = "bencode")]
pub fn tsv_to_bencode(tsv: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(tsv.as_bytes(), &TsvEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to TSV string.
#[cfg(feature = "bencode")]
pub fn bencode_to_tsv(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to INI configuration string.
pub fn tsv_to_ini(tsv: &str) -> Result<String, BabbelError> {
    convert_format(tsv, &TsvEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert INI configuration string to TSV string.
pub fn ini_to_tsv(ini: &str) -> Result<String, BabbelError> {
    convert_format(ini, &IniEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert TSV string to JSON Lines string.
#[cfg(feature = "json")]
pub fn tsv_to_jsonlines(tsv: &str) -> Result<String, BabbelError> {
    convert_format(tsv, &TsvEngine, &JsonLinesEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to TSV string.
#[cfg(feature = "json")]
pub fn jsonlines_to_tsv(jsonl: &str) -> Result<String, BabbelError> {
    convert_format(jsonl, &JsonLinesEngine, &TsvEngine, &ConversionOptions::default())
}

/// Convert INI string to JSON object string.
#[cfg(feature = "json")]
pub fn ini_to_json(ini: &str) -> Result<String, BabbelError> {
    convert_format(ini, &IniEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert JSON object string to INI configuration string.
#[cfg(feature = "json")]
pub fn json_to_ini(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert INI string to YAML string.
#[cfg(feature = "yaml")]
pub fn ini_to_yaml(ini: &str) -> Result<String, BabbelError> {
    convert_format(ini, &IniEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert YAML object string to INI configuration string.
#[cfg(feature = "yaml")]
pub fn yaml_to_ini(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert INI string to XML string.
#[cfg(feature = "xml")]
pub fn ini_to_xml(ini: &str) -> Result<String, BabbelError> {
    convert_format(ini, &IniEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert XML string to INI configuration string.
#[cfg(feature = "xml")]
pub fn xml_to_ini(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert INI string to Bencode byte vector.
#[cfg(feature = "bencode")]
pub fn ini_to_bencode(ini: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(ini.as_bytes(), &IniEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to INI string.
#[cfg(feature = "bencode")]
pub fn bencode_to_ini(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert INI string to JSON Lines string.
#[cfg(feature = "json")]
pub fn ini_to_jsonlines(ini: &str) -> Result<String, BabbelError> {
    convert_format(ini, &IniEngine, &JsonLinesEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to INI string.
#[cfg(feature = "json")]
pub fn jsonlines_to_ini(jsonl: &str) -> Result<String, BabbelError> {
    convert_format(jsonl, &JsonLinesEngine, &IniEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to JSON array string.
#[cfg(feature = "json")]
pub fn jsonlines_to_json(jsonl: &str) -> Result<String, BabbelError> {
    convert_format(jsonl, &JsonLinesEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert JSON string to JSON Lines string.
#[cfg(feature = "json")]
pub fn json_to_jsonlines(json: &str) -> Result<String, BabbelError> {
    convert_format(json, &JsonEngine, &JsonLinesEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to CSV string.
#[cfg(feature = "json")]
pub fn jsonlines_to_csv(jsonl: &str) -> Result<String, BabbelError> {
    convert_format(jsonl, &JsonLinesEngine, &CsvEngine, &ConversionOptions::default())
}

/// Convert CSV string to JSON Lines string.
#[cfg(feature = "json")]
pub fn csv_to_jsonlines(csv: &str) -> Result<String, BabbelError> {
    convert_format(csv, &CsvEngine, &JsonLinesEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to YAML string.
#[cfg(all(feature = "json", feature = "yaml"))]
pub fn jsonlines_to_yaml(jsonl: &str) -> Result<String, BabbelError> {
    convert_format(jsonl, &JsonLinesEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert YAML string to JSON Lines string.
#[cfg(all(feature = "yaml", feature = "json"))]
pub fn yaml_to_jsonlines(yaml: &str) -> Result<String, BabbelError> {
    convert_format(yaml, &YamlEngine, &JsonLinesEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to XML string.
#[cfg(all(feature = "json", feature = "xml"))]
pub fn jsonlines_to_xml(jsonl: &str) -> Result<String, BabbelError> {
    convert_format(jsonl, &JsonLinesEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert XML string to JSON Lines string.
#[cfg(all(feature = "xml", feature = "json"))]
pub fn xml_to_jsonlines(xml: &str) -> Result<String, BabbelError> {
    convert_format(xml, &XmlEngine, &JsonLinesEngine, &ConversionOptions::default())
}

/// Convert JSON Lines string to Bencode byte vector.
#[cfg(all(feature = "json", feature = "bencode"))]
pub fn jsonlines_to_bencode(jsonl: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(jsonl.as_bytes(), &JsonLinesEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to JSON Lines string.
#[cfg(all(feature = "bencode", feature = "json"))]
pub fn bencode_to_jsonlines(bencode: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(bencode, &BencodeEngine, &JsonLinesEngine, &ConversionOptions::default())
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

// =========================================================================
// MessagePack Cross-Format Conversions
// =========================================================================

/// Convert JSON string to MessagePack byte vector.
#[cfg(all(feature = "json", feature = "msgpack"))]
pub fn json_to_msgpack(json: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(json.as_bytes(), &JsonEngine, &MsgPackEngine, &ConversionOptions::default())
}

/// Convert MessagePack byte payload to JSON string.
#[cfg(all(feature = "msgpack", feature = "json"))]
pub fn msgpack_to_json(msgpack: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(msgpack, &MsgPackEngine, &JsonEngine, &ConversionOptions::default())
}

/// Convert YAML string to MessagePack byte vector.
#[cfg(all(feature = "yaml", feature = "msgpack"))]
pub fn yaml_to_msgpack(yaml: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(yaml.as_bytes(), &YamlEngine, &MsgPackEngine, &ConversionOptions::default())
}

/// Convert MessagePack byte payload to YAML string.
#[cfg(all(feature = "msgpack", feature = "yaml"))]
pub fn msgpack_to_yaml(msgpack: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(msgpack, &MsgPackEngine, &YamlEngine, &ConversionOptions::default())
}

/// Convert XML string to MessagePack byte vector.
#[cfg(all(feature = "xml", feature = "msgpack"))]
pub fn xml_to_msgpack(xml: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(xml.as_bytes(), &XmlEngine, &MsgPackEngine, &ConversionOptions::default())
}

/// Convert MessagePack byte payload to XML string.
#[cfg(all(feature = "msgpack", feature = "xml"))]
pub fn msgpack_to_xml(msgpack: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(msgpack, &MsgPackEngine, &XmlEngine, &ConversionOptions::default())
}

/// Convert TOML string to MessagePack byte vector.
#[cfg(all(feature = "toml", feature = "msgpack"))]
pub fn toml_to_msgpack(toml: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(toml.as_bytes(), &TomlEngine, &MsgPackEngine, &ConversionOptions::default())
}

/// Convert MessagePack byte payload to TOML string.
#[cfg(all(feature = "msgpack", feature = "toml"))]
pub fn msgpack_to_toml(msgpack: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(msgpack, &MsgPackEngine, &TomlEngine, &ConversionOptions::default())
}

/// Convert Bencode binary payload to MessagePack byte vector.
#[cfg(all(feature = "bencode", feature = "msgpack"))]
pub fn bencode_to_msgpack(bencode: &[u8]) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(bencode, &BencodeEngine, &MsgPackEngine, &ConversionOptions::default())
}

/// Convert MessagePack byte payload to Bencode byte vector.
#[cfg(all(feature = "msgpack", feature = "bencode"))]
pub fn msgpack_to_bencode(msgpack: &[u8]) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(msgpack, &MsgPackEngine, &BencodeEngine, &ConversionOptions::default())
}

/// Convert CSV string to MessagePack byte vector.
#[cfg(feature = "msgpack")]
pub fn csv_to_msgpack(csv: &str) -> Result<Vec<u8>, BabbelError> {
    convert_format_bytes(csv.as_bytes(), &CsvEngine, &MsgPackEngine, &ConversionOptions::default())
}

/// Convert MessagePack byte payload to CSV string.
#[cfg(feature = "msgpack")]
pub fn msgpack_to_csv(msgpack: &[u8]) -> Result<String, BabbelError> {
    convert_format_bytes_to_str(msgpack, &MsgPackEngine, &CsvEngine, &ConversionOptions::default())
}


