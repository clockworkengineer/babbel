# Babbel Schema Validation Guide

Babbel provides comprehensive schema validation capabilities across multiple specification standards:
1. **Universal JSON Schema Validation** (Draft 7 / Draft 2020-12) evaluating universal [`Value`](../crates/babbel_core/src/model.rs) AST trees.
2. **W3C XML DTD & XSD Validation** in `babbel_xml`.
3. **Apache Avro Schema Models** in `babbel_avro`.

---

## 1. Universal JSON Schema Validator

The core schema validator compiles standard JSON Schema documents in-memory and evaluates them against any document parsed into a `Value` AST:

### Rust API Usage
```rust
use babbel_core::{CompiledSchema, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_json = r#"{
        "type": "object",
        "required": ["name", "port"],
        "properties": {
            "name": { "type": "string" },
            "port": { "type": "integer", "minimum": 1, "maximum": 65535 },
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        }
    }"#;

    // 1. Compile schema in-memory
    let schema = CompiledSchema::compile_str(schema_json)?;

    // 2. Validate a valid document
    let valid_doc = Value::Object(vec![
        ("name".into(), Value::String("babbel-server".into())),
        ("port".into(), Value::Integer(8080)),
        ("tags".into(), Value::Array(vec![Value::String("prod".into())])),
    ]);
    assert!(schema.validate(&valid_doc).is_ok());

    // 3. Validate an invalid document (out of range port)
    let invalid_doc = Value::Object(vec![
        ("name".into(), Value::String("babbel-server".into())),
        ("port".into(), Value::Integer(70000)),
    ]);
    assert!(schema.validate(&invalid_doc).is_err());

    Ok(())
}
```

---

## 2. Validating Non-JSON Formats Against JSON Schema

Because all format engines in Babbel produce universal `Value` AST representations, you can validate **YAML**, **TOML**, **HCL**, **KDL**, or **CBOR** documents directly against standard JSON Schemas:

```rust
use babbel_core::CompiledSchema;
use babbel::yaml;
use babbel::toml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_text = r#"{"type": "object", "required": ["version"]}"#;
    let schema = CompiledSchema::compile_str(schema_text)?;

    // Validate a YAML configuration
    let yaml_doc = yaml::from_str("version: 2\nenvironment: production\n")?;
    assert!(schema.validate(&yaml_doc).is_ok());

    // Validate a TOML configuration
    let toml_doc = toml::from_str("version = 2\n")?;
    assert!(schema.validate(&toml_doc).is_ok());

    Ok(())
}
```

---

## 3. W3C XML DTD & XSD Validation

In `babbel_xml`, XML documents can be validated during parsing against Document Type Definitions (DTD) or W3C XML Schemas (XSD):

```rust
use babbel_xml::{parse_with_dtd_validation, XmlEngine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml_with_dtd = r#"<?xml version="1.0"?>
        <!DOCTYPE note [
            <!ELEMENT note (to,from,heading,body)>
            <!ELEMENT to (#PCDATA)>
            <!ELEMENT from (#PCDATA)>
            <!ELEMENT heading (#PCDATA)>
            <!ELEMENT body (#PCDATA)>
        ]>
        <note>
            <to>Alice</to>
            <from>Bob</from>
            <heading>Reminder</heading>
            <body>Meeting at 10</body>
        </note>
    "#;

    let doc = parse_with_dtd_validation(xml_with_dtd)?;
    assert_eq!(doc.root().name(), "note");

    Ok(())
}
```

---

## 4. Apache Avro Schema Validation

In `babbel_avro`, schemas are first-class citizens parsed via [`AvroSchema`](../crates/avro/src/schema.rs):

```rust
use babbel_avro::AvroSchema;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema_json = r#"{
        "type": "record",
        "name": "Transaction",
        "fields": [
            {"name": "id", "type": "string"},
            {"name": "amount", "type": "double"},
            {"name": "status", "type": ["null", "string"]}
        ]
    }"#;

    let schema = AvroSchema::parse_str(schema_json)?;
    assert!(matches!(schema, AvroSchema::Record { .. }));

    Ok(())
}
```

---

## 5. Schema Validation via CLI

You can validate files from the command line using `babbel validate`:

```bash
# Validate Kubernetes YAML manifest against JSON Schema
babbel validate k8s-deployment.yaml -s schema.json

# Validate TOML configuration
babbel validate config.toml -s schema.json
```
