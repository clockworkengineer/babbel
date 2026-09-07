# Babbel Text File Support Guide

Babbel provides high-performance, specification-compliant text processing engines covering:
- **Core Line-by-Line Streaming** (`ILineReader`, `LineIter`)
- **Delimited Tabular Data** (RFC 4180 CSV / TSV with delimiter auto-detection)
- **Configuration Formats** (Section-based INI, Java `.properties`, and `.env`)
- **Stream-Oriented Data** (Line-delimited JSON / NDJSON / `.jsonl`)
- **Document Metadata** (Frontmatter extraction for YAML and TOML, text indentation manipulation)

---

## 1. Core Line Streaming (`ILineReader`)

Traditional line reading with `std::io::BufRead::lines` often allocates a new string per line and may choke on legacy Mac (`\r`) or mixed Windows (`\r\n`) / Unix (`\n`) newlines.

Babbel's [`ILineReader`](../crates/babbel_core/src/io/traits.rs) is an Interface Segregation Principle (ISP) compliant trait that standardizes line reading across all stream sources.

### Key Capabilities
- **Universal Newline Normalization**: Seamlessly terminates lines on `\n`, `\r\n`, or lone `\r`.
- **In-Place Buffer Reuse**: `read_line_into(&mut String)` reuses existing allocations to eliminate heap thrashing in tight loops.
- **Zero-Allocation Slicing**: `SliceSource::read_line_slice()` returns `Option<&'a str>` directly pointing into memory.

### Basic Usage
```rust
use babbel::core::io::{ILineReader, SliceSource};

let input = "first line\r\nsecond line\nthird line\rfourth line";
let mut source = SliceSource::new(input);

// 1. Owned String iteration
while let Some(line) = source.read_line() {
    println!("Read line: {}", line);
}

// 2. Iterator pattern
let mut source2 = SliceSource::new(input);
for line in source2.lines() {
    println!("Iter line: {}", line);
}

// 3. Zero-allocation borrowed slices
let mut source3 = SliceSource::new(input);
while let Some(line_slice) = source3.read_line_slice() {
    println!("Zero-copy slice: {}", line_slice);
}
```

---

## 2. Tabular Text Engine (CSV / TSV)

Babbel's CSV engine ([`babbel_core::csv`](../crates/babbel_core/src/csv.rs)) conforms strictly to **RFC 4180** while providing developer-friendly conveniences like automatic type inference and delimiter sniffing.

### Features
- **RFC 4180 Escaping**: Embedded quotes escaped as doubled quotes (`""`), commas and newlines preserved within quoted fields.
- **Header Mapping**: When `has_header = true`, rows parse into `Value::Object(Vec<(header, cell)>)`; when `false`, into `Value::Array(Vec<cell>)`.
- **Type Inference**: Automatically parses `true`/`false`, `i128` integers, `f64` floats, and `null`/`~`/empty values.
- **Delimiter Sniffing**: Inspects input samples to automatically distinguish `,`, `\t`, `;`, and `|`.

### Parsing CSV
```rust
use babbel::core::{parse_csv, CsvOptions};

let csv_data = "id,name,salary,active\n1,Alice,125000,true\n2,Bob,95000.5,false\n";
let value = parse_csv(csv_data, &CsvOptions::default())?;

let rows = value.as_array().unwrap();
assert_eq!(rows.len(), 2);
```

### Parsing TSV
```rust
use babbel::core::{parse_csv, CsvOptions};

let tsv_data = "id\tname\n1\tAlice\n2\tBob\n";
let value = parse_csv(tsv_data, &CsvOptions::tsv())?;
```

### Delimiter Auto-Detection
```rust
use babbel::core::sniff_delimiter;

assert_eq!(sniff_delimiter("col1,col2\n1,2\n"), ',');
assert_eq!(sniff_delimiter("col1\tcol2\n1\t2\n"), '\t');
assert_eq!(sniff_delimiter("col1;col2\n1;2\n"), ';');
assert_eq!(sniff_delimiter("col1|col2\n1|2\n"), '|');
```

### Serializing to CSV
```rust
use babbel::core::{emit_csv, CsvOptions, Value};

let records = Value::Array(vec![
    Value::Object(vec![
        ("name".to_string(), Value::String("Alice".into())),
        ("score".to_string(), Value::Integer(95)),
    ]),
    Value::Object(vec![
        ("name".to_string(), Value::String("Bob".into())),
        ("score".to_string(), Value::Integer(88)),
    ]),
]);

let csv_output = emit_csv(&records, &CsvOptions::default())?;
println!("{}", csv_output);
```

---

## 3. Configuration Text Engine (INI / Properties / .env)

The configuration engine ([`babbel_core::ini`](../crates/babbel_core/src/ini.rs)) parses and emits structured configuration files.

### Features
- **Sections**: Maps `[section]` headers to nested `Value::Object`s.
- **Global Keys**: Keys declared before any section header are preserved at root level.
- **Comments**: Supports line comments beginning with `#`, `;`, or `!`.
- **Delimiters**: Supports `=` and `:` key-value separators.
- **Type Inference**: Parses booleans (`true`/`false`, `yes`/`no`, `on`/`off`), numbers, and nulls.

### Parsing Sectioned INI
```rust
use babbel::core::{parse_ini, IniOptions};

let config = r#"
# Server settings
[server]
host = 127.0.0.1
port = 8080
workers = 4

[database]
url = postgresql://localhost/db
pool_size = 10
"#;

let value = parse_ini(config, &IniOptions::default())?;
let server = value.get("server").unwrap();
assert_eq!(server.get("port").and_then(|v| v.as_i64()), Some(8080));
```

### Parsing Environment Files (`.env`)
```rust
use babbel::core::{parse_ini, IniOptions};

let env_file = "DATABASE_URL=sqlite://app.db\nPORT=3000\nDEBUG=true\n";
let env_val = parse_ini(env_file, &IniOptions::env())?;
assert_eq!(env_val.get("PORT").and_then(|v| v.as_i64()), Some(3000));
```

---

## 4. Line-Delimited JSON (JSON Lines / NDJSON)

JSON Lines is standard for log processing, analytics pipelines, and AI dataset streaming. Each line contains a single-line JSON value terminated by `\n`.

### Streaming Reader with `JsonLinesReader`
```rust
use babbel::core::io::SliceSource;
use json_lib::lines::{JsonLinesConfig, JsonLinesReader};

let stream_data = "{\"event\": \"login\", \"user\": 10}\n{\"event\": \"logout\", \"user\": 10}\n";
let source = SliceSource::new(stream_data);

let mut reader = JsonLinesReader::new(source);
while let Some(record) = reader.next_node() {
    let node = record.unwrap();
    println!("Event: {:?}", node.get("event"));
}
```

### Batch Parsing & Serializing
```rust
use json_lib::lines::{parse_json_lines, to_json_lines};

let jsonl = "{\"id\": 1}\n{\"id\": 2}\n";
let nodes = parse_json_lines(jsonl)?;
assert_eq!(nodes.len(), 2);

let serialized = to_json_lines(&nodes)?;
assert_eq!(serialized, "{\"id\":1}\n{\"id\":2}\n");
```

---

## 5. Frontmatter & Text Utilities

Located in [`babbel_core::text`](../crates/babbel_core/src/text.rs).

### Frontmatter Extraction
Extracts document frontmatter headers (e.g. Hugo, Jekyll, VitePress, Obsidian markdown files):
```rust
use babbel::core::{split_frontmatter, FrontmatterFormat};

let post = r#"---
title: Getting Started with Babbel
date: 2026-09-07
tags: [rust, serialization]
---
# Welcome
Here is the document content.
"#;

let parsed = split_frontmatter(post);
assert_eq!(parsed.format, Some(FrontmatterFormat::Yaml));
assert!(parsed.frontmatter.unwrap().contains("title: Getting Started"));
assert!(parsed.content.starts_with("# Welcome"));
```

### Indentation Utilities
```rust
use babbel::core::text::{dedent, indent, trim_lines};

// Indent
let s = indent("foo\nbar", "  ");
assert_eq!(s, "  foo\n  bar");

// Common prefix dedent
let raw_code = "    fn hello() {\n        42\n    }";
assert_eq!(dedent(raw_code), "fn hello() {\n    42\n}");

// Line trimming
let messy = "\n\n  text with trailing space   \n\n";
assert_eq!(trim_lines(messy), "  text with trailing space");
```
