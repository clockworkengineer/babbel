# babbel

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)

The master facade crate for the **Babbel** multi-format serialization and document processing ecosystem. It provides unified ergonomics, re-exports all domain engines (JSON, TOML, YAML, XML, Bencode, CSV, TSV, INI, JSON Lines), provides dynamic format resolution via `FormatRegistry`, and powers open-ended, $O(N)$ cross-format conversion pipelines.

---

## Features

- **Unified Prelude**: Access all formats and text processing primitives with a single dependency.
- **Polyglot Parsing**: Parse JSON, TOML, YAML, XML, Bencode, CSV/TSV, and INI into idiomatic typed representations.
- **Universal Cross-Conversion**: Convert documents between 9 arbitrary formats without manual intermediate representations:
  - JSON $\leftrightarrow$ TOML $\leftrightarrow$ YAML $\leftrightarrow$ XML $\leftrightarrow$ Bencode
  - CSV / TSV $\leftrightarrow$ JSON $\leftrightarrow$ TOML $\leftrightarrow$ YAML
  - INI / .env $\leftrightarrow$ JSON $\leftrightarrow$ TOML $\leftrightarrow$ YAML
  - JSON Lines $\leftrightarrow$ JSON $\leftrightarrow$ TOML $\leftrightarrow$ CSV
- **Engine-Driven Pipelines (`convert_format`)**: Open-ended pipelines accepting any `FormatEngine` (OCP & DIP).
- **Dynamic Format Discovery**: Look up format engines dynamically by format ID, MIME type, or file extension via `FormatRegistry`.
- **Zero-Allocation Embedded Module (`babbel::embedded`)**:
  - Stack allocation: `StackBuffer<const N>`, `MemoryTracker`, `EmbeddedLimits`, `CompactError`.
  - Zero-allocation destinations: `SliceDestination` and `ArrayVecDestination`.
  - Zero-allocation streaming pull parsers: `JsonPullParser`, `TomlPullParser`, `XmlPullParser`, `CsvPullParser`, and `IniPullParser`.
- **First-Class Text Processing**:
  - RFC 4180 CSV & TSV parsing/emission with automatic delimiter sniffing and scalar type inference.
  - Section-based INI (`[section]`), Java `.properties`, and `.env` parsing.
  - Line-delimited JSON (`.jsonl`/`.ndjson`) streaming reader and writer.
  - Document frontmatter extraction (`split_frontmatter` for YAML `---` and TOML `+++`).
- **Extensible Streaming I/O**: Stream to and from files or memory buffers powered by [`babbel_core`](../babbel_core).
- **Fine-Grained Feature Flags**: Enable only the formats your application requires.

---

## Installation

Add `babbel` to your `Cargo.toml`:

```toml
[dependencies]
babbel = "0.1.2"
```

Or as a workspace path dependency:

```toml
[dependencies]
babbel = { path = "crates/babbel" }
```

### Feature Flags

By default, all formats and the conversion pipeline are enabled (`["std", "json", "toml", "yaml", "xml", "bencode", "convert"]`). To minimize binary size, enable only what you need:

| Feature | Description |
| :--- | :--- |
| `std` *(default)* | Enables standard library I/O and OS integration. |
| `alloc` | Enables heap allocation without full `std` (`no_std` environments). |
| `json` *(default)* | Re-exports `babbel_json` (`babbel::json`, `JsonEngine`, pull parser). |
| `toml` *(default)* | Re-exports `babbel_toml` (`babbel::toml`, `TomlEngine`, pull parser). |
| `yaml` *(default)* | Re-exports `babbel_yaml` (`babbel::yaml`, `YamlEngine`). |
| `xml` *(default)* | Re-exports `babbel_xml` (`babbel::xml`, `XmlEngine`, pull parser). |
| `bencode` *(default)* | Re-exports `babbel_bencode` (`babbel::bencode`, `BencodeEngine`). |
| `convert` *(default)* | Enables the `babbel::convert` cross-format conversion pipeline across all 9 formats. |

---

## Quickstart

### 1. Document Parsing & Navigation

```rust
use babbel::json;
use babbel::toml;
use babbel::yaml;
use babbel::xml;
use babbel::bencode;
use babbel::{parse_csv, parse_ini, split_frontmatter, CsvOptions, IniOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse JSON
    let json_doc = json::from_str(r#"{"service": "babbel", "port": 8080}"#)?;
    assert_eq!(json_doc.get("service").and_then(|n| n.as_str()), Some("babbel"));

    // 2. Parse TOML
    let toml_doc = toml::parse_string("service = \"babbel\"\nport = 8080\n")?;
    assert_eq!(toml_doc.get("service").and_then(|n| n.as_str()), Some("babbel"));

    // 3. Parse YAML
    let yaml_doc = yaml::parse_string("service: babbel\nport: 8080\n")?;
    assert_eq!(yaml_doc.get("service").and_then(|n| n.as_str()), Some("babbel"));

    // 4. Parse XML
    let xml_doc = xml::parse("<service port=\"8080\">babbel</service>")?;
    assert_eq!(xml_doc.get_root_element_name(), Some("service"));

    // 5. Parse Bencode
    let bencode_node = bencode::parse_bytes(b"d7:service6:babbel6:status6:activee")?;

    // 6. Parse CSV with type inference
    let csv_val = parse_csv("id,name\n1,Alice\n", &CsvOptions::default())?;
    assert_eq!(csv_val.as_array().unwrap().len(), 1);

    // 7. Parse INI with sections
    let ini_val = parse_ini("[app]\nname = Babbel\n", &IniOptions::default())?;
    assert!(ini_val.get("app").is_some());

    // 8. Split Markdown Frontmatter
    let doc = "---\ntitle: Guide\n---\n# Content";
    let parsed = split_frontmatter(doc);
    assert_eq!(parsed.content, "# Content");

    Ok(())
}
```

### 2. Universal Cross-Format Conversions

```rust
use babbel::convert::{convert_format, ConversionOptions};
use babbel_json::JsonEngine;
use babbel_toml::TomlEngine;
use babbel_yaml::YamlEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Engine Pipeline: JSON <-> TOML
    let toml_output = convert_format(
        r#"{"title": "Babbel", "version": 1}"#,
        &JsonEngine,
        &TomlEngine,
        &ConversionOptions::pretty(),
    )?;

    // TOML <-> YAML
    let yaml_output = babbel::convert::toml_to_yaml(&toml_output)?;

    // TOML <-> Bencode
    let bencode_bytes = babbel::convert::toml_to_bencode(&toml_output)?;

    // CSV <-> JSON
    let csv_data = "id,name\n1,Alice\n2,Bob\n";
    let json_arr = babbel::convert::csv_to_json(csv_data)?;
    let csv_back = babbel::convert::json_to_csv(&json_arr)?;

    Ok(())
}
```

### 3. Dynamic Format Registry

```rust
use babbel::{default_registry, convert::convert_format, convert::ConversionOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = default_registry();
    if let (Some(from), Some(to)) = (registry.get_by_id("json"), registry.get_by_id("toml")) {
        let toml_doc = convert_format(
            r#"{"status": "ok"}"#,
            from.as_ref(),
            to.as_ref(),
            &ConversionOptions::default(),
        )?;
        println!("{}", toml_doc);
    }
    Ok(())
}
```

### 4. Zero-Allocation Streaming Pull Parsers

```rust
use babbel::embedded::{JsonPullParser, JsonPullEvent, JsonScalar};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = JsonPullParser::new(r#"{"temp": 24.5, "sensor": "DHT22"}"#);
    while let Some(event) = parser.next_event()? {
        if let JsonPullEvent::Scalar(JsonScalar::Float(val)) = event {
            println!("Sensor value: {}", val);
        }
    }
    Ok(())
}
```

---

## Sub-Libraries

| Crate | Link | Description |
| :--- | :--- | :--- |
| `babbel_core` | [`crates/babbel_core`](../babbel_core) | Core I/O traits, `FormatEngine`, universal `Value` AST, RFC 4180 CSV/TSV, INI/.env, frontmatter, BOM detection, and embedded primitives. |
| `babbel_json` | [`crates/json`](../json) | RFC 6901 JSON Pointer, RFC 7396 Merge Patch, JSON Lines (`.jsonl`/`.ndjson`) streaming, and zero-allocation `JsonPullParser`. |
| `babbel_toml` | [`crates/toml`](../toml) | Strict TOML v1.1.0 parser/emitter with micro-allocations, typed RFC 3339 datetimes, and `TomlPullParser`. |
| `babbel_yaml` | [`crates/yaml`](../yaml) | YAML 1.2 specification compliance, anchors, aliases, and custom tags. |
| `babbel_xml` | [`crates/xml`](../xml) | XML DOM, C14N Canonical XML, DTD validation, XSD schema, XPath 1.0, and `XmlPullParser`. |
| `babbel_bencode` | [`crates/bencode`](../bencode) | Zero-copy borrowed parsing and streaming for BitTorrent Bencode. |

---

## Documentation

See the [Documentation Hub](../../docs/README.md) for full workspace guides:
- [Architecture Guide](../../docs/ARCHITECTURE.md)
- [SOLID Architecture Whitepaper](../../docs/SOLID_ARCHITECTURE_GUIDE.md)
- [Format Engine Plugin Guide](../../docs/FORMAT_ENGINE_PLUGIN_GUIDE.md)
- [Embedded Systems Guide](../../docs/EMBEDDED_GUIDE.md)
- [Conversion Matrix](../../docs/CONVERSION_MATRIX.md)
- [Memory & Benchmarks](../../docs/BENCHMARKS_AND_MEMORY.md)
- [Migration Guide & Changelog](../../docs/MIGRATION_AND_CHANGELOG.md)

---

## License

Licensed under the [MIT License](../../LICENSE).
