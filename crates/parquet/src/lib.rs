//! # Babbel Parquet
//!
//! Fast, pure-Rust columnar reader, writer, and [`FormatEngine`] implementation for
//! **Apache Parquet (`.parquet`)**, the industry-standard columnar big data format.
//!
//! ## Features
//!
//! - **Pure Rust**: Zero external dependencies, self-contained Apache Thrift Compact Protocol codec.
//! - **Standard Parquet Encoding**: Valid `PAR1` magic bytes, standard `FileMetaData`, `RowGroup`, and `ColumnChunk` structures.
//! - **Rich Types**: Integers (`INT32`, `INT64`), Floating-point (`FLOAT`, `DOUBLE`), Strings (`BYTE_ARRAY` UTF-8), Booleans, and Nullability.
//! - **Unified Architecture**: Integrates into Babbel's universal `Value` AST and cross-format conversion matrix.
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_core::Value;
//! use babbel_parquet::{read_parquet, write_parquet};
//!
//! // Create tabular data as an Array of row Objects
//! let table = Value::Array(vec![
//!     Value::Object(vec![
//!         ("id".into(), Value::Integer(1)),
//!         ("name".into(), Value::String("Alice".into())),
//!         ("score".into(), Value::Float(98.5)),
//!     ]),
//!     Value::Object(vec![
//!         ("id".into(), Value::Integer(2)),
//!         ("name".into(), Value::String("Bob".into())),
//!         ("score".into(), Value::Float(85.0)),
//!     ]),
//! ]);
//!
//! // Write to standard Parquet bytes
//! let parquet_bytes = write_parquet(&table).expect("write failed");
//! assert_eq!(&parquet_bytes[0..4], b"PAR1");
//!
//! // Read back into universal Value AST
//! let roundtrip = read_parquet(&parquet_bytes).expect("read failed");
//! assert_eq!(roundtrip, table);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub mod column;
pub mod engine;
pub mod error;
pub mod metadata;
pub mod reader;
pub mod thrift;
pub mod writer;

pub use column::ColumnData;
pub use engine::ParquetEngine;
pub use error::ParquetError;
pub use metadata::CompressionCodec;
pub use reader::{ParquetReadOptions, read_parquet, read_parquet_with_options};
pub use writer::{ParquetWriteOptions, write_parquet, write_parquet_with_options};

pub use babbel_core::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_parquet_snappy_and_lz4_roundtrip() {
        let table = Value::Array(vec![
            Value::Object(vec![
                ("id".into(), Value::Integer(1)),
                ("name".into(), Value::String("Alice".into())),
                ("score".into(), Value::Float(98.5)),
            ]),
            Value::Object(vec![
                ("id".into(), Value::Integer(2)),
                ("name".into(), Value::String("Bob".into())),
                ("score".into(), Value::Null),
            ]),
            Value::Object(vec![
                ("id".into(), Value::Integer(3)),
                ("name".into(), Value::String("Alice".into())),
                ("score".into(), Value::Float(91.0)),
            ]),
        ]);

        // Snappy compressed
        let snappy_bytes =
            write_parquet_with_options(&table, &ParquetWriteOptions::snappy()).unwrap();
        let snappy_rt = read_parquet(&snappy_bytes).unwrap();
        assert_eq!(snappy_rt, table);

        // LZ4_RAW compressed
        let lz4_bytes =
            write_parquet_with_options(&table, &ParquetWriteOptions::lz4_raw()).unwrap();
        let lz4_rt = read_parquet(&lz4_bytes).unwrap();
        assert_eq!(lz4_rt, table);

        // Column projection + offset + limit
        let projected = read_parquet_with_options(
            &snappy_bytes,
            &ParquetReadOptions::new()
                .with_columns(["name"])
                .with_offset(1)
                .with_limit(1),
        )
        .unwrap();
        let expected_proj = Value::Array(vec![Value::Object(vec![(
            "name".into(),
            Value::String("Bob".into()),
        )])]);
        assert_eq!(projected, expected_proj);
    }
}
