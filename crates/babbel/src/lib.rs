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

pub use babbel_core as core;

#[cfg(feature = "json")]
pub use json_lib as json;

#[cfg(feature = "yaml")]
pub use yaml_lib as yaml;

#[cfg(feature = "xml")]
pub use xml_lib as xml;

#[cfg(feature = "bencode")]
pub use bencode_lib as bencode;

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
    pub use json_lib::parser::pull_parser::{JsonPullEvent, JsonPullParser, JsonScalar};
    #[cfg(feature = "xml")]
    pub use xml_lib::parser::{XmlPullAttribute, XmlPullEvent, XmlPullParser};
}

// Re-export text, CSV, and INI processing from core
pub use babbel_core::{
    csv, emit_csv, emit_ini, ini, parse_csv, parse_ini, sniff_delimiter, split_frontmatter, text,
    ArrayVecDestination, CompactError, CsvFieldsIter, CsvOptions, CsvPullParser, CsvRecord,
    DocumentWithFrontmatter, EmbeddedLimits, FrontmatterFormat, IniEvent, IniOptions, IniPullParser,
    MemoryTracker, SliceDestination, StackBuffer,
};
