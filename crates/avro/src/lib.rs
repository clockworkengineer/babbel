//! # Babbel Avro
//!
//! Fast, pure-Rust parser, serializer, and [`FormatEngine`] implementation for
//! **Apache Avro** binary encoding and Object Container Files (`.avro`).
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_avro::{from_bytes, to_vec};
//! use babbel_core::Value;
//!
//! let record = Value::Object(vec![
//!     ("id".into(), Value::Integer(42)),
//!     ("name".into(), Value::String("Babbel".into())),
//! ]);
//!
//! let encoded = to_vec(&record).unwrap();
//! let decoded = from_bytes(&encoded).unwrap();
//! assert_eq!(decoded.get("name").and_then(|v| v.as_str()), Some("Babbel"));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod codec;
pub mod engine;
pub mod error;
pub mod ocf;
pub mod schema;

pub use babbel_core::Value;
pub use codec::{
    AvroDecoder, AvroEncoder, from_bytes, from_bytes_with_schema, to_vec, to_vec_with_schema,
};
pub use engine::AvroEngine;
pub use error::AvroError;
pub use ocf::{OCF_MAGIC, from_bytes_ocf, to_vec_ocf};
pub use schema::{AvroField, AvroSchema};
