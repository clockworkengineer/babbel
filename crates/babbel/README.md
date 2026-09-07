# babbel

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)

The master facade crate for the **Babbel** multi-format serialization and document processing ecosystem. It provides unified ergonomics, re-exports all domain engines (JSON, YAML, XML, Bencode), and powers open-ended, $O(N)$ cross-format conversion pipelines.

---

## Features

- **Unified Prelude**: Access all formats with a single dependency.
- **Polyglot Parsing**: Parse JSON, YAML, XML, and BitTorrent Bencode into idiomatic typed representations.
- **Universal Cross-Conversion**: Convert documents between arbitrary formats without manual intermediate representations:
  - JSON $\leftrightarrow$ YAML
  - JSON $\leftrightarrow$ XML
  - JSON $\leftrightarrow$ Bencode
  - YAML $\leftrightarrow$ XML
  - YAML $\leftrightarrow$ Bencode
  - XML $\leftrightarrow$ Bencode
- **Extensible Streaming I/O**: Stream to and from files or memory buffers powered by [`babbel_core`](../babbel_core).
- **Fine-Grained Feature Flags**: Enable only the formats your application requires.

---

## Installation

Add `babbel` to your `Cargo.toml`:

```toml
[dependencies]
babbel = "0.1.0"
```

Or as a workspace path dependency:

```toml
[dependencies]
babbel = { path = "crates/babbel" }
```

### Feature Flags

By default, all formats and the conversion pipeline are enabled (`["std", "json", "yaml", "xml", "bencode", "convert"]`). To minimize binary size, enable only what you need:

| Feature | Description |
| :--- | :--- |
| `std` *(default)* | Enables standard library I/O and OS integration. |
| `alloc` | Enables heap allocation without full `std` (`no_std` environments). |
| `json` *(default)* | Re-exports `json_lib` for JSON parsing, pointers, and patch operations. |
| `yaml` *(default)* | Re-exports `yaml_lib` for full YAML 1.2 parsing and emission. |
| `xml` *(default)* | Re-exports `xml_lib` for validating XML DOM, C14N, and XPath 1.0. |
| `bencode` *(default)* | Re-exports `bencode_lib` for BitTorrent Bencode processing. |
| `convert` *(default)* | Enables the `babbel::convert` cross-format conversion pipeline. |

---

## Quickstart

### 1. Document Parsing & Navigation

```rust
use babbel::json;
use babbel::yaml;
use babbel::xml;
use babbel::bencode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse JSON
    let json_doc = json::from_str(r#"{"service": "babbel", "port": 8080}"#)?;
    assert_eq!(json_doc.get("service").and_then(|n| n.as_str()), Some("babbel"));

    // Parse YAML
    let yaml_doc = yaml::parse_string("service: babbel\nport: 8080\n")?;
    assert_eq!(yaml_doc.get("service").and_then(|n| n.as_str()), Some("babbel"));

    // Parse XML
    let xml_doc = xml::parse("<service port=\"8080\">babbel</service>")?;
    assert_eq!(xml_doc.get_root_element_name(), Some("service"));

    // Parse Bencode
    let bencode_doc = bencode::parse_bytes(b"d4:porti8080e7:service6:babbele")?;
    
    Ok(())
}
```

### 2. Universal Cross-Format Conversions

```rust
use babbel::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = r#"{"title": "Babbel", "active": true, "version": 1}"#;

    // Convert JSON to YAML
    let yaml_output = convert::json_to_yaml(json_data)?;

    // Convert YAML back to JSON
    let json_output = convert::yaml_to_json(&yaml_output)?;

    // Convert JSON to XML
    let xml_output = convert::json_to_xml(json_data)?;

    // Convert JSON to binary Bencode
    let bencode_bytes = convert::json_to_bencode(json_data)?;

    // Convert Bencode directly to YAML
    let yaml_from_bencode = convert::bencode_to_yaml(&bencode_bytes)?;

    Ok(())
}
```

### 3. Streaming I/O

```rust
use babbel::core::io::{Buffer, BufferSource};
use babbel::xml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Stream from an in-memory buffer source
    let mut source = BufferSource::new(b"<app><version>1.0</version></app>");
    let doc = xml::parse_source(&mut source)?;

    // Stream directly into an in-memory destination
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
| `babbel_core` | [`crates/babbel_core`](../babbel_core) | Core I/O traits, universal `Value` AST, BOM detection, and numeric utilities. |
| `json_lib` | [`crates/json`](../json) | RFC 6901 JSON Pointer, RFC 7396 Merge Patch, JSON5 comment stripping. |
| `yaml_lib` | [`crates/yaml`](../yaml) | YAML 1.2 specification compliance, anchors, aliases, and custom tags. |
| `xml_lib` | [`crates/xml`](../xml) | XML DOM, C14N Canonical XML, DTD validation, XSD schema, XPath 1.0. |
| `bencode_lib` | [`crates/bencode`](../bencode) | Zero-copy borrowed parsing and streaming for BitTorrent Bencode. |

---

## License

Licensed under the [MIT License](../../LICENSE).
