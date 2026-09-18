//! Cross-format conversion matrix for the Babbel ecosystem.
//!
//! Provides effortless conversion pipelines between JSON, YAML, XML, Bencode,
//! CSV, TSV, INI, JSON Lines, TOML, MsgPack, CBOR, BSON, RON, KDL, and Parquet.

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "bencode")]
use babbel_bencode::BencodeEngine;
#[cfg(feature = "bson")]
use babbel_bson::BsonEngine;
#[cfg(feature = "cbor")]
use babbel_cbor::CborEngine;
#[cfg(feature = "json")]
use babbel_json::{Json5Engine, JsonEngine, JsonLinesEngine};
#[cfg(feature = "kdl")]
use babbel_kdl::KdlEngine;
#[cfg(feature = "msgpack")]
use babbel_msgpack::MsgPackEngine;
#[cfg(feature = "parquet")]
use babbel_parquet::ParquetEngine;
#[cfg(feature = "ron")]
use babbel_ron::RonEngine;
#[cfg(feature = "toml")]
use babbel_toml::TomlEngine;
#[cfg(feature = "xml")]
use babbel_xml::XmlEngine;
#[cfg(feature = "yaml")]
use babbel_yaml::YamlEngine;

pub use babbel_core::TomlEmitter;
use babbel_core::{
    BabbelError, Buffer, BufferDestination, CsvEngine, CsvOptions, FormatEmitter, FormatEngine,
    FormatOptions, FormatParser, IniEngine, IniOptions, JsonEmitter, TsvEngine, Value,
    csv::emit_csv_to, ini::emit_ini_to, parse_csv, parse_ini,
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
    Cbor,
    Bson,
    Ron,
    Kdl,
    Parquet,
    Json5,
    Csv,
    Tsv,
    Ini,
    JsonLines,
}

impl Format {
    /// Returns the canonical format string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Xml => "xml",
            Self::Bencode => "bencode",
            Self::Toml => "toml",
            Self::MsgPack => "msgpack",
            Self::Cbor => "cbor",
            Self::Bson => "bson",
            Self::Ron => "ron",
            Self::Kdl => "kdl",
            Self::Parquet => "parquet",
            Self::Json5 => "json5",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Ini => "ini",
            Self::JsonLines => "jsonlines",
        }
    }
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

// =========================================================================
// Universal Open Engine Pipelines (OCP & DIP)
// =========================================================================

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

// =========================================================================
// Dynamic Format Registry Conversion Pipeline (OCP)
// =========================================================================

/// Dynamically convert text input between formats looked up in the default registry.
#[cfg(feature = "alloc")]
pub fn convert(
    input: &str,
    from_format: &str,
    to_format: &str,
    options: &ConversionOptions,
) -> Result<String, BabbelError> {
    let registry = crate::default_registry();
    let from_engine = registry
        .get_by_id(from_format)
        .ok_or_else(|| BabbelError::syntax(alloc::format!("unknown format '{from_format}'")))?;
    let to_engine = registry
        .get_by_id(to_format)
        .ok_or_else(|| BabbelError::syntax(alloc::format!("unknown format '{to_format}'")))?;
    convert_format(input, &*from_engine, &*to_engine, options)
}

/// Dynamically convert binary or text input bytes between formats looked up in the default registry.
#[cfg(feature = "alloc")]
pub fn convert_dynamic_bytes(
    input: &[u8],
    from_format: &str,
    to_format: &str,
    options: &ConversionOptions,
) -> Result<Vec<u8>, BabbelError> {
    let registry = crate::default_registry();
    let from_engine = registry
        .get_by_id(from_format)
        .ok_or_else(|| BabbelError::syntax(alloc::format!("unknown format '{from_format}'")))?;
    let to_engine = registry
        .get_by_id(to_format)
        .ok_or_else(|| BabbelError::syntax(alloc::format!("unknown format '{to_format}'")))?;
    convert_format_bytes(input, &*from_engine, &*to_engine, options)
}

/// Dynamically convert text input between two `Format` enum values.
#[cfg(feature = "alloc")]
pub fn convert_between(
    input: &str,
    from: Format,
    to: Format,
    options: &ConversionOptions,
) -> Result<String, BabbelError> {
    convert(input, from.as_str(), to.as_str(), options)
}

// =========================================================================
// Declarative Pairwise Conversion Macros
// =========================================================================

macro_rules! text_to_text {
    ($fn_name:ident, ($($cfg:tt)*), $from:expr, $to:expr, $doc:expr) => {
        #[doc = $doc]
        #[cfg($($cfg)*)]
        pub fn $fn_name(input: &str) -> Result<String, BabbelError> {
            convert_format(input, &$from, &$to, &ConversionOptions::default())
        }
    };
}

macro_rules! text_to_bytes {
    ($fn_name:ident, ($($cfg:tt)*), $from:expr, $to:expr, $doc:expr) => {
        #[doc = $doc]
        #[cfg($($cfg)*)]
        pub fn $fn_name(input: &str) -> Result<Vec<u8>, BabbelError> {
            convert_format_bytes(input.as_bytes(), &$from, &$to, &ConversionOptions::default())
        }
    };
}

macro_rules! bytes_to_text {
    ($fn_name:ident, ($($cfg:tt)*), $from:expr, $to:expr, $doc:expr) => {
        #[doc = $doc]
        #[cfg($($cfg)*)]
        pub fn $fn_name(input: &[u8]) -> Result<String, BabbelError> {
            convert_format_bytes_to_str(input, &$from, &$to, &ConversionOptions::default())
        }
    };
}

macro_rules! bytes_to_bytes {
    ($fn_name:ident, ($($cfg:tt)*), $from:expr, $to:expr, $doc:expr) => {
        #[doc = $doc]
        #[cfg($($cfg)*)]
        pub fn $fn_name(input: &[u8]) -> Result<Vec<u8>, BabbelError> {
            convert_format_bytes(input, &$from, &$to, &ConversionOptions::default())
        }
    };
}

// =========================================================================
// Convenience Cross-Format Conversions (Generated via Macros)
// =========================================================================

// JSON, YAML, XML, Bencode
text_to_text!(
    json_to_yaml,
    (all(feature = "json", feature = "yaml")),
    JsonEngine,
    YamlEngine,
    "Convert JSON string to YAML string."
);
text_to_text!(
    json_to_xml,
    (all(feature = "json", feature = "xml")),
    JsonEngine,
    XmlEngine,
    "Convert JSON string to XML string."
);
text_to_bytes!(
    json_to_bencode,
    (all(feature = "json", feature = "bencode")),
    JsonEngine,
    BencodeEngine,
    "Convert JSON string to Bencode byte vector."
);
text_to_text!(
    yaml_to_json,
    (all(feature = "yaml", feature = "json")),
    YamlEngine,
    JsonEngine,
    "Convert YAML string to JSON string."
);
text_to_text!(
    yaml_to_xml,
    (all(feature = "yaml", feature = "xml")),
    YamlEngine,
    XmlEngine,
    "Convert YAML string to XML string."
);
bytes_to_text!(
    bencode_to_json,
    (all(feature = "bencode", feature = "json")),
    BencodeEngine,
    JsonEngine,
    "Convert Bencode binary payload to JSON string."
);
bytes_to_text!(
    bencode_to_yaml,
    (all(feature = "bencode", feature = "yaml")),
    BencodeEngine,
    YamlEngine,
    "Convert Bencode binary payload to YAML string."
);
bytes_to_text!(
    bencode_to_xml,
    (all(feature = "bencode", feature = "xml")),
    BencodeEngine,
    XmlEngine,
    "Convert Bencode binary payload to XML string."
);
text_to_text!(
    xml_to_json,
    (all(feature = "xml", feature = "json")),
    XmlEngine,
    JsonEngine,
    "Convert XML string to JSON string."
);
text_to_text!(
    xml_to_yaml,
    (all(feature = "xml", feature = "yaml")),
    XmlEngine,
    YamlEngine,
    "Convert XML string to YAML string."
);
text_to_bytes!(
    xml_to_bencode,
    (all(feature = "xml", feature = "bencode")),
    XmlEngine,
    BencodeEngine,
    "Convert XML string to Bencode byte vector."
);
text_to_bytes!(
    yaml_to_bencode,
    (all(feature = "yaml", feature = "bencode")),
    YamlEngine,
    BencodeEngine,
    "Convert YAML string to Bencode byte vector."
);

// Text formats (CSV, TSV, INI, JSON Lines)
text_to_text!(
    csv_to_json,
    (feature = "json"),
    CsvEngine,
    JsonEngine,
    "Convert CSV string to JSON string."
);
text_to_text!(
    json_to_csv,
    (feature = "json"),
    JsonEngine,
    CsvEngine,
    "Convert JSON string to CSV string."
);
text_to_text!(
    tsv_to_json,
    (feature = "json"),
    TsvEngine,
    JsonEngine,
    "Convert TSV string to JSON string."
);
text_to_text!(
    json_to_tsv,
    (feature = "json"),
    JsonEngine,
    TsvEngine,
    "Convert JSON string to TSV string."
);
text_to_text!(
    csv_to_yaml,
    (feature = "yaml"),
    CsvEngine,
    YamlEngine,
    "Convert CSV string to YAML string."
);
text_to_text!(
    yaml_to_csv,
    (feature = "yaml"),
    YamlEngine,
    CsvEngine,
    "Convert YAML string to CSV string."
);
text_to_text!(
    csv_to_xml,
    (feature = "xml"),
    CsvEngine,
    XmlEngine,
    "Convert CSV string to XML string."
);
text_to_text!(
    xml_to_csv,
    (feature = "xml"),
    XmlEngine,
    CsvEngine,
    "Convert XML string to CSV string."
);
text_to_bytes!(
    csv_to_bencode,
    (feature = "bencode"),
    CsvEngine,
    BencodeEngine,
    "Convert CSV string to Bencode byte vector."
);
bytes_to_text!(
    bencode_to_csv,
    (feature = "bencode"),
    BencodeEngine,
    CsvEngine,
    "Convert Bencode binary payload to CSV string."
);
text_to_text!(
    csv_to_ini,
    (all()),
    CsvEngine,
    IniEngine,
    "Convert CSV string to INI string."
);
text_to_text!(
    ini_to_csv,
    (all()),
    IniEngine,
    CsvEngine,
    "Convert INI string to CSV string."
);
text_to_text!(
    tsv_to_yaml,
    (feature = "yaml"),
    TsvEngine,
    YamlEngine,
    "Convert TSV string to YAML string."
);
text_to_text!(
    yaml_to_tsv,
    (feature = "yaml"),
    YamlEngine,
    TsvEngine,
    "Convert YAML string to TSV string."
);
text_to_text!(
    tsv_to_xml,
    (feature = "xml"),
    TsvEngine,
    XmlEngine,
    "Convert TSV string to XML string."
);
text_to_text!(
    xml_to_tsv,
    (feature = "xml"),
    XmlEngine,
    TsvEngine,
    "Convert XML string to TSV string."
);
text_to_text!(
    tsv_to_toml,
    (feature = "toml"),
    TsvEngine,
    TomlEngine,
    "Convert TSV string to TOML string."
);
text_to_text!(
    toml_to_tsv,
    (feature = "toml"),
    TomlEngine,
    TsvEngine,
    "Convert TOML string to TSV string."
);
text_to_bytes!(
    tsv_to_bencode,
    (feature = "bencode"),
    TsvEngine,
    BencodeEngine,
    "Convert TSV string to Bencode byte vector."
);
bytes_to_text!(
    bencode_to_tsv,
    (feature = "bencode"),
    BencodeEngine,
    TsvEngine,
    "Convert Bencode binary payload to TSV string."
);
text_to_text!(
    tsv_to_ini,
    (all()),
    TsvEngine,
    IniEngine,
    "Convert TSV string to INI string."
);
text_to_text!(
    ini_to_tsv,
    (all()),
    IniEngine,
    TsvEngine,
    "Convert INI string to TSV string."
);
text_to_text!(
    tsv_to_jsonlines,
    (feature = "json"),
    TsvEngine,
    JsonLinesEngine,
    "Convert TSV string to JSON Lines string."
);
text_to_text!(
    jsonlines_to_tsv,
    (feature = "json"),
    JsonLinesEngine,
    TsvEngine,
    "Convert JSON Lines string to TSV string."
);
text_to_text!(
    ini_to_json,
    (feature = "json"),
    IniEngine,
    JsonEngine,
    "Convert INI string to JSON string."
);
text_to_text!(
    json_to_ini,
    (feature = "json"),
    JsonEngine,
    IniEngine,
    "Convert JSON string to INI string."
);
text_to_text!(
    ini_to_yaml,
    (feature = "yaml"),
    IniEngine,
    YamlEngine,
    "Convert INI string to YAML string."
);
text_to_text!(
    yaml_to_ini,
    (feature = "yaml"),
    YamlEngine,
    IniEngine,
    "Convert YAML string to INI string."
);
text_to_text!(
    ini_to_xml,
    (feature = "xml"),
    IniEngine,
    XmlEngine,
    "Convert INI string to XML string."
);
text_to_text!(
    xml_to_ini,
    (feature = "xml"),
    XmlEngine,
    IniEngine,
    "Convert XML string to INI string."
);
text_to_bytes!(
    ini_to_bencode,
    (feature = "bencode"),
    IniEngine,
    BencodeEngine,
    "Convert INI string to Bencode byte vector."
);
bytes_to_text!(
    bencode_to_ini,
    (feature = "bencode"),
    BencodeEngine,
    IniEngine,
    "Convert Bencode binary payload to INI string."
);
text_to_text!(
    ini_to_jsonlines,
    (feature = "json"),
    IniEngine,
    JsonLinesEngine,
    "Convert INI string to JSON Lines string."
);
text_to_text!(
    jsonlines_to_ini,
    (feature = "json"),
    JsonLinesEngine,
    IniEngine,
    "Convert JSON Lines string to INI string."
);
text_to_text!(
    jsonlines_to_json,
    (feature = "json"),
    JsonLinesEngine,
    JsonEngine,
    "Convert JSON Lines string to JSON string."
);
text_to_text!(
    json_to_jsonlines,
    (feature = "json"),
    JsonEngine,
    JsonLinesEngine,
    "Convert JSON string to JSON Lines string."
);
text_to_text!(
    jsonlines_to_csv,
    (feature = "json"),
    JsonLinesEngine,
    CsvEngine,
    "Convert JSON Lines string to CSV string."
);
text_to_text!(
    csv_to_jsonlines,
    (feature = "json"),
    CsvEngine,
    JsonLinesEngine,
    "Convert CSV string to JSON Lines string."
);
text_to_text!(
    jsonlines_to_yaml,
    (all(feature = "json", feature = "yaml")),
    JsonLinesEngine,
    YamlEngine,
    "Convert JSON Lines string to YAML string."
);
text_to_text!(
    yaml_to_jsonlines,
    (all(feature = "yaml", feature = "json")),
    YamlEngine,
    JsonLinesEngine,
    "Convert YAML string to JSON Lines string."
);
text_to_text!(
    jsonlines_to_xml,
    (all(feature = "json", feature = "xml")),
    JsonLinesEngine,
    XmlEngine,
    "Convert JSON Lines string to XML string."
);
text_to_text!(
    xml_to_jsonlines,
    (all(feature = "xml", feature = "json")),
    XmlEngine,
    JsonLinesEngine,
    "Convert XML string to JSON Lines string."
);
text_to_bytes!(
    jsonlines_to_bencode,
    (all(feature = "json", feature = "bencode")),
    JsonLinesEngine,
    BencodeEngine,
    "Convert JSON Lines string to Bencode byte vector."
);
bytes_to_text!(
    bencode_to_jsonlines,
    (all(feature = "bencode", feature = "json")),
    BencodeEngine,
    JsonLinesEngine,
    "Convert Bencode binary payload to JSON Lines string."
);

// TOML
text_to_text!(
    toml_to_json,
    (all(feature = "toml", feature = "json")),
    TomlEngine,
    JsonEngine,
    "Convert TOML string to JSON string."
);
text_to_text!(
    json_to_toml,
    (all(feature = "json", feature = "toml")),
    JsonEngine,
    TomlEngine,
    "Convert JSON string to TOML string."
);
text_to_text!(
    toml_to_yaml,
    (all(feature = "toml", feature = "yaml")),
    TomlEngine,
    YamlEngine,
    "Convert TOML string to YAML string."
);
text_to_text!(
    yaml_to_toml,
    (all(feature = "yaml", feature = "toml")),
    YamlEngine,
    TomlEngine,
    "Convert YAML string to TOML string."
);
text_to_text!(
    toml_to_xml,
    (all(feature = "toml", feature = "xml")),
    TomlEngine,
    XmlEngine,
    "Convert TOML string to XML string."
);
text_to_text!(
    xml_to_toml,
    (all(feature = "xml", feature = "toml")),
    XmlEngine,
    TomlEngine,
    "Convert XML string to TOML string."
);
text_to_bytes!(
    toml_to_bencode,
    (all(feature = "toml", feature = "bencode")),
    TomlEngine,
    BencodeEngine,
    "Convert TOML string to Bencode byte vector."
);
bytes_to_text!(
    bencode_to_toml,
    (all(feature = "bencode", feature = "toml")),
    BencodeEngine,
    TomlEngine,
    "Convert Bencode binary payload to TOML string."
);
text_to_text!(
    toml_to_csv,
    (feature = "toml"),
    TomlEngine,
    CsvEngine,
    "Convert TOML string to CSV string."
);
text_to_text!(
    csv_to_toml,
    (feature = "toml"),
    CsvEngine,
    TomlEngine,
    "Convert CSV string to TOML string."
);
text_to_text!(
    toml_to_ini,
    (feature = "toml"),
    TomlEngine,
    IniEngine,
    "Convert TOML string to INI string."
);
text_to_text!(
    ini_to_toml,
    (feature = "toml"),
    IniEngine,
    TomlEngine,
    "Convert INI string to TOML string."
);
text_to_text!(
    toml_to_jsonlines,
    (all(feature = "toml", feature = "json")),
    TomlEngine,
    JsonLinesEngine,
    "Convert TOML string to JSON Lines string."
);
text_to_text!(
    jsonlines_to_toml,
    (all(feature = "json", feature = "toml")),
    JsonLinesEngine,
    TomlEngine,
    "Convert JSON Lines string to TOML string."
);

// MsgPack
text_to_bytes!(
    json_to_msgpack,
    (all(feature = "json", feature = "msgpack")),
    JsonEngine,
    MsgPackEngine,
    "Convert JSON string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_json,
    (all(feature = "msgpack", feature = "json")),
    MsgPackEngine,
    JsonEngine,
    "Convert MsgPack binary payload to JSON string."
);
text_to_bytes!(
    yaml_to_msgpack,
    (all(feature = "yaml", feature = "msgpack")),
    YamlEngine,
    MsgPackEngine,
    "Convert YAML string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_yaml,
    (all(feature = "msgpack", feature = "yaml")),
    MsgPackEngine,
    YamlEngine,
    "Convert MsgPack binary payload to YAML string."
);
text_to_bytes!(
    xml_to_msgpack,
    (all(feature = "xml", feature = "msgpack")),
    XmlEngine,
    MsgPackEngine,
    "Convert XML string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_xml,
    (all(feature = "msgpack", feature = "xml")),
    MsgPackEngine,
    XmlEngine,
    "Convert MsgPack binary payload to XML string."
);
text_to_bytes!(
    toml_to_msgpack,
    (all(feature = "toml", feature = "msgpack")),
    TomlEngine,
    MsgPackEngine,
    "Convert TOML string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_toml,
    (all(feature = "msgpack", feature = "toml")),
    MsgPackEngine,
    TomlEngine,
    "Convert MsgPack binary payload to TOML string."
);
bytes_to_bytes!(
    bencode_to_msgpack,
    (all(feature = "bencode", feature = "msgpack")),
    BencodeEngine,
    MsgPackEngine,
    "Convert Bencode binary payload to MsgPack byte vector."
);
bytes_to_bytes!(
    msgpack_to_bencode,
    (all(feature = "msgpack", feature = "bencode")),
    MsgPackEngine,
    BencodeEngine,
    "Convert MsgPack binary payload to Bencode byte vector."
);
text_to_bytes!(
    csv_to_msgpack,
    (feature = "msgpack"),
    CsvEngine,
    MsgPackEngine,
    "Convert CSV string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_csv,
    (feature = "msgpack"),
    MsgPackEngine,
    CsvEngine,
    "Convert MsgPack binary payload to CSV string."
);

// CBOR
text_to_bytes!(
    json_to_cbor,
    (all(feature = "json", feature = "cbor")),
    JsonEngine,
    CborEngine,
    "Convert JSON string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_json,
    (all(feature = "cbor", feature = "json")),
    CborEngine,
    JsonEngine,
    "Convert CBOR binary payload to JSON string."
);
text_to_bytes!(
    yaml_to_cbor,
    (all(feature = "yaml", feature = "cbor")),
    YamlEngine,
    CborEngine,
    "Convert YAML string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_yaml,
    (all(feature = "cbor", feature = "yaml")),
    CborEngine,
    YamlEngine,
    "Convert CBOR binary payload to YAML string."
);
text_to_bytes!(
    xml_to_cbor,
    (all(feature = "xml", feature = "cbor")),
    XmlEngine,
    CborEngine,
    "Convert XML string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_xml,
    (all(feature = "cbor", feature = "xml")),
    CborEngine,
    XmlEngine,
    "Convert CBOR binary payload to XML string."
);
text_to_bytes!(
    toml_to_cbor,
    (all(feature = "toml", feature = "cbor")),
    TomlEngine,
    CborEngine,
    "Convert TOML string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_toml,
    (all(feature = "cbor", feature = "toml")),
    CborEngine,
    TomlEngine,
    "Convert CBOR binary payload to TOML string."
);
bytes_to_bytes!(
    msgpack_to_cbor,
    (all(feature = "msgpack", feature = "cbor")),
    MsgPackEngine,
    CborEngine,
    "Convert MsgPack binary payload to CBOR byte vector."
);
bytes_to_bytes!(
    cbor_to_msgpack,
    (all(feature = "cbor", feature = "msgpack")),
    CborEngine,
    MsgPackEngine,
    "Convert CBOR binary payload to MsgPack byte vector."
);
bytes_to_bytes!(
    bencode_to_cbor,
    (all(feature = "bencode", feature = "cbor")),
    BencodeEngine,
    CborEngine,
    "Convert Bencode binary payload to CBOR byte vector."
);
bytes_to_bytes!(
    cbor_to_bencode,
    (all(feature = "cbor", feature = "bencode")),
    CborEngine,
    BencodeEngine,
    "Convert CBOR binary payload to Bencode byte vector."
);
text_to_bytes!(
    csv_to_cbor,
    (feature = "cbor"),
    CsvEngine,
    CborEngine,
    "Convert CSV string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_csv,
    (feature = "cbor"),
    CborEngine,
    CsvEngine,
    "Convert CBOR binary payload to CSV string."
);

// BSON
text_to_bytes!(
    json_to_bson,
    (all(feature = "json", feature = "bson")),
    JsonEngine,
    BsonEngine,
    "Convert JSON string to BSON byte vector."
);
bytes_to_text!(
    bson_to_json,
    (all(feature = "bson", feature = "json")),
    BsonEngine,
    JsonEngine,
    "Convert BSON binary payload to JSON string."
);
text_to_bytes!(
    yaml_to_bson,
    (all(feature = "yaml", feature = "bson")),
    YamlEngine,
    BsonEngine,
    "Convert YAML string to BSON byte vector."
);
bytes_to_text!(
    bson_to_yaml,
    (all(feature = "bson", feature = "yaml")),
    BsonEngine,
    YamlEngine,
    "Convert BSON binary payload to YAML string."
);
text_to_bytes!(
    xml_to_bson,
    (all(feature = "xml", feature = "bson")),
    XmlEngine,
    BsonEngine,
    "Convert XML string to BSON byte vector."
);
bytes_to_text!(
    bson_to_xml,
    (all(feature = "bson", feature = "xml")),
    BsonEngine,
    XmlEngine,
    "Convert BSON binary payload to XML string."
);
text_to_bytes!(
    toml_to_bson,
    (all(feature = "toml", feature = "bson")),
    TomlEngine,
    BsonEngine,
    "Convert TOML string to BSON byte vector."
);
bytes_to_text!(
    bson_to_toml,
    (all(feature = "bson", feature = "toml")),
    BsonEngine,
    TomlEngine,
    "Convert BSON binary payload to TOML string."
);
bytes_to_bytes!(
    msgpack_to_bson,
    (all(feature = "msgpack", feature = "bson")),
    MsgPackEngine,
    BsonEngine,
    "Convert MsgPack binary payload to BSON byte vector."
);
bytes_to_bytes!(
    bson_to_msgpack,
    (all(feature = "bson", feature = "msgpack")),
    BsonEngine,
    MsgPackEngine,
    "Convert BSON binary payload to MsgPack byte vector."
);
bytes_to_bytes!(
    cbor_to_bson,
    (all(feature = "cbor", feature = "bson")),
    CborEngine,
    BsonEngine,
    "Convert CBOR binary payload to BSON byte vector."
);
bytes_to_bytes!(
    bson_to_cbor,
    (all(feature = "bson", feature = "cbor")),
    BsonEngine,
    CborEngine,
    "Convert BSON binary payload to CBOR byte vector."
);
bytes_to_bytes!(
    bencode_to_bson,
    (all(feature = "bencode", feature = "bson")),
    BencodeEngine,
    BsonEngine,
    "Convert Bencode binary payload to BSON byte vector."
);
bytes_to_bytes!(
    bson_to_bencode,
    (all(feature = "bson", feature = "bencode")),
    BsonEngine,
    BencodeEngine,
    "Convert BSON binary payload to Bencode byte vector."
);
text_to_bytes!(
    csv_to_bson,
    (feature = "bson"),
    CsvEngine,
    BsonEngine,
    "Convert CSV string to BSON byte vector."
);
bytes_to_text!(
    bson_to_csv,
    (feature = "bson"),
    BsonEngine,
    CsvEngine,
    "Convert BSON binary payload to CSV string."
);

// JSON5
text_to_text!(
    json5_to_json,
    (feature = "json"),
    Json5Engine,
    JsonEngine,
    "Convert JSON5 string to JSON string."
);
text_to_text!(
    json_to_json5,
    (feature = "json"),
    JsonEngine,
    Json5Engine,
    "Convert JSON string to JSON5 string."
);
text_to_text!(
    json5_to_yaml,
    (all(feature = "json", feature = "yaml")),
    Json5Engine,
    YamlEngine,
    "Convert JSON5 string to YAML string."
);
text_to_text!(
    yaml_to_json5,
    (all(feature = "yaml", feature = "json")),
    YamlEngine,
    Json5Engine,
    "Convert YAML string to JSON5 string."
);
text_to_text!(
    json5_to_toml,
    (all(feature = "json", feature = "toml")),
    Json5Engine,
    TomlEngine,
    "Convert JSON5 string to TOML string."
);
text_to_text!(
    toml_to_json5,
    (all(feature = "toml", feature = "json")),
    TomlEngine,
    Json5Engine,
    "Convert TOML string to JSON5 string."
);
text_to_text!(
    json5_to_xml,
    (all(feature = "json", feature = "xml")),
    Json5Engine,
    XmlEngine,
    "Convert JSON5 string to XML string."
);
text_to_text!(
    xml_to_json5,
    (all(feature = "xml", feature = "json")),
    XmlEngine,
    Json5Engine,
    "Convert XML string to JSON5 string."
);
text_to_bytes!(
    json5_to_msgpack,
    (all(feature = "json", feature = "msgpack")),
    Json5Engine,
    MsgPackEngine,
    "Convert JSON5 string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_json5,
    (all(feature = "msgpack", feature = "json")),
    MsgPackEngine,
    Json5Engine,
    "Convert MsgPack binary payload to JSON5 string."
);
text_to_bytes!(
    json5_to_cbor,
    (all(feature = "json", feature = "cbor")),
    Json5Engine,
    CborEngine,
    "Convert JSON5 string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_json5,
    (all(feature = "cbor", feature = "json")),
    CborEngine,
    Json5Engine,
    "Convert CBOR binary payload to JSON5 string."
);
text_to_bytes!(
    json5_to_bson,
    (all(feature = "json", feature = "bson")),
    Json5Engine,
    BsonEngine,
    "Convert JSON5 string to BSON byte vector."
);
bytes_to_text!(
    bson_to_json5,
    (all(feature = "bson", feature = "json")),
    BsonEngine,
    Json5Engine,
    "Convert BSON binary payload to JSON5 string."
);

// RON
text_to_text!(
    ron_to_json,
    (all(feature = "ron", feature = "json")),
    RonEngine,
    JsonEngine,
    "Convert RON string to JSON string."
);
text_to_text!(
    json_to_ron,
    (all(feature = "json", feature = "ron")),
    JsonEngine,
    RonEngine,
    "Convert JSON string to RON string."
);
text_to_text!(
    ron_to_yaml,
    (all(feature = "ron", feature = "yaml")),
    RonEngine,
    YamlEngine,
    "Convert RON string to YAML string."
);
text_to_text!(
    yaml_to_ron,
    (all(feature = "yaml", feature = "ron")),
    YamlEngine,
    RonEngine,
    "Convert YAML string to RON string."
);
text_to_text!(
    ron_to_toml,
    (all(feature = "ron", feature = "toml")),
    RonEngine,
    TomlEngine,
    "Convert RON string to TOML string."
);
text_to_text!(
    toml_to_ron,
    (all(feature = "toml", feature = "ron")),
    TomlEngine,
    RonEngine,
    "Convert TOML string to RON string."
);
text_to_text!(
    ron_to_xml,
    (all(feature = "ron", feature = "xml")),
    RonEngine,
    XmlEngine,
    "Convert RON string to XML string."
);
text_to_text!(
    xml_to_ron,
    (all(feature = "xml", feature = "ron")),
    XmlEngine,
    RonEngine,
    "Convert XML string to RON string."
);
text_to_bytes!(
    ron_to_msgpack,
    (all(feature = "ron", feature = "msgpack")),
    RonEngine,
    MsgPackEngine,
    "Convert RON string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_ron,
    (all(feature = "msgpack", feature = "ron")),
    MsgPackEngine,
    RonEngine,
    "Convert MsgPack binary payload to RON string."
);
text_to_bytes!(
    ron_to_cbor,
    (all(feature = "ron", feature = "cbor")),
    RonEngine,
    CborEngine,
    "Convert RON string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_ron,
    (all(feature = "cbor", feature = "ron")),
    CborEngine,
    RonEngine,
    "Convert CBOR binary payload to RON string."
);
text_to_bytes!(
    ron_to_bson,
    (all(feature = "ron", feature = "bson")),
    RonEngine,
    BsonEngine,
    "Convert RON string to BSON byte vector."
);
bytes_to_text!(
    bson_to_ron,
    (all(feature = "bson", feature = "ron")),
    BsonEngine,
    RonEngine,
    "Convert BSON binary payload to RON string."
);
text_to_text!(
    ron_to_json5,
    (all(feature = "ron", feature = "json")),
    RonEngine,
    Json5Engine,
    "Convert RON string to JSON5 string."
);
text_to_text!(
    json5_to_ron,
    (all(feature = "json", feature = "ron")),
    Json5Engine,
    RonEngine,
    "Convert JSON5 string to RON string."
);

// KDL
text_to_text!(
    kdl_to_json,
    (all(feature = "kdl", feature = "json")),
    KdlEngine,
    JsonEngine,
    "Convert KDL string to JSON string."
);
text_to_text!(
    json_to_kdl,
    (all(feature = "json", feature = "kdl")),
    JsonEngine,
    KdlEngine,
    "Convert JSON string to KDL string."
);
text_to_text!(
    kdl_to_yaml,
    (all(feature = "kdl", feature = "yaml")),
    KdlEngine,
    YamlEngine,
    "Convert KDL string to YAML string."
);
text_to_text!(
    yaml_to_kdl,
    (all(feature = "yaml", feature = "kdl")),
    YamlEngine,
    KdlEngine,
    "Convert YAML string to KDL string."
);
text_to_text!(
    kdl_to_toml,
    (all(feature = "kdl", feature = "toml")),
    KdlEngine,
    TomlEngine,
    "Convert KDL string to TOML string."
);
text_to_text!(
    toml_to_kdl,
    (all(feature = "toml", feature = "kdl")),
    TomlEngine,
    KdlEngine,
    "Convert TOML string to KDL string."
);
text_to_text!(
    kdl_to_xml,
    (all(feature = "kdl", feature = "xml")),
    KdlEngine,
    XmlEngine,
    "Convert KDL string to XML string."
);
text_to_text!(
    xml_to_kdl,
    (all(feature = "xml", feature = "kdl")),
    XmlEngine,
    KdlEngine,
    "Convert XML string to KDL string."
);
text_to_bytes!(
    kdl_to_msgpack,
    (all(feature = "kdl", feature = "msgpack")),
    KdlEngine,
    MsgPackEngine,
    "Convert KDL string to MsgPack byte vector."
);
bytes_to_text!(
    msgpack_to_kdl,
    (all(feature = "msgpack", feature = "kdl")),
    MsgPackEngine,
    KdlEngine,
    "Convert MsgPack binary payload to KDL string."
);
text_to_bytes!(
    kdl_to_cbor,
    (all(feature = "kdl", feature = "cbor")),
    KdlEngine,
    CborEngine,
    "Convert KDL string to CBOR byte vector."
);
bytes_to_text!(
    cbor_to_kdl,
    (all(feature = "cbor", feature = "kdl")),
    CborEngine,
    KdlEngine,
    "Convert CBOR binary payload to KDL string."
);
text_to_bytes!(
    kdl_to_bson,
    (all(feature = "kdl", feature = "bson")),
    KdlEngine,
    BsonEngine,
    "Convert KDL string to BSON byte vector."
);
bytes_to_text!(
    bson_to_kdl,
    (all(feature = "bson", feature = "kdl")),
    BsonEngine,
    KdlEngine,
    "Convert BSON binary payload to KDL string."
);
text_to_text!(
    kdl_to_ron,
    (all(feature = "kdl", feature = "ron")),
    KdlEngine,
    RonEngine,
    "Convert KDL string to RON string."
);
text_to_text!(
    ron_to_kdl,
    (all(feature = "ron", feature = "kdl")),
    RonEngine,
    KdlEngine,
    "Convert RON string to KDL string."
);
text_to_text!(
    kdl_to_json5,
    (all(feature = "kdl", feature = "json")),
    KdlEngine,
    Json5Engine,
    "Convert KDL string to JSON5 string."
);
text_to_text!(
    json5_to_kdl,
    (all(feature = "json", feature = "kdl")),
    Json5Engine,
    KdlEngine,
    "Convert JSON5 string to KDL string."
);

// Parquet
bytes_to_text!(
    parquet_to_json,
    (all(feature = "parquet", feature = "json")),
    ParquetEngine,
    JsonEngine,
    "Convert Parquet binary payload to JSON string."
);
text_to_bytes!(
    json_to_parquet,
    (all(feature = "json", feature = "parquet")),
    JsonEngine,
    ParquetEngine,
    "Convert JSON string to Parquet byte vector."
);
bytes_to_text!(
    parquet_to_csv,
    (feature = "parquet"),
    ParquetEngine,
    CsvEngine,
    "Convert Parquet binary payload to CSV string."
);
text_to_bytes!(
    csv_to_parquet,
    (feature = "parquet"),
    CsvEngine,
    ParquetEngine,
    "Convert CSV string to Parquet byte vector."
);
bytes_to_text!(
    parquet_to_yaml,
    (all(feature = "parquet", feature = "yaml")),
    ParquetEngine,
    YamlEngine,
    "Convert Parquet binary payload to YAML string."
);
text_to_bytes!(
    yaml_to_parquet,
    (all(feature = "yaml", feature = "parquet")),
    YamlEngine,
    ParquetEngine,
    "Convert YAML string to Parquet byte vector."
);
bytes_to_text!(
    parquet_to_toml,
    (all(feature = "parquet", feature = "toml")),
    ParquetEngine,
    TomlEngine,
    "Convert Parquet binary payload to TOML string."
);
text_to_bytes!(
    toml_to_parquet,
    (all(feature = "toml", feature = "parquet")),
    TomlEngine,
    ParquetEngine,
    "Convert TOML string to Parquet byte vector."
);
bytes_to_bytes!(
    parquet_to_msgpack,
    (all(feature = "parquet", feature = "msgpack")),
    ParquetEngine,
    MsgPackEngine,
    "Convert Parquet binary payload to MsgPack byte vector."
);
bytes_to_bytes!(
    msgpack_to_parquet,
    (all(feature = "msgpack", feature = "parquet")),
    MsgPackEngine,
    ParquetEngine,
    "Convert MsgPack binary payload to Parquet byte vector."
);
bytes_to_bytes!(
    parquet_to_cbor,
    (all(feature = "parquet", feature = "cbor")),
    ParquetEngine,
    CborEngine,
    "Convert Parquet binary payload to CBOR byte vector."
);
bytes_to_bytes!(
    cbor_to_parquet,
    (all(feature = "cbor", feature = "parquet")),
    CborEngine,
    ParquetEngine,
    "Convert CBOR binary payload to Parquet byte vector."
);
bytes_to_bytes!(
    parquet_to_bson,
    (all(feature = "parquet", feature = "bson")),
    ParquetEngine,
    BsonEngine,
    "Convert Parquet binary payload to BSON byte vector."
);
bytes_to_bytes!(
    bson_to_parquet,
    (all(feature = "bson", feature = "parquet")),
    BsonEngine,
    ParquetEngine,
    "Convert BSON binary payload to Parquet byte vector."
);

// =========================================================================
// Backward Compatibility Format Adapters
// =========================================================================

macro_rules! define_format_adapters {
    ($parser:ident, $emitter:ident, $engine:expr, ($($cfg:tt)*)) => {
        #[cfg($($cfg)*)]
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $parser;

        #[cfg($($cfg)*)]
        impl FormatParser for $parser {
            #[inline]
            fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
                FormatEngine::parse_str(&$engine, input)
            }
            #[inline]
            fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
                FormatEngine::parse_bytes(&$engine, input)
            }
        }

        #[cfg($($cfg)*)]
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $emitter;

        #[cfg($($cfg)*)]
        impl FormatEmitter for $emitter {
            #[inline]
            fn emit(&self, value: &Value, dest: &mut dyn babbel_core::IDestination) -> Result<(), BabbelError> {
                FormatEmitter::emit(&$engine, value, dest)
            }
            #[inline]
            fn emit_pretty(&self, value: &Value, dest: &mut dyn babbel_core::IDestination, indent: usize) -> Result<(), BabbelError> {
                FormatEmitter::emit_pretty(&$engine, value, dest, indent)
            }
        }
    };
}

#[cfg(feature = "json")]
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonParser;

#[cfg(feature = "json")]
impl FormatParser for JsonParser {
    #[inline]
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        FormatEngine::parse_str(&JsonEngine, input)
    }
    #[inline]
    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        FormatEngine::parse_bytes(&JsonEngine, input)
    }
}

define_format_adapters!(Json5Parser, Json5Emitter, Json5Engine, (feature = "json"));
define_format_adapters!(YamlParser, YamlEmitter, YamlEngine, (feature = "yaml"));
define_format_adapters!(XmlParser, XmlEmitter, XmlEngine, (feature = "xml"));
define_format_adapters!(
    BencodeParser,
    BencodeEmitter,
    BencodeEngine,
    (feature = "bencode")
);
#[cfg(feature = "toml")]
#[derive(Debug, Default, Clone, Copy)]
pub struct TomlParser;

#[cfg(feature = "toml")]
impl FormatParser for TomlParser {
    #[inline]
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        FormatEngine::parse_str(&TomlEngine, input)
    }
    #[inline]
    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        FormatEngine::parse_bytes(&TomlEngine, input)
    }
}
define_format_adapters!(
    MsgPackParser,
    MsgPackEmitter,
    MsgPackEngine,
    (feature = "msgpack")
);
define_format_adapters!(CborParser, CborEmitter, CborEngine, (feature = "cbor"));
define_format_adapters!(BsonParser, BsonEmitter, BsonEngine, (feature = "bson"));
define_format_adapters!(RonParser, RonEmitter, RonEngine, (feature = "ron"));
define_format_adapters!(KdlParser, KdlEmitter, KdlEngine, (feature = "kdl"));
define_format_adapters!(
    ParquetParser,
    ParquetEmitter,
    ParquetEngine,
    (feature = "parquet")
);

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
    fn emit(
        &self,
        value: &Value,
        destination: &mut dyn babbel_core::IDestination,
    ) -> Result<(), BabbelError> {
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
    fn emit(
        &self,
        value: &Value,
        destination: &mut dyn babbel_core::IDestination,
    ) -> Result<(), BabbelError> {
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
    fn emit(
        &self,
        value: &Value,
        destination: &mut dyn babbel_core::IDestination,
    ) -> Result<(), BabbelError> {
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
    fn emit(
        &self,
        value: &Value,
        destination: &mut dyn babbel_core::IDestination,
    ) -> Result<(), BabbelError> {
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
