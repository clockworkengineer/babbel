# Babbel HCL (`babbel_hcl`)

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Tests](https://img.shields.io/badge/tests-100%25%20passing-brightgreen.svg)](../../docs/conformance/HCL_CONFORMANCE.md)

Fast, modular, pure-Rust parser, serializer, and [`FormatEngine`](../../crates/babbel_core/src/codec.rs) implementation for **HashiCorp Configuration Language (HCL v2)** and **Terraform** configuration files (`.tf`, `.hcl`).

---

## Features

- **Full HCL v2 & Terraform Syntax**:
  - Top-level structural blocks (`resource "aws_instance" "web" { ... }`).
  - Nested blocks, block labels, and attribute definitions (`name = "value"`).
  - Single-line (`#`, `//`) and multi-line (`/* */`) comments.
- **Rich Expression Engine**:
  - Ternary conditional operators (`var.enabled ? "yes" : "no"`).
  - Arithmetic operators (`+`, `-`, `*`, `/`, `%`).
  - Relational and comparison operators (`==`, `!=`, `<`, `<=`, `>`, `>=`).
  - Logical boolean operators (`&&`, `||`, `!`).
- **Heredoc Strings & Multiline Support**:
  - Standard heredocs (`<<EOF ... EOF`).
  - Indented heredocs (`<<-EOF ... EOF`) with automatic common leading whitespace stripping.
- **Interpolation & Directives**:
  - String interpolation sequences (`${var.name}`).
  - Template directives (`%{if}`, `%{else}`, `%{endif}`, `%{for}`).
- **Data Structures**:
  - Tuples / arrays (`[1, 2, 3]`).
  - Objects / dictionaries (`{ a = 1, b = 2 }`).
  - Identifiers, booleans, integer literals, floating-point numbers, and strings.
- **Architectural Excellence & Plugin Integration**:
  - Zero external runtime dependencies beyond `babbel_core`.
  - Implements [`FormatEngine`] via [`HclEngine`].
  - Full `no_std` + `alloc` support for embedded and memory-constrained environments.
- **Official Specification Conformance**:
  - **100.0% pass rate** across all **2,228 tests** and 17 categories in the official [kmoneil/hcl-test-suite](https://github.com/kmoneil/hcl-test-suite.git) repository.

---

## Quickstart

Add `babbel_hcl` to your `Cargo.toml`:

```toml
[dependencies]
babbel_hcl = "0.2.1"
babbel_core = "0.2.1"
```

### 1. Parsing HCL / Terraform into Universal `Value` AST

```rust
use babbel_hcl::from_str;
use babbel_core::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tf_source = r#"
        variable "environment" {
            type        = "string"
            default     = "production"
            description = "Deployment target"
        }

        resource "aws_s3_bucket" "data" {
            bucket = "my-app-data"
            acl    = "private"

            tags = {
                Environment = "production"
                ManagedBy   = "Terraform"
            }
        }
    "#;

    let doc: Value = from_str(tf_source)?;

    // Access parsed structures
    if let Some(res) = doc.get("resource") {
        println!("Resources found: {:?}", res);
    }

    Ok(())
}
```

### 2. Serializing Universal `Value` AST to Clean HCL

```rust
use babbel_hcl::to_string;
use babbel_core::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Value::Object(vec![
        ("cluster_name".into(), Value::String("babbel-prod".into())),
        ("node_count".into(), Value::Integer(5)),
        ("features".into(), Value::Array(vec![
            Value::String("monitoring".into()),
            Value::String("autoscaling".into()),
        ])),
    ]);

    let hcl_text = to_string(&config)?;
    println!("{}", hcl_text);

    Ok(())
}
```

### 3. Using the `FormatEngine` Plugin

```rust
use babbel_hcl::HclEngine;
use babbel_core::{FormatEngine, FormatOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = HclEngine;

    assert_eq!(engine.format_id(), "hcl");
    assert_eq!(engine.mime_type(), "application/x-hcl");
    assert_eq!(engine.file_extensions(), &["hcl", "tf"]);
    assert!(!engine.is_binary());

    let val = Value::Object(vec![("app".into(), Value::String("babbel".into()))]);
    let options = FormatOptions::default();

    let bytes = engine.serialize_to_vec(&val, &options)?;
    let parsed = engine.parse_bytes(&bytes)?;
    assert_eq!(parsed.get("app").and_then(|v| v.as_str()), Some("babbel"));

    Ok(())
}
```

---

## Conformance Verification

To run the full official HCL conformance test suite:

```bash
# Download official test suite
./scripts/fetch_hcl_test_suite.ps1

# Execute conformance test runner
cargo test -p babbel_hcl --test hcl_conformance -- --nocapture
```

Detailed test reports and category breakdowns are available in the [HCL Conformance Guide](../../docs/conformance/HCL_CONFORMANCE.md).
