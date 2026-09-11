# Babbel

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-3500%2B%20passing-brightgreen.svg)]()
[![Architecture: SOLID](https://img.shields.io/badge/architecture-SOLID%20%26%20DRY-purple.svg)](docs/SOLID_ARCHITECTURE_GUIDE.md)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Donate-FFDD00?logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/roberttizz1)

A high-performance, polyglot serialization, parsing, and document manipulation workspace in Rust. Babbel brings together **JSON**, **YAML**, **Bencode**, **XML**, **TOML**, **CSV / TSV**, **INI / Properties**, and **JSON Lines** under a unified, modular architecture adhering strictly to **DRY** (Don't Repeat Yourself) and **SOLID** engineering principles.

📖 **Documentation & Guides**:
- [Documentation Hub](docs/README.md) — Central directory of all specifications, guides, and tutorials
- [SOLID Architecture Whitepaper](docs/SOLID_ARCHITECTURE_GUIDE.md) — Comprehensive guide to the 6-phase SOLID implementation
- [Format Engine Plugin Guide](docs/FORMAT_ENGINE_PLUGIN_GUIDE.md) — Tutorial on implementing and registering custom format engines
- [Architecture Guide](docs/ARCHITECTURE.md) — 3-tier layering model, memory bounds, and design principles
- [Conversion Matrix](docs/CONVERSION_MATRIX.md) — Universal $O(N)$ cross-format conversion reference & options
- [Embedded Systems Guide](docs/EMBEDDED_GUIDE.md) — `no_std`, stack buffers (`StackBuffer`), and zero-allocation pull parsers
- [Memory & Benchmarks](docs/BENCHMARKS_AND_MEMORY.md) — Struct size bounds verification (`size_checks.rs`)
- [YAML 1.2 Conformance](docs/YAML_CONFORMANCE.md) — Official YAML test suite results (1,085+ tests passing)
- [Bencode Spec & Conformance](docs/BENCODE_SPEC_AND_CONFORMANCE.md) — BitTorrent BEP 0003 specification & test breakdown
- [TOML Conformance](docs/TOML_CONFORMANCE.md) — Official skystrife/toml-test suite results (100.0% pass rate)
- [JSON Conformance](docs/JSON_CONFORMANCE.md) — Official nst/JSONTestSuite results (100.0% pass rate)
- [W3C XML Conformance](docs/XML_CONFORMANCE.md) — W3C XML Conformance Suite results (100.0% across 1,834 tests)
- [Text Support Guide](docs/TEXT_SUPPORT_GUIDE.md) — RFC 4180 CSV/TSV, INI/.env, JSON Lines, and frontmatter
- [Migration Guide & Changelog](docs/MIGRATION_AND_CHANGELOG.md) — Release notes and backward compatibility guarantees
- [Security Policy](docs/SECURITY.md) — Threat model, Billion Laughs mitigations, and 64 MB DoS limits
- [Development Guide](docs/DEVELOPMENT_GUIDE.md) — Contributor onboarding, toolchain, testing, and release builds
- [Contributing Guidelines](docs/CONTRIBUTING.md) — Engineering standards and code conventions

---

## Workspace Architecture

Babbel is structured as an interconnected multi-crate workspace:

| Crate | Package Docs | Directory | Description |
| :--- | :--- | :--- | :--- |
| **`babbel`** | [README](crates/babbel/README.md) | [`crates/babbel`](crates/babbel) | Master facade crate providing high-level ergonomics, prelude, dynamic `FormatRegistry`, and open-ended conversion pipelines across 9 formats. |
| **`babbel_core`** | [README](crates/babbel_core/README.md) | [`crates/babbel_core`](crates/babbel_core) | Core architectural kernel containing streaming traits (`ISource`, `IDestination`, `ILineReader`), `FormatEngine` interface, universal `Value` AST, RFC 4180 CSV/TSV, sectioned INI/.env, frontmatter processing, Unicode BOM detection, and embedded primitives. |
| **`babbel_json`** | [README](crates/json/README.md) | [`crates/json`](crates/json) | Full-featured JSON DOM engine, `JsonEngine`, RFC 6901 JSON Pointer, RFC 7396 JSON Merge Patch, JSON Lines streaming, and zero-allocation `JsonPullParser`. |
| **`babbel_yaml`** | [README](crates/yaml/README.md) | [`crates/yaml`](crates/yaml) | YAML 1.2 parser and emitter, `YamlEngine`, anchors, aliases, custom tags, multiline block scalars, and SRP-decomposed modules. |
| **`babbel_bencode`** | [README](crates/bencode/README.md) | [`crates/bencode`](crates/bencode) | High-speed, binary-safe BitTorrent Bencode parser and serializer, `BencodeEngine`, supporting zero-copy borrowed slices and iterative streaming. |
| **`babbel_xml`** | [README](crates/xml/README.md) | [`crates/xml`](crates/xml) | Robust XML DOM parser, `XmlEngine`, W3C Canonical XML (C14N 1.0/1.1), DTD validation, XSD schema validator, zero-allocation `XmlPullParser`, and XPath 1.0 query engine. |
| **`babbel_toml`** | [README](crates/toml/README.md) | [`crates/toml`](crates/toml) | Fast, modular, pure-Rust TOML v1.1.0 parser, `TomlEngine`, serializer, streaming pull parser, and compacted DOM with zero external parser dependencies. |

---

## Key Design Principles

### 1. Unified Streaming I/O (`ISource` & `IDestination`)
All format crates across Babbel share the same streaming abstractions from [`babbel_core::io`](crates/babbel_core/src/io):
- **Streaming Input (`ISource`)**: Every parser ingests data via `&mut dyn ISource`.
- **Streaming Output (`IDestination`)**: Every emitter and serializer writes sequentially to `&mut dyn IDestination`.
- **Interface Segregation (ISP)**: Minimal sub-traits allow clients to bind only to the capabilities they require:
  - [`ILineReader`](crates/babbel_core/src/io/traits.rs): Line-by-line reading across mixed `\r\n`, `\n`, `\r` endings without full buffering.
  - [`ICharStream`](crates/babbel_core/src/io/traits.rs): Minimal pull-based character stream (`current()`, `next()`, `more()`).
  - [`IByteStream`](crates/babbel_core/src/io/traits.rs): Raw byte reading for binary protocols (Bencode).
  - [`IByteWriter`](crates/babbel_core/src/io/traits.rs): Binary byte writing for output destinations.
  - [`IRewindable`](crates/babbel_core/src/io/traits.rs): Reset streams to initial state.
  - [`IPositionAware`](crates/babbel_core/src/io/traits.rs): Absolute byte offset tracking.
  - [`ILocationAware`](crates/babbel_core/src/io/traits.rs): 1-based line and column metric tracking.
  - [`ITailInspectable`](crates/babbel_core/src/io/traits.rs): Inspect last written byte in-memory without OS file seek syscalls.
  - [`IClearable`](crates/babbel_core/src/io/traits.rs): Buffer/destination truncation.
  - [`IFlushable`](crates/babbel_core/src/io/traits.rs): Storage buffer synchronization.
  - [`IIndentationAware`](crates/babbel_core/src/io/traits.rs): Indentation calculation for whitespace-sensitive grammars.

### 2. Open-Closed & Dependency Inversion (OCP & DIP)
- **Extensible `FormatEngine`**: New serialization formats can be integrated simply by implementing [`FormatEngine`](crates/babbel_core/src/engine.rs) (or `FormatParser` + `FormatEmitter`).
- **Dynamic `FormatRegistry`**: Register and discover engines at runtime by format ID, MIME type, or file extension.
- **Universal Data Pipeline**: Rather than $O(N^2)$ hand-rolled cross-serializers, conversions flow through the universal [`Value`](crates/babbel_core/src/model.rs) AST:
  ```rust
  let output = babbel::convert::convert_format(input_str, &from_engine, &to_engine, &options)?;
  let bytes  = babbel::convert::convert_format_bytes(input_bytes, &from_engine, &to_engine, &options)?;
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
babbel = { path = "crates/babbel", features = ["json", "yaml", "xml", "bencode", "toml", "convert"] }
```

### Parsing & Document Access

```rust
use babbel::json;
use babbel::toml;
use babbel::yaml;
use babbel::xml;
use babbel::bencode;

// 1. Parse JSON
let json_node = json::from_str(r#"{"service": "babbel", "status": "active"}"#)?;
assert_eq!(json_node.get("status").and_then(|n| n.as_str()), Some("active"));

// 2. Parse TOML
let toml_node = toml::parse_string("service = \"babbel\"\nport = 8080\n")?;
assert_eq!(toml_node.get("service").and_then(|n| n.as_str()), Some("babbel"));

// 3. Parse YAML
let yaml_node = yaml::parse_string("service: babbel\nstatus: active\n")?;
assert_eq!(yaml_node.get("service").and_then(|n| n.as_str()), Some("babbel"));

// 4. Parse XML
let xml_doc = xml::parse("<service name=\"babbel\"><status>active</status></service>")?;
assert_eq!(xml_doc.get_root_element_name(), Some("service"));

// 5. Parse Bencode
let bencode_node = bencode::parse_bytes(b"d7:service6:babbel6:status6:activee")?;
```

### Universal Cross-Format Conversions

```rust
use babbel::convert::{convert_format, ConversionOptions};
use babbel_json::JsonEngine;
use babbel_toml::TomlEngine;
use babbel_yaml::YamlEngine;

// High-level engine pipeline: JSON <-> TOML
let toml_str = convert_format(
    r#"{"host": "localhost", "port": 8080}"#,
    &JsonEngine,
    &TomlEngine,
    &ConversionOptions::pretty(),
)?;

// Convenience functions:
let yaml_str = babbel::convert::toml_to_yaml(&toml_str)?;
let json_str = babbel::convert::yaml_to_json(&yaml_str)?;
let bencode_bytes = babbel::convert::toml_to_bencode(&toml_str)?;
```

### Dynamic Format Resolution via `FormatRegistry`

```rust
use babbel::{default_registry, convert::convert_format, convert::ConversionOptions};

let registry = default_registry();
if let (Some(from), Some(to)) = (registry.get_by_id("json"), registry.get_by_id("xml")) {
    let xml_output = convert_format(
        r#"{"status": "ok"}"#,
        from.as_ref(),
        to.as_ref(),
        &ConversionOptions::default(),
    )?;
}
```

### Embedded Zero-Allocation Streaming

```rust
use babbel::embedded::{JsonPullParser, JsonPullEvent, JsonScalar};

let mut parser = JsonPullParser::new(r#"{"temp": 24.5, "sensor": "DHT22"}"#);
while let Some(event) = parser.next_event()? {
    if let JsonPullEvent::Scalar(JsonScalar::Float(val)) = event {
        println!("Temperature: {}", val);
    }
}
```

---

## Building and Testing

Babbel features a rigorous test suite of over **3,500 unit, integration, and doc tests** across all crates.

```bash
# Check all workspace crates
cargo check --workspace

# Run tests across all workspace crates
cargo test --workspace --jobs 2

# Verify struct size bounds
cargo test -p babbel --test size_checks
```

### Official Specification Conformance Suites

- **Official YAML 1.2 Test Suite**: **1,085+ tests passed** via `cargo test -p babbel_yaml`.
- **Official W3C XML Conformance Test Suite (XML TS 20130923)**: **1,834 / 1,834 tests passed (100.0%)** across all 12 sub-catalogs (0 failed, 0 skipped, 0 panics) via `cargo test -p babbel_xml --test w3c_conformance`.
- **Official JSONTestSuite (RFC 8259 Conformance)**: **340 / 340 tests passed (100.0%)** across all categories (95/95 `y_` accepted, 188/188 `n_` rejected, 35/35 `i_` safe, 22/22 transform, 0 panics) via `cargo test -p babbel_json --test nst_conformance`.
- **Official TOML skystrife/toml-test**: **148 / 148 tests passed (100.0%)** via `cargo test -p babbel_toml --test toml_test_suite`.
- **BitTorrent Bencode BEP 0003**: Full conformance with zero-copy and recursive limits verification.

---

## Support

If you find Babbel helpful and want to support ongoing development, performance optimizations, and format additions, you can buy me a coffee:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Donate-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/roberttizz1)

👉 **[https://buymeacoffee.com/roberttizz1](https://buymeacoffee.com/roberttizz1)**

---

## License

This project is licensed under the [MIT License](LICENSE).
