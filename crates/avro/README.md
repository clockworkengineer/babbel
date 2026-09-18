# Babbel Avro (`babbel_avro`)

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Tests](https://img.shields.io/badge/tests-100%25%20passing-brightgreen.svg)](../../docs/conformance/AVRO_CONFORMANCE.md)

Fast, modular, pure-Rust parser, serializer, and [`FormatEngine`](../../crates/babbel_core/src/codec.rs) implementation for **Apache Avro** binary encoding and Object Container Files (`.avro`).

---

## Features

- **Schema-Driven Binary Serialization & Deserialization**:
  - Deserialize raw Avro byte payloads using an explicit [`AvroSchema`] AST (`from_bytes_with_schema`).
  - Serialize universal [`Value`] ASTs directly into compact Avro binary payloads (`to_vec_with_schema`).
- **Object Container File (OCF) Support**:
  - Parse `.avro` container files containing embedded JSON schemas (`"avro.schema"`), compression codecs (`"avro.codec"`), and 16-byte random sync markers (`from_bytes_ocf`).
  - Generate standards-compliant OCF files with automatic sync marker insertion and block headers (`to_vec_ocf`).
- **Rich Schema AST & Parser**:
  - Primitives: `null`, `boolean`, `int`, `long`, `float`, `double`, `bytes`, `string`.
  - Complex types: `record` (fields, defaults, namespaces), `enum` (symbols), `array` (items), `map` (values), `union` (tagged variants), `fixed` (fixed-size bytes).
  - Environment-based resolution of recursive and named schema references.
- **Variable-Length Zigzag Encoding**:
  - High-performance, zero-allocation zigzag integer codec for 32-bit and 64-bit integers (`AvroEncoder::write_long`, `AvroDecoder::read_long`).
- **IEEE 754 Float & Double Codec**:
  - Little-Endian encoding and decoding for single-precision (`f32`) and double-precision (`f64`) numbers.
- **Architectural Excellence & Plugin Integration**:
  - Zero external runtime dependencies beyond Babbel workspace crates (`babbel_core`, `babbel_json`).
  - Implements [`FormatEngine`] via [`AvroEngine`] with automatic OCF magic (`Obj\x01`) detection.
  - Full `no_std` + `alloc` support for embedded and memory-constrained environments.
- **Official Specification Conformance**:
  - **100.0% pass rate** across all 80 test vectors in the official [drnice/AvroTest](https://github.com/drnice/AvroTest.git) corpus and Apache Avro 1.x specification.

---

## Quickstart

Add `babbel_avro` to your `Cargo.toml`:

```toml
[dependencies]
babbel_avro = "0.2.2"
babbel_core = "0.2.2"
```

### 1. Schema-Driven Encoding and Decoding

```rust
use babbel_avro::{from_bytes_with_schema, to_vec_with_schema, AvroSchema, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_json = r#"{
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "name", "type": "string"},
            {"name": "favorite_number", "type": ["int", "null"]},
            {"name": "favorite_color", "type": ["string", "null"]}
        ]
    }"#;

    let schema = AvroSchema::parse_str(schema_json)?;

    let user = Value::Object(vec![
        ("name".into(), Value::String("Alyssa".into())),
        ("favorite_number".into(), Value::Integer(256)),
        ("favorite_color".into(), Value::Null),
    ]);

    // Serialize to binary according to the schema
    let encoded_bytes = to_vec_with_schema(&user, &schema)?;

    // Deserialize back from binary
    let decoded_user = from_bytes_with_schema(&encoded_bytes, &schema)?;
    assert_eq!(decoded_user, user);

    Ok(())
}
```

### 2. Reading and Writing Object Container Files (`.avro`)

```rust
use babbel_avro::{from_bytes_ocf, to_vec_ocf, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let users = Value::Array(vec![
        Value::Object(vec![
            ("name".into(), Value::String("Ben".into())),
            ("favorite_number".into(), Value::Integer(7)),
            ("favorite_color".into(), Value::String("red".into())),
        ]),
        Value::Object(vec![
            ("name".into(), Value::String("Charlie".into())),
            ("favorite_number".into(), Value::Null),
            ("favorite_color".into(), Value::String("blue".into())),
        ]),
    ]);

    let schema_json = r#"{"type":"record","name":"User","fields":[{"name":"name","type":"string"},{"name":"favorite_number":["int","null"]},{"name":"favorite_color":["string","null"]}]}"#;

    // Create a complete Object Container File (OCF)
    let ocf_bytes = to_vec_ocf(&users, schema_json)?;

    // Read an OCF file (the schema is parsed automatically from file metadata)
    let decoded_records = from_bytes_ocf(&ocf_bytes)?;
    assert_eq!(decoded_records.as_array().unwrap().len(), 2);

    Ok(())
}
```

### 3. Using the `FormatEngine` Plugin

```rust
use babbel_avro::AvroEngine;
use babbel_core::{FormatEngine, FormatOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = AvroEngine;

    assert_eq!(engine.format_id(), "avro");
    assert_eq!(engine.mime_type(), "application/avro");
    assert_eq!(engine.file_extensions(), &["avro"]);
    assert!(engine.is_binary());

    let val = Value::Object(vec![("id".into(), Value::Integer(42))]);
    let options = FormatOptions::default();

    let bytes = engine.serialize_to_vec(&val, &options)?;
    let parsed = engine.parse_bytes(&bytes)?;
    assert_eq!(parsed, val);

    Ok(())
}
```

---

## Conformance Verification

To run the full official Avro conformance test suite:

```bash
# Download official test suite
./scripts/fetch_avro_test_suite.ps1

# Execute conformance test runner
cargo test -p babbel_avro --test avro_conformance -- --nocapture
```

Detailed test reports, vector breakdowns, and framing specifications are available in the [Avro Conformance Guide](../../docs/conformance/AVRO_CONFORMANCE.md).
