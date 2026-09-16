# Babbel CLI (`babbel-cli`)

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)

Universal, high-performance command-line utility for multi-format serialization, document querying, structural AST diffing, RFC 6902 patching, schema validation, and formatting across **16 data formats**.

---

## Installation

### From Source
```bash
cargo install --path crates/cli
```

Verify installation:
```bash
babbel --version
```

---

## Command Reference

### 1. `babbel convert` — Universal Cross-Format Conversion
Converts documents between any of the 16 supported formats with automatic format autodetection.

```bash
# Convert Terraform HCL to JSON
babbel convert main.tf -t json -o main.json

# Convert JSON to Apache Parquet
babbel convert data.json -t parquet -o data.parquet

# Pipe XML from stdin to YAML on stdout
cat config.xml | babbel convert - -f xml -t yaml --pretty

# Convert CBOR binary payload to compact MessagePack
babbel convert payload.cbor -t msgpack -o payload.msgpack
```

**Options**:
- `-f, --from <FORMAT>`: Source format (auto-detected from file extension if omitted).
- `-t, --to <FORMAT>`: Target format (`json`, `yaml`, `toml`, `xml`, `cbor`, `msgpack`, `bson`, `ron`, `kdl`, `parquet`, `hcl`, `avro`, `csv`, `tsv`, `ini`, `bencode`).
- `-o, --output <PATH>`: Output file path (defaults to stdout).
- `--pretty`: Pretty-print formatted output (default: true for text formats).
- `--indent <N>`: Indentation spaces for formatted text (default: 2).

---

### 2. `babbel query` — RFC 9535 JSONPath Document Querying
Executes standard JSONPath query expressions across any format without requiring format-specific tools.

```bash
# Query YAML configuration with filter expression
babbel query servers.yaml -q '$.servers[?(@.active == true)].ip'

# Extract the first matching element from a TOML file
babbel query Cargo.toml -q '$.package.name' --first

# Query an Avro container file
babbel query users.avro -q '$[*].name'
```

**Options**:
- `-q, --query <EXPR>`: RFC 9535 JSONPath expression (e.g. `$.store.book[*].author`).
- `--first`: Return only the first match instead of an array of matches.

---

### 3. `babbel diff` — Structural Document Diffing
Computes structural deltas between two documents, producing standard RFC 6902 JSON Patch operations or RFC 7396 Merge Patch representations.

```bash
# Compute RFC 6902 patch between original and modified configs
babbel diff config.v1.json config.v2.yaml

# Compute RFC 7396 Merge Patch
babbel diff old.toml new.toml --merge-patch
```

---

### 4. `babbel patch` — Atomic Document Patching
Applies an RFC 6902 JSON Patch or RFC 7396 Merge Patch file to a document.

```bash
# Apply RFC 6902 patch to update a configuration
babbel patch base.yaml -p update.patch.json -o updated.yaml

# Apply RFC 7396 Merge Patch
babbel patch config.json -p delta.json --merge-patch
```

---

### 5. `babbel validate` — JSON Schema Validation
Validates any document (JSON, YAML, TOML, XML, KDL, HCL, etc.) against a standard JSON Schema (Draft 7 / Draft 2020-12).

```bash
# Validate Kubernetes YAML manifest against schema
babbel validate deployment.yaml -s k8s-schema.json

# Validate TOML configuration
babbel validate config.toml -s schema.json
```

---

### 6. `babbel fmt` — Document Formatting & Pretty-Printing
Formats, validates syntax, and pretty-prints documents with customizable indentation.

```bash
# Pretty-print and format a minified JSON file in-place
babbel fmt messy.json -o clean.json --indent 4
```

---

### 7. `babbel inspect` — Document Diagnostics
Analyzes a document, displaying its detected format, AST data type, record count, byte size, and maximum nesting depth.

```bash
babbel inspect complex.avro
```

Output:
```
File: complex.avro
Detected Format: avro (application/avro)
Data Type: Array
Record Count: 3
Size: 265 bytes
Max Nesting Depth: 2
```

---

## Supported Formats

| Format ID | MIME Type | Extensions | Classification |
| :--- | :--- | :--- | :--- |
| `json` | `application/json` | `.json` | Text |
| `yaml` | `application/yaml` | `.yaml`, `.yml` | Text |
| `toml` | `application/toml` | `.toml` | Text |
| `xml` | `application/xml` | `.xml` | Text |
| `hcl` | `application/x-hcl` | `.hcl`, `.tf` | Text |
| `kdl` | `application/kdl` | `.kdl` | Text |
| `ron` | `application/ron` | `.ron` | Text |
| `jsonl` | `application/x-ndjson` | `.jsonl`, `.ndjson` | Text |
| `csv` | `text/csv` | `.csv` | Text |
| `tsv` | `text/tab-separated-values` | `.tsv` | Text |
| `ini` | `text/plain` | `.ini`, `.env`, `.properties` | Text |
| `cbor` | `application/cbor` | `.cbor` | Binary |
| `msgpack` | `application/msgpack` | `.msgpack`, `.mp` | Binary |
| `bson` | `application/bson` | `.bson` | Binary |
| `avro` | `application/avro` | `.avro` | Binary |
| `parquet` | `application/x-parquet` | `.parquet` | Binary |
| `bencode` | `application/x-bencode` | `.bencode`, `.torrent` | Binary |
