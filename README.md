# Babbel

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-3500%2B%20passing-brightgreen.svg)]()
[![Architecture: DRY & SOLID](https://img.shields.io/badge/architecture-DRY%20%26%20SOLID-purple.svg)](docs/ARCHITECTURE.md)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Donate-FFDD00?logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/roberttizz1)

A high-performance, polyglot serialization, parsing, and document manipulation workspace in Rust. Babbel brings together **JSON**, **YAML**, **Bencode**, **XML**, **CSV / TSV**, **INI / Properties**, and **JSON Lines** under a unified, modular architecture adhering strictly to **DRY** (Don't Repeat Yourself) and **SOLID** engineering principles.

📖 **Documentation & Guides**:
- [Documentation Hub](docs/README.md) — Central directory of all specifications, guides, and tutorials
- [Architecture Guide](docs/ARCHITECTURE.md) — 3-tier layering model and SOLID design principles
- [Embedded Systems Guide](docs/EMBEDDED_GUIDE.md) — `no_std`, stack buffers (`StackBuffer`), and zero-allocation streaming
- [Text Support Guide](docs/TEXT_SUPPORT_GUIDE.md) — RFC 4180 CSV/TSV, INI/.env, JSON Lines, and frontmatter
- [Conversion Matrix](docs/CONVERSION_MATRIX.md) — $O(N)$ cross-format conversion reference & options
- [Memory & Benchmarks](docs/BENCHMARKS_AND_MEMORY.md) — Struct size bounds verification (`size_checks.rs`)
- [Security Policy](docs/SECURITY.md) — Threat model, Billion Laughs mitigations, and 64 MB DoS limits
- [Development Guide](docs/DEVELOPMENT_GUIDE.md) — Contributor onboarding, toolchain, testing, and release builds
- [Contributing Guidelines](docs/CONTRIBUTING.md) — Engineering standards and code conventions

---

## Workspace Architecture

Babbel is structured as an interconnected multi-crate workspace:

| Crate | Package Docs | Directory | Description |
| :--- | :--- | :--- | :--- |
| **`babbel`** | [README](crates/babbel/README.md) | [`crates/babbel`](crates/babbel) | Master facade crate providing high-level ergonomics, prelude, and open-ended cross-format conversion pipelines across 8 formats. |
| **`babbel_core`** | [README](crates/babbel_core/README.md) | [`crates/babbel_core`](crates/babbel_core) | Core architectural kernel containing streaming traits (`ISource`, `IDestination`, `ILineReader`), universal `Value` AST, RFC 4180 CSV/TSV, sectioned INI/.env, frontmatter processing, Unicode BOM detection, and format codec abstractions. |
| **`babbel_json`** | [README](crates/json/README.md) | [`crates/json`](crates/json) | Full-featured JSON DOM engine supporting RFC 6901 JSON Pointer, RFC 7396 JSON Merge Patch, JSON Lines (`.jsonl`/`.ndjson`) streaming, and zero-copy parsing. |
| **`babbel_yaml`** | [README](crates/yaml/README.md) | [`crates/yaml`](crates/yaml) | YAML 1.2 parser and emitter with full support for anchors, aliases, custom tags, multiline block scalars, and multi-document streams. |
| **`babbel_bencode`** | [README](crates/bencode/README.md) | [`crates/bencode`](crates/bencode) | High-speed, binary-safe BitTorrent Bencode parser and serializer supporting zero-copy borrowed slices and iterative streaming. |
| **`babbel_xml`** | [README](crates/xml/README.md) | [`crates/xml`](crates/xml) | Robust XML DOM parser, W3C Canonical XML (C14N 1.0/1.1), DTD validation, XSD schema validator, and XPath 1.0 query engine. |

---

## Key Design Principles

### 1. Unified Streaming I/O (`ISource` & `IDestination`)
All format crates across Babbel share the same streaming abstractions from [`babbel_core::io`](crates/babbel_core/src/io):
- **Streaming Input (`ISource`)**: Every parser (`babbel_json::parse`, `babbel_yaml::parse`, `babbel_bencode::parse`, `babbel_xml::parse_source`) ingests data via `&mut dyn ISource`.
- **Streaming Output (`IDestination`)**: Every emitter and serializer (`to_json`, `to_yaml`, `to_bencode`, `to_xml`, `stringify_to`, `emit_csv_to`, `emit_ini_to`) writes sequentially to `&mut dyn IDestination`.
- **Interface Segregation (ISP)**: Minimal sub-traits allow clients to bind only to the capabilities they require:
  - [`ILineReader`](crates/babbel_core/src/io/traits.rs): Line-by-line reading across mixed `\r\n`, `\n`, `\r` endings without full buffering.
  - [`ICharStream`](crates/babbel_core/src/io/traits.rs): Minimal pull-based character stream (`current()`, `next()`, `more()`).
  - [`IByteStream`](crates/babbel_core/src/io/traits.rs): Raw byte reading for binary protocols (Bencode).
  - [`IByteWriter`](crates/babbel_core/src/io/traits.rs): Binary byte writing for output destinations.
  - [`IRewindable`](crates/babbel_core/src/io/traits.rs): Reset streams to initial state.
  - [`IPositionAware`](crates/babbel_core/src/io/traits.rs): Absolute byte offset tracking.
  - [`ILocationAware`](crates/babbel_core/src/io/traits.rs): 1-based line and column metric tracking.
  - [`ITailInspectable`](crates/babbel_core/src/io/traits.rs): Inspect last written byte without file reopening.
  - [`IClearable`](crates/babbel_core/src/io/traits.rs): Buffer/destination truncation.
  - [`IFlushable`](crates/babbel_core/src/io/traits.rs): Storage buffer synchronization.
  - [`IIndentationAware`](crates/babbel_core/src/io/traits.rs): Indentation calculation for whitespace-sensitive grammars.

### 2. Open-Closed & Dependency Inversion (OCP & DIP)
- **Extensible Codecs**: New serialization formats can be integrated simply by implementing [`FormatParser`](crates/babbel_core/src/codec.rs) and [`FormatEmitter`](crates/babbel_core/src/codec.rs).
- **Universal Data Pipeline**: Rather than $O(N^2)$ hand-rolled cross-serializers, conversions flow through the universal [`Value`](crates/babbel_core/src/model.rs) AST:
  ```rust
  let output = babbel::convert::convert_text(input_str, &parser, &emitter)?;
  let bytes  = babbel::convert::convert_bytes(input_bytes, &parser, &emitter)?;
  ```
- **Unified Error Handling**: Format-specific errors implement `Into<BabbelError>` with normalized classification codes (`ErrorCode::Syntax`, `ErrorCode::Encoding`, `ErrorCode::Io`, etc.).

### 3. DRY Consolidation
- **Shared Unicode Engine**: Automatic BOM detection (UTF-8, UTF-16 LE, UTF-16 BE) and newline normalization (`\r\n` / `\r` $\rightarrow$ `\n`).
- **Canonical Escaping**: Standardized escaping algorithms in `babbel_core::text` for JSON string literals, XML character entities, and YAML delimiters.
- **Universal Buffers**: Reusable in-memory `Buffer` and `BufferSource` implementations eliminating duplicate stream wrappers across format libraries.

---

## Quick Start & Usage

Add `babbel` to your `Cargo.toml`:

```toml
[dependencies]
babbel = { path = "crates/babbel", features = ["json", "yaml", "xml", "bencode"] }
```

### Parsing & Document Access

```rust
use babbel::json;
use babbel::yaml;
use babbel::xml;
use babbel::bencode;

// 1. Parse JSON
let json_node = json::from_str(r#"{"service": "babbel", "status": "active"}"#)?;
assert_eq!(json_node.get("status").and_then(|n| n.as_str()), Some("active"));

// 2. Parse YAML
let yaml_node = yaml::parse_string("service: babbel\nstatus: active\n")?;
assert_eq!(yaml_node.get("service").and_then(|n| n.as_str()), Some("babbel"));

// 3. Parse XML
let xml_doc = xml::parse("<service name=\"babbel\"><status>active</status></service>")?;
assert_eq!(xml_doc.get_root_element_name(), Some("service"));

// 4. Parse Bencode
let bencode_node = bencode::parse_bytes(b"d7:service6:babbel6:status6:activee")?;
```

### Streaming I/O

```rust
use babbel::core::io::{Buffer, BufferSource, ISource, IDestination};
use babbel::xml;

// Stream from any ISource
let mut source = BufferSource::new(b"<config><timeout>30</timeout></config>");
let doc = xml::parse_source(&mut source)?;

// Stream directly to any IDestination
let mut dest = Buffer::new();
xml::stringify_to(&doc, &mut dest);
println!("Serialized XML: {}", dest.to_string());
```

### Cross-Format Conversions

```rust
use babbel::convert;

// JSON <-> YAML
let yaml_str = convert::json_to_yaml(r#"{"host": "localhost", "port": 8080}"#)?;
let json_str = convert::yaml_to_json(&yaml_str)?;

// JSON <-> XML
let xml_str = convert::json_to_xml(r#"{"message": "hello"}"#)?;

// CSV <-> JSON
let csv_data = "id,name\n1,Alice\n2,Bob\n";
let json_from_csv = convert::csv_to_json(csv_data)?;
let csv_roundtrip = convert::json_to_csv(&json_from_csv)?;

// INI <-> JSON
let ini_doc = "[server]\nhost = 127.0.0.1\nport = 8080\n";
let json_from_ini = convert::ini_to_json(ini_doc)?;

// JSON Lines <-> JSON
let jsonl = "{\"id\":1}\n{\"id\":2}\n";
let json_arr = convert::jsonlines_to_json(jsonl)?;

// Binary Bencode conversions
let bencode_bytes = convert::json_to_bencode(r#"{"id": 101}"#)?;
let yaml_from_bencode = convert::bencode_to_yaml(&bencode_bytes)?;
```

---

## Building and Testing

Babbel features a rigorous test suite of over **3,000 unit, integration, and doc tests** across all crates.

```bash
# Check all workspace crates
cargo check --workspace

# Run tests across all workspace crates
cargo test --workspace --jobs 2

# Run tests for a specific crate
cargo test -p babbel_xml --jobs 2
cargo test -p babbel_json --jobs 2
cargo test -p babbel_yaml --jobs 2
cargo test -p babbel_bencode --jobs 2
cargo test -p babbel_core --jobs 2
cargo test -p babbel --jobs 2
```

> [!TIP]
> On Windows, passing `--jobs 2` ensures smooth concurrent file handling across test artifacts.

---

## Support

If you find Babbel helpful and want to support ongoing development, performance optimizations, and format additions, you can buy me a coffee:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Donate-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/roberttizz1)

👉 **[https://buymeacoffee.com/roberttizz1](https://buymeacoffee.com/roberttizz1)**

---

## License

This project is licensed under the [MIT License](LICENSE).
