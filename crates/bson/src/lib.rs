//! # Babbel BSON
//!
//! Fast, binary-safe BSON (Binary JSON) parser, serializer, and DOM engine in pure Rust.
//! Adheres strictly to the official BSON specification (bsonspec.org) and Babbel's SOLID architecture.
//!
//! BSON maps cleanly to Babbel's universal [`Value`] AST:
//! - Null -> `Value::Null`
//! - Boolean -> `Value::Bool`
//! - 32-bit & 64-bit Signed integers -> `Value::Integer(i128)`
//! - 64-bit IEEE 754 floats -> `Value::Float(f64)`
//! - UTF-8 Strings -> `Value::String(String)`
//! - Binary byte arrays -> `Value::Bytes(Vec<u8>)`
//! - Arrays -> `Value::Array(Vec<Value>)`
//! - Embedded documents / Objects -> `Value::Object(Vec<(String, Value)>)`
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_bson::{from_bytes, to_vec, BsonEngine};
//! use babbel_core::Value;
//!
//! let original = Value::Object(vec![
//!     ("database".into(), Value::String("mongodb".into())),
//!     ("port".into(), Value::Integer(27017)),
//! ]);
//!
//! let encoded = to_vec(&original).unwrap();
//! let decoded = from_bytes(&encoded).unwrap();
//! assert_eq!(original, decoded);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod constants;
pub mod error;
pub mod parser;
pub mod serializer;
pub mod engine;

pub use constants::*;
pub use error::BsonError;
pub use parser::{from_bytes, Decoder, DecoderConfig};
pub use serializer::{serialize_to_dest, to_vec};
pub use engine::BsonEngine;
pub use babbel_core::Value;
