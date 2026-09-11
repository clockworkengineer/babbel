# babbel_core

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)

Foundational architectural kernel for the **Babbel** multi-format serialization ecosystem. `babbel_core` provides unified streaming I/O abstractions adhering strictly to **SOLID** principles, the universal `FormatEngine` and `FormatRegistry` architecture, universal `Value` AST, RFC 4180 CSV/TSV, sectioned INI/.env, frontmatter processing, Unicode BOM detection, zero-allocation numeric formatting, diagnostic error reporting, and string escaping.

---

## Features

- **SOLID Streaming I/O (`babbel_core::io`)**:
  - Segregated capability traits: [`ILineReader`](src/io/traits.rs), `IByteStream`, `IByteWriter`, `ICharStream`, `IRewindable`, `IPositionAware`, `ILocationAware`, `IClearable`, `ITailInspectable`, `IFlushable`, `IIndentationAware`, `IStatefulStream`.
  - Unified input sources: `BufferSource`, `FileSource` (with 64 MB DoS guard), `SliceSource`, `StringSource`, `ReaderSource` (generic `Read` streaming with `BufReader`).
  - Unified output destinations: `Buffer`, `FileDestination`, `StringDestination`, `SliceDestination`, `ArrayVecDestination`.
  - Stream state snapshots: `SaveState` and `IStatefulStream` for transactional backtracking.
  - Line-by-line reading across mixed CRLF, LF, and CR newlines.
  - Zero-allocation line slicing (`SliceSource::read_line_slice()`).
  - Safe in-memory tail tracking (`last()`) without file system re-opening.
  - Full Unicode scalar decoding preventing multi-byte UTF-8 corruption.
- **Format Engine & Extensibility (`babbel_core::engine` & `babbel_core::codec`)**:
  - [`FormatEngine`](src/engine.rs) trait unifying format ID, MIME types, file extensions, parsing, and serialization.
  - [`FormatRegistry`](src/engine.rs) for dynamic format discovery and lookup.
  - Static resolution helpers: `find_engine`, `find_engine_by_mime`, `find_engine_by_extension`.
  - Codec traits: `FormatParser`, `FormatEmitter`, `FormatCodec`.
  - Visitor pattern: `ValueVisitor` and `NodeVisitor` with default no-op methods.
- **Embedded & Zero-Allocation Primitives (`babbel_core::embedded`)**:
  - Stack allocation: `StackBuffer<const N>`, `MemoryTracker`, `EmbeddedLimits`, `CompactError` (8 bytes).
  - Zero-allocation destinations: `SliceDestination<'a>` and `ArrayVecDestination<const N>`.
- **Universal Data Model (`babbel_core::model`)**:
  - `Value` AST (`Null`, `Bool`, `Integer(i128)`, `Float`, `String`, `Array`, `Object`, `Bytes`) compacted to **32 bytes** (half a cache line).
- **Tabular Text Engine (`babbel_core::csv`)**:
  - RFC 4180 CSV and TSV parsing and serialization.
  - Delimiter auto-detection (`sniff_delimiter`) across `,`, `\t`, `;`, `|`.
  - Multi-line quoted fields, double-quote escaping (`""`), and automatic scalar type inference.
- **Configuration Text Engine (`babbel_core::ini`)**:
  - Sectioned INI (`[section]`), Java `.properties`, and `.env` parsing and serialization.
  - Comments (`#`, `;`, `!`), delimiters (`=`, `:`), and global root keys.
- **Document Frontmatter & Text Utilities (`babbel_core::text`)**:
  - Frontmatter splitter (`split_frontmatter`) supporting YAML (`---`) and TOML (`+++`).
  - Indentation manipulation: `indent`, `dedent`, `trim_lines`, `line_count`.
- **Unicode & Text Engine (`babbel_core::encoding` & `babbel_core::escape`)**:
  - Automatic BOM detection for UTF-8, UTF-16 LE/BE, and UTF-32 LE/BE.
  - Cross-platform newline normalization (`\r\n` / `\r` $\rightarrow$ `\n`).
  - Standardized escaping for JSON, XML, and YAML.
- **Zero-Allocation Numeric Formatting (`babbel_core::num`)**:
  - Fast integer formatting via `itoa` and float formatting via `dtoa`.
- **Diagnostic Error Reporting (`babbel_core::error`)**:
  - Structured `BabbelError`, `ErrorCode`, `Span`, and formatted context snippets.
- **`no_std` Support**:
  - Compiles cleanly in embedded and bare-metal environments with the `alloc` feature.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
babbel_core = "0.1.2"
```

Or as a workspace path dependency:

```toml
[dependencies]
babbel_core = { path = "crates/babbel_core" }
```

### Feature Flags

| Feature | Default | Description |
| :--- | :--- | :--- |
| `std` | **Yes** | Enables standard library support and OS integration. |
| `alloc` | **Yes** (via `std`) | Enables heap allocation primitives (`String`, `Vec`) for `no_std`. |
| `file-io` | **Yes** | Enables `FileSource`, `FileDestination`, and disk file utilities. |

---

## Quickstart & Code Examples

### 1. Line-by-Line Text Streaming (`ILineReader`)

```rust
use babbel_core::io::{ILineReader, SliceSource};

let text = "alpha\r\nbeta\ngamma\rdelta";
let mut source = SliceSource::new(text);

while let Some(line) = source.read_line() {
    println!("Line: {}", line);
}

// Zero-copy borrowed slice iteration
let mut source2 = SliceSource::new(text);
while let Some(slice) = source2.read_line_slice() {
    println!("Slice: {}", slice);
}
```

### 2. Format Engine Architecture

```rust
use babbel_core::{FormatEngine, FormatOptions, FormatRegistry, IDestination, Value, BabbelError};

pub struct CustomEngine;

impl FormatEngine for CustomEngine {
    fn format_id(&self) -> &'static str { "custom" }
    fn mime_types(&self) -> &'static [&'static str] { &["application/x-custom"] }
    fn file_extensions(&self) -> &'static [&'static str] { &["custom"] }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        Ok(Value::String(input.to_string()))
    }

    fn serialize(&self, value: &Value, dest: &mut dyn IDestination, _options: &FormatOptions) -> Result<(), BabbelError> {
        dest.add_bytes(value.as_str().unwrap_or(""));
        Ok(())
    }
}
```

### 3. Delimited Text (CSV / TSV)

```rust
use babbel_core::csv::{parse_csv, emit_csv, sniff_delimiter, CsvOptions};

// 1. Sniff delimiter
let data = "col1\tcol2\nval1\tval2\n";
assert_eq!(sniff_delimiter(data), '\t');

// 2. Parse TSV
let val = parse_csv(data, &CsvOptions::tsv()).unwrap();

// 3. Emit CSV
let csv_out = emit_csv(&val, &CsvOptions::default()).unwrap();
println!("{}", csv_out);
```

### 4. Configuration Text (INI / .env)

```rust
use babbel_core::ini::{parse_ini, emit_ini, IniOptions};

// Parse INI with sections
let ini_text = "[server]\nhost = 127.0.0.1\nport = 8080\n";
let val = parse_ini(ini_text, &IniOptions::default()).unwrap();

// Parse .env
let env_text = "PORT=3000\nDATABASE_URL=sqlite://data.db\n";
let env_val = parse_ini(env_text, &IniOptions::env()).unwrap();
```

---

## Universal `Value` Model

The `Value` enum enables lossless structural representation across format boundaries:

```rust
use babbel_core::model::Value;

let node = Value::Object(vec![
    ("name".to_string(), Value::String("Babbel".to_string())),
    ("version".to_string(), Value::Integer(1)),
    ("enabled".to_string(), Value::Bool(true)),
]);

assert!(matches!(node, Value::Object(_)));
assert_eq!(core::mem::size_of::<Value>(), 32);
```

---

## Documentation

See the [Documentation Hub](../../docs/README.md) for full workspace guides:
- [Architecture Guide](../../docs/ARCHITECTURE.md)
- [SOLID Architecture Whitepaper](../../docs/SOLID_ARCHITECTURE_GUIDE.md)
- [Format Engine Plugin Guide](../../docs/FORMAT_ENGINE_PLUGIN_GUIDE.md)
- [Embedded Systems Guide](../../docs/EMBEDDED_GUIDE.md)
- [Conversion Matrix](../../docs/CONVERSION_MATRIX.md)
- [Development Guide](../../docs/DEVELOPMENT_GUIDE.md)
- [Contributing Guidelines](../../docs/CONTRIBUTING.md)

---

## License

Licensed under the [MIT License](../../LICENSE).
