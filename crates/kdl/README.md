# Babbel KDL

A fast, zero-dependency, pure-Rust parser, serializer, and `FormatEngine` for **KDL (KDL Document Language)**, fully integrated into the **Babbel** serialization and document processing ecosystem.

## Features

- **Document & Node Model**: Seamless mapping between KDL node hierarchies, positional arguments, named properties (`key=val`), and Babbel's universal `Value` AST.
- **Full Comment Support**: Line comments (`//`), nested multiline block comments (`/* /* */ */`), and slashdash comments (`/- node`, `/- "arg"`, `/- key="val"`, `/- { children }`).
- **Flexible Numeric Literals**: Decimal, hex (`0x`), octal (`0o`), binary (`0b`), floating point, exponents, explicit signs (`+`/`-`), and numeric underscores (`1_000_000`).
- **Rich String Types**: Standard escaped strings, Unicode escapes (`\u{1F980}`), and raw strings (`r#"..."#`).
- **FormatEngine & Cross-Format Conversion**: Bidirectional conversions between KDL and JSON, YAML, TOML, XML, MessagePack, CBOR, BSON, RON, and JSON5.
- **Embedded & `no_std` Ready**: Zero runtime dependencies with `alloc` support.

## Usage

```rust
use babbel_kdl::{from_str, to_string_pretty};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = r#"
    // Zellij terminal layout
    layout {
        pane size=1 borderless=true {
            plugin location="zellij:tab-bar"
        }
        pane split_direction="vertical" {
            pane
            pane
        }
        pane size=2 {
            plugin location="zellij:status-bar"
        }
    }
    "#;

    let value = from_str(doc)?;
    println!("Parsed KDL into Value AST: {:?}", value);

    let pretty = to_string_pretty(&value, 2)?;
    println!("Serialized:\n{}", pretty);

    Ok(())
}
```
