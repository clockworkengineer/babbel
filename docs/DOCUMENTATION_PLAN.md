# Babbel Documentation Master Plan

A comprehensive, actionable plan to expand, update, and align all technical documentation across the Babbel workspace with recent architectural additions—including first-class **Text File Support** (RFC 4180 CSV/TSV, sectioned INI/Properties/.env, JSON Lines/NDJSON, frontmatter extraction, `ILineReader`), the **`xml_lib`** rename, and memory compaction optimizations.

---

## 1. Executive Summary & Goals

### 1.1 Objectives
1. **Complete Coverage**: Ensure every feature, format engine, conversion pipeline, and public abstraction is documented with clear explanations and executable code examples.
2. **Link & Reference Integrity**: Fix all broken links (such as missing `CONTRIBUTING.md` and incorrect relative paths to `docs/ARCHITECTURE.md`).
3. **Architectural Synchronization**: Bring `docs/ARCHITECTURE.md` into 100% parity with the codebase (adding `ILineReader`, CSV, TSV, INI, JSON Lines, `i128` integer model, and memory compaction guarantees).
4. **Actionable Developer Guides**: Provide dedicated, in-depth guides for text processing, cross-format conversion matrix, and memory optimization.
5. **Verified Doctests**: Ensure all doc-comments contain valid Rust examples that pass `cargo test --workspace --doc --jobs 2`.

---

## 2. Current State Audit

| Document | Location | Status | Action Required |
| :--- | :--- | :--- | :--- |
| **Workspace README** | `README.md` | Outdated links & missing text features | Fix broken relative links (`ARCHITECTURE.md` $\rightarrow$ `docs/ARCHITECTURE.md`, missing `CONTRIBUTING.md`), add text formats to tables and conversion snippets. |
| **Architecture Guide** | `docs/ARCHITECTURE.md` | Missing Phase 1-5 text engines & `ILineReader` | Update 3-tier layering model diagram, add `ILineReader` to ISP table, update `Value` variant types (`Integer(i128)`), and add CSV/TSV/INI/JSONL to format mapping matrix. |
| **Facade README** | `crates/babbel/README.md` | Lacks text format APIs | Add CSV, TSV, INI, JSON Lines to features, prelude examples, and conversion pipeline table. |
| **Core README** | `crates/babbel_core/README.md`| Lacks text modules & `ILineReader` | Document `ILineReader`, `babbel_core::csv`, `babbel_core::ini`, and `babbel_core::text`. |
| **JSON README** | `crates/json/README.md` | Missing JSON Lines streaming | Document `json_lib::lines` (`JsonLinesReader`, `to_json_lines`, `parse_json_lines`). Fix broken reference to `EMBEDDING_GUIDE.md`. |
| **XML README** | `crates/xml/README.md` | Up-to-date with `xml_lib` | Minor polish: ensure links to `babbel_core` and `babbel` facade are consistent. |
| **YAML README** | `crates/yaml/README.md` | Up-to-date | Ensure consistency with cross-conversion matrix. |
| **Bencode README** | `crates/bencode/README.md` | Up-to-date | Ensure consistency with cross-conversion matrix. |
| **Contributing Guide** | `CONTRIBUTING.md` | **Missing** (broken link in `README.md`) | Create comprehensive guide covering SOLID standards, testing commands (`--jobs 2`), and PR workflow. |
| **Text Support Guide** | `docs/TEXT_SUPPORT_GUIDE.md` | **Missing** | Create dedicated guide covering CSV, TSV, INI, JSON Lines, and frontmatter. |
| **Conversion Matrix** | `docs/CONVERSION_MATRIX.md` | **Missing** | Create reference documentation detailing $O(N)$ conversions across all 8 supported formats. |
| **Memory & Performance**| `docs/BENCHMARKS_AND_MEMORY.md` | **Missing** | Document memory compaction sizes (NodeKind $\le$ 48B, Value $\le$ 32B) and zero-copy streaming guidelines. |

---

## 3. New Documents to Create

### 3.1 `docs/TEXT_SUPPORT_GUIDE.md` (Comprehensive Text Processing Guide)
- **Purpose**: Serve as the authoritative user manual for text-based file formats and line streaming in Babbel.
- **Sections**:
  1. **Overview & Philosophy**: Why text file support belongs in Babbel; zero-allocation streaming and RFC conformance.
  2. **Core Line Streaming (`ILineReader`)**:
     - Handling mixed line endings (`\r\n`, `\n`, `\r`) without reading entire files into memory.
     - Reading lines from `SliceSource`, `BufferSource`, `FileSource`.
     - Zero-copy line slicing with `read_line_slice()`.
     - Using `LineIter` as standard Rust `Iterator`.
  3. **Delimited Text Engine (CSV / TSV)**:
     - RFC 4180 compliance: double-quote escaping (`""`), multi-line quoted fields, commas inside quotes.
     - `CsvOptions` configuration (custom delimiters, quote characters, header toggles, type inference).
     - Delimiter auto-detection (`sniff_delimiter`) across `,`, `\t`, `;`, `|`.
     - Bidirectional `Value` mapping: header-based `Value::Object` rows vs. flat `Value::Array` rows.
     - `parse_csv`, `emit_csv`, `emit_csv_to`.
  4. **Configuration Text Engine (INI / Properties / .env)**:
     - Sectioned INI parsing: `[section]` headers mapping to nested `Value::Object`s.
     - Global/root properties before any section header.
     - Key-value delimiters (`=`, `:`) and comments (`#`, `;`, `!`).
     - Specialized profiles: `IniOptions::env()` for `.env` files, `IniOptions::properties()` for Java properties.
     - `parse_ini`, `emit_ini`, `emit_ini_to`.
  5. **JSON Lines / NDJSON Streaming**:
     - Streaming record-by-record processing using `JsonLinesReader`.
     - Comment stripping (`#`, `//`) and blank line suppression.
     - `parse_json_lines`, `to_json_lines`, and streaming to `IDestination`.
  6. **Document Frontmatter & Line Utilities**:
     - Extracting YAML (`---`) and TOML (`+++`) metadata blocks with `split_frontmatter`.
     - Indentation manipulation: `indent`, `dedent`, `trim_lines`, `line_count`.
  7. **Cookbook & Real-World Recipes**:
     - Streaming a 1GB CSV file to JSON Lines without OOM.
     - Parsing a Markdown blog post with YAML frontmatter.
     - Loading and mutating application `.env` configurations.

---

### 3.2 `docs/CONVERSION_MATRIX.md` (Universal Cross-Format Matrix)
- **Purpose**: Document the cross-format interoperability layer (`babbel::convert`).
- **Sections**:
  1. **The $O(N)$ Conversion Architecture**:
     - Why $O(N^2)$ point-to-point converters are an anti-pattern.
     - How `Value` acts as the universal canonical intermediary.
     - Direct streaming conversions vs. DOM-based conversions.
  2. **Comprehensive Format Matrix (64 Combinations)**:
     - Table mapping sources (JSON, YAML, XML, Bencode, CSV, TSV, INI, JSONL) to destinations.
     - Type preservation guarantees and semantic fallbacks (e.g. how binary byte arrays serialize into JSON base64 or CSV strings).
  3. **High-Level Conversion API Reference**:
     - Detailed signatures and examples for:
       - `json_to_yaml`, `yaml_to_json`
       - `json_to_xml`, `yaml_to_xml`, `bencode_to_xml`
       - `json_to_bencode`, `yaml_to_bencode`, `bencode_to_json`, `bencode_to_yaml`
       - `csv_to_json`, `json_to_csv`, `tsv_to_json`, `json_to_tsv`
       - `csv_to_yaml`, `yaml_to_csv`
       - `ini_to_json`, `json_to_ini`, `ini_to_yaml`, `yaml_to_ini`
       - `jsonlines_to_json`, `json_to_jsonlines`
       - `jsonlines_to_csv`, `csv_to_jsonlines`
  4. **Open-Ended Extension (OCP & DIP)**:
     - How to implement `FormatParser` and `FormatEmitter` for a custom format (e.g. TOML, MessagePack, CBOR) and plug it directly into `convert_text` and `convert_bytes`.

---

### 3.3 `CONTRIBUTING.md` (Contributor Guidelines & Standards)
- **Purpose**: Provide clear instructions for developers contributing to Babbel (resolving the dead link in root `README.md`).
- **Sections**:
  1. **Architecture Principles**: Adhering to SOLID, DRY, and clean layering (no format crates depending on each other; all depending on `babbel_core`).
  2. **Setting Up the Workspace**: Prerequisites, cloning, and cargo tools.
  3. **Building & Running Tests**:
     - Explaining the mandatory Windows flag `--jobs 2` (`cargo test --workspace --jobs 2`).
     - Running package-specific tests (`cargo test -p <crate> --jobs 2`).
     - Running doctests (`cargo test --workspace --doc --jobs 2`).
     - Running memory size checks (`cargo test -p babbel --test size_checks`).
  4. **Code Style & Guidelines**:
     - `no_std` + `alloc` compatibility in `babbel_core` and format crates.
     - Error handling with `BabbelError` and normalized `ErrorCode`.
     - Preserving existing comments and docstrings.
  5. **Pull Request & Commit Workflow**:
     - Conventional Commits (`feat:`, `fix:`, `refactor:`, `docs:`, `perf:`).
     - CI checks and requirements.

---

### 3.4 `docs/BENCHMARKS_AND_MEMORY.md` (Memory & Performance Specifications)
- **Purpose**: Document memory compacting achievements, struct size bounds, and zero-allocation techniques.
- **Sections**:
  1. **Memory Compacted Layouts**:
     - `Value`: 32 bytes (optimized discriminant and storage layout).
     - `json_lib::Node`: 56 bytes.
     - `xml_lib::NodeKind`: 48 bytes (compacted from 72 bytes via boxing).
     - `xml_lib::NodeData`: 88 bytes (compacted from 112 bytes).
     - `yaml_lib::Node`: 40 bytes.
     - `bencode_lib::Node`: 56 bytes.
  2. **Zero-Allocation Primitives**:
     - `itoa` and `dtoa` buffers for integer and float formatting.
     - `SliceSource::read_line_slice` returning `&'a str` without heap allocation.
     - Small-vector optimizations (`smallvec::SmallVec`, `arrayvec::ArrayVec`).
  3. **Streaming I/O Performance**:
     - Elimination of Windows file handle re-opening locks via in-memory tail tracking.
     - Buffered file I/O eliminating 1-byte system calls.

---

## 4. Existing Documents to Modify

### 4.1 Root `README.md`
- **Line 10**: Fix broken relative links:
  - Change `[Architecture Guide](ARCHITECTURE.md)` to `[Architecture Guide](docs/ARCHITECTURE.md)`.
  - Ensure `[Contributing Guide](CONTRIBUTING.md)` links to the new `CONTRIBUTING.md`.
  - Add links to `[Text Support Guide](docs/TEXT_SUPPORT_GUIDE.md)` and `[Conversion Matrix](docs/CONVERSION_MATRIX.md)`.
- **Section "Workspace Architecture" (Lines 18-26)**:
  - Update `babbel_core` description to highlight CSV, TSV, INI, and text streaming capabilities.
  - Update `json_lib` description to mention JSON Lines / NDJSON.
- **Section "Key Design Principles" (Lines 35-46)**:
  - Add `ILineReader` to the ISP trait listing.
- **Section "Code Examples"**:
  - Add snippets for CSV auto-detection, frontmatter extraction, and JSON Lines streaming.

---

### 4.2 `docs/ARCHITECTURE.md`
- **Section 1: 3-Tier Layering Model**:
  - Update Mermaid diagram:
    - Tier 0 (`babbel_core`): Add `babbel_core::csv`, `babbel_core::ini`, `babbel_core::text`, and `ILineReader`.
    - Tier 1: Add `json_lib::lines` (JSON Lines).
    - Tier 2 (`babbel::convert`): Add CSV, TSV, INI, JSON Lines pipelines.
  - Tier 0 Responsibilities: Document delimited tabular parsing, section-based INI parsing, and frontmatter handling.
- **Section 2: SOLID Principles (ISP Table, Lines 85-97)**:
  - Add row for `ILineReader`: forward line-by-line reading across CRLF/LF/CR (`read_line`, `read_line_into`, `lines`).
- **Section 3: Universal Data Model (Lines 105-134)**:
  - Fix type definition: `Integer(i128)` (was documented as `i64`).
  - Expand Format Mapping Matrix with columns for CSV/TSV, INI, and JSON Lines.
- **Section 4: Performance & Memory (Lines 137-151)**:
  - Add explicit mention of struct size bounds verified by `tests/size_checks.rs`.

---

### 4.3 `crates/babbel/README.md`
- **Features (Lines 10-23)**:
  - Add bullet points for CSV/TSV with type inference, section-based INI/.env, JSON Lines streaming, and frontmatter extraction.
  - Expand cross-conversion list to include all 8 formats.
- **Quickstart**:
  - Add text parsing example (`parse_csv`, `parse_ini`, `split_frontmatter`).
  - Add CSV $\leftrightarrow$ JSON and INI $\leftrightarrow$ JSON conversion examples.
- **Sub-Libraries table**:
  - Update `babbel_core` and `json_lib` rows.

---

### 4.4 `crates/babbel_core/README.md`
- **Features (Lines 10-33)**:
  - Add `ILineReader` to the streaming I/O list.
  - Add section for Delimited Text (CSV/TSV): RFC 4180 parsing, sniffing, and emission.
  - Add section for Configuration Text (INI/.properties/.env).
  - Add section for Text Utilities: frontmatter splitting and indentation manipulation.
- **Trait Architecture**:
  - Include `ILineReader` definition and inherent `SliceSource::read_line_slice` example.
- **Quickstart Code Examples**:
  - Add CSV parsing and roundtrip example.
  - Add INI and `.env` parsing example.
  - Add `split_frontmatter` example.

---

### 4.5 `crates/json/README.md`
- **Header & Features (Lines 3-38)**:
  - Add JSON Lines (`.jsonl` / `.ndjson`) streaming reader and writer.
- **Quickstart**:
  - Add a dedicated "JSON Lines / NDJSON Streaming" section demonstrating `JsonLinesReader`, `parse_json_lines`, and `to_json_lines`.
- **Line 89**:
  - Replace non-existent `EMBEDDING_GUIDE.md` link with clear inline guidance or link to root documentation.

---

## 5. In-Code Documentation & Doc-Comment Integrity

### 5.1 Crate Root Lib Comments
- [`crates/babbel/src/lib.rs`](crates/babbel/src/lib.rs):
  - Ensure module-level docstring introduces all 8 formats and text utilities.
  - Include doctests for `parse_csv`, `parse_ini`, and `split_frontmatter`.
- [`crates/babbel_core/src/lib.rs`](crates/babbel_core/src/lib.rs):
  - Update top-level docstring to list `csv`, `ini`, and `text` modules alongside `codec`, `error`, `io`, etc.
- [`crates/json/src/lib.rs`](crates/json/src/lib.rs):
  - Add doc comments for `pub mod lines` and re-exports.

### 5.2 Doctest Verification
- All newly added and modified code blocks in documentation and doc-comments must be verified using:
  ```bash
  cargo test --workspace --doc --jobs 2
  ```

---

## 6. Implementation Roadmap & Execution Order

```
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Fix Existing Broken Links & Root Documentation               │
│ - Create CONTRIBUTING.md                                               │
│ - Update README.md (fix links, add text formats & conversion snippets) │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Core Architecture & Interoperability Documentation            │
│ - Update docs/ARCHITECTURE.md (diagram, ISP table, Value i128, matrix) │
│ - Create docs/CONVERSION_MATRIX.md (full 64-combo reference)           │
│ - Create docs/BENCHMARKS_AND_MEMORY.md (size checks & perf rules)      │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Text Support Documentation                                    │
│ - Create docs/TEXT_SUPPORT_GUIDE.md (complete CSV/TSV/INI/JSONL guide) │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Crate-Level README Updates                                    │
│ - Update crates/babbel/README.md                                       │
│ - Update crates/babbel_core/README.md                                  │
│ - Update crates/json/README.md                                         │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Verification & Quality Assurance                              │
│ - Run markdown link check                                              │
│ - Run cargo test --workspace --doc --jobs 2                            │
│ - Verify cargo check --workspace                                       │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Acceptance Criteria

1. **Zero Broken Links**: All internal Markdown links (`[text](path.md)`) resolve to existing files.
2. **100% Feature Parity**: All newly added features (`ILineReader`, CSV, TSV, INI, JSON Lines, frontmatter, `xml_lib`) are thoroughly documented.
3. **Valid Executable Examples**: Every doctest in doc-comments compiles and passes.
4. **Architectural Accuracy**: `docs/ARCHITECTURE.md` accurately reflects the 3-tier layering model and current codebase semantics.
