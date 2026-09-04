//! # Babbel
//!
//! A unified, high-performance polyglot serialization and document processing ecosystem.
//! Provides first-class support for XML, JSON, YAML, and BitTorrent Bencode, backed by a
//! shared core (`babbel_core`).
//!
//! ## Sub-libraries
//!
//! - **`core`**: Common streaming I/O, BOM auto-detection, and numeric/error utilities.
//! - **`json`**: Full-featured JSON DOM, JSON Pointer (RFC 6901), and Merge Patch (RFC 7396).
//! - **`yaml`**: YAML 1.2 parser/emitter with anchors, aliases, tags, and multi-document streams.
//! - **`xml`**: Validating XML parser, C14N canonicalization, and XPath 1.0 engine.
//! - **`bencode`**: Fast, binary-safe BitTorrent Bencode serializer and DOM.

pub use babbel_core as core;

#[cfg(feature = "json")]
pub use json_lib as json;

#[cfg(feature = "yaml")]
pub use yaml_lib as yaml;

#[cfg(feature = "xml")]
pub use xml_lib_rust as xml;

#[cfg(feature = "bencode")]
pub use bencode_lib as bencode;

#[cfg(feature = "convert")]
pub mod convert;
