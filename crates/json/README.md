# babbel_json

**Version 0.1.0** — A lightweight, modular JSON toolkit for Rust with pluggable I/O sources/destinations, a simple in-memory Node tree, multiple serializers (JSON, YAML, XML, Bencode, TOML), and `no_std` support. Designed for small binaries, predictable behavior, and easy embedding.

- Core `Node` type representing JSON structures
- Parser to build Node trees from streams, strings, or byte slices
- Zero-allocation JSON validation
- Stringifiers to JSON, YAML, XML, Bencode, and TOML
- Performance-optimized serialization with lazy escaping and arena allocation
- File and in-memory buffer I/O abstractions
- Pretty-printing utilities
- Line-delimited JSON (JSON Lines / NDJSON / `.jsonl`) streaming reader and writer
- JSON Pointer (RFC 6901), JSON Patch (RFC 6902), and JSON Merge Patch (RFC 7386)
- JSON Schema validation (Draft 7 subset)
- JSON5 comment stripping
- `json!` macro for ergonomic Node construction
- Unicode-aware file helpers (BOM detection/handling)
- `no_std` compatible with the `alloc` feature

Minimum supported Rust version: 1.88.0

## Features at a glance

- `Node` and `Numeric` types for representing all JSON values
- Parse with `parse`, `from_str`, `from_bytes`, or `parse_with_config`
- `lines::JsonLinesReader` — stream record-by-record without buffering entire files
- `validate_json` — check syntax without building a Node tree
- Streaming-friendly `ISource`/`IDestination` traits
- `FileSource`/`FileDestination` and `BufferSource`/`BufferDestination`
- Pretty printer (`print`) and optimized serializer (`stringify_optimized`)
- Convert a `Node` to alternate formats (YAML, XML, Bencode, TOML)
- JSON Pointer navigation and mutation (`pointer_get`, `pointer_set`, `pointer_remove`)
- JSON Patch operations (`patch::apply_operation`, `patch::apply_patch`)
- JSON Merge Patch (`merge_patch::merge_patch`)
- JSON Schema validation (`schema::SchemaValidator`)
- JSON5 comment stripping (`parser::json5::strip_comments`)
- `json!` macro for inline Node construction
- `ParserConfig` for resource limits; `ParseStats` for profiling
- Unicode file format detection and read/write helpers

## Cargo features

| Feature | Default | Description |
|---|---|---|
| `std` | yes | Enables standard library support (includes `alloc`) |
| `alloc` | yes (via `std`) | Enables heap allocation without full `std` |
| `file-io` | yes | `FileSource`, `FileDestination`, and Unicode file helpers |
| `format-yaml` | no | `to_yaml` encoder |
| `format-xml` | no | `to_xml` encoder |
| `format-bencode` | no | `to_bencode` encoder |
| `format-toml` | no | `to_toml` encoder |
| `json-pointer` | no | JSON Pointer, JSON Patch support |

### `no_std` usage

Disable default features and enable `alloc` for embedded / bare-metal targets:

```toml
[dependencies]
babbel_json = { version = "0.1.0", default-features = false, features = ["alloc"] }
# Or within the workspace:
# babbel_json = { path = "crates/json", default-features = false, features = ["alloc"] }
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
babbel_json = "0.1.0"
```

Or as a workspace path dependency:

```toml
[dependencies]
babbel_json = { path = "crates/json" }
```

All streaming I/O in `babbel_json` (`FileSource`, `FileDestination`, `BufferSource`, `Buffer`) is unified with and powered by [`babbel_core::io`](../babbel_core).

## Embedding Guide

To embed this library in another Rust project:
1. Add the path or version dependency as shown above.
2. Import the required items in your code:
   ```rust
   use babbel_json::{Node, parse, stringify};
   ```
3. For comprehensive architecture and conversion pipelines across the Babbel ecosystem, refer to the [Babbel Architecture Guide](../../docs/ARCHITECTURE.md).

## Quick start

### Parse from a string or bytes

```rust
use babbel_json::{from_str, from_bytes};

let node = from_str(r#"{"name": "Alice", "age": 30}"#).unwrap();
let node = from_bytes(b"[1, 2, 3]").unwrap();
```

### Stream Line-Delimited JSON (JSON Lines / NDJSON)

```rust
use babbel_json::lines::{JsonLinesReader, parse_json_lines, to_json_lines};
use babbel_core::io::SliceSource;

// 1. Batch parsing
let data = "{\"id\": 1}\n{\"id\": 2}\n";
let records = parse_json_lines(data).unwrap();
assert_eq!(records.len(), 2);

// 2. Stream-by-record iteration
let source = SliceSource::new(data);
let mut reader = JsonLinesReader::new(source);
while let Some(Ok(record)) = reader.next_node() {
    println!("ID: {:?}", record.get("id"));
}

// 3. Serialization
let jsonl_output = to_json_lines(&records).unwrap();
```

### Build a Node with the `json!` macro

```rust
use babbel_json::{json, Node};

let value = json!({
    "name": "Alice",
    "age": 30,
    "active": true,
    "scores": [85, 92, 78]
});

assert_eq!(value["name"].as_str(), Some("Alice"));
assert_eq!(value["age"].as_i64(), Some(30));
```

### Parse a file, then pretty-print to another file

```rust
use babbel_json::{FileSource, FileDestination, parse, print};
use std::path::Path;

fn main() -> Result<(), String> {
    let input = "data.json";
    let mut src = FileSource::new(input).map_err(|e| e.to_string())?;
    let node = parse(&mut src).map_err(|e| e.to_string())?;

    let output = Path::new(input).with_extension("pretty.json");
    let mut dst = FileDestination::new(output.to_string_lossy().as_ref())
        .map_err(|e| e.to_string())?;

    print(&node, &mut dst, 4, 0); // 4-space indentation
    Ok(())
}
```

### Build a Node in memory and stringify to a buffer

```rust
use babbel_json::{Node, Numeric, stringify, BufferDestination};

fn main() {
    let node = Node::Object(vec![
        ("name".into(), Node::Str("example".into())),
        ("count".into(), Node::Number(Numeric::Integer(3))),
        ("items".into(), Node::Array(vec![
            Node::Boolean(true),
            Node::None,
            Node::Str("text".into())
        ])),
    ].into_iter().collect());

    let mut buf = BufferDestination::new();
    stringify(&node, &mut buf);
    println!("{}", buf.to_string());
}
```

### Validate JSON without allocating

```rust
use babbel_json::{validate_json, BufferSource, ParserConfig};

let mut source = BufferSource::new(br#"{"valid": true}"#);
let config = ParserConfig::new();
assert!(validate_json(&mut source, &config).is_ok());
```

### Parse with resource limits

```rust
use babbel_json::{parse_with_config, BufferSource, ParserConfig};

let config = ParserConfig {
    max_depth: Some(16),
    max_string_length: Some(1024),
    max_array_size: Some(256),
    max_object_size: Some(64),
};
let mut src = BufferSource::new(b"{}");
let node = parse_with_config(&mut src, &config).unwrap();
```

### Convert a Node to YAML / XML / Bencode

```rust
use babbel_json::{Node, Numeric, to_yaml, to_xml, to_bencode, BufferDestination};

let node = Node::Array(vec![
    Node::Number(Numeric::Integer(1)),
    Node::Number(Numeric::Integer(2)),
    Node::Number(Numeric::Integer(3)),
]);

let mut yaml = BufferDestination::new();
to_yaml(&node, &mut yaml);
println!("YAML:\n{}", yaml.to_string());

let mut xml = BufferDestination::new();
to_xml(&node, &mut xml);
println!("XML:\n{}", xml.to_string());

let mut bencode = BufferDestination::new();
to_bencode(&node, &mut bencode);
println!("Bencode:\n{}", bencode.to_string());
```

### JSON Pointer (RFC 6901)

```rust
use babbel_json::{json, pointer_get, pointer_set};

let mut doc = json!({"users": [{"name": "Alice"}, {"name": "Bob"}]});

// Read a value
let name = pointer_get(&doc, "/users/0/name");
assert_eq!(name.and_then(|n| n.as_str()), Some("Alice"));

// Write a value
pointer_set(&mut doc, "/users/1/name", json!("Charlie")).unwrap();
```

### JSON Patch (RFC 6902)

```rust
use babbel_json::{json, nodes::patch::{apply_patch, PatchOp}};

let mut doc = json!({"a": 1, "b": 2});
let ops = vec![
    PatchOp::Add { path: "/c".into(), value: json!(3) },
    PatchOp::Remove { path: "/b".into() },
];
apply_patch(&mut doc, &ops).unwrap();
// doc is now {"a": 1, "c": 3}
```

### JSON Merge Patch (RFC 7386)

```rust
use babbel_json::{json, nodes::merge_patch::merge_patch};

let mut doc = json!({"a": 1, "b": 2});
let patch = json!({"b": 3, "c": 4});
merge_patch(&mut doc, &patch);
// doc is now {"a": 1, "b": 3, "c": 4}
```

### JSON Schema validation

```rust
use babbel_json::{json, nodes::schema::SchemaValidator};

let schema = json!({
    "type": "object",
    "required": ["name"],
    "properties": {
        "name": {"type": "string"},
        "age":  {"type": "number", "minimum": 0}
    }
});
let validator = SchemaValidator::new(schema);

assert!(validator.validate(&json!({"name": "Alice", "age": 30})).is_ok());
assert!(validator.validate(&json!({"age": 30})).is_err()); // missing "name"
```

### JSON5 comment stripping

```rust
use babbel_json::parser::json5::strip_comments;

let input = r#"{
    "name": "Alice", // single-line comment
    /* multi-line
       comment */
    "age": 30
}"#;
let clean = strip_comments(input);
let node = babbel_json::from_str(&clean).unwrap();
```

### Read/write text files with Unicode BOM handling

```rust
use babbel_json::{Format, detect_format, read_file_to_string, write_file_from_string};

fn main() -> Result<(), String> {
    let fmt = detect_format("input.txt").unwrap_or(Format::UTF8);
    let content = read_file_to_string("input.txt")?;
    write_file_from_string("output.txt", &content, Format::UTF8)
}
```

## API overview

### Version

- `version() -> &'static str`

### I/O

- `FileSource` / `FileDestination` — file-backed I/O (requires `file-io` feature)
- `BufferSource` / `BufferDestination` — in-memory I/O

### Node model

- `Node`: `Object`, `Array`, `Str`, `Number(Numeric)`, `Boolean`, `None`
- `Numeric`: `Integer(i64)`, `UInteger(u64)`, `Float(f64)`, `Byte(u8)`, `Int8(i8)`, `Int16(i16)`, `UInt16(u16)`, `Int32(i32)`, `UInt32(u32)`

### Parsing

- `parse(&mut Source) -> Result<Node, String>`
- `parse_with_config(&mut Source, &ParserConfig) -> Result<Node, String>`
- `from_str(&str) -> Result<Node, String>`
- `from_bytes(&[u8]) -> Result<Node, String>`
- `validate_json(&mut Source, &ParserConfig) -> Result<(), String>` — zero-allocation syntax check

### Stringifying

- `stringify(&Node, &mut Destination)`
- `stringify_pretty(...)` — alias for `print`
- `print(&Node, &mut Destination, indent, current_indent)` — human-readable output
- `stringify_optimized(&Node, &mut Destination) -> Result<(), String>` — lazy escaping, fast paths
- `strip_whitespace(src, dst)` — compact whitespace removal

### Alternative format encoders (feature-gated)

- `to_yaml(&Node, &mut Destination)` — requires `format-yaml`
- `to_xml(&Node, &mut Destination)` — requires `format-xml`
- `to_bencode(&Node, &mut Destination)` — requires `format-bencode`
- `to_toml(&Node, &mut Destination)` — requires `format-toml`

### JSON Pointer (requires `json-pointer`)

- `pointer_get(&Node, &str) -> Option<&Node>`
- `pointer_get_mut(&mut Node, &str) -> Option<&mut Node>`
- `pointer_set(&mut Node, &str, Node) -> Result<(), String>`
- `pointer_remove(&mut Node, &str) -> Result<Node, String>`

### JSON Patch (requires `json-pointer`)

- `nodes::patch::apply_operation(&mut Node, &PatchOp) -> Result<(), PatchError>`
- `nodes::patch::apply_patch(&mut Node, &[PatchOp]) -> Result<(), PatchError>`
- `PatchOp`: `Add`, `Remove`, `Replace`, `Move`, `Copy`, `Test`

### JSON Merge Patch

- `nodes::merge_patch::merge_patch(&mut Node, &Node)`

### JSON Schema validation

- `nodes::schema::SchemaValidator::new(schema: Node) -> SchemaValidator`
- `SchemaValidator::validate(&Node) -> Result<(), Vec<ValidationError>>`

### JSON5

- `parser::json5::strip_comments(&str) -> String`

### Parser configuration and profiling

- `ParserConfig::new()` — conservative limits suitable for embedded systems
- `ParserConfig::unlimited()` — no limits
- `ParseStats` — peak depth, node count, string bytes, estimated memory, parse time

### Unicode-aware file helpers (requires `file-io`)

- `detect_format(path) -> Result<Format, String>`
- `read_file_to_string(path) -> Result<String, String>`
- `write_file_from_string(path, &str, Format) -> Result<(), String>`

### `json!` macro

- `json!(null)`, `json!(true)`, `json!(42)`, `json!("text")`, `json!([...])`, `json!({...})`

## Performance features

Several `alloc`-gated modules provide lower-overhead alternatives for hot paths:

- `parser::arena` — arena allocator to reduce per-node heap allocations
- `parser::fast` — SIMD-friendly fast parsing utilities
- `parser::sso` — Small String Optimization for short keys and values
- `stringify::optimized::stringify_optimized` — lazy escaping and batch writes

## Design goals

- Small, explicit API focused on Node-based manipulation
- Works with streams and simple abstractions
- Deterministic and predictable output
- `no_std` compatible for embedded and constrained targets
- Easy to integrate for CLI tools, small services, and firmware

## Roadmap ideas

- Optional parse/stringify options (e.g., key sorting, non-ASCII escaping)
- Streaming parser and incremental writer
- Canonical JSON and stable hashing
- Optional serde interop and async I/O

If you're interested in any of these, contributions are welcome.

## Documentation

See the [Documentation Hub](../../docs/README.md) for complete guides:
- [Architecture Guide](../../docs/ARCHITECTURE.md)
- [Development Guide](../../docs/DEVELOPMENT_GUIDE.md)
- [Contributing Guidelines](../../docs/CONTRIBUTING.md)
- [Code of Conduct](../../docs/CODE_OF_CONDUCT.md)
- [Embedded Systems Guide](../../docs/EMBEDDED_GUIDE.md)
- [Security Policy](../../docs/SECURITY.md)

## License

This project is licensed under the terms of the LICENSE file included in the repository.

## Contributing

- Open issues for bugs and feature requests
- PRs with tests are welcome
- Please format and lint your code before submitting

## Support

If you run into issues or have questions, please open an issue in the repository.