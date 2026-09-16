# Babbel JSON Patch & Structural Diff Guide

Babbel provides comprehensive support for **RFC 6902 JSON Patch**, **RFC 7396 JSON Merge Patch**, and automated **structural document diffing** across universal [`Value`](../crates/babbel_core/src/model.rs) trees. This enables declarative, atomic configuration updates and delta calculations across all 16 supported formats.

---

## 1. RFC 6902 JSON Patch

[RFC 6902](https://datatracker.ietf.org/doc/html/rfc6902) specifies an atomic sequence of operations applied to a target document.

### Supported Operations
- `add`: Inserts a value at a target location or pushes to an array.
- `remove`: Removes the value at a target path.
- `replace`: Replaces the value at a target path.
- `move`: Moves a value from one location to another.
- `copy`: Copies a value from one location to another.
- `test`: Asserts that a target location matches an expected value (aborts transaction on mismatch).

### Rust API Example
```rust
use babbel_core::{Patch, PatchOp, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Value::Object(vec![
        ("app".into(), Value::String("babbel".into())),
        ("port".into(), Value::Integer(8080)),
        ("features".into(), Value::Array(vec![Value::String("metrics".into())])),
    ]);

    // Construct an atomic RFC 6902 patch
    let patch = Patch::new(vec![
        PatchOp::Replace {
            path: "/port".into(),
            value: Value::Integer(9090),
        },
        PatchOp::Add {
            path: "/features/1".into(),
            value: Value::String("tracing".into()),
        },
    ]);

    // Apply patch atomically
    patch.apply(&mut config)?;

    assert_eq!(config.get("port").and_then(|v| v.as_i64()), Some(9090));
    assert_eq!(config.get("features").and_then(|v| v.as_array()).unwrap().len(), 2);

    Ok(())
}
```

---

## 2. Automated Structural Diffing

Babbel can automatically compute the minimal set of RFC 6902 operations needed to transform a source document into a target document:

```rust
use babbel_core::{diff, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Value::Object(vec![
        ("title".into(), Value::String("Old Title".into())),
        ("active".into(), Value::Bool(true)),
    ]);

    let target = Value::Object(vec![
        ("title".into(), Value::String("New Title".into())),
        ("active".into(), Value::Bool(true)),
        ("version".into(), Value::Integer(2)),
    ]);

    // Generate diff
    let patch = diff(&source, &target);

    // Apply patch to source and verify it matches target
    let mut transformed = source.clone();
    patch.apply(&mut transformed)?;
    assert_eq!(transformed, target);

    Ok(())
}
```

---

## 3. RFC 7396 JSON Merge Patch

[RFC 7396](https://datatracker.ietf.org/doc/html/rfc7396) provides a simplified merge syntax where nulls delete keys and objects are merged recursively:

```rust
use babbel_core::{apply_merge_patch, diff_merge_patch, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Value::Object(vec![
        ("name".into(), Value::String("Server 1".into())),
        ("status".into(), Value::String("degraded".into())),
        ("unneeded_key".into(), Value::Integer(123)),
    ]);

    let patch = Value::Object(vec![
        ("status".into(), Value::String("healthy".into())),
        ("unneeded_key".into(), Value::Null), // Deletes the key
    ]);

    // Apply merge patch
    apply_merge_patch(&mut doc, &patch);

    assert_eq!(doc.get("status").and_then(|v| v.as_str()), Some("healthy"));
    assert!(doc.get("unneeded_key").is_none());

    Ok(())
}
```

---

## 4. Cross-Format Diffing & Patching with the CLI

The CLI provides seamless cross-format diffing and patching:

```bash
# Compute structural diff between a JSON file and a YAML file
babbel diff original.json updated.yaml

# Generate RFC 7396 Merge Patch delta
babbel diff original.toml modified.toml --merge-patch

# Apply patch to a Kubernetes YAML deployment
babbel patch deployment.yaml -p patch.json -o deployment.updated.yaml
```
