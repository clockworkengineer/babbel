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

pub mod error;
pub mod codec;
pub mod ocf;
pub mod engine;

pub use error::AvroError;
pub use codec::{from_bytes, to_vec, AvroDecoder, AvroEncoder};
pub use ocf::{from_bytes_ocf, to_vec_ocf, OCF_MAGIC};
pub use engine::AvroEngine;
pub use babbel_core::Value;
