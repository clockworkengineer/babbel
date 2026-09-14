# Babbel Documentation Hub

Welcome to the **Babbel** documentation hub. Babbel is a unified, high-performance polyglot serialization, document processing, and cross-format conversion ecosystem in Rust.

---

## 📚 Categorized Documentation Directory

The documentation suite is organized into four structured categories:

### 1. Core Architectural Guides
Architectural blueprints, SOLID engineering realizations, and extensibility manuals.

| Document | Description |
| :--- | :--- |
| **[Architecture & Design Principles](ARCHITECTURE.md)** | Core 3-tier layering model, SOLID principles in Rust, DRY consolidation, and memory bounds. |
| **[SOLID Architecture Whitepaper](SOLID_ARCHITECTURE_GUIDE.md)** | Comprehensive whitepaper detailing the 6-phase refactoring (SRP, OCP, LSP, ISP, and DIP). |
| **[Format Engine Plugin Guide](FORMAT_ENGINE_PLUGIN_GUIDE.md)** | Step-by-step tutorial on implementing custom format engines and registering them in `FormatRegistry`. |

### 2. Specification & Conformance
Official test suites, standards verification, and formal specification conformance reports.

| Document | Standard / Test Suite | Verified Results |
| :--- | :--- | :--- |
| **[JSON Conformance](JSON_CONFORMANCE.md)** | RFC 8259 / nst/JSONTestSuite | **100.0% pass rate** across all 340 test cases (0 failures, 0 panics). |
| **[YAML 1.2 Conformance](YAML_CONFORMANCE.md)** | YAML 1.2 Core Spec / Test Suite | **1,085+ test cases** passing across all 10 feature categories. |
| **[W3C XML Conformance](XML_CONFORMANCE.md)** | W3C XML TS 20130923 | **100.0% pass rate** across all 12 sub-catalogs (1,834/1,834 tests). |
| **[TOML Conformance](TOML_CONFORMANCE.md)** | TOML v1.1.0 / skystrife toml-test | **100.0% pass rate** across all 148 test cases (0 failures, 0 panics). |
| **[Bencode Spec & Conformance](BENCODE_SPEC_AND_CONFORMANCE.md)** | BitTorrent BEP 0003 | Full specification compliance, zero-copy borrowed slices, and stack parsing. |

### 3. Specialized Ecosystem Guides
Targeted guides for performance engineering, embedded targets, and text formats.

| Document | Description |
| :--- | :--- |
| **[Cross-Format Conversion Matrix](CONVERSION_MATRIX.md)** | Universal $O(N)$ conversion matrix between all 9 formats, engine pipelines, and dynamic registry lookups. |
| **[Embedded Systems & Low-Memory Guide](EMBEDDED_GUIDE.md)** | Guide to `no_std`, stack buffers (`StackBuffer`), zero-allocation destinations, and $O(1)$ RAM pull parsers. |
| **[Memory & Performance Architecture](BENCHMARKS_AND_MEMORY.md)** | Struct size bounds verification (`size_checks.rs`), zero-allocation formatting, and Windows lock elimination. |
| **[Text Support Guide](TEXT_SUPPORT_GUIDE.md)** | RFC 4180 CSV/TSV, sectioned INI/.env, JSON Lines (`.jsonl`), frontmatter extraction, and line readers. |

### 4. Governance, Releases & Development
Contribution workflows, release history, and security policies.

| Document | Description |
| :--- | :--- |
| **[Migration Guide & Changelog](MIGRATION_AND_CHANGELOG.md)** | Release notes, breaking change analysis, backward compatibility guarantees, and migration paths. |
| **[Security Policy](SECURITY.md)** | Threat model, Billion Laughs entity expansion mitigations, 64 MB DoS limits, and vulnerability reporting. |
| **[Development Guide](DEVELOPMENT_GUIDE.md)** | Contributor onboarding, toolchain requirements, debugging, release profiles, and profiling. |
| **[Contributing Guidelines](CONTRIBUTING.md)** | Engineering standards, code conventions, and pull request verification workflows. |
| **[Code of Conduct](CODE_OF_CONDUCT.md)** | Contributor Covenant v2.1 community standards. |

---

## 📦 Workspace Crates

Babbel is partitioned into seven decoupled, focused crates:

| Crate | Package Docs | Path | Role |
| :--- | :--- | :--- | :--- |
| **`babbel`** | [README](../crates/babbel/README.md) | `crates/babbel` | Master facade crate with unified prelude, cross-format conversion pipeline, and ergonomics. |
| **`babbel_core`** | [README](../crates/babbel_core/README.md) | `crates/babbel_core` | Architectural kernel: streaming I/O traits, universal `Value` AST, `FormatEngine` interface, CSV/TSV, INI, frontmatter. |
| **`babbel_json`** | [README](../crates/json/README.md) | `crates/json` | JSON DOM engine, `JsonEngine`, RFC 6901 Pointer, RFC 7396 Merge Patch, JSON Lines, pull parser. |
| **`babbel_yaml`** | [README](../crates/yaml/README.md) | `crates/yaml` | YAML 1.2 compliant DOM, `YamlEngine`, anchors/aliases, custom tags, multiline block scalars, SRP submodules. |
| **`babbel_bencode`** | [README](../crates/bencode/README.md) | `crates/bencode` | BitTorrent Bencode parser/serializer, `BencodeEngine`, borrowed zero-copy DOM, stack-based iterative parser. |
| **`babbel_xml`** | [README](../crates/xml/README.md) | `crates/xml` | W3C XML DOM, `XmlEngine`, validating pull parser, C14N 1.0/1.1 canonicalization, DTD, XSD, XPath 1.0. |
| **`babbel_toml`** | [README](../crates/toml/README.md) | `crates/toml` | TOML v1.1.0 DOM engine, `TomlEngine`, streaming pull parser, AST serializer, strict validation. |

---

## 🚀 Quick Reference by Task

### 1. Document Parsing & Serialization
```rust
use babbel::prelude::*;

// Standardized verbs across all formats:
let json_node = babbel::json::from_str("{\"status\": \"ok\"}")?;
let yaml_text = babbel::yaml::to_string(&yaml_node)?;
let bencode_bytes = babbel::bencode::to_vec(&bencode_node)?;
let toml_text = babbel::toml::to_string(&toml_node)?;
```

### 2. Universal Cross-Format Conversion
```rust
use babbel::convert::{convert_format, ConversionOptions};
use babbel_json::JsonEngine;
use babbel_toml::TomlEngine;

// Convert directly between any formats using their engines:
let toml_str = convert_format(
    r#"{"service": "babbel", "port": 8080}"#,
    &JsonEngine,
    &TomlEngine,
    &ConversionOptions::pretty(),
)?;
```

### 3. Dynamic Format Resolution
```rust
use babbel::{default_registry, convert::convert_format, convert::ConversionOptions};

let registry = default_registry();
if let (Some(from), Some(to)) = (registry.get_by_id("json"), registry.get_by_id("yaml")) {
    let yaml_out = convert_format(
        r#"{"item": "val"}"#,
        from.as_ref(),
        to.as_ref(),
        &ConversionOptions::default(),
    )?;
}
```

### 4. Zero-Allocation Streaming Pull Parsing
```rust
use babbel::embedded::{JsonPullParser, JsonPullEvent};

let json = r#"{"temp": 24.5, "sensor": "DHT22"}"#;
let mut parser = JsonPullParser::new(json);

while let Some(event) = parser.next_event()? {
    // Process events with 0 heap allocations
}
```
