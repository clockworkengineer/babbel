# babbel_toml

A high-performance, modular, pure-Rust library for parsing, constructing, and serializing TOML v1.1.0 documents. Engineered for embedded systems, microcontrollers, resource-constrained environments, and general-purpose applications.

[![Repository](https://img.shields.io/badge/github-clockworkengineer%2Fbabbel-blue)](https://github.com/clockworkengineer/babbel)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)

---

## Features

- **TOML v1.1.0 Conformance**: Fully compliant with the official TOML v1.1.0 specification:
  - **Inline Tables**: Multiline layout, trailing commas, and arbitrary internal comments.
  - **Escape Sequences**: `\e` (escape character `U+001B`) and `\xHH` (hexadecimal byte literals).
  - **Datetimes with Optional Seconds**: RFC 3339 datetimes, local date-times, and local times with optional seconds (e.g. `07:32`, `1979-05-27 07:32Z`).
  - **CRLF Normalization**: Automatic newline normalization in multi-line literal and basic strings.
  - Bare keys, quoted keys, dotted keys, integer bases (hex, octal, binary), underscores in numbers, and special floats (`inf`, `nan`).
- **Pure Rust & Zero Parser Dependencies**: Zero dependencies on external parser crates (no `toml` or `toml_edit` crates). Only relies on lightweight Babbel core utilities (`babbel_core`, `itoa`, `dtoa`, `smallvec`, `arrayvec`).
- **Memory Compact DOM**: `Node` memory footprint strictly constrained (`size_of::<Node>() <= 48` bytes).
- **Embedded & `no_std` Ready**: Supports `no_std` + `alloc` bare-metal microcontrollers (ARM Cortex-M, ESP32, RISC-V).
- **Streaming Pull Parser**: Zero-allocation event pull parser (`TomlPullParser`) emitting `TomlPullEvent` for $O(1)$ stack-memory document processing.
- **Polyglot Interoperability**: Bi-directional conversions with `babbel_core::Value` and integration into `babbel::convert`.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
babbel_toml = "0.1.2"
```

Or within the Babbel workspace:

```toml
[dependencies]
babbel_toml = { path = "crates/toml" }
```

To run in bare-metal / `no_std` environments:

```toml
[dependencies]
babbel_toml = { version = "0.1.2", default-features = false, features = ["alloc"] }
```

---

## Quickstart

### Parsing TOML

```rust
use babbel_toml::from_str;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let toml_src = r#"
    [server]
    host = "127.0.0.1"
    port = 8080
    workers = 4
    enabled = true
    tags = [ "web", "api" ]
    "#;

    let doc = from_str(toml_src)?;
    let server = doc.get("server").expect("server table missing");

    assert_eq!(server.get("host").and_then(|n| n.as_str()), Some("127.0.0.1"));
    assert_eq!(server.get("port").and_then(|n| n.as_integer()), Some(8080));
    assert_eq!(server.get("enabled").and_then(|n| n.as_bool()), Some(true));

    Ok(())
}
```

### Serializing TOML

```rust
use babbel_toml::{to_string, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut root = Node::new_table();
    root.insert("service".to_string(), Node::from("babbel-gateway"));
    root.insert("timeout_ms".to_string(), Node::from(5000));

    let output = to_string(&root)?;
    println!("{}", output);
    Ok(())
}
```

### Zero-Allocation Streaming Pull Parser

```rust
use babbel_toml::{TomlPullEvent, TomlPullParser};

let input = r#"
title = "Service"
port = 9000
"#;

let mut parser = TomlPullParser::new(input);
while let Ok(Some(event)) = parser.next_event() {
    match event {
        TomlPullEvent::Key(k) => println!("Key: {}", k),
        TomlPullEvent::ValueString(s) => println!("String: {}", s),
        TomlPullEvent::ValueInteger(i) => println!("Int: {}", i),
        _ => {}
    }
}
```

---

## Conformance & Testing

Run the automated test suite:

```bash
cargo test -p babbel_toml
cargo test -p babbel_toml --test conformance
```
