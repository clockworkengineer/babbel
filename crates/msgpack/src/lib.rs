//! # Babbel MessagePack
//!
//! Fast, binary-safe, zero-copy MessagePack parser, serializer, and DOM engine in pure Rust.
//! Adheres strictly to the official MessagePack specification and Babbel's SOLID architecture.
//!
//! MessagePack maps with 100% fidelity to Babbel's universal [`Value`] AST:
//! - Nil -> `Value::Null`
//! - Boolean -> `Value::Bool`
//! - Signed & Unsigned integers -> `Value::Integer(i128)`
//! - 32-bit & 64-bit IEEE floats -> `Value::Float(f64)`
//! - UTF-8 Strings -> `Value::String(String)`
//! - Binary byte buffers (`bin 8/16/32`) -> `Value::Bytes(Vec<u8>)`
//! - Arrays -> `Value::Array(Vec<Value>)`
//! - Maps -> `Value::Object(Vec<(String, Value)>)`
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_msgpack::{from_bytes, to_vec, MsgPackEngine};
//! use babbel_core::Value;
//!
//! let original = Value::Object(vec![
//!     ("compact".into(), Value::Bool(true)),
//!     ("schema".into(), Value::Integer(0)),
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
pub use error::MsgPackError;
pub use parser::{from_bytes, Decoder, DecoderConfig};
pub use serializer::{serialize_to_dest, to_vec};
pub use engine::MsgPackEngine;
pub use babbel_core::Value;
