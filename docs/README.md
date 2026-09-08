# Babbel Documentation Hub

Welcome to the **Babbel** documentation hub. Babbel is a high-performance, polyglot serialization, parsing, and document manipulation ecosystem in Rust.

---

## 📚 Guides & Specifications

| Guide | Description |
| :--- | :--- |
| **[Architecture & Design Principles](ARCHITECTURE.md)** | Detailed 3-tier layering model, SOLID principles in Rust, and DRY consolidation. |
| **[Embedded Systems Guide](EMBEDDED_GUIDE.md)** | Guide to `no_std`, stack buffers (`StackBuffer`), zero-allocation destinations, and $O(1)$ RAM pull parsers. |
| **[Text Support Guide](TEXT_SUPPORT_GUIDE.md)** | RFC 4180 CSV/TSV, sectioned INI/.env, JSON Lines (`.jsonl`), frontmatter extraction, and line readers. |
| **[Conversion Matrix](CONVERSION_MATRIX.md)** | $O(N)$ universal cross-format conversion pipeline and `ConversionOptions` reference. |
| **[Memory & Benchmarks](BENCHMARKS_AND_MEMORY.md)** | Struct size bounds verification (`size_checks.rs`), zero-allocation formatters, and I/O optimizations. |
| **[W3C XML Conformance](XML_CONFORMANCE.md)** | Official W3C XML Conformance Test Suite (XML TS 20130923) results: 100.0% pass rate across all 12 sub-catalogs (1,834/1,834 tests, 0 skipped). |
| **[Security Policy](SECURITY.md)** | Threat model, Billion Laughs entity expansion mitigations, 64 MB DoS limits, and vulnerability reporting. |
| **[Development Guide](DEVELOPMENT_GUIDE.md)** | Contributor onboarding, toolchain requirements, debugging, release profiles, and profiling. |
| **[Contributing Guidelines](CONTRIBUTING.md)** | Engineering standards, code formatting, pull request process, and test guidelines. |
| **[Code of Conduct](CODE_OF_CONDUCT.md)** | Contributor Covenant v2.1 community standards. |

---

## 📦 Workspace Crates

Babbel is partitioned into six decoupled, focused crates:

| Crate | Package Docs | Path | Role |
| :--- | :--- | :--- | :--- |
| **`babbel`** | [README](../crates/babbel/README.md) | `crates/babbel` | Master facade crate with unified prelude, cross-format conversion pipeline, and ergonomics. |
| **`babbel_core`** | [README](../crates/babbel_core/README.md) | `crates/babbel_core` | Architectural kernel: streaming I/O traits, universal `Value` AST, CSV/TSV, INI, frontmatter, BOM detection. |
| **`babbel_json`** | [README](../crates/json/README.md) | `crates/json` | JSON DOM engine, RFC 6901 Pointer, RFC 7396 Merge Patch, JSON Lines, JSON5 comments. |
| **`babbel_yaml`** | [README](../crates/yaml/README.md) | `crates/yaml` | YAML 1.2 compliant DOM, anchors/aliases, custom tags, multiline block scalars, fluent builders. |
| **`babbel_bencode`** | [README](../crates/bencode/README.md) | `crates/bencode` | BitTorrent Bencode parser/serializer, borrowed zero-copy DOM, stack-based iterative parser. |
| **`babbel_xml`** | [README](../crates/xml/README.md) | `crates/xml` | W3C XML DOM, validating pull parser, C14N 1.0/1.1 canonicalization, DTD, XSD, XPath 1.0. |

---

## 🚀 Quick Reference by Task

### 1. Simple Parsing & Stringifying
```rust
use babbel::prelude::*;

// Standardized verbs across all formats:
let json_node = babbel_json::from_str("{\"status\": \"ok\"}")?;
let yaml_text = babbel_yaml::to_string(&yaml_node)?;
let bencode_bytes = babbel_bencode::to_vec(&bencode_node)?;
```

### 2. Universal Cross-Format Conversion
```rust
use babbel::convert::{convert_text_with_options, ConversionOptions};
use babbel_json::codec::JsonCodec;
use babbel_yaml::codec::YamlCodec;

let options = ConversionOptions::default().with_pretty(true);
let yaml_out = convert_text_with_options(
    "{\"name\": \"babbel\", \"version\": 1}",
    &JsonCodec::new(),
    &YamlCodec::new(),
    &options,
)?;
```

### 3. Embedded & Low-Memory Parsing
```rust
use babbel_core::embedded::JsonPullParser;
use babbel_core::io::SliceSource;

let mut source = SliceSource::new(r#"{"temp": 24.5, "sensor": "DHT22"}"#);
let mut parser = JsonPullParser::new(&mut source);
while let Some(event) = parser.next_event()? {
    // Process stream with O(1) stack memory and 0 dynamic allocations
}
```
