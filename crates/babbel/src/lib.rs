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
//! - **`csv`**: RFC 4180 CSV and TSV parsing/emission with delimiter sniffing and type inference.
//! - **`ini`**: Section-based INI, Java `.properties`, and `.env` parsing/emission.
//! - **`text`**: Document frontmatter extraction (`split_frontmatter`) and line indentation utilities.
//! - **`convert`**: Universal $O(N)$ cross-format conversion matrix.
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

// Backwards-compatible aliases (prefer babbel::json, babbel::yaml, babbel::xml, babbel::bencode, babbel::toml)
#[cfg(feature = "json")]
#[deprecated(since = "0.2.0", note = "Use `babbel::json` instead")]
pub use babbel_json as json_lib;
#[cfg(feature = "yaml")]
#[deprecated(since = "0.2.0", note = "Use `babbel::yaml` instead")]
pub use babbel_yaml as yaml_lib;
#[cfg(feature = "xml")]
#[deprecated(since = "0.2.0", note = "Use `babbel::xml` instead")]
pub use babbel_xml as xml_lib;
#[cfg(feature = "bencode")]
#[deprecated(since = "0.2.0", note = "Use `babbel::bencode` instead")]
pub use babbel_bencode as bencode_lib;
#[cfg(feature = "toml")]
#[deprecated(since = "0.2.0", note = "Use `babbel::toml` instead")]
pub use babbel_toml as toml_lib;

#[cfg(feature = "convert")]
pub mod convert;

/// Unified embedded systems module aggregating zero-allocation streaming parsers,
/// stack-allocated destinations, and memory limits for microcontrollers.
pub mod embedded {
    pub use babbel_core::embedded::*;
    pub use babbel_core::io::{ArrayVecDestination, SliceDestination};
    pub use babbel_core::csv::{CsvFieldsIter, CsvPullParser, CsvRecord};
    pub use babbel_core::ini::{IniEvent, IniPullParser};
    #[cfg(feature = "json")]
    pub use babbel_json::parser::pull_parser::{JsonPullEvent, JsonPullParser, JsonScalar};
    #[cfg(feature = "xml")]
    pub use babbel_xml::parser::{XmlPullAttribute, XmlPullEvent, XmlPullParser};
    #[cfg(feature = "toml")]
    pub use babbel_toml::parser::{TomlPullEvent, TomlPullParser};
}

// Re-export text, CSV, and INI processing from core
pub use babbel_core::{
    csv, emit_csv, emit_ini, ini, parse_csv, parse_ini, sniff_delimiter, split_frontmatter, text,
    ArrayVecDestination, BabbelError, CompactError, CsvFieldsIter, CsvOptions, CsvPullParser, CsvRecord,
    DocumentWithFrontmatter, EmbeddedLimits, ErrorCode, FrontmatterFormat, IniEvent, IniOptions, IniPullParser,
    Location, MemoryTracker, SliceDestination, Span, StackBuffer, Value,
};

// Re-export FormatEngine architecture (OCP & DIP)
#[cfg(feature = "json")]
pub use babbel_json::{JsonEngine, JsonLinesEngine};
#[cfg(feature = "yaml")]
pub use babbel_yaml::YamlEngine;
#[cfg(feature = "xml")]
pub use babbel_xml::XmlEngine;
#[cfg(feature = "bencode")]
pub use babbel_bencode::BencodeEngine;
#[cfg(feature = "toml")]
pub use babbel_toml::TomlEngine;
#[cfg(feature = "msgpack")]
pub use babbel_msgpack::MsgPackEngine;
#[cfg(feature = "cbor")]
pub use babbel_cbor::CborEngine;
#[cfg(feature = "bson")]
pub use babbel_bson::BsonEngine;

pub use babbel_core::{
    find_engine, find_engine_by_extension, find_engine_by_mime, CsvEngine, FormatCodec,
    FormatEmitter, FormatEngine, FormatOptions, FormatParser, IniEngine, TsvEngine,
};
#[cfg(feature = "alloc")]
pub use babbel_core::FormatRegistry;

/// Creates a format registry pre-populated with all enabled built-in format engines.
#[cfg(feature = "alloc")]
pub fn default_registry() -> babbel_core::FormatRegistry {
    #[allow(unused_mut)]
    let mut registry = babbel_core::FormatRegistry::new();
    #[cfg(feature = "json")]
    {
        registry.register(alloc::sync::Arc::new(babbel_json::JsonEngine));
        registry.register(alloc::sync::Arc::new(babbel_json::JsonLinesEngine));
    }
    #[cfg(feature = "yaml")]
    registry.register(alloc::sync::Arc::new(babbel_yaml::YamlEngine));
    #[cfg(feature = "xml")]
    registry.register(alloc::sync::Arc::new(babbel_xml::XmlEngine));
    #[cfg(feature = "bencode")]
    registry.register(alloc::sync::Arc::new(babbel_bencode::BencodeEngine));
    #[cfg(feature = "toml")]
    registry.register(alloc::sync::Arc::new(babbel_toml::TomlEngine));
    #[cfg(feature = "msgpack")]
    registry.register(alloc::sync::Arc::new(babbel_msgpack::MsgPackEngine));
    #[cfg(feature = "cbor")]
    registry.register(alloc::sync::Arc::new(babbel_cbor::CborEngine));
    #[cfg(feature = "bson")]
    registry.register(alloc::sync::Arc::new(babbel_bson::BsonEngine));

    registry.register(alloc::sync::Arc::new(babbel_core::CsvEngine));
    registry.register(alloc::sync::Arc::new(babbel_core::TsvEngine));
    registry.register(alloc::sync::Arc::new(babbel_core::IniEngine));
    registry
}

#[macro_use]
pub mod macros;

#[cfg(feature = "serde")]
pub use babbel_core::{from_value, to_value, SerdeError};

/// Convenient prelude re-exporting key format engines, universal Value AST, I/O traits,
/// the declarative `value!` macro, and conversion routines.
pub mod prelude {
    pub use babbel_core::{
        find_engine, find_engine_by_extension, find_engine_by_mime, BabbelError, ErrorCode,
        FormatCodec, FormatEmitter, FormatEngine, FormatOptions, FormatParser, Location, Span,
        Value,
    };
    pub use babbel_core::io::{
        Buffer, BufferDestination, BufferSource, IDestination, ISource,
    };
    #[cfg(feature = "alloc")]
    pub use babbel_core::FormatRegistry;
    #[cfg(feature = "alloc")]
    pub use crate::default_registry;
    pub use crate::value;
    #[cfg(feature = "convert")]
    pub use crate::convert::*;
    #[cfg(feature = "serde")]
    pub use babbel_core::{from_value, to_value, SerdeError};
}

