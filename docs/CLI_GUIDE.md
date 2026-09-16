# Babbel CLI User Guide & Command Reference

The **Babbel CLI** (`babbel`) is a universal, high-performance command-line developer utility and automation tool for multi-format serialization, document querying, structural AST diffing, RFC 6902 patching, schema validation, and formatting across **16 data formats**.

---

## 1. Installation & Setup

### Building & Installing from Source
```bash
cargo install --path crates/cli
```

Verify your installation:
```bash
babbel --version
```

### Shell Autocompletion
The CLI supports standard POSIX command patterns and flags, making it ideal for integration into bash/zsh scripts, CI/CD runners, and Kubernetes operations.

---

## 2. Global Usage & Flags

```
babbel <SUBCOMMAND> [OPTIONS]
```

### Global Options
- `-h, --help`: Displays help information for the tool or specific subcommand.
- `-V, --version`: Displays version information.

### Stdin / Stdout Piping Convention
For commands accepting input paths, passing `-` instructs the CLI to read from standard input (`stdin`). When the `-o, --output` flag is omitted, results are emitted directly to standard output (`stdout`).

---

## 3. Subcommand Reference

### 3.1 `babbel convert` — Cross-Format Conversion

Converts documents between any of the 16 supported formats with automatic format autodetection based on file extension.

```bash
babbel convert <INPUT> -t <TO_FORMAT> [OPTIONS]
```

#### Options
| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --from <FORMAT>` | Source format identifier | Inferred from file extension |
| `-t, --to <FORMAT>` | Target format identifier (required) | N/A |
| `-o, --output <PATH>` | Output file destination | Standard output (`stdout`) |
| `--pretty` | Pretty-print formatted text output | `true` for text formats |
| `--indent <N>` | Indentation spaces for formatted text | `2` |

#### Examples
```bash
# Convert Terraform HCL configuration to JSON
babbel convert main.tf -t json -o main.json

# Convert JSON to Apache Parquet columnar table
babbel convert dataset.json -t parquet -o dataset.parquet

# Convert CBOR binary payload to YAML from stdin
cat payload.cbor | babbel convert - -f cbor -t yaml

# Convert an Avro container file to TOML
babbel convert users.avro -t toml -o users.toml
```

---

### 3.2 `babbel query` — RFC 9535 JSONPath Document Querying

Executes standard RFC 9535 JSONPath query expressions across any document format, eliminating the need to install separate tools like `jq`, `yq`, or `xq`.

```bash
babbel query <INPUT> -q <JSONPATH_EXPRESSION> [OPTIONS]
```

#### Options
| Option | Description | Default |
| :--- | :--- | :--- |
| `-q, --query <EXPR>` | RFC 9535 JSONPath expression | N/A (Required) |
| `-f, --from <FORMAT>` | Document format identifier | Inferred from file extension |
| `--first` | Return only the first matching element | `false` (returns array of matches) |

#### Examples
```bash
# Query YAML Kubernetes manifest for pod container images
babbel query deployment.yaml -q '$.spec.template.spec.containers[*].image'

# Extract the package name from Cargo.toml
babbel query Cargo.toml -q '$.package.name' --first

# Filter active servers from a JSON file
babbel query infrastructure.json -q '$.servers[?(@.active == true)].ip'
```

---

### 3.3 `babbel diff` — Structural Document Diffing

Compares two documents structurally regardless of formatting, key order, or representation formats, generating standard RFC 6902 JSON Patch operations or RFC 7396 Merge Patch representations.

```bash
babbel diff <SOURCE> <TARGET> [OPTIONS]
```

#### Options
| Option | Description | Default |
| :--- | :--- | :--- |
| `--merge-patch` | Output an RFC 7396 Merge Patch object instead of RFC 6902 Patch operations | `false` |

#### Examples
```bash
# Compare a JSON configuration against a YAML deployment
babbel diff config.v1.json config.v2.yaml

# Generate RFC 7396 merge patch delta
babbel diff original.toml modified.toml --merge-patch
```

---

### 3.4 `babbel patch` — Atomic Document Patching

Applies an RFC 6902 JSON Patch (an array of `add`, `remove`, `replace`, `move`, `copy`, `test` operations) or an RFC 7396 Merge Patch to any document.

```bash
babbel patch <INPUT> -p <PATCH_FILE> [OPTIONS]
```

#### Options
| Option | Description | Default |
| :--- | :--- | :--- |
| `-p, --patch <PATH>` | Path to patch document (required) | N/A |
| `--merge-patch` | Treat patch as RFC 7396 Merge Patch | `false` |
| `-o, --output <PATH>` | Output destination | Standard output (`stdout`) |

#### Examples
```bash
# Apply atomic RFC 6902 patch to a YAML config
babbel patch settings.yaml -p updates.patch.json -o settings.updated.yaml

# Apply merge patch delta in-place
babbel patch app.json -p delta.json --merge-patch -o app.json
```

---

### 3.5 `babbel validate` — JSON Schema Validation

Validates documents in any format (YAML, TOML, XML, KDL, HCL, etc.) against a standard JSON Schema (Draft 7 / Draft 2020-12).

```bash
babbel validate <INPUT> -s <SCHEMA_FILE>
```

#### Options
| Option | Description | Default |
| :--- | :--- | :--- |
| `-s, --schema <PATH>` | Path to JSON Schema file (required) | N/A |

#### Examples
```bash
# Validate a YAML configuration against a JSON Schema
babbel validate cluster.yaml -s schema.json

# Exit codes: returns 0 on success, 1 on validation error
if babbel validate config.toml -s schema.json; then
    echo "Configuration valid"
fi
```

---

### 3.6 `babbel fmt` — Document Formatting

Formats, validates syntax, and pretty-prints documents with configurable indentation.

```bash
babbel fmt <INPUT> [OPTIONS]
```

#### Options
| Option | Description | Default |
| :--- | :--- | :--- |
| `-o, --output <PATH>` | Output destination | Standard output (`stdout`) |
| `--indent <N>` | Number of indentation spaces | `2` |

#### Examples
```bash
# Pretty-print a minified JSON file
babbel fmt compact.json -o formatted.json --indent 4
```

---

### 3.7 `babbel inspect` — Document Diagnostics

Inspects a document without re-serializing, reporting detected format, AST data type, record counts, file size, and maximum hierarchy depth.

```bash
babbel inspect users.avro
```

Output:
```
File: users.avro
Detected Format: avro (application/avro)
Data Type: Array
Record Count: 3
Size: 265 bytes
Max Nesting Depth: 2
```

---

## 4. Supported Formats Reference

| Format Identifier | Format Name | MIME Type | File Extensions | Category |
| :--- | :--- | :--- | :--- | :--- |
| `json` | JavaScript Object Notation | `application/json` | `.json` | Text |
| `yaml` | YAML Ain't Markup Language | `application/yaml` | `.yaml`, `.yml` | Text |
| `toml` | Tom's Obvious Minimal Language | `application/toml` | `.toml` | Text |
| `xml` | Extensible Markup Language | `application/xml` | `.xml` | Text |
| `hcl` | HashiCorp Configuration Language | `application/x-hcl` | `.hcl`, `.tf` | Text |
| `kdl` | KDL Document Language | `application/kdl` | `.kdl` | Text |
| `ron` | Rusty Object Notation | `application/ron` | `.ron` | Text |
| `jsonl` | JSON Lines (NDJSON) | `application/x-ndjson` | `.jsonl`, `.ndjson` | Text |
| `csv` | Comma-Separated Values | `text/csv` | `.csv` | Text |
| `tsv` | Tab-Separated Values | `text/tab-separated-values` | `.tsv` | Text |
| `ini` | Initialization Configuration | `text/plain` | `.ini`, `.env`, `.properties` | Text |
| `cbor` | Concise Binary Object Representation | `application/cbor` | `.cbor` | Binary |
| `msgpack` | MessagePack | `application/msgpack` | `.msgpack`, `.mp` | Binary |
| `bson` | Binary JSON | `application/bson` | `.bson` | Binary |
| `avro` | Apache Avro Binary & OCF | `application/avro` | `.avro` | Binary |
| `parquet` | Apache Parquet Columnar | `application/x-parquet` | `.parquet` | Binary |
| `bencode` | BitTorrent Bencode | `application/x-bencode` | `.bencode`, `.torrent` | Binary |
