//! # Babbel KDL
//!
//! Fast, pure-Rust parser, serializer, and [`FormatEngine`] implementation for
//! **KDL (KDL Document Language)**, a human-friendly configuration and document language.
//!
//! ## Features
//!
//! - **Full KDL Syntax**: Nodes, positional arguments, named properties (`key=val`), and nested children blocks (`{ ... }`).
//! - **Comments**: Single-line (`//`), nested multi-line block (`/* /* */ */`), and node/entry slashdash comments (`/-`).
//! - **Rich Types**: Raw strings (`r#"..."#`), unicode escape sequences (`\u{...}`), number bases (hex `0x`, octal `0o`, bin `0b`, float, exponents, underscores), booleans (`true`/`false`), and `null`.
//! - **Zero Dependencies**: Pure Rust with optional `no_std` + `alloc` support.
//! - **Unified Architecture**: Integrates into Babbel's universal `Value` AST and cross-format conversion matrix.
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_kdl::{from_str, to_string_pretty};
//!
//! let kdl_source = r#"
//! package {
//!     name "babbel"
//!     version "0.2.1"
//!     features "kdl" "ron"
//! }
//! "#;
//!
//! let value = from_str(kdl_source).expect("parse failed");
//! assert!(value.get("package").is_some());
//!
//! let pretty_kdl = to_string_pretty(&value, 2).expect("serialize failed");
//! println!("{}", pretty_kdl);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

pub mod ast;
pub mod engine;
pub mod error;
pub mod parser;
pub mod serializer;

pub use ast::{KdlDocument, KdlEntry, KdlNode, KdlValue};
pub use engine::KdlEngine;
pub use error::KdlError;
pub use parser::parse_document;
pub use serializer::{to_string, to_string_pretty};

pub use babbel_core::Value;

/// Deserialize a universal Babbel [`Value`] from a KDL string.
pub fn from_str(input: &str) -> Result<Value, KdlError> {
    let doc = parser::parse_document(input)?;
    Ok(doc.to_value())
}

/// Deserialize a universal Babbel [`Value`] from UTF-8 KDL bytes.
pub fn from_bytes(input: &[u8]) -> Result<Value, KdlError> {
    let s = core::str::from_utf8(input).map_err(|_| KdlError::InvalidUtf8)?;
    from_str(s)
}
