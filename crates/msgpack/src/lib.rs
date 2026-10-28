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
pub mod engine;
pub mod error;
pub mod parser;
pub mod pull;
pub mod serializer;

pub use babbel_core::Value;
pub use constants::*;
pub use engine::MsgPackEngine;
pub use error::MsgPackError;
pub use parser::{Decoder, DecoderConfig, from_bytes};
pub use pull::{MsgPackPullEvent, MsgPackPullParser};
pub use serializer::{
    MsgPackTimestamp, decode_timestamp, encode_timestamp, serialize_to_dest, to_vec,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msgpack_timestamp_32_64_96_roundtrip() {
        // 32-bit timestamp (nanoseconds == 0, fits in u32)
        let ts32 = MsgPackTimestamp::new(1_700_000_000, 0);
        let b32 = encode_timestamp(ts32);
        assert_eq!(b32.len(), 6);
        assert_eq!(decode_timestamp(&b32).unwrap(), ts32);

        // 64-bit timestamp (has nanoseconds)
        let ts64 = MsgPackTimestamp::new(1_700_000_000, 500_000_000);
        let b64 = encode_timestamp(ts64);
        assert_eq!(b64.len(), 10);
        assert_eq!(decode_timestamp(&b64).unwrap(), ts64);

        // 96-bit timestamp (negative seconds before 1970)
        let ts96 = MsgPackTimestamp::new(-86_400, 123_456_789);
        let b96 = encode_timestamp(ts96);
        assert_eq!(b96.len(), 15);
        assert_eq!(decode_timestamp(&b96).unwrap(), ts96);
    }
}
