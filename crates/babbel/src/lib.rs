//! # Babbel
//!
//! A unified, high-performance polyglot serialization and document processing ecosystem.
//! Provides first-class support for JSON, YAML, XML, BitTorrent Bencode, RFC 4180 CSV/TSV,
//! sectioned INI/.env, and JSON Lines, backed by a shared core (`babbel_core`).
//!
//! ## Sub-libraries & Modules
//!
//! - **`core`**: Common streaming I/O (`ILineReader`), universal `Value` AST, BOM detection, and numeric utilities.
//! - **`json`**: Full-featured JSON DOM, JSON Pointer (RFC 6901), Merge Patch (RFC 7396), and JSON Lines streaming.
//! - **`yaml`**: YAML 1.2 parser/emitter with anchors, aliases, tags, and multi-document streams.
//! - **`xml`**: Validating XML parser, C14N canonicalization, and XPath 1.0 engine.
//! - **`bencode`**: Fast, binary-safe BitTorrent Bencode serializer and DOM.
//! - **`toml`**: Fast, modular, pure-Rust TOML v1.1.0 parser and serializer.
//! - **`msgpack`**: High-speed, binary-safe MessagePack encoder and decoder.
//! - **`cbor`**: RFC 8949 Concise Binary Object Representation (CBOR) encoder/decoder.
//! - **`bson`**: High-performance Binary JSON (BSON v1.1) specification encoder/decoder.
//! - **`ron`**: Rusty Object Notation (RON) parser and serializer for Rust configurations and games.
//! - **`kdl`**: KDL Document Language (KDL v2) parser and serializer for modern CLI and document configs.
//! - **`parquet`**: Apache Parquet columnar storage reader, writer, and FormatEngine for analytical data.
//! - **`hcl`**: HashiCorp Configuration Language (HCL v2) and Terraform parser and serializer.
//! - **`avro`**: Apache Avro binary and Object Container File (OCF) parser and serializer.
//! - **`csv`**: RFC 4180 CSV and TSV parsing/emission with delimiter sniffing and type inference.
//! - **`ini`**: Section-based INI, Java `.properties`, and `.env` parsing/emission.
//! - **`text`**: Document frontmatter extraction (`split_frontmatter`) and line indentation utilities.
//! - **`convert`**: Universal $O(N)$ cross-format conversion matrix across 16 formats.
//!
//! ## Architectural Data Flow
//!
//! ```text
//! Text / Bytes ──► FormatParser ──► Universal Value AST ──► FormatEmitter ──► Destination
//!                        │                                        │
//!                        ▼                                        ▼
//!                  Error Snippet                            Buffer / File
//! ```
//!
//! ## Universal Facade & Fluent Builders
//!
//! ```rust
//! use babbel::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Fluent AST construction
//! let config = Value::object()
//!     .set("service", "gateway")
//!     .set("port", 8080)
//!     .set("features", Value::array().push("metrics").push("tracing").build())
//!     .build();
//!
//! // 2. Direct serialization across formats
//! let json_text = babbel::to_string(Format::Json, &config)?;
//! assert!(json_text.contains("\"service\":\"gateway\""));
//!
//! // 3. Universal cross-format conversion
//! let yaml_text = babbel::convert(Format::Json, Format::Yaml, &json_text)?;
//! assert!(yaml_text.contains("service: gateway"));
//! # Ok(())
//! # }
//! ```
//!
//! ## Quickstart
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! #[cfg(feature = "json")]
//! {
//!     let json_node = babbel::json::from_str(r#"{"service": "babbel", "port": 8080}"#)?;
//!     assert_eq!(json_node.get("service").and_then(|n| n.as_str()), Some("babbel"));
//! }
//!
//! // Parse CSV
//! let csv_val = babbel::parse_csv("name,score\nAlice,100\n", &babbel::CsvOptions::default())?;
//! assert_eq!(csv_val.as_array().unwrap().len(), 1);
//!
//! #[cfg(feature = "convert")]
//! {
//!     let yaml_str = babbel::convert::json_to_yaml(r#"{"name": "test"}"#)?;
//!     assert!(yaml_str.contains("name: test"));
//!
//!     let json_str = babbel::convert::csv_to_json("id,item\n1,rust\n")?;
//!     assert!(json_str.contains("rust"));
//! }
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

pub use babbel_core as core;

#[cfg(feature = "json")]
pub use babbel_json as json;

#[cfg(feature = "yaml")]
pub use babbel_yaml as yaml;

#[cfg(feature = "xml")]
pub use babbel_xml as xml;

#[cfg(feature = "bencode")]
pub use babbel_bencode as bencode;

#[cfg(feature = "toml")]
pub use babbel_toml as toml;

#[cfg(feature = "msgpack")]
pub use babbel_msgpack as msgpack;

#[cfg(feature = "cbor")]
pub use babbel_cbor as cbor;

#[cfg(feature = "bson")]
pub use babbel_bson as bson;

#[cfg(feature = "ron")]
pub use babbel_ron as ron;

#[cfg(feature = "kdl")]
pub use babbel_kdl as kdl;

#[cfg(feature = "parquet")]
pub use babbel_parquet as parquet;

#[cfg(feature = "hcl")]
pub use babbel_hcl as hcl;

#[cfg(feature = "avro")]
pub use babbel_avro as avro;

// Backwards-compatible aliases (prefer babbel::json, babbel::yaml, babbel::xml, babbel::bencode, babbel::toml)
#[cfg(feature = "bencode")]
#[deprecated(since = "0.2.0", note = "Use `babbel::bencode` instead")]
pub use babbel_bencode as bencode_lib;
#[cfg(feature = "json")]
#[deprecated(since = "0.2.0", note = "Use `babbel::json` instead")]
pub use babbel_json as json_lib;
#[cfg(feature = "toml")]
#[deprecated(since = "0.2.0", note = "Use `babbel::toml` instead")]
pub use babbel_toml as toml_lib;
#[cfg(feature = "xml")]
#[deprecated(since = "0.2.0", note = "Use `babbel::xml` instead")]
pub use babbel_xml as xml_lib;
#[cfg(feature = "yaml")]
#[deprecated(since = "0.2.0", note = "Use `babbel::yaml` instead")]
pub use babbel_yaml as yaml_lib;

#[cfg(feature = "convert")]
pub mod convert;

/// Unified embedded systems module aggregating zero-allocation streaming parsers,
/// stack-allocated destinations, and memory limits for microcontrollers.
pub mod embedded {
    #[cfg(feature = "bson")]
    pub use babbel_bson::pull::{BsonPullEvent, BsonPullParser};
    #[cfg(feature = "cbor")]
    pub use babbel_cbor::pull::{CborPullEvent, CborPullParser};
    pub use babbel_core::csv::{CsvFieldsIter, CsvPullParser, CsvRecord};
    pub use babbel_core::embedded::*;
    pub use babbel_core::ini::{IniEvent, IniPullParser};
    pub use babbel_core::io::{ArrayVecDestination, SliceDestination};
    #[cfg(feature = "json")]
    pub use babbel_json::parser::pull_parser::{JsonPullEvent, JsonPullParser, JsonScalar};
    #[cfg(feature = "msgpack")]
    pub use babbel_msgpack::pull::{MsgPackPullEvent, MsgPackPullParser};
    #[cfg(feature = "toml")]
    pub use babbel_toml::parser::{TomlPullEvent, TomlPullParser};
    #[cfg(feature = "xml")]
    pub use babbel_xml::parser::{XmlPullAttribute, XmlPullEvent, XmlPullParser};
}

// Re-export text, CSV, and INI processing from core
pub use babbel_core::{
    ArrayBuilder, ArrayVecDestination, BabbelError, CompactError, CsvFieldsIter, CsvOptions,
    CsvPullParser, CsvRecord, DocumentWithFrontmatter, DuplicateKeyMode, EmbeddedLimits, ErrorCode,
    FormatOptionsBuilder, FrontmatterFormat, IniEvent, IniOptions, IniPullParser, Location,
    MemoryTracker, ObjectBuilder, ParseOptions, SliceDestination, Span, StackBuffer, Value,
    csv, emit_csv, emit_ini, ini, parse_csv, parse_ini, sniff_delimiter, split_frontmatter, text,
};

// Re-export FormatEngine architecture (OCP & DIP)
#[cfg(feature = "avro")]
pub use babbel_avro::AvroEngine;
#[cfg(feature = "bencode")]
pub use babbel_bencode::BencodeEngine;
#[cfg(feature = "bson")]
pub use babbel_bson::BsonEngine;
#[cfg(feature = "cbor")]
pub use babbel_cbor::CborEngine;
#[cfg(feature = "hcl")]
pub use babbel_hcl::HclEngine;
#[cfg(feature = "json")]
pub use babbel_json::{Json5Engine, JsonEngine, JsonLinesEngine};
#[cfg(feature = "kdl")]
pub use babbel_kdl::KdlEngine;
#[cfg(feature = "msgpack")]
pub use babbel_msgpack::MsgPackEngine;
#[cfg(feature = "parquet")]
pub use babbel_parquet::ParquetEngine;
#[cfg(feature = "ron")]
pub use babbel_ron::RonEngine;
#[cfg(feature = "toml")]
pub use babbel_toml::TomlEngine;
#[cfg(feature = "xml")]
pub use babbel_xml::XmlEngine;
#[cfg(feature = "yaml")]
pub use babbel_yaml::YamlEngine;

pub use babbel_core::{
    CompiledSchema, CsvEngine, FormatCodec, FormatEmitter, FormatEngine, FormatOptions,
    FormatParser, IniEngine, JsonPath, LocatedNode, Patch, PatchOp, Regex, SchemaValidationError,
    TsvEngine, apply_merge_patch, compression, diff_merge_patch, find_engine,
    find_engine_by_extension, find_engine_by_mime, format_unified_diff, infer_schema,
    jsonpath_query, jsonpath_query_located, jsonpath_query_mut, transform, validate_schema,
};
pub mod diff {
    pub use babbel_core::diff::*;
}
pub mod patch {
    pub use babbel_core::patch::*;
}
pub mod query {
    pub use babbel_core::query::*;
}
pub mod schema {
    pub use babbel_core::schema::*;
}
#[cfg(feature = "alloc")]
pub use babbel_core::FormatRegistry;

macro_rules! register_feature_engines {
    ($registry:ident, $( ($feature:literal, $engine:expr) ),* $(,)?) => {
        $(
            #[cfg(feature = $feature)]
            $registry.register(alloc::sync::Arc::new($engine));
        )*
    };
}

/// Creates a format registry pre-populated with all enabled built-in format engines.
#[cfg(feature = "alloc")]
pub fn default_registry() -> babbel_core::FormatRegistry {
    let mut registry = babbel_core::FormatRegistry::new();
    register_feature_engines!(
        registry,
        ("json", babbel_json::JsonEngine),
        ("json", babbel_json::JsonLinesEngine),
        ("json", babbel_json::Json5Engine),
        ("yaml", babbel_yaml::YamlEngine),
        ("xml", babbel_xml::XmlEngine),
        ("bencode", babbel_bencode::BencodeEngine),
        ("toml", babbel_toml::TomlEngine),
        ("msgpack", babbel_msgpack::MsgPackEngine),
        ("cbor", babbel_cbor::CborEngine),
        ("bson", babbel_bson::BsonEngine),
        ("ron", babbel_ron::RonEngine),
        ("kdl", babbel_kdl::KdlEngine),
        ("parquet", babbel_parquet::ParquetEngine),
        ("hcl", babbel_hcl::HclEngine),
        ("avro", babbel_avro::AvroEngine),
    );

    registry.register(alloc::sync::Arc::new(babbel_core::CsvEngine));
    registry.register(alloc::sync::Arc::new(babbel_core::TsvEngine));
    registry.register(alloc::sync::Arc::new(babbel_core::IniEngine));
    registry
}

#[cfg(feature = "convert")]
pub use convert::Format;

/// Universal parse from string into [`Value`] using the specified format.
#[cfg(feature = "convert")]
pub fn from_str(format: Format, input: &str) -> Result<Value, BabbelError> {
    let registry = default_registry();
    let engine = registry
        .get_by_id(format.as_str())
        .ok_or_else(|| BabbelError::unsupported(alloc::format!("format '{}' is not registered", format.as_str())))?;
    engine.parse_str(input)
}

/// Universal parse from byte slice into [`Value`] using the specified format.
#[cfg(feature = "convert")]
pub fn from_slice(format: Format, input: &[u8]) -> Result<Value, BabbelError> {
    let registry = default_registry();
    let engine = registry
        .get_by_id(format.as_str())
        .ok_or_else(|| BabbelError::unsupported(alloc::format!("format '{}' is not registered", format.as_str())))?;
    engine.parse_bytes(input)
}

/// Universal serialize [`Value`] into compact UTF-8 string using the specified format.
#[cfg(feature = "convert")]
pub fn to_string(format: Format, value: &Value) -> Result<alloc::string::String, BabbelError> {
    let registry = default_registry();
    let engine = registry
        .get_by_id(format.as_str())
        .ok_or_else(|| BabbelError::unsupported(alloc::format!("format '{}' is not registered", format.as_str())))?;
    engine.serialize_to_string(value, &FormatOptions::compact())
}

/// Universal serialize [`Value`] into pretty-printed UTF-8 string with default indentation.
#[cfg(feature = "convert")]
pub fn to_string_pretty(format: Format, value: &Value) -> Result<alloc::string::String, BabbelError> {
    let registry = default_registry();
    let engine = registry
        .get_by_id(format.as_str())
        .ok_or_else(|| BabbelError::unsupported(alloc::format!("format '{}' is not registered", format.as_str())))?;
    engine.serialize_to_string(value, &FormatOptions::pretty())
}

/// Universal serialize [`Value`] into byte vector using the specified format.
#[cfg(feature = "convert")]
pub fn to_vec(format: Format, value: &Value) -> Result<alloc::vec::Vec<u8>, BabbelError> {
    let registry = default_registry();
    let engine = registry
        .get_by_id(format.as_str())
        .ok_or_else(|| BabbelError::unsupported(alloc::format!("format '{}' is not registered", format.as_str())))?;
    engine.serialize_to_vec(value, &FormatOptions::compact())
}

/// Universal cross-format string conversion.
#[cfg(feature = "convert")]
pub fn convert(from: Format, to: Format, input: &str) -> Result<alloc::string::String, BabbelError> {
    let val = from_str(from, input)?;
    to_string(to, &val)
}

/// Universal cross-format byte conversion.
#[cfg(feature = "convert")]
pub fn convert_bytes(from: Format, to: Format, input: &[u8]) -> Result<alloc::vec::Vec<u8>, BabbelError> {
    let val = from_slice(from, input)?;
    to_vec(to, &val)
}

/// Parse input string based on file path extension.
#[cfg(all(feature = "convert", feature = "std"))]
pub fn from_file_str(path: impl AsRef<std::path::Path>, input: &str) -> Result<Value, BabbelError> {
    let format = Format::from_path(&path)
        .ok_or_else(|| BabbelError::unsupported("unrecognized or missing file extension"))?;
    from_str(format, input)
}

/// Parse input byte slice based on file path extension.
#[cfg(all(feature = "convert", feature = "std"))]
pub fn from_file_bytes(path: impl AsRef<std::path::Path>, input: &[u8]) -> Result<Value, BabbelError> {
    let format = Format::from_path(&path)
        .ok_or_else(|| BabbelError::unsupported("unrecognized or missing file extension"))?;
    from_slice(format, input)
}

#[macro_use]
pub mod macros;

#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "serde")]
pub use babbel_core::{
    FormatVisitorSerializer, SerdeError, from_value, from_value_owned, serialize_to_visitor,
    to_value,
};

/// Convenient prelude re-exporting key format engines, universal Value AST, I/O traits,
/// the declarative `value!` macro, and conversion routines.
pub mod prelude {
    #[cfg(feature = "convert")]
    pub use crate::convert::*;
    #[cfg(feature = "convert")]
    pub use crate::{
        convert, convert_bytes, from_slice, from_str, to_string, to_string_pretty, to_vec,
    };
    #[cfg(all(feature = "convert", feature = "std"))]
    pub use crate::{from_file_bytes, from_file_str};
    #[cfg(feature = "alloc")]
    pub use crate::default_registry;
    pub use crate::value;
    #[cfg(feature = "alloc")]
    pub use babbel_core::FormatRegistry;
    pub use babbel_core::io::{Buffer, BufferDestination, BufferSource, IDestination, ISource};
    pub use babbel_core::{
        ArrayBuilder, BabbelError, CompiledSchema, DuplicateKeyMode, ErrorCode, FormatCodec,
        FormatEmitter, FormatEngine, FormatOptions, FormatOptionsBuilder, FormatParser, JsonPath,
        Location, ObjectBuilder, ParseOptions, Patch, PatchOp, SchemaValidationError, Span, Value,
        apply_merge_patch, diff, diff_merge_patch, find_engine, find_engine_by_extension,
        find_engine_by_mime, jsonpath_query, jsonpath_query_mut, validate_schema,
    };
    #[cfg(feature = "serde")]
    pub use babbel_core::{
        FormatVisitorSerializer, SerdeError, from_value, from_value_owned, serialize_to_visitor,
        to_value,
    };
}
