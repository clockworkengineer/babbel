# Babbel Cookbook & Practical Recipes

Comprehensive, end-to-end recipe guide for real-world document translation, high-throughput analytics, zero-allocation embedded streaming, schema evolution, and automated CLI workflows with **Babbel**.

---

## Table of Contents
1. [Recipe 1: Microservice Config Translation](#recipe-1-microservice-config-translation)
2. [Recipe 2: High-Throughput Analytics & JSON Schema Validation](#recipe-2-high-throughput-analytics--json-schema-validation)
3. [Recipe 3: Zero-Allocation Embedded Parsing on Microcontrollers](#recipe-3-zero-allocation-embedded-parsing-on-microcontrollers)
4. [Recipe 4: Schema Evolution & Binary Storage](#recipe-4-schema-evolution--binary-storage)
5. [Recipe 5: CI/CD Pipeline Automation with the CLI](#recipe-5-cicd-pipeline-automation-with-the-cli)
6. [Diagnostic Troubleshooting Guide](#diagnostic-troubleshooting-guide)

---

## Recipe 1: Microservice Config Translation

### Problem
You need to convert infrastructure configurations between Kubernetes YAML, HashiCorp HCL (Terraform), and TOML with custom formatting rules (sorted keys, CRLF line endings, and custom indentations).

### Solution

```rust
use babbel::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let k8s_yaml = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  labels:
    app: nginx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
"#;

    // 1. Parse YAML into universal Value AST
    let config = babbel::from_str(Format::Yaml, k8s_yaml)?;

    // 2. Query and inspect fields with dot-separated path navigation
    if let Some(replicas) = config.get_path("spec.replicas").and_then(|v| v.as_i64()) {
        println!("Configured replicas: {}", replicas);
    }

    // 3. Configure formatting options via FormatOptionsBuilder
    let pretty_options = FormatOptions::builder()
        .pretty(true)
        .indent(4)
        .sort_keys(true)
        .build();

    // 4. Translate directly to HashiCorp HCL and TOML
    let registry = babbel::default_registry();
    let toml_engine = registry.get_by_id("toml").unwrap();
    let hcl_engine = registry.get_by_id("hcl").unwrap();

    let toml_out = toml_engine.serialize_to_string(&config, &pretty_options)?;
    let hcl_out = hcl_engine.serialize_to_string(&config, &pretty_options)?;

    println!("--- TOML Output ---\n{}", toml_out);
    println!("--- HCL Output ---\n{}", hcl_out);

    Ok(())
}
```

---

## Recipe 2: High-Throughput Analytics & JSON Schema Validation

### Problem
Ingest multi-megabyte CSV telemetry streams, validate each record against a strict JSON Schema Draft 2020-12 specification, and transcode valid records into compact binary CBOR or MessagePack.

### Solution

```rust
use babbel::prelude::*;
use babbel_core::schema::{infer_schema, validate_schema};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_csv = "sensor_id,temperature,humidity,status\nsens-101,23.5,45.2,ONLINE\nsens-102,28.1,51.0,ONLINE\n";

    // 1. Ingest CSV tabular data into Value::Array of row Objects
    let records = babbel::from_str(Format::Csv, raw_csv)?;

    // 2. Define or compile a JSON Schema
    let schema_json = r#"{
        "type": "array",
        "items": {
            "type": "object",
            "required": ["sensor_id", "temperature", "status"],
            "properties": {
                "sensor_id": { "type": "string" },
                "temperature": { "type": "number" },
                "status": { "type": "string" }
            }
        }
    }"#;
    let schema_ast = babbel::from_str(Format::Json, schema_json)?;
    let compiled_schema = babbel_core::schema::compile_schema(&schema_ast)?;

    // 3. Validate entire batch against schema
    validate_schema(&compiled_schema, &records)?;
    println!("All CSV telemetry records successfully validated!");

    // 4. Compress and store directly as binary MessagePack
    let msgpack_bytes = babbel::to_vec(Format::MsgPack, &records)?;
    println!("Serialized {} bytes of binary MessagePack", msgpack_bytes.len());

    Ok(())
}
```

---

## Recipe 3: Zero-Allocation Embedded Parsing on Microcontrollers

### Problem
Parse JSON or CBOR sensor telemetry on resource-constrained microcontrollers (`no_std` / Cortex-M) with zero dynamic heap allocations.

### Solution

```rust
use babbel_core::embedded::{StackBuffer, MemoryTracker};
use babbel_core::io::SliceDestination;

fn main() {
    // 1. Fixed 256-byte stack-allocated buffer
    let mut stack_buf = StackBuffer::<256>::new();
    let _ = stack_buf.as_mut_slice();

    // 2. Track heap memory usage to ensure 0 allocations
    let tracker = MemoryTracker::with_limit(1024);

    // 3. Stream data using SliceDestination (zero allocation)
    let mut memory_buffer = [0u8; 128];
    let mut dest = SliceDestination::new(&mut memory_buffer);

    dest.add_bytes(r#"{"status":"OK","code":200}"#);
    assert_eq!(dest.as_str().unwrap(), r#"{"status":"OK","code":200}"#);

    assert_eq!(tracker.peak(), 0, "Zero dynamic heap allocations occurred!");
}
```

---

## Recipe 4: Schema Evolution & Binary Storage

### Problem
Safely convert schemas and binary representations between Avro, BSON, and Parquet columnar layouts.

### Solution

```rust
use babbel::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build a strongly typed AST using the fluent builder
    let user_record = Value::object()
        .set("user_id", 42001)
        .set("email", "engineer@domain.com")
        .set("roles", Value::array().push("admin").push("developer").build())
        .set("metadata", Value::object().set("created_at", 1700000000).build())
        .build();

    // 2. Encode to Avro binary stream
    let avro_bytes = babbel::to_vec(Format::Avro, &user_record)?;

    // 3. Encode to BSON binary stream
    let bson_bytes = babbel::to_vec(Format::Bson, &user_record)?;

    // 4. Verify cross-format roundtrip fidelity
    let decoded_from_avro = babbel::from_slice(Format::Avro, &avro_bytes)?;
    let decoded_from_bson = babbel::from_slice(Format::Bson, &bson_bytes)?;

    assert_eq!(decoded_from_avro.get("user_id"), decoded_from_bson.get("user_id"));
    Ok(())
}
```

---

## Recipe 5: CI/CD Pipeline Automation with the CLI

Use the unified `babbel` CLI directly in your CI/CD shell scripts.

### 1. Cross-Format Conversion
```bash
# Convert Kubernetes YAML to JSON
babbel convert deploy.yaml deploy.json

# Convert CSV analytics data to compressed Parquet or CBOR
babbel convert --to cbor telemetry.csv telemetry.cbor
```

### 2. Querying Values with JSONPath
```bash
# Extract list of container images from a Kubernetes Deployment
babbel query -q "$.spec.template.spec.containers[*].image" deploy.yaml
```

### 3. Mutating Configurations in Place
```bash
# Update replica count in CI deployment step
babbel set spec.replicas 5 deploy.yaml -o deploy.prod.yaml
```

### 4. Merging Configuration Overlays
```bash
# Merge production overlay into base configuration
babbel merge base.yaml prod-overlay.yaml -o final.yaml
```

---

## Diagnostic Troubleshooting Guide

| Error Code | Common Cause | Recommended Resolution |
|---|---|---|
| `SyntaxError` | Malformed punctuation, mismatched braces, or invalid format tokens. | Verify source syntax at line/col span indicated in `BabbelError`. |
| `UnexpectedEof` | Truncated input, missing closing bracket, or incomplete transmission. | Ensure all input bytes are received before parsing; verify file transfer integrity. |
| `InvalidEncoding` | Invalid UTF-8 sequence, invalid hex digit, or corrupted base64 string. | Check file encoding (BOM stripped automatically) and validate binary text representations. |
| `SchemaValidation` | Input document violates required properties, type constraints, or ranges. | Inspect schema error path and adjust input data to conform to expected structure. |
| `UnsupportedType` | Requested format is not registered or disabled by feature flags. | Ensure format feature is enabled in `Cargo.toml` (e.g. `features = ["avro", "parquet"]`). |
| `RecursionLimit` | Nesting depth exceeds parser limit (default: 128 standard, 16 embedded). | Simplify document hierarchy or increase `EmbeddedLimits::max_depth`. |
| `IoError` | Disk write error, permissions failure, or destination capacity exhaustion. | Check disk space, file permissions, or increase buffer slice capacity. |

