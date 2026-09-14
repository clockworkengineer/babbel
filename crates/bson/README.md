# `babbel_bson`

Fast, binary-safe BSON (Binary JSON) parser, serializer, and `FormatEngine` in pure Rust with zero external runtime dependencies.

## Features

- **Full BSON Specification Support**: Implements official BSON (bsonspec.org) data types:
  - 64-bit IEEE 754 Floating point (`0x01`)
  - UTF-8 Length-prefixed Strings (`0x02`)
  - Embedded Documents (`0x03` -> `Value::Object`)
  - Arrays (`0x04` -> `Value::Array`)
  - Binary Data with Subtypes (`0x05` -> `Value::Bytes`)
  - Boolean (`0x08` -> `Value::Bool`)
  - UTC Datetime (`0x09` -> `Value::Integer`)
  - Null (`0x0A` -> `Value::Null`)
  - 32-bit Integers (`0x10` -> `Value::Integer`)
  - 64-bit Timestamps (`0x11` -> `Value::Integer`)
  - 64-bit Integers (`0x12` -> `Value::Integer`)
  - 12-byte ObjectIds (`0x07` -> `Value::Bytes`)
- **Universal AST**: Maps 1:1 with Babbel's [`Value`](../babbel_core/src/model.rs).
- **Security & DoS Protection**: Built-in maximum recursion depth (default 128) and payload size limit (default 64 MB).
- **Zero Heavy Dependencies**: Pure Rust implementation with zero external crates.
- **OCP & DIP Architecture**: Implements `FormatEngine`, enabling instant integration into `babbel::FormatRegistry` and cross-format conversion.

## Usage

```toml
[dependencies]
babbel_bson = "0.2.1"
```

### Parsing and Serializing

```rust
use babbel_bson::{from_bytes, to_vec, BsonEngine};
use babbel_core::{FormatEngine, FormatOptions, Value};

let data = Value::Object(vec![
    ("database".into(), Value::String("mongodb".into())),
    ("port".into(), Value::Integer(27017)),
    ("active".into(), Value::Bool(true)),
]);

// Encode to BSON binary
let bytes = to_vec(&data)?;

// Decode back to universal Value AST
let decoded = from_bytes(&bytes)?;
assert_eq!(data, decoded);

// Use via FormatEngine trait
let engine = BsonEngine;
let bytes2 = engine.serialize_to_vec(&data, &FormatOptions::compact())?;
let decoded2 = engine.parse_bytes(&bytes2)?;
assert_eq!(data, decoded2);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Examples

Run any of the included examples with cargo:

```bash
# Basic parse, serialize, and typed document exploration
cargo run --package babbel_bson --example bson_parse_and_serialize

# Database record file I/O (writing and reading .bson files)
cargo run --package babbel_bson --example bson_file_io

# Babbel FormatEngine integration and trait usage
cargo run --package babbel_bson --example bson_format_engine
```

