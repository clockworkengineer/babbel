# Babbel RFC 9535 JSONPath Query Guide

Babbel includes a high-performance, zero-dependency **RFC 9535 JSONPath** query engine operating across universal [`Value`](../crates/babbel_core/src/model.rs) AST trees. This allows developers to query documents in **any supported format** (JSON, YAML, TOML, XML, KDL, HCL, CBOR, Avro, etc.) using unified JSONPath syntax.

---

## 1. Syntax Overview

Babbel implements the official IETF [RFC 9535](https://datatracker.ietf.org/doc/html/rfc9535) specification:

| Operator | Syntax | Description | Example |
| :--- | :--- | :--- | :--- |
| **Root** | `$` | Represents the root document node | `$` |
| **Dot Child** | `.key` | Selects a named property on an object | `$.store.name` |
| **Bracket Child** | `['key']` | Selects a named property (supports special characters) | `$['store']['street-address']` |
| **Wildcard** | `*` | Selects all children of an object or all elements of an array | `$.store.book[*]` |
| **Recursive Descent** | `..` | Recursively searches all descendants for matches | `$..author` |
| **Array Index** | `[i]` | Selects a specific element (supports negative indices) | `$.books[0]`, `$.books[-1]` |
| **Array Slice** | `[start:end:step]` | Selects a sub-slice of array elements | `$.books[0:5:2]` |
| **Filter Expression** | `[?(expr)]` | Filters array elements by predicate | `$.books[?(@.price < 30)]` |
| **Current Node** | `@` | Refers to the current element inside a filter expression | `@.price` |

---

## 2. Rust API Usage

The query engine is accessible via `babbel::core::jsonpath_query` or convenience methods on `Value`:

```rust
use babbel_core::{jsonpath_query, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = Value::Object(vec![
        ("store".into(), Value::Object(vec![
            ("book".into(), Value::Array(vec![
                Value::Object(vec![
                    ("title".into(), Value::String("Sayings of the Century".into())),
                    ("price".into(), Value::Float(8.95)),
                ]),
                Value::Object(vec![
                    ("title".into(), Value::String("Sword of Honour".into())),
                    ("price".into(), Value::Float(12.99)),
                ]),
            ])),
        ])),
    ]);

    // Query 1: Extract all titles via recursive descent
    let titles = jsonpath_query(&doc, "$..title")?;
    assert_eq!(titles.len(), 2);
    assert_eq!(titles[0].as_str(), Some("Sayings of the Century"));

    // Query 2: Filter books by price
    let cheap_books = jsonpath_query(&doc, "$.store.book[?(@.price < 10.0)]")?;
    assert_eq!(cheap_books.len(), 1);

    Ok(())
}
```

---

## 3. Format-Agnostic Querying

Because Babbel routes all format parsing through the universal `Value` AST, the same query expression works identically across any document format:

```rust
use babbel::core::jsonpath_query;
use babbel::yaml;
use babbel::toml;
use babbel::hcl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Query a YAML document
    let yaml_text = "servers:\n  - host: 10.0.0.1\n    active: true\n  - host: 10.0.0.2\n    active: false\n";
    let yaml_val = yaml::from_str(yaml_text)?;
    let active_servers = jsonpath_query(&yaml_val, "$.servers[?(@.active == true)].host")?;
    assert_eq!(active_servers.len(), 1);

    // 2. Query a TOML document
    let toml_text = "[package]\nname = \"my-app\"\nversion = \"1.0.0\"\n";
    let toml_val = toml::from_str(toml_text)?;
    let app_name = jsonpath_query(&toml_val, "$.package.name")?;
    assert_eq!(app_name[0].as_str(), Some("my-app"));

    // 3. Query a Terraform HCL document
    let hcl_text = "resource \"aws_s3_bucket\" \"logs\" { bucket = \"my-logs\" }";
    let hcl_val = hcl::from_str(hcl_text)?;
    let bucket = jsonpath_query(&hcl_val, "$.resource.aws_s3_bucket.logs.bucket")?;
    assert_eq!(bucket[0].as_str(), Some("my-logs"));

    Ok(())
}
```

---

## 4. CLI Querying

You can execute JSONPath queries directly from your shell using `babbel query`:

```bash
# Query YAML file
babbel query deployment.yaml -q '$.spec.template.spec.containers[*].name'

# Query TOML file and return first match
babbel query Cargo.toml -q '$.dependencies.*.version' --first

# Filter Avro records
babbel query users.avro -q '$[?(@.favorite_number > 10)].name'
```
