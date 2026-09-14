# `babbel_cbor`

Fast, binary-safe RFC 8949 CBOR (Concise Binary Object Representation) parser, serializer, and `FormatEngine` in pure Rust with zero external runtime dependencies.

## Features

- **Full RFC 8949 Specification Support**: Encodes and decodes all major types:
  - Major 0: Unsigned integers (`0` up to `u64::MAX`)
  - Major 1: Negative integers (`-1` down to `-1 - u64::MAX`)
  - Major 2: Byte strings (definite & indefinite-length chunks)
  - Major 3: UTF-8 Text strings (definite & indefinite-length chunks)
  - Major 4: Arrays (definite & indefinite-length)
  - Major 5: Maps (definite & indefinite-length key-value pairs)
  - Major 6: Semantic tags (unwrapping & payload preservation)
  - Major 7: Simple values (`false`, `true`, `null`, `undefined`) & IEEE 754 floats (half, single, double precision)
- **Universal AST**: Maps 1:1 with Babbel's [`Value`](../babbel_core/src/model.rs).
- **Security & DoS Protection**: Built-in maximum recursion depth (default 128) and payload size limit (default 64 MB).
- **Embedded & `no_std` Ready**: Zero dynamic allocation requirements for streaming; works in `alloc` environments.
- **OCP & DIP Architecture**: Implements `FormatEngine`, enabling instant integration into `babbel::FormatRegistry` and cross-format conversion.

## Usage

```toml
[dependencies]
babbel_cbor = "0.2.1"
```

### Parsing and Serializing

```rust
use babbel_cbor::{from_bytes, to_vec, CborEngine};
use babbel_core::{FormatEngine, FormatOptions, Value};

let data = Value::Object(vec![
    ("endpoint".into(), Value::String("/api/v1/sensors".into())),
    ("coap_code".into(), Value::Integer(69)),
    ("active".into(), Value::Bool(true)),
]);

// Encode to CBOR binary
let bytes = to_vec(&data)?;

// Decode back to universal Value AST
let decoded = from_bytes(&bytes)?;
assert_eq!(data, decoded);

// Use via FormatEngine trait
let engine = CborEngine;
let bytes2 = engine.serialize_to_vec(&data, &FormatOptions::compact())?;
let decoded2 = engine.parse_bytes(&bytes2)?;
assert_eq!(data, decoded2);
# Ok::<(), Box<dyn std::error::Error>>(())
```
