# bencode_lib

A Rust library for parsing, constructing, and converting Bencode data. Designed for embedded systems, resource-constrained environments, and general-purpose use. Supports round-tripping Bencode and conversion to JSON, YAML, XML, and TOML.

[![Repository](https://img.shields.io/badge/github-clockworkengineer%2Fbencode-blue)](https://github.com/clockworkengineer/bencode)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)
[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Donate-orange.svg)](https://buymeacoffee.com/roberttizz1)

## Features

- Parse Bencode into a typed tree (`Node`)
- Serialize `Node` back to canonical Bencode
- Convert `Node` to JSON, YAML, XML, or TOML (optional Cargo features)
- Zero-copy borrowed parsing via `BorrowedNode` — no heap allocation
- `no_std` compatible (disable the default `std` feature)
- Memory pool / arena allocation (`Arena`, `StackBuffer`, `MemoryTracker`)
- Stack-based iterative parser — safe for deeply nested structures
- Validation helpers for ergonomic field extraction (`get_required`, `get_int_required`, …)
- Configurable parsing depth (`ParserConfig`) and canonicalisation enforcement (`EncoderConfig`)
- Read/write from files or in-memory buffers

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bencode_lib = "0.2.1"
```

Or as a path dependency within this workspace:

```toml
[dependencies]
bencode_lib = { path = "library" }
```

To minimise binary size, disable unused format-conversion features:

```toml
[dependencies]
bencode_lib = { version = "0.2.1", default-features = false, features = ["std", "json"] }
```

Available features: `std` (default), `json`, `toml`, `xml`, `yaml`.

## Release Builds & LTO for Optimal Size

For the smallest and fastest binaries, enable Link Time Optimization (LTO):

```toml
[profile.release]
lto = true
```

Then build with:

```sh
cargo build --release
```

## Quick Examples

### Parse a `.torrent` file and convert to YAML

```rust
use bencode_lib::{FileSource, FileDestination, parse, to_yaml};

let mut src = FileSource::new("example.torrent")?;
let node = parse(&mut src)?;
let mut dst = FileDestination::new("example.yaml")?;
to_yaml(&node, &mut dst);
```

### Round-trip a Bencode buffer

```rust
use bencode_lib::{parse_bytes, stringify_to_bytes};

let raw = b"d3:foo3:bar4:spamli1ei2ei3eee";
let node = parse_bytes(raw)?;
let encoded = stringify_to_bytes(&node);
assert_eq!(raw.as_slice(), encoded.as_slice());
```

### Construct a `Node` and render as JSON

```rust
use bencode_lib::{Node, make_node, to_json, BufferDestination};

// Using the From trait with an array of key-value pairs
let node = Node::from([
    ("name", Node::from("hello")),
    ("count", Node::from(42_i64)),
]);
let mut dst = BufferDestination::new();
to_json(&node, &mut dst);
```

### Validate and extract fields ergonomically

```rust
use bencode_lib::parse_bytes;

let node = parse_bytes(b"d4:name5:Alice3:agei30ee")?;
let name: &str = node.get_string_required("name")?;
let age:  i64  = node.get_int_required("age")?;
```

## Data Model

```
Node::Integer(i64)                      — Bencode integer
Node::Str(String)                       — Bencode string (UTF-8)
Node::List(Vec<Node>)                   — Bencode list
Node::Dictionary(HashMap<String, Node>) — Bencode dictionary
Node::None                              — Empty / uninitialized node
```

Nodes implement `Clone`, `Debug`, `PartialEq`, and `Display`.

### Creating Nodes

```rust
use bencode_lib::{Node, make_node};
use std::collections::HashMap;

// Direct variants
let i = Node::Integer(42);
let s = Node::Str("hello".to_string());

// Via the generic helper (uses From conversions)
let n = make_node(99_i64);          // -> Node::Integer(99)
let n = make_node("world");         // -> Node::Str("world")
let n = make_node(vec![1_i64, 2]);  // -> Node::List([Integer(1), Integer(2)])

// Array literal short-hand
let list = Node::from([1_i64, 2, 3]);
let dict = Node::from([("key", Node::Integer(1))]);
```

## API Overview

### Parsing

| Function | Description |
|---|---|
| `parse(&mut src)` | Parse from any `Source` (file or buffer) |
| `parse_bytes(data: &[u8])` | Parse directly from a byte slice |
| `parse_str(data: &str)` | Parse directly from a string slice |
| `parse_iterative(&mut src)` | Stack-based iterative parse (deep nesting safe) |
| `parse_bytes_iterative(data)` | Iterative parse from byte slice |
| `parse_str_iterative(data)` | Iterative parse from string slice |
| `parse_borrowed(data: &[u8])` | Zero-copy parse returning `BorrowedNode` |
| `validate_bencode(data: &[u8])` | Validate without building a node tree |

### Stringifying

| Function | Description |
|---|---|
| `stringify(&node, &mut dst)` | Write canonical Bencode to any `Destination` |
| `stringify_to_bytes(&node)` | Return Bencode as `Vec<u8>` |
| `stringify_to_string(&node)` | Return Bencode as `String` |
| `to_json(&node, &mut dst)` | Convert to JSON (`json` feature) |
| `to_toml(&node, &mut dst)` | Convert to TOML (`toml` feature) |
| `to_xml(&node, &mut dst)` | Convert to XML (`xml` feature) |
| `to_yaml(&node, &mut dst)` | Convert to YAML (`yaml` feature) |

### I/O

| Type | Description |
|---|---|
| `BufferSource` | Read Bencode from an in-memory buffer |
| `FileSource` | Read Bencode from a file (`std` feature) |
| `BufferDestination` | Write output to an in-memory buffer |
| `FileDestination` | Write output to a file (`std` feature) |

### Node Methods

**Type checking:** `is_integer()`, `is_string()`, `is_list()`, `is_dictionary()`, `is_none()`

**Value access:** `as_integer()`, `as_string()`, `as_list()`, `as_list_mut()`, `as_dictionary()`, `as_dictionary_mut()`

**Dictionary access:** `get(key)`, `get_mut(key)`

**Validation helpers:**

| Method | Returns |
|---|---|
| `get_required(key)` | `Result<&Node, String>` |
| `get_int_required(key)` | `Result<i64, String>` |
| `get_string_required(key)` | `Result<&str, String>` |
| `get_list_required(key)` | `Result<&Vec<Node>, String>` |
| `get_dict_required(key)` | `Result<&HashMap<String, Node>, String>` |
| `get_int_optional(key)` | `Option<i64>` |
| `get_string_optional(key)` | `Option<&str>` |
| `get_list_optional(key)` | `Option<&Vec<Node>>` |
| `get_dict_optional(key)` | `Option<&HashMap<String, Node>>` |

**Utility:** `len()`, `is_empty()`, `type_name()`

### Embedded / `no_std` API

| Type / Function | Description |
|---|---|
| `Arena` | Bump allocator from a fixed buffer |
| `StackBuffer<N>` | Stack-allocated byte buffer |
| `MemoryTracker` | Allocation accounting for embedded systems |
| `FixedSizeBuffer<N>` | Stack-allocated fixed-size buffer (const generic) |
| `MemoryBounds` | Const-generic memory bounds calculator |
| `BorrowedNode` | Zero-copy borrowed node (no allocation) |

### Configuration

```rust
use bencode_lib::{ParserConfig, EncoderConfig};

let parser = ParserConfig::new().with_max_depth(50);   // default: 100

let encoder = EncoderConfig::new()
    .with_canonical(true)               // enforce sorted dict keys, no leading zeros
    .with_dict_order_verification(true);
```

### Utilities

| Function | Description |
|---|---|
| `version()` | Returns the library version string |
| `read_file(path)` | Read a file to `String` (`std` feature) → `Result<String, io::Error>` |
| `write_file(path, content)` | Write a string to a file (`std` feature) → `Result<(), io::Error>` |

## Error Handling

`BencodeError` is a lightweight, allocation-free enum suitable for `no_std` environments:

```rust
pub enum BencodeError {
    EmptyInput,
    InvalidInteger,
    UnterminatedInteger,
    InvalidStringLength,
    StringTooShort,
    UnterminatedList,
    UnterminatedDictionary,
    DictKeysOutOfOrder,
    DictKeyMustBeString,
    UnexpectedCharacter(char),
    FileNotFound,
    IoError,
}
```

Each variant exposes `.code() -> u8` for compact logging and `.as_str() -> &'static str` for human-readable messages. In `std` environments, `BencodeError` implements `std::error::Error`.

> **Note:** `read_file` / `write_file` return `std::io::Error`, not `BencodeError`.

## Minimum Supported Rust Version

Rust **1.85.0** (edition 2024).

## Documentation

- [`../docs/API_OVERVIEW.md`](../docs/API_OVERVIEW.md) — API reference
- [`../docs/DEVELOPER_GUIDE.md`](../docs/DEVELOPER_GUIDE.md) — contributing and project structure
- [`../docs/EMBEDDED_GUIDE.md`](../docs/EMBEDDED_GUIDE.md) — `no_std` / embedded usage
- [`../docs/CHANGELOG.md`](../docs/CHANGELOG.md) — release history
- [`../examples/README.md`](../examples/README.md) — example programs

## Support

If you find this project useful, you can support its development by buying me a coffee:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-Donate-orange.svg)](https://buymeacoffee.com/roberttizz1)

Or visit [buymeacoffee.com/roberttizz1](https://buymeacoffee.com/roberttizz1).

## License

MIT License. See [LICENSE](../LICENSE) for details.
