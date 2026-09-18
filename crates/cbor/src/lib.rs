//! # Babbel CBOR
//!
//! Fast, binary-safe, zero-copy RFC 8949 CBOR (Concise Binary Object Representation)
//! parser, serializer, and DOM engine in pure Rust.
//! Adheres strictly to the IETF CBOR standard and Babbel's SOLID architecture.
//!
//! CBOR maps with 100% fidelity to Babbel's universal [`Value`] AST:
//! - Null / Undefined -> `Value::Null`
//! - Boolean -> `Value::Bool`
//! - Unsigned & Negative integers -> `Value::Integer(i128)`
//! - 16-bit, 32-bit & 64-bit IEEE floats -> `Value::Float(f64)`
//! - UTF-8 Strings (definite & indefinite) -> `Value::String(String)`
//! - Byte arrays (definite & indefinite) -> `Value::Bytes(Vec<u8>)`
//! - Arrays (definite & indefinite) -> `Value::Array(Vec<Value>)`
//! - Maps (definite & indefinite) -> `Value::Object(Vec<(String, Value)>)`
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_cbor::{from_bytes, to_vec, CborEngine};
//! use babbel_core::Value;
//!
//! let original = Value::Object(vec![
//!     ("standard".into(), Value::String("RFC 8949".into())),
//!     ("compact".into(), Value::Bool(true)),
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
pub mod edn;
pub mod engine;
pub mod error;
pub mod parser;
pub mod pull;
pub mod serializer;

pub use babbel_core::Value;
pub use constants::*;
pub use edn::{from_edn, to_edn};
pub use engine::CborEngine;
pub use error::CborError;
pub use parser::{Decoder, DecoderConfig, from_bytes};
pub use pull::{CborPullEvent, CborPullParser};
pub use serializer::{serialize_to_dest, to_vec};
