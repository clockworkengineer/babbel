//! # Babbel RON
//!
//! Fast, lightweight, zero-dependency RON (Rusty Object Notation) parser, serializer,
//! and `FormatEngine` in pure Rust.
//! Adheres strictly to the RON specification and Babbel's SOLID architecture.
//!
//! RON maps naturally to Babbel's universal [`Value`] AST:
//! - Unit `()` / `None` -> `Value::Null`
//! - `Some(v)` -> unwrap to `v`
//! - Boolean -> `Value::Bool`
//! - Integers (dec, hex, oct, bin) -> `Value::Integer(i128)`
//! - Floats -> `Value::Float(f64)`
//! - Strings & Characters -> `Value::String(String)`
//! - Byte strings `b"..."` -> `Value::Bytes(Vec<u8>)`
//! - Sequences `[...]` & Tuples `(...)` -> `Value::Array(Vec<Value>)`
//! - Structs `(field: val)` & Maps `{key: val}` -> `Value::Object(Vec<(String, Value)>)`
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_ron::{from_str, to_string, RonEngine};
//! use babbel_core::Value;
//!
//! let original = Value::Object(vec![
//!     ("game".into(), Value::String("space-shooter".into())),
//!     ("fps_limit".into(), Value::Integer(144)),
//!     ("fullscreen".into(), Value::Bool(true)),
//! ]);
//!
//! let ron_str = to_string(&original).unwrap();
//! let decoded = from_str(&ron_str).unwrap();
//! assert_eq!(original, decoded);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod parser;
pub mod serializer;
pub mod engine;

pub use error::RonError;
pub use parser::{from_bytes, from_str, RonParser};
pub use serializer::{serialize_to_dest, to_string, to_string_pretty, to_vec, RonSerializerConfig};
pub use engine::RonEngine;
pub use babbel_core::Value;
