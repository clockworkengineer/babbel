# babbel

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)

The master facade crate for the **Babbel** multi-format serialization and document processing ecosystem. It provides unified ergonomics, re-exports all domain engines (JSON, TOML, YAML, XML, Bencode, CSV, TSV, INI, JSON Lines), and powers open-ended, $O(N)$ cross-format conversion pipelines.

---

## Features

- **Unified Prelude**: Access all formats and text processing primitives with a single dependency.
- **Polyglot Parsing**: Parse JSON, TOML, YAML, XML, Bencode, CSV/TSV, and INI into idiomatic typed representations.
- **Universal Cross-Conversion**: Convert documents between 9 arbitrary formats without manual intermediate representations:
  - JSON $\leftrightarrow$ TOML $\leftrightarrow$ YAML $\leftrightarrow$ XML $\leftrightarrow$ Bencode
  - CSV / TSV $\leftrightarrow$ JSON $\leftrightarrow$ TOML $\leftrightarrow$ YAML
  - INI / .env $\leftrightarrow$ JSON $\leftrightarrow$ TOML $\leftrightarrow$ YAML
  - JSON Lines $\leftrightarrow$ JSON $\leftrightarrow$ TOML $\leftrightarrow$ CSV
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
| `json` *(default)* | Re-exports `babbel_json` for JSON parsing, pointers, patch, and JSON Lines streaming. |
| `toml` *(default)* | Re-exports `babbel_toml` for TOML v1.1.0 parsing, typed datetimes, and emission. |
| `yaml` *(default)* | Re-exports `babbel_yaml` for full YAML 1.2 parsing and emission. |
| `xml` *(default)* | Re-exports `babbel_xml` for validating XML DOM, C14N, and XPath 1.0. |
| `bencode` *(default)* | Re-exports `babbel_bencode` for BitTorrent Bencode processing. |
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

    // 5. Parse CSV with type inference
    let csv_val = parse_csv("id,name\n1,Alice\n", &CsvOptions::default())?;
    assert_eq!(csv_val.as_array().unwrap().len(), 1);

    // 6. Parse INI with sections
    let ini_val = parse_ini("[app]\nname = Babbel\n", &IniOptions::default())?;
    assert!(ini_val.get("app").is_some());

    // 7. Split Markdown Frontmatter
    let doc = "---\ntitle: Guide\n---\n# Content";
    let parsed = split_frontmatter(doc);
    assert_eq!(parsed.content, "# Content");

    Ok(())
}
```

### 2. Universal Cross-Format Conversions

```rust
use babbel::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // JSON <-> TOML
    let toml_output = convert::json_to_toml(r#"{"title": "Babbel", "version": 1}"#)?;
    let json_output = convert::toml_to_json(&toml_output)?;

    // JSON <-> YAML
    let yaml_output = convert::json_to_yaml(r#"{"title": "Babbel", "version": 1}"#)?;
    let json_output = convert::yaml_to_json(&yaml_output)?;

    // CSV <-> JSON
    let csv_data = "id,name\n1,Alice\n2,Bob\n";
    let json_arr = convert::csv_to_json(csv_data)?;
    let csv_back = convert::json_to_csv(&json_arr)?;

    // INI <-> JSON
    let ini_doc  = "[server]\nhost = 127.0.0.1\nport = 8080\n";
    let json_ini = convert::ini_to_json(ini_doc)?;
    let ini_back = convert::json_to_ini(&json_ini)?;

    // JSON Lines <-> JSON
    let jsonl    = "{\"id\":1}\n{\"id\":2}\n";
    let json_arr = convert::jsonlines_to_json(jsonl)?;
    let jsonl_out= convert::json_to_jsonlines(&json_arr)?;

    // Bencode <-> JSON
    let bencode_bytes = convert::json_to_bencode(r#"{"user": "alice"}"#)?;
    let json_from_benc = convert::bencode_to_json(&bencode_bytes)?;

    Ok(())
}
```

### 3. Streaming I/O

```rust
use babbel::core::io::{Buffer, BufferSource};
use babbel::xml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = BufferSource::new(b"<app><version>1.0</version></app>");
    let doc = xml::parse_source(&mut source)?;

    let mut dest = Buffer::new();
    xml::stringify_to(&doc, &mut dest);
    assert!(dest.to_string().contains("1.0"));

    Ok(())
}
```

---

## Sub-Libraries

| Crate | Link | Description |
| :--- | :--- | :--- |
| `babbel_core` | [`crates/babbel_core`](../babbel_core) | Core I/O traits (`ILineReader`), universal `Value` AST, RFC 4180 CSV/TSV, INI/.env, frontmatter, BOM detection, and numeric utilities. |
| `babbel_json` | [`crates/json`](../json) | RFC 6901 JSON Pointer, RFC 7396 Merge Patch, JSON Lines (`.jsonl`/`.ndjson`) streaming, and JSON5 comment stripping. |
| `babbel_toml` | [`crates/toml`](../toml) | Strict TOML v1.1.0 parser/emitter with micro-allocations, typed RFC 3339 datetimes, and streaming pull parser. |
| `babbel_yaml` | [`crates/yaml`](../yaml) | YAML 1.2 specification compliance, anchors, aliases, and custom tags. |
| `babbel_xml` | [`crates/xml`](../xml) | XML DOM, C14N Canonical XML, DTD validation, XSD schema, XPath 1.0. |
| `babbel_bencode` | [`crates/bencode`](../bencode) | Zero-copy borrowed parsing and streaming for BitTorrent Bencode. |

---

## Documentation

See the [Documentation Hub](../../docs/README.md) for full workspace guides:
- [Architecture Guide](../../docs/ARCHITECTURE.md)
- [Embedded Systems Guide](../../docs/EMBEDDED_GUIDE.md)
- [Text Support Guide](../../docs/TEXT_SUPPORT_GUIDE.md)
- [Conversion Matrix](../../docs/CONVERSION_MATRIX.md)
- [Development Guide](../../docs/DEVELOPMENT_GUIDE.md)
- [Contributing Guidelines](../../docs/CONTRIBUTING.md)

---

## License

Licensed under the [MIT License](../../LICENSE).
