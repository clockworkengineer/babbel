# Babbel Parquet

A fast, zero-dependency, pure-Rust reader, writer, and `FormatEngine` for **Apache Parquet (`.parquet`)**, fully integrated into the **Babbel** serialization and document processing ecosystem.

## Features

- **Standard Parquet Columnar Storage**: Emits and parses valid `PAR1` files conforming to the Apache Parquet specification.
- **Pure-Rust Thrift Compact Protocol**: Self-contained Apache Thrift Compact Protocol encoder and decoder for `FileMetaData`, `SchemaElement`, `RowGroup`, and `PageHeader`.
- **Supported Data Types**: `INT32`, `INT64`, `FLOAT`, `DOUBLE`, `BYTE_ARRAY` (UTF-8 strings and binary), `BOOLEAN`, and nullable columns.
- **Unified Tabular Model**: Matches Babbel's CSV, TSV, and JSON tabular array-of-objects data model for seamless interoperability.
- **Cross-Format Conversion Matrix**: Bidirectional conversion between Parquet and JSON, CSV, TSV, YAML, TOML, MessagePack, CBOR, BSON, RON, and KDL.

## Usage

```rust
use babbel_core::Value;
use babbel_parquet::{read_parquet, write_parquet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dataset = Value::Array(vec![
        Value::Object(vec![
            ("id".into(), Value::Integer(101)),
            ("city".into(), Value::String("Berlin".into())),
            ("temperature".into(), Value::Float(18.5)),
            ("active".into(), Value::Bool(true)),
        ]),
        Value::Object(vec![
            ("id".into(), Value::Integer(102)),
            ("city".into(), Value::String("Tokyo".into())),
            ("temperature".into(), Value::Float(24.0)),
            ("active".into(), Value::Bool(false)),
        ]),
    ]);

    // Serialize to Parquet
    let bytes = write_parquet(&dataset)?;
    println!("Parquet byte length: {} bytes", bytes.len());

    // Deserialize back into Value AST
    let decoded = read_parquet(&bytes)?;
    assert_eq!(decoded, dataset);
    println!("Parquet roundtrip successful!");

    Ok(())
}
```
