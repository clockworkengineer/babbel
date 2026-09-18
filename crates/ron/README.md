# `babbel_ron`

Fast, lightweight, zero-dependency RON (Rusty Object Notation) parser, serializer, and `FormatEngine` in pure Rust.

## Features

- **Full Specification Support**:
  - Rust-style structs: `( field: value, ... )` or `Name( field: value, ... )`
  - Maps: `{ "key": value, ... }`
  - Sequences and Tuples: `[1, 2, 3]` and `(1, "two", 3.0)`
  - Unit identifiers and Enum variants: `Active`, `Color::Red`, `None`, `Some(value)`
  - Raw strings: `r"..."`, `r#"..."#`, `r##"..."##`
  - Byte strings: `b"..."` and `br"..."`
  - Nested multi-line comments `/* outer /* inner */ outer */` and single-line comments `//`
  - Numeric formats: hex (`0x10`), binary (`0b1010`), octal (`0o77`), underscores (`1_000_000`), floats, `inf`, `-inf`, `NaN`
  - Trailing commas everywhere in structs, maps, tuples, and lists
- **Universal AST**: Maps with 100% fidelity to Babbel's universal [`Value`](../babbel_core/src/model.rs).
- **Embedded & `no_std` Support**: Works in `alloc` environments without requiring standard library OS capabilities.
- **OCP & DIP Architecture**: Implements `FormatEngine`, allowing dynamic registration into `babbel::FormatRegistry` and cross-format conversion.

## Usage

```toml
[dependencies]
babbel_ron = "0.2.2"
```

### Parsing and Serializing

```rust
use babbel_ron::{from_str, to_string, RonEngine};
use babbel_core::{FormatEngine, FormatOptions, Value};

let data = Value::Object(vec![
    ("service".into(), Value::String("bevy_game".into())),
    ("framerate".into(), Value::Integer(60)),
    ("vsync".into(), Value::Bool(true)),
]);

// Encode to RON
let ron_str = to_string(&data)?;

// Decode back to universal Value AST
let decoded = from_str(&ron_str)?;
assert_eq!(data, decoded);

// Use via FormatEngine trait
let engine = RonEngine;
let ron_bytes = engine.serialize_to_vec(&data, &FormatOptions::compact())?;
let decoded2 = engine.parse_bytes(&ron_bytes)?;
assert_eq!(data, decoded2);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Examples

Run any of the included examples with cargo:

```bash
# Basic parse, serialize, and AST exploration
cargo run --package babbel_ron --example ron_parse_and_serialize

# Babbel FormatEngine integration and trait usage
cargo run --package babbel_ron --example ron_format_engine
```
