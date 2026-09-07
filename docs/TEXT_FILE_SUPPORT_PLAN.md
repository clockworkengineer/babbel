# Text File Support - Architectural Analysis & Feature Proposal

This document presents a comprehensive analysis of the existing Babbel codebase and proposes a concrete, modular architecture for introducing first-class **Text File Support** across the Babbel workspace.

---

## 1. Analysis of Existing Codebase & Text Capabilities

### 1.1 Current Architecture & Scope
The Babbel ecosystem currently supports four major hierarchical document formats:
1. **JSON (`json_lib`)**: RFC 6901 Pointer, RFC 7396 Merge Patch, JSON5 comment stripping.
2. **YAML (`yaml_lib`)**: Full YAML 1.2 specification, anchors/aliases, multiline block scalars, custom tags.
3. **XML (`xml_lib`)**: W3C DOM, C14N 1.0/1.1 Canonical XML, DTD validation, XSD schema, XPath 1.0.
4. **Bencode (`bencode_lib`)**: BitTorrent protocol parser, zero-copy borrowed slices, iterative parser.

All four engines interface with `babbel_core` via:
- **Streaming Input (`ISource` / `ICharStream`)**: Pull-based character streaming (`next()`, `current()`, `more()`).
- **Streaming Output (`IDestination`)**: Character/byte destinations (`add_byte()`, `add_bytes()`).
- **Universal AST (`babbel_core::Value`)**: Canonical intermediate representation (`Null`, `Bool`, `Integer`, `Float`, `String`, `Bytes`, `Array`, `Object`).
- **Cross-Format Conversion (`babbel::convert`)**: $O(N)$ translation between formats through `FormatParser` and `FormatEmitter`.

### 1.2 Identified Gaps in Text File Support
Despite Babbel's rich support for structured trees, working with everyday **text files** currently has substantial gaps:

1. **No Delimited / Tabular Text Support (CSV / TSV)**:
   - CSV and TSV files are the most common text data exchange formats in data engineering, science, and spreadsheeting.
   - Users cannot currently convert a JSON or YAML array of records into a `.csv` or `.tsv` file, nor ingest tabular text files into `babbel_core::Value`.
2. **No Configuration / Key-Value Text Support (INI / `.properties` / `.env`)**:
   - Systems, game engines, and DevOps rely heavily on flat or sectioned text configs (`.ini`, `.properties`, `.cfg`, `.env`).
   - `yaml_lib` has an internal one-way serializer `to_toml`, but there is no dedicated key-value text engine for INI/properties files.
3. **No Line-Delimited JSON (NDJSON / JSONL) Text Streams**:
   - Modern log pipelines, LLM fine-tuning datasets, and large-scale data feeds use `.jsonl` / `.ndjson` (one JSON value per line).
   - Currently, `json_lib` requires an entire JSON file to have a single root element (`{...}` or `[...]`). Attempting to parse a `.jsonl` file fails with a syntax error at the second line.
4. **Lack of Streaming Line Reader (`ILineReader`) in `babbel_core`**:
   - `ICharStream` only provides character-by-character consumption (`c: Option<char>`).
   - There is no streaming `read_line()` or `lines()` iterator in `babbel_core::io::traits`. Callers must either read the entire file into a giant heap string via `babbel_core::file::read_file_to_string` or write custom loop logic to buffer characters until `\n`.
5. **No Document Text with Metadata (Markdown + Frontmatter)**:
   - Documentation, static sites, and blogging platforms store text files with YAML, TOML, or JSON frontmatter (`---` ... `---`) followed by a Markdown/plaintext body.
   - Babbel cannot currently parse such hybrid text files without the caller manually splitting the file.

---

## 2. Proposed Text File Features

We propose a cohesive, phased addition of **Text File Support** organized into four functional pillars:

```
+-------------------------------------------------------------------------------+
|                            Babbel Text File Support                           |
+-------------------------------------------------------------------------------+
|                                                                               |
|  1. Tabular Text Engine         2. Config Text Engine      3. Stream Engine   |
|     (CSV / TSV)                    (INI / Properties)         (JSON Lines)    |
|     * RFC 4180 compliance          * [sections] & keys        * .jsonl/.ndjson|
|     * Delimiter auto-detection     * Comments (#, ;)          * Line streaming|
|     * Header & row mapping         * Type inference           * Record iter   |
|                                                                               |
|  4. Core Text Streaming Utilities (babbel_core::io::text)                     |
|     * ILineReader & LineIterator (mixed \r\n, \n, \r handling)               |
|     * Chunked text streaming & sliding windows                                |
|     * Frontmatter splitter (YAML/TOML/JSON metadata + body)                   |
|                                                                               |
|  5. Universal Conversion Integration (babbel::convert)                        |
|     * json_to_csv, csv_to_json, yaml_to_csv, ini_to_json, json_to_jsonl       |
+-------------------------------------------------------------------------------+
```

---

### Feature 1: Tabular Text Support (CSV / TSV)

#### Objective
Provide high-performance, RFC 4180-compliant CSV and TSV parsing and serialization that bi-directionally maps to `babbel_core::Value` and streaming destinations.

#### Key Capabilities
- **Format Dialects**:
  - Comma-Separated (`.csv`, delimiter `,`)
  - Tab-Separated (`.tsv`, delimiter `\t`)
  - Semicolon-Separated (European `.csv`, delimiter `;`)
  - Pipe-Separated (`.psv`, delimiter `|`)
  - **Auto-Detection**: Sniffer that inspects the first $N$ lines to deduce the delimiter.
- **Header Mapping & Row Types**:
  - **With Headers (Default)**: Maps rows to `Value::Object(vec![(header_name, field_value), ...])`. Entire file maps to `Value::Array(vec![row1, row2, ...])`.
  - **Without Headers**: Maps rows to `Value::Array(vec![field1, field2, ...])`.
- **RFC 4180 Quoting & Escaping**:
  - Handles multiline values enclosed in double quotes (`"line 1\nline 2"`).
  - Handles escaped quotes (`"He said, ""Hello!"""`).
- **Automatic Type Inference**:
  - Strings matching integers $\to$ `Value::Integer`.
  - Strings matching floats $\to$ `Value::Float`.
  - `"true"` / `"false"` $\to$ `Value::Bool`.
  - Empty fields $\to$ `Value::Null` (or `Value::String("")` depending on config).

#### Proposed API Blueprint
```rust
pub struct CsvOptions {
    pub delimiter: u8,            // b',', b'\t', b';', etc.
    pub quote_char: u8,           // b'"'
    pub escape_char: Option<u8>,  // b'\\' or quote doubling
    pub has_headers: bool,        // true by default
    pub infer_types: bool,        // parse numbers and booleans
    pub trim_whitespace: bool,    // trim leading/trailing spaces
}

pub fn parse_csv(input: &str) -> Result<Value, CsvError>;
pub fn parse_csv_with_options(input: &str, options: &CsvOptions) -> Result<Value, CsvError>;
pub fn parse_tsv(input: &str) -> Result<Value, CsvError>;

pub fn to_csv(value: &Value, dest: &mut dyn IDestination) -> Result<(), CsvError>;
pub fn to_tsv(value: &Value, dest: &mut dyn IDestination) -> Result<(), CsvError>;
```

---

### Feature 2: Configuration Text Support (INI & Properties)

#### Objective
Enable parsing and serialization of sectioned key-value text files (`.ini`, `.properties`, `.env`, `.conf`) into hierarchical `Value` maps.

#### Key Capabilities
- **Sections**: Group keys under `[section]` headers:
  ```ini
  [server]
  port = 8080
  host = 0.0.0.0

  [database]
  url = postgres://localhost/db
  pool_size = 10
  ```
  Maps to:
  ```rust
  Value::Object(vec![
      ("server".into(), Value::Object(vec![
          ("port".into(), Value::Integer(8080)),
          ("host".into(), Value::String("0.0.0.0".into())),
      ])),
      ("database".into(), Value::Object(vec![
          ("url".into(), Value::String("postgres://localhost/db".into())),
          ("pool_size".into(), Value::Integer(10)),
      ])),
  ])
  ```
- **Global / Top-level Keys**: Keys preceding any section header map to root-level keys.
- **Comments**: Supports `;` (INI) and `#` (Properties / Shell / .env).
- **Key-Value Delimiters**: Supports `=` and `:` as assignment operators.
- **Boolean & Number Coercion**: Automatically coerces `true`, `false`, `yes`, `no`, `1`, `0`, and numbers.

#### Proposed API Blueprint
```rust
pub struct IniOptions {
    pub allow_duplicate_keys: bool,
    pub comment_chars: &'static [char], // [';', '#']
    pub assign_chars: &'static [char],  // ['=', ':']
    pub infer_types: bool,
}

pub fn parse_ini(input: &str) -> Result<Value, IniError>;
pub fn parse_ini_source(source: &mut dyn ISource) -> Result<Value, IniError>;
pub fn to_ini(value: &Value, dest: &mut dyn IDestination) -> Result<(), IniError>;
```

---

### Feature 3: Line-Delimited JSON (JSON Lines / NDJSON)

#### Objective
Provide high-throughput, memory-bounded streaming for text files where each line is an independent JSON document (`.jsonl`, `.ndjson`).

#### Key Capabilities
- **Streaming Iterator**: Yields parsed `Value` records one by one as lines are read, allowing multi-gigabyte log files to be processed with $O(1)$ memory.
- **Batch Serialization**: Writes arrays or iterators of `Value` as newline-delimited JSON strings into any `IDestination`.
- **Fault Tolerance**: Option to skip malformed lines or collect error diagnostics with line numbers.

#### Proposed API Blueprint
```rust
pub struct JsonLinesReader<'a> {
    source: &'a mut dyn ISource,
}

impl<'a> Iterator for JsonLinesReader<'a> {
    type Item = Result<Value, json_lib::JsonError>;
    fn next(&mut self) -> Option<Self::Item> { ... }
}

pub fn parse_json_lines(input: &str) -> Result<Vec<Value>, json_lib::JsonError>;
pub fn to_json_lines<'a, I>(records: I, dest: &mut dyn IDestination) -> Result<(), json_lib::JsonError>
where
    I: IntoIterator<Item = &'a Value>;
```

---

### Feature 4: Core Text Streaming Primitives (`babbel_core::io::text`)

#### Objective
Add dedicated, zero-copy line-reading and text processing abstractions directly to `babbel_core` so that all crates and consumers can efficiently process text streams.

#### Key Capabilities
- **`ILineReader` Capability Trait**:
  ```rust
  pub trait ILineReader: ICharStream {
      /// Reads the next line up to '\n' or '\r\n', returning it without the newline terminator.
      fn read_line(&mut self) -> Option<String>;
      
      /// Reads the next line into an existing reusable buffer to avoid allocations.
      fn read_line_into(&mut self, buf: &mut String) -> bool;

      /// Returns an iterator yielding lines sequentially.
      fn lines(&mut self) -> LineIter<'_, Self> where Self: Sized;
  }
  ```
- **Unified Implementation**:
  - Implement `ILineReader` for `BufferSource`, `FileSource`, `SliceSource`, and `StringSource`.
  - Correctly normalizes all three line endings: Windows (`\r\n`), Unix (`\n`), and Classic Mac (`\r`).
- **Markdown Frontmatter Extractor**:
  - A lightweight, zero-copy splitter:
    ```rust
    pub struct DocumentWithFrontmatter<'a> {
        pub format: FrontmatterFormat, // Yaml, Toml, Json
        pub frontmatter_raw: &'a str,
        pub body: &'a str,
    }
    
    pub fn split_frontmatter(text: &str) -> Option<DocumentWithFrontmatter<'_>>;
    ```

---

### Feature 5: Universal Conversion Matrix Integration (`babbel::convert`)

#### Objective
Expand `babbel::convert::Format` to include the new text-based formats and add direct, high-level conversion helpers.

#### Extended Matrix
```rust
pub enum Format {
    Json,
    Yaml,
    Xml,
    Bencode,
    Csv,        // NEW
    Tsv,        // NEW
    Ini,        // NEW
    JsonLines,  // NEW
}
```

#### New High-Level Convenience Conversions
- `json_to_csv(json: &str) -> Result<String, String>`
- `csv_to_json(csv: &str) -> Result<String, String>`
- `csv_to_yaml(csv: &str) -> Result<String, String>`
- `yaml_to_csv(yaml: &str) -> Result<String, String>`
- `ini_to_json(ini: &str) -> Result<String, String>`
- `json_to_ini(json: &str) -> Result<String, String>`
- `json_to_json_lines(json: &str) -> Result<String, String>`
- `json_lines_to_json(jsonl: &str) -> Result<String, String>`

---

## 3. Phased Implementation Roadmap

### Phase 1: `babbel_core` Foundation
- Add `ILineReader` trait and `read_line_into` methods to `crates/babbel_core/src/io/traits.rs`.
- Implement `ILineReader` on `FileSource`, `BufferSource`, `SliceSource`, and `StringSource`.
- Add frontmatter parser utility in `crates/babbel_core/src/text.rs`.
- Unit test coverage for line reading with mixed `\r\n`, `\n`, `\r`.

### Phase 2: JSON Lines Streaming
- Add `json_lib::lines` module to `crates/json`.
- Implement `JsonLinesReader` and `to_json_lines`.
- Add round-trip tests and memory benchmarks.

### Phase 3: Delimited Text Module (CSV & TSV)
- Implement RFC 4180 CSV/TSV parser and emitter in `crates/babbel_core::csv` (or separate `csv_lib`).
- Implement delimiter auto-detection and quote escaping.
- Support `Value::Array` of `Value::Object` (header mode) and `Value::Array` of `Value::Array` (raw row mode).

### Phase 4: Configuration Text Module (INI & Properties)
- Implement INI/Properties parser and emitter.
- Section nesting and top-level property support.
- Comment stripping (`;`, `#`) and type inference.

### Phase 5: Facade & Conversion Integration
- Add feature flags to `crates/babbel/Cargo.toml` (`csv`, `ini`).
- Update `crates/babbel/src/convert.rs` with `Format::Csv`, `Format::Tsv`, `Format::Ini`, `Format::JsonLines`.
- Implement conversion pairs (`json_to_csv`, `csv_to_json`, `ini_to_yaml`, etc.).
- Update documentation and create comprehensive examples.
