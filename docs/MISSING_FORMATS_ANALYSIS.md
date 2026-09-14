# Babbel: Unsupported & Candidate Formats Analysis

This document evaluates prominent data serialization, configuration, and document formats currently not implemented in Babbel (v0.2.1), analyzing their industry adoption, ecosystem relevance, alignment with Babbel's universal [`Value`](../crates/babbel_core/src/model.rs) AST, and implementation feasibility.

---

## 1. Current Baseline (v0.2.1 Supported Formats)

Babbel natively supports **9 data formats** spanning hierarchical, tabular, configuration, and streaming formats:

| Format Family | Formats Supported | Primary Crate | File Extensions |
| :--- | :--- | :--- | :--- |
| **Hierarchical Documents** | JSON, YAML 1.2, W3C XML, TOML v1.1, Bencode | `babbel_json`, `babbel_yaml`, `babbel_xml`, `babbel_toml`, `babbel_bencode` | `.json`, `.yaml`, `.yml`, `.xml`, `.toml`, `.torrent`, `.bencode` |
| **Tabular Records** | RFC 4180 CSV, TSV | `babbel_core` | `.csv`, `.tsv` |
| **Configuration** | INI, Java `.properties`, `.env` | `babbel_core` | `.ini`, `.env`, `.properties` |
| **Streaming Records** | JSON Lines / NDJSON | `babbel_json` | `.jsonl`, `.ndjson` |

---

## 2. High-Priority Binary Interchange Formats

These formats share a 1:1 conceptual mapping with Babbel's universal `Value` AST (scalars, maps, arrays, binary buffers) and represent high-demand candidates.

### 2.1 MessagePack (`.msgpack`, `.mp`) — [IMPLEMENTED in `babbel_msgpack`]
- **Status**: **Implemented** in `crates/msgpack` (`babbel_msgpack`), registered in `babbel::FormatRegistry` and `babbel::convert`.
- **MIME Type**: `application/msgpack`
- **Adoption**: Redis, Fluentd, Neovim RPC, WebSockets, gaming backends, microservices.
- **Value Proposition**:
  - Extremely compact binary encoding of JSON-like structures.
  - Zero schema requirements; self-describing type tags.
  - Native binary byte-array (`bin`) support matching `Value::Bytes`.
  - Up to 64-bit and 128-bit integer support matching `Value::Integer(i128)`.
- **Implementation**: Pure-Rust `babbel_msgpack::MsgPackEngine` implementing `FormatEngine`, zero runtime dependencies, DoS memory and recursion safeguards.

### 2.2 CBOR (Concise Binary Object Representation - RFC 8949) — [IMPLEMENTED in `babbel_cbor`]
- **Status**: **Implemented** in `crates/cbor` (`babbel_cbor`), registered in `babbel::FormatRegistry` and `babbel::convert`.
- **MIME Type**: `application/cbor`
- **Adoption**: IETF standard, WebAuthn / FIDO2 tokens, COSE cryptography, IoT / CoAP networks, Cardano blockchain.
- **Value Proposition**:
  - The official Internet standard binary format.
  - Extensible semantic tagging system (e.g. timestamps, UUIDs, BigNums).
  - Can be parsed in $O(1)$ memory on embedded systems (`no_std`).
- **Implementation**: Pure-Rust `babbel_cbor::CborEngine` implementing `FormatEngine`, zero runtime dependencies, full RFC 8949 major types 0–7, definite and indefinite length streaming, DoS memory and recursion bounds.

### 2.3 BSON (Binary JSON) — [IMPLEMENTED in `babbel_bson`]
- **Status**: **Implemented** in `crates/bson` (`babbel_bson`), registered in `babbel::FormatRegistry` and `babbel::convert`.
- **MIME Type**: `application/bson`
- **Adoption**: MongoDB, document database exports, NoSQL data storage.
- **Value Proposition**:
  - Length-prefixed binary encoding of JSON documents.
  - Native support for binary data, embedded documents, and typed integers/floats.
- **Implementation**: Pure-Rust `babbel_bson::BsonEngine` implementing `FormatEngine`, zero runtime dependencies, full bsonspec.org types, DoS memory and recursion safeguards.

---

## 3. Human-Centric Configuration Formats

### 3.1 JSON5 / JSONC (`.json5`, `.jsonc`)
- **Adoption**: Visual Studio Code configuration (`settings.json`, `launch.json`), TypeScript compiler (`tsconfig.json`), Babel configuration.
- **Value Proposition**:
  - Permits single-line (`//`) and multiline (`/* */`) comments.
  - Allows trailing commas in arrays and objects.
  - Permits single-quoted strings (`'str'`) and unquoted identifier keys.
- **Current State in Babbel**: Babbel currently offers a regex/lexer utility `strip_comments` in `babbel_json::parser::json5`, but lacks a full standalone `Json5Engine`.

### 3.2 RON (Rusty Object Notation - `.ron`)
- **Adoption**: Rust ecosystem, game engines (Bevy, Amethyst, Veloren), Rust CLI configurations.
- **Value Proposition**:
  - Native syntax matching Rust literals (`Struct(field: "value")`, `Enum::Variant`).
  - Supports trailing commas, comments, and named or tuple variants.
- **Implementation Complexity**: Medium. Matches Rust data models naturally.

### 3.3 KDL Document Language (`.kdl`)
- **Adoption**: Terminal multiplexers (Zellij), CLI configuration, modern system tooling.
- **Value Proposition**:
  - Node-and-attribute syntax designed specifically for human-authored configuration documents.
  - Type annotations and child nodes.
- **Implementation Complexity**: Medium. Requires a node-attribute AST mapping to `Value::Object`.

### 3.4 HCL (HashiCorp Configuration Language - `.hcl`, `.tf`)
- **Adoption**: Terraform, Nomad, Consul, Packer, cloud infrastructure-as-code.
- **Trade-off**: Complex grammar with built-in interpolation functions; typically better handled by dedicated grammar runtimes.

---

## 4. Analytical, Columnar & Big Data Formats

### 4.1 Apache Parquet (`.parquet`)
- **Adoption**: DuckDB, Polars, Apache Spark, Snowflake, Amazon Athena, analytical data lakes.
- **Value Proposition**:
  - Columnar storage with dictionary encoding and Snappy/ZSTD compression.
- **Trade-off**: Binary columnar layout designed for analytical batch processing rather than single-document conversion pipelines.

### 4.2 Apache Avro (`.avro`)
- **Adoption**: Apache Kafka, event streaming pipelines, Hadoop ecosystem.
- **Value Proposition**:
  - Row-oriented binary serialization with embedded JSON schema.
- **Trade-off**: Requires schema negotiation or schema parsing to deserialize binary rows.

---

## 5. Summary Evaluation Matrix

| Format | Category | Ecosystem Demand | AST Mapping Alignment | Status / Priority |
| :--- | :--- | :--- | :--- | :--- |
| **MessagePack** (`.msgpack`) | Binary Interchange | Very High | 100% (Direct `Value` mapping) | **IMPLEMENTED (`babbel_msgpack`)** |
| **CBOR** (`.cbor`) | Binary Interchange / IoT | High | 95% (Direct + Tag mapping) | **IMPLEMENTED (`babbel_cbor`)** |
| **JSON5 / JSONC** (`.json5`) | Human Configuration | High | 100% (Maps directly to JSON) | **Priority 1** (Next) |
| **RON** (`.ron`) | Rust Configuration | Medium | 90% (Rust literal mapping) | **Priority 2** |
| **KDL** (`.kdl`) | Modern CLI Config | Medium | 85% (Node/attribute mapping) | Priority 3 |
| **BSON** (`.bson`) | Database Storage | Medium | 90% (JSON-extended mapping) | **IMPLEMENTED (`babbel_bson`)** |
| **Apache Parquet** (`.parquet`)| Columnar Big Data | High (Data Science) | 70% (Batch tabular only) | Future / Specialized |
| **Apache Avro** (`.avro`) | Event Streaming | High (Kafka) | 75% (Schema-bound) | Future / Specialized |

---

## 6. Architecture Integration Blueprint

Thanks to Babbel's Open-Closed Principle (OCP) architecture, integrating any new format (e.g. MessagePack) requires **only two steps**:

1. **Implement `FormatEngine`**:
   ```rust
   use babbel_core::{BabbelError, FormatEngine, FormatOptions, IDestination, ISource, Value};

   #[derive(Debug, Default, Clone, Copy)]
   pub struct MsgPackEngine;

   impl FormatEngine for MsgPackEngine {
       fn format_id(&self) -> &'static str { "msgpack" }
       fn mime_type(&self) -> &'static str { "application/msgpack" }
       fn file_extensions(&self) -> &'static [&'static str] { &["msgpack", "mp"] }

       fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
           // Parse MessagePack byte stream into universal Value AST
           todo!()
       }

       fn serialize(&self, value: &Value, dest: &mut dyn IDestination, options: &FormatOptions) -> Result<(), BabbelError> {
           // Serialize universal Value AST into MessagePack bytes
           todo!()
       }
   }
   ```

2. **Instant Matrix Integration**:
   Registering the engine in `FormatRegistry` automatically grants $O(N)$ bidirectional conversion between the new format and all 9 existing formats.
