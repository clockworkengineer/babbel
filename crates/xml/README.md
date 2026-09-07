# babbel_xml

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2021-orange)](Cargo.toml)

A robust, full-featured pure Rust XML toolkit providing W3C-compliant DOM construction, validating SAX/pull-parser, W3C Canonical XML (C14N 1.0/1.1), DTD validation, XSD schema validation, and an XPath 1.0 query engine. Backed by the unified [`babbel_core`](../babbel_core) streaming I/O architecture.

---

## Features

- **W3C XML DOM Engine**:
  - Full support for elements, text nodes, CDATA sections, comments, and processing instructions.
  - Complete XML Namespaces (XMLNS) resolution and prefix inheritance.
  - Serialization with customizable indentation, attribute sorting, and entity escaping.
- **W3C Canonical XML (C14N 1.0 & 1.1)**:
  - Canonicalize documents and subtrees for cryptographic signature verification.
  - Handles namespace prefix rewrites, attribute ordering, and character normalization.
- **Validation**:
  - **DTD Validation**: Element content models, attribute types (`CDATA`, `ID`, `IDREF`), and entity expansion limits.
  - **XSD Schema Validator**: Simple types, complex types, sequences, choices, and facet validation.
- **XPath 1.0 Engine**:
  - Full XPath 1.0 expression evaluator with predicate filters, axes (`child`, `descendant`, `attribute`, `parent`), and core function library (`count`, `string`, `concat`, `contains`, `starts-with`, `substring`).
- **Security Hardening**:
  - Billion Laughs exponential entity expansion mitigation.
  - Configurable nesting depth and text node limits to prevent unbounded memory allocation.
- **Unified Streaming I/O**:
  - Implements `ICharStream`, `IRewindable`, `ILocationAware`, `IClearable`, and `ITailInspectable` from `babbel_core::io`.
- **`no_std` Support**:
  - Operates in memory-constrained environments with the `alloc` feature.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
babbel_xml = "0.1.0"
```

Or as a workspace path dependency:

```toml
[dependencies]
babbel_xml = { path = "crates/xml" }
```

### Feature Flags

| Feature | Default | Description |
| :--- | :--- | :--- |
| `std` | **Yes** | Standard library file I/O and stream reading. |
| `alloc` | **Yes** (via `std`) | Heap allocation primitives for `no_std` environments. |
| `dtd` | **Yes** | DTD parsing and document validation engine. |
| `xsd` | **Yes** | XSD schema compilation and validation. |
| `xpath` | **Yes** | XPath 1.0 expression evaluator. |
| `stringify` | **Yes** | XML serialization and C14N canonicalization. |
| `serde` | No | Serde derive support for XML data structures. |

---

## Quickstart

### 1. Parsing & DOM Navigation

```rust
use babbel_xml::Document;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"
        <catalog xmlns:bk="urn:book">
            <bk:book id="b1" price="19.99">
                <bk:title>Rust in Action</bk:title>
            </bk:book>
        </catalog>
    "#;

    let doc = Document::parse_str(xml)?;
    let root = doc.root_element().unwrap();
    assert_eq!(root.name(), "catalog");

    let book = root.children_elements().next().unwrap();
    assert_eq!(book.attribute_value("id"), Some("b1"));
    assert_eq!(book.attribute_value("price"), Some("19.99"));

    let title = book.children_elements().next().unwrap();
    assert_eq!(title.text_content(), "Rust in Action");

    Ok(())
}
```

### 2. XPath 1.0 Queries

```rust
use babbel_xml::Document;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"
        <store>
            <item category="electronics"><name>Laptop</name><price>999</price></item>
            <item category="books"><name>Book</name><price>15</price></item>
        </store>
    "#;

    let doc = Document::parse_str(xml)?;
    
    // Evaluate XPath expression
    let items = doc.select_nodes("//item[@category='electronics']/name")?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].text_content(), "Laptop");

    Ok(())
}
```

### 3. Canonical XML (C14N)

```rust
use babbel_xml::stringify::canonicalize;
use babbel_xml::Document;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"<doc b="2" a="1">  <child/>  </doc>"#;
    let doc = Document::parse_str(xml)?;

    // Canonicalize attributes and whitespace
    let c14n_xml = canonicalize(&doc)?;
    assert!(c14n_xml.starts_with(r#"<doc a="1" b="2">"#));

    Ok(())
}
```

### 4. Streaming I/O with `babbel_core`

```rust
use babbel_xml::io::{XmlSource, XmlDestination};
use babbel_core::io::traits::{ICharStream, IDestination};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Zero-allocation character source
    let mut source = XmlSource::from_string("<root>data</root>");
    assert_eq!(source.current(), Some('<'));
    source.next();
    assert_eq!(source.current(), Some('r'));

    // Reusable output destination
    let mut dest = XmlDestination::new();
    dest.add_bytes("<output>ok</output>");
    assert_eq!(dest.as_str(), "<output>ok</output>");

    Ok(())
}
```

---

## Security Limits

`babbel_xml` provides built-in defenses against malicious XML payloads:

* `max_depth`: Limits nesting to prevent stack overflows (default: 256).
* `max_entity_expansions`: Mitigates Billion Laughs exponential entity expansion (default: 10,000).
* `max_stream_size`: Limits unbounded streaming inputs (default: 50 MB).

```rust
use babbel_xml::io::XmlSource;

// Read streaming input with a custom safety limit (10 MB)
let file = std::fs::File::open("large.xml")?;
let source = XmlSource::from_reader_with_limit(file, 10 * 1024 * 1024)?;
```

---

## License

Licensed under the [MIT License](../../LICENSE).
