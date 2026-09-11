# Babbel Migration Guide & Changelog

This document details the architectural enhancements, release highlights, backward compatibility guarantees, and migration pathways introduced in the **Babbel SOLID & Extensibility Release (v0.1.2)**.

---

## 1. Release Highlights

The v0.1.2 release represents a comprehensive architectural modernization across all seven workspace crates (`babbel`, `babbel_core`, `babbel_json`, `babbel_yaml`, `babbel_xml`, `babbel_bencode`, and `babbel_toml`), delivering strict adherence to **SOLID** design principles without sacrificing high throughput or breaking backwards compatibility.

### Key Additions
1. **Universal $O(N)$ Conversion Matrix**:
   - Universal conversion pipeline connecting all **9 supported formats**: JSON, YAML, XML, BitTorrent Bencode, TOML v1.1.0, CSV, TSV, sectioned INI/.env, and JSON Lines.
   - Point-to-point convenience wrappers for newly added pairs, including TOML $\leftrightarrow$ Bencode, XML $\leftrightarrow$ TOML, YAML $\leftrightarrow$ TOML, and JSON Lines $\leftrightarrow$ TOML.
2. **Standardized `FormatEngine` Plugin System**:
   - Standardized format engine facade ([`FormatEngine`](ARCHITECTURE.md#formatengine-trait-definition)) providing unified parsing, serialization, MIME detection, and file extension queries.
   - Dynamic format discovery via [`FormatRegistry`](FORMAT_ENGINE_PLUGIN_GUIDE.md#the-formatregistry) and static lookup functions (`find_engine_by_mime`, `find_engine_by_extension`).
3. **Embedded & Zero-Allocation Primitives**:
   - Unified embedded namespace (`babbel::embedded`) exporting `StackBuffer`, `SliceDestination`, `ArrayVecDestination`, `MemoryTracker`, `EmbeddedLimits`, and `CompactError` (8 bytes).
   - Zero-allocation streaming event pull parsers (`JsonPullParser`, `TomlPullParser`, `XmlPullParser`, `CsvPullParser`, and `IniPullParser`) enabling AST-free parsing on microcontrollers.
4. **Compact Memory Footprint & Verified Bounds**:
   - Compaction of core AST nodes verified by automated tests in `tests/size_checks.rs`:
     - `babbel_core::Value`: **32 bytes** (exactly half a 64-byte cache line)
     - `babbel_yaml::Node`: **$\le 40$ bytes**
     - `babbel_xml::NodeKind`: **$\le 48$ bytes**, `NodeData`: **$\le 88$ bytes**
     - `babbel_json::Node`: **$\le 56$ bytes**
     - `babbel_bencode::Node`: **$\le 56$ bytes**
     - `babbel_toml::Node`: **$\le 48$ bytes**
5. **100% Specification Conformance**:
   - **3,500+ automated tests** passing across the workspace with 0 failures and 0 panics.
   - **1,085+ tests** in the YAML test suite.
   - **1,834 tests (100.0%)** in the official W3C XML Conformance Test Suite (XML TS 20130923).
   - **340 tests (100.0%)** in the official nst/JSONTestSuite (RFC 8259).
   - **148 tests (100.0%)** in the official skystrife/toml-test suite.
   - Full BitTorrent Bencode BEP 0003 specification compliance.

---

## 2. Backward Compatibility Matrix

All existing code written against Babbel v0.1.0/v0.1.1 will continue to compile and function without modification.

| Component | Status | Compatibility Notes |
| :--- | :--- | :--- |
| `babbel::json`, `yaml`, `xml`, `bencode`, `toml` | **Stable** | Preferred canonical namespaces for format engines. |
| `babbel::json_lib`, `yaml_lib`, `xml_lib`, `bencode_lib`, `toml_lib` | **Deprecated** | Legacy crate aliases maintained for backward compatibility. Emits compiler deprecation warnings directing users to canonical modules. |
| `babbel::convert::convert_text`, `convert_bytes` | **Stable** | Maintained with full backward compatibility. Supports any types implementing `FormatParser` and `FormatEmitter`. |
| `babbel::convert::convert_format`, `convert_format_bytes` | **New** | Recommended engine-driven conversion pipelines accepting `FormatEngine` references or trait objects. |
| `babbel_core::io::*` | **Enhanced** | All existing sources (`BufferSource`, `FileSource`, `SliceSource`) and destinations (`Buffer`, `FileDestination`) implement new segregated ISP traits seamlessly. |
| `babbel_core::model::Value` | **Compacted** | Public enum variants unchanged; memory layout compacted from 48 bytes to 32 bytes via boxed complex payload representations. |

---

## 3. Performance & Memory Comparison

| Metric / Struct | Pre-SOLID Baseline | v0.1.2 (Current) | Improvement |
| :--- | :---: | :---: | :--- |
| **`babbel_core::Value`** | 48 bytes | **32 bytes** | **-33.3%** (Fits two values per 64-byte cache line) |
| **`babbel_xml::NodeKind`** | 72 bytes | **$\le 48$ bytes** | **-33.3%** (Boxed ElementData and processing instructions) |
| **`babbel_xml::NodeData`** | 112 bytes | **$\le 88$ bytes** | **-21.4%** (Compacted attribute storage) |
| **`babbel_yaml::Node`** | 56 bytes | **$\le 40$ bytes** | **-28.6%** (Inlined scalar storage & alias tagging) |
| **`babbel_json::Node`** | 64 bytes | **$\le 56$ bytes** | **-12.5%** (Small string optimization & discriminant union) |
| **`babbel_bencode::Node`** | 64 bytes | **$\le 56$ bytes** | **-12.5%** (Zero-copy byte slices) |
| **`babbel_toml::Node`** | 64 bytes | **$\le 48$ bytes** | **-25.0%** (Boxed table arrays & compact discriminants) |
| **File Tail Inspection (`last()`)** | OS File Seek | **0 Syscalls** | Fast in-memory byte cache eliminates Windows lock contention |
| **Numeric Serialization** | Heap `String` | **0 Allocations** | Direct stack formatting via `itoa` and `dtoa` |

---

## 4. Migration Walkthroughs

### 4.1 Migrating Legacy Module Aliases
Update any references to deprecated `*_lib` re-exports to the canonical modules:

```rust
// Before (v0.1.0):
use babbel::json_lib;
use babbel::yaml_lib;

let node = json_lib::from_str(data)?;

// After (v0.1.2 recommended):
use babbel::json;
use babbel::yaml;

let node = json::from_str(data)?;
```

### 4.2 Adopting `FormatEngine` and `convert_format`
While legacy `convert_text` remains fully supported, `convert_format` offers cleaner syntax and unified option propagation:

```rust
use babbel::convert::{convert_format, ConversionOptions};
use babbel_json::JsonEngine;
use babbel_toml::TomlEngine;

// Modern engine-driven conversion:
let toml_output = convert_format(
    r#"{"server": {"port": 8080}}"#,
    &JsonEngine,
    &TomlEngine,
    &ConversionOptions::pretty(),
)?;
```

### 4.3 Using Dynamic Format Discovery via `FormatRegistry`
Applications processing dynamic user inputs or MIME payloads can replace hand-rolled `match` statements with `FormatRegistry`:

```rust
use babbel::default_registry;
use babbel::convert::{convert_format, ConversionOptions};

let registry = default_registry();

// Resolve engines by format ID ("json", "yaml", "xml", "toml", "bencode")
if let (Some(from), Some(to)) = (registry.get_by_id("json"), registry.get_by_id("toml")) {
    let result = convert_format(
        r#"{"app": "gateway"}"#,
        from.as_ref(),
        to.as_ref(),
        &ConversionOptions::default(),
    )?;
}
```

### 4.4 Migrating Embedded Pipelines to Streaming Pull Parsers
To parse documents on microcontrollers without heap allocations, migrate from DOM-based parsing to `babbel::embedded` pull parsers:

```rust
// Before (builds full AST on heap):
let json_node = babbel_json::from_str(payload)?;

// After (zero-allocation streaming pull parser):
use babbel::embedded::{JsonPullParser, JsonPullEvent, JsonScalar};

let mut parser = JsonPullParser::new(payload);
while let Some(event) = parser.next_event()? {
    if let JsonPullEvent::Scalar(JsonScalar::String(val)) = event {
        // Stream process strings directly from source slice
    }
}
```

---

## 5. Detailed Changelog

### Added
- **`babbel_core`**:
  - `FormatEngine` trait unifying format ID, MIME types, file extensions, parsing, and serialization.
  - `FormatRegistry` for dynamic format discovery and lookup.
  - Static lookup helpers: `find_engine`, `find_engine_by_mime`, `find_engine_by_extension`.
  - Segregated streaming traits: `ILineReader`, `ICharStream`, `IByteStream`, `IByteWriter`, `IRewindable`, `IPositionAware`, `ILocationAware`, `ITailInspectable`, `IClearable`, `IFlushable`, `IIndentationAware`.
  - Embedded primitives in `babbel_core::embedded`: `StackBuffer`, `SliceDestination`, `ArrayVecDestination`, `MemoryTracker`, `EmbeddedLimits`, `CompactError`.
  - Visitor abstraction: `ValueVisitor` and `NodeVisitor` with default no-op methods.
- **`babbel`**:
  - Universal pipelines: `convert_format`, `convert_format_bytes`, and `convert_format_bytes_to_str`.
  - Convenience functions: `toml_to_bencode`, `bencode_to_toml`, `toml_to_xml`, `xml_to_toml`, `yaml_to_toml`, `toml_to_yaml`, `toml_to_json`, `json_to_toml`, `toml_to_csv`, `csv_to_toml`, `toml_to_ini`, `ini_to_toml`, `toml_to_jsonlines`, `jsonlines_to_toml`.
  - `babbel::default_registry()` pre-populated with all enabled built-in format engines.
  - Unified `babbel::embedded` module re-exporting all embedded utilities and streaming pull parsers.
  - Canonical format aliases: `babbel::json`, `babbel::yaml`, `babbel::xml`, `babbel::bencode`, `babbel::toml`.
- **`babbel_json`**:
  - `JsonEngine` implementing `FormatEngine`.
  - Zero-allocation `JsonPullParser`.
- **`babbel_yaml`**:
  - `YamlEngine` implementing `FormatEngine`.
  - Deconstruction of monolithic `node.rs` into SRP submodules: `access.rs`, `search.rs`, `convert.rs`, `scalar.rs`.
- **`babbel_xml`**:
  - `XmlEngine` implementing `FormatEngine`.
  - Zero-allocation `XmlPullParser`.
  - Element and node memory compaction.
- **`babbel_bencode`**:
  - `BencodeEngine` implementing `FormatEngine`.
  - Zero-copy `BorrowedNode<'a>`.
- **`babbel_toml`**:
  - `TomlEngine` implementing `FormatEngine`.
  - Zero-allocation `TomlPullParser`.
  - Node memory compaction ($\le 48$ bytes).
- **Documentation**:
  - `docs/YAML_CONFORMANCE.md` (1,085+ YAML test cases).
  - `docs/BENCODE_SPEC_AND_CONFORMANCE.md` (BEP 0003 specification & test suite).
  - `docs/FORMAT_ENGINE_PLUGIN_GUIDE.md` (extensibility and custom format tutorial).
  - `docs/SOLID_ARCHITECTURE_GUIDE.md` (whitepaper on 6-phase SOLID implementation).
  - `docs/MIGRATION_AND_CHANGELOG.md` (release notes & migration guide).

### Changed
- Refactored `babbel::convert` to depend on `FormatEngine` and `Value` rather than concrete format crates (DIP).
- Compacted `babbel_core::Value` discriminant and payload layout to 32 bytes.
- Compacted `babbel_xml::NodeKind` and `NodeData` via boxed element payloads.
- Replaced file seek system calls in `FileDestination` with in-memory tail tracking.

### Deprecated
- Deprecated legacy aliases `babbel::json_lib`, `babbel::yaml_lib`, `babbel::xml_lib`, `babbel::bencode_lib`, `babbel::toml_lib` in favor of canonical module names.
