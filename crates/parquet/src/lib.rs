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
pub use reader::read_parquet;
pub use writer::write_parquet;

pub use babbel_core::Value;
