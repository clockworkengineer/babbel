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
pub use parser::{Decoder, DecoderConfig, from_bytes, from_bytes_seq};
pub use pull::{CborPullEvent, CborPullParser};
pub use serializer::{
    EncoderConfig, serialize_to_dest, serialize_to_dest_canonical, to_vec, to_vec_canonical,
    to_vec_seq,
};

/// Convenience alias parsing CBOR from a byte slice into [`Value`].
#[inline]
pub fn from_slice(bytes: &[u8]) -> Result<Value, CborError> {
    from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_cbor_canonical_map_key_sorting_and_sequences() {
        let obj = Value::Object(vec![
            ("longer_key".into(), Value::Integer(1)),
            ("z".into(), Value::Integer(2)),
            ("a".into(), Value::Integer(3)),
        ]);
        let canon = to_vec_canonical(&obj).unwrap();
        let decoded = from_bytes(&canon).unwrap();
        // In canonical CBOR, 1-byte keys "a" and "z" come before 10-byte key "longer_key"
        let keys: alloc::vec::Vec<&str> = decoded
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, vec!["a", "z", "longer_key"]);

        let seq_items = vec![
            Value::Integer(10),
            Value::String("seq".into()),
            Value::Bool(true),
        ];
        let seq_bytes = to_vec_seq(&seq_items).unwrap();
        let seq_decoded = from_bytes_seq(&seq_bytes).unwrap();
        assert_eq!(seq_decoded, seq_items);
    }
}
