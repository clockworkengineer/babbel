# Babbel Serde Integration Guide

Babbel provides idiomatic, high-throughput integration with the **[Serde](https://serde.rs)** ecosystem through the optional `serde` feature in `babbel` and `babbel_core`.

This allows any custom Rust data structure implementing `serde::Serialize` and `serde::Deserialize` to be seamlessly serialized to, or deserialized from, **any of Babbel's 16 supported formats** (including formats that traditionally lack Serde drivers, such as KDL v2, BitTorrent Bencode, or Apache Parquet).

---

## 1. Enabling Serde Support

Add `babbel` to your `Cargo.toml` with the `serde` feature enabled:

```toml
[dependencies]
babbel = { version = "0.2.2", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

---

## 2. Converting Between Rust Structs and `Value`

The bridge between Rust types and Babbel's universal AST is provided by `babbel::core::serde::to_value` and `from_value`:

```rust
use serde::{Deserialize, Serialize};
use babbel::core::serde::{from_value, to_value};
use babbel::core::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    max_connections: u32,
    ssl_enabled: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig {
        host: "db.internal".to_string(),
        port: 5432,
        max_connections: 100,
        ssl_enabled: true,
    };

    // 1. Convert strongly-typed Rust struct to universal Value AST
    let value_ast: Value = to_value(&config)?;

    // 2. Deserialize universal Value AST back into a Rust struct
    let recovered: DatabaseConfig = from_value(&value_ast)?;
    assert_eq!(recovered, config);

    Ok(())
}
```

---

## 3. End-to-End Serialization Across Formats

By combining `to_value` and `from_value` with Babbel format engines, you can serialize any Rust struct directly into any format:

### Serializing a Struct to Terraform HCL
```rust
use serde::Serialize;
use babbel::core::serde::to_value;
use babbel::hcl;

#[derive(Serialize)]
struct Server {
    instance_type: String,
    ami: String,
    tags: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server {
        instance_type: "t3.medium".into(),
        ami: "ami-12345678".into(),
        tags: vec!["web".into(), "production".into()],
    };

    let value = to_value(&server)?;
    let hcl_text = hcl::to_string(&value)?;
    println!("{}", hcl_text);

    Ok(())
}
```

### Deserializing an Avro Container File into Rust Structs
```rust
use serde::Deserialize;
use babbel::core::serde::from_value;
use babbel::avro;

#[derive(Debug, Deserialize)]
struct UserRecord {
    name: String,
    favorite_number: Option<i32>,
    favorite_color: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ocf_bytes = std::fs::read("users.avro")?;
    let doc = avro::from_bytes_ocf(&ocf_bytes)?;

    let users: Vec<UserRecord> = from_value(&doc)?;
    for user in users {
        println!("User: {} (fav num: {:?})", user.name, user.favorite_number);
    }

    Ok(())
}
```

---

## 4. Universal Deserialization Helper

You can define a simple helper function to deserialize any text or binary input through a format engine directly into a typed struct:

```rust
use serde::de::DeserializeOwned;
use babbel_core::{FormatEngine, Value};
use babbel::core::serde::from_value;

pub fn from_format<T: DeserializeOwned>(
    input: &[u8],
    engine: &dyn FormatEngine,
) -> Result<T, Box<dyn std::error::Error>> {
    let value: Value = engine.parse_bytes(input)?;
    let typed: T = from_value(&value)?;
    Ok(typed)
}
```
