# `babbel_msgpack`

Fast, binary-safe MessagePack parser, serializer, and `FormatEngine` in pure Rust with zero external runtime dependencies.

## Features

- **Full Specification Support**: Encodes and decodes all standard MessagePack types:
  - FixInt, UInt 8/16/32/64, Int 8/16/32/64
  - Nil, Boolean (`false` / `true`)
  - Float 32 / Float 64
  - FixStr, Str 8/16/32 (validated UTF-8)
  - Bin 8/16/32 (`Value::Bytes`)
  - FixArray, Array 16/32 (`Value::Array`)
  - FixMap, Map 16/32 (`Value::Object`)
  - FixExt 1/2/4/8/16, Ext 8/16/32
- **Universal AST**: Maps 1:1 with Babbel's [`Value`](../babbel_core/src/model.rs).
- **Security & DoS Protection**: Built-in maximum recursion depth (default 128) and payload size limit (default 64 MB).
- **Zero Heavy Dependencies**: Pure Rust implementation matching Babbel's lightweight footprint.
- **OCP & DIP Architecture**: Implements `FormatEngine`, allowing dynamic registration into `babbel::FormatRegistry` and instant cross-format conversion.

## Usage

```toml
[dependencies]
babbel_msgpack = "0.2.1"
```

### Parsing and Serializing

```rust
use babbel_msgpack::{from_bytes, to_vec, MsgPackEngine};
use babbel_core::{FormatEngine, FormatOptions, Value};

let data = Value::Object(vec![
    ("service".into(), Value::String("babbel-api".into())),
    ("port".into(), Value::Integer(8080)),
    ("enabled".into(), Value::Bool(true)),
]);

// Encode to MessagePack binary
let bytes = to_vec(&data)?;

// Decode back to universal Value AST
let decoded = from_bytes(&bytes)?;
assert_eq!(data, decoded);

// Use via FormatEngine trait
let engine = MsgPackEngine;
let bytes2 = engine.serialize_to_vec(&data, &FormatOptions::compact())?;
let decoded2 = engine.parse_bytes(&bytes2)?;
assert_eq!(data, decoded2);
# Ok::<(), Box<dyn std::error::Error>>(())
```
