# Babbel

A unified polyglot serialization, parsing, and document manipulation workspace in Rust.

## Architecture

The workspace combines four high-performance format libraries while keeping their original upstream codebases disjoint:

- [`xml`](file:///C:/Users/User/.gemini/antigravity-ide/scratch/babbel/projects/xml) (`xml_lib_rust`): XML DOM parser, C14N canonicalization, DTD/XSD validation, and XPath 1.0 engine.
- [`json`](file:///C:/Users/User/.gemini/antigravity-ide/scratch/babbel/projects/json) (`json_lib`): JSON DOM tree, RFC 6901 JSON Pointer, RFC 7396 JSON Merge Patch, and zero-allocation parsing.
- [`yaml`](file:///C:/Users/User/.gemini/antigravity-ide/scratch/babbel/projects/yaml) (`yaml_lib`): YAML 1.2 parser/emitter with anchors, aliases, custom tags, and multi-document streams.
- [`bencode`](file:///C:/Users/User/.gemini/antigravity-ide/scratch/babbel/projects/bencode) (`bencode_lib`): High-speed, binary-safe BitTorrent Bencode parser and serializer.

### Common Foundation: `babbel_core`

Located at [`crates/babbel_core`](file:///C:/Users/User/.gemini/antigravity-ide/scratch/babbel/crates/babbel_core), this shared library encapsulates common capabilities:
- **I/O Streaming**: `ISource`, `IByteStream`, `IDestination`, `IIndentationAware`, `SliceSource`, `StringSource`, `BufferDestination`, `StringDestination`.
- **Text & Encoding**: BOM detection (UTF-8, UTF-16LE, UTF-16BE), newline normalization (`normalize_newlines`).
- **Character Lexing**: Fast ASCII classification (`is_whitespace`, `is_digit`, `is_hex_digit`, `is_newline`).
- **Fast Numeric Formatting**: Zero-allocation formatting via `itoa` and `dtoa`.
- **Diagnostics**: `Location` (1-based line/col, byte offset), `Span`, and rustc-style error snippet formatter.
- **Universal Data Model**: `Value` enum and `FormatVisitor` for polyglot format interchange.

### Facade Crate: `babbel`

Located at [`crates/babbel`](file:///C:/Users/User/.gemini/antigravity-ide/scratch/babbel/crates/babbel), providing unified access:

```rust
use babbel::core::StringDestination;

// 1. Core utilities
let mut dest = StringDestination::new();
babbel::core::format_integer(42, &mut dest);

// 2. JSON
let json_doc = babbel::json::from_str(r#"{"status": "ok"}"#).unwrap();

// 3. YAML
let yaml_doc = babbel::yaml::parse_string("status: ok\n").unwrap();

// 4. XML
let xml_doc = babbel::xml::parse("<status>ok</status>").unwrap();

// 5. Bencode
let bencode_doc = babbel::bencode::parse_bytes(b"d6:status2:oke").unwrap();

// 6. Cross-Format Conversions
let yaml = babbel::convert::json_to_yaml(r#"{"service": "api"}"#).unwrap();
let json = babbel::convert::yaml_to_json("service: api\n").unwrap();
```

## Cross-Format Matrix

The `babbel::convert` module enables direct conversions between formats:
- `babbel::convert::json_to_yaml` / `yaml_to_json`
- `babbel::convert::json_to_xml` / `yaml_to_xml`
- `babbel::convert::json_to_bencode` / `bencode_to_json`
- `babbel::convert::bencode_to_yaml` / `bencode_to_xml`

## Building & Testing

```bash
# Check all workspace members
cargo check --workspace

# Run babbel_core unit tests (10 tests)
cargo test -p babbel_core

# Run polyglot smoke test
cargo test -p babbel --test smoke_test
```
