# babbel_core

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)
[![Rust Edition](https://img.shields.io/badge/edition-2024-orange)](Cargo.toml)

Foundational architectural kernel for the **Babbel** multi-format serialization ecosystem. `babbel_core` provides unified streaming I/O abstractions adhering strictly to **SOLID** principles, a universal `Value` AST, Unicode BOM detection, zero-allocation numeric formatting, diagnostic error reporting, and string escaping.

---

## Features

- **SOLID Streaming I/O (`babbel_core::io`)**:
  - Segregated capability traits (`IByteStream`, `IByteWriter`, `ICharStream`, `IRewindable`, `IPositionAware`, `ILocationAware`, `IClearable`, `ITailInspectable`, `IFlushable`, `IIndentationAware`).
  - Unified input sources: `BufferSource`, `FileSource`, `SliceSource`, `StringSource`.
  - Unified output destinations: `Buffer`, `FileDestination`, `StringDestination`.
  - Safe in-memory tail tracking (`last()`) without file system re-opening.
  - Full Unicode scalar decoding preventing multi-byte UTF-8 corruption.
- **Universal Data Model (`babbel_core::model`)**:
  - `Value` AST (`Null`, `Bool`, `Integer`, `Float`, `String`, `Array`, `Object`, `Bytes`) powering cross-format conversions.
- **Unified Codec Interfaces (`babbel_core::codec`)**:
  - `FormatParser`, `FormatEmitter`, and `FormatCodec` abstractions.
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
babbel_core = "0.1.0"
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

## I/O Streaming Architecture

### Core Traits (`babbel_core::io::traits`)

```rust
use babbel_core::io::traits::*;

// 1. Forward character stream (ISP-segregated pull parser)
pub trait ICharStream {
    fn next(&mut self);
    fn current(&mut self) -> Option<char>;
    fn more(&mut self) -> bool;
}

// 2. Binary byte stream (ISP-segregated for binary protocols)
pub trait IByteStream {
    fn peek_byte(&mut self) -> Option<u8>;
    fn read_byte(&mut self) -> Option<u8>;
    fn advance(&mut self);
    fn has_more(&mut self) -> bool;
}

// 3. Composite source
pub trait ISource: ICharStream + IRewindable {}

// 4. Output destination
pub trait IDestination: IClearable + ITailInspectable {
    fn add_byte(&mut self, byte: u8);
    fn add_bytes(&mut self, bytes: &str);
}
```

### Reading from In-Memory Buffers & Slices

```rust
use babbel_core::io::{BufferSource, SliceSource, ISource};

// 1. Reading from a byte buffer
let mut source = BufferSource::new("Hello, 世界!".as_bytes());
assert_eq!(source.current(), Some('H'));
source.next();
assert_eq!(source.current(), Some('e'));

// 2. Reading directly from a borrowed slice with zero allocations
let data = b"42";
let mut slice_source = SliceSource::new(data);
assert_eq!(slice_source.current(), Some('4'));
```

### Writing with In-Memory Buffers & Files

```rust
use babbel_core::io::{Buffer, IDestination};

let mut dest = Buffer::new();
dest.add_bytes("hello");
dest.add_byte(b' ');
dest.add_bytes("world");

assert_eq!(dest.to_string(), "hello world");
assert_eq!(dest.last(), Some(b'd'));
```

### BOM Detection & Newline Normalization

```rust
use babbel_core::encoding::{detect_encoding_and_strip_bom, normalize_newlines, Encoding};

// Auto-detect UTF-8 BOM
let raw_bytes = b"\xEF\xBB\xBFcontent\r\n";
let (content, encoding) = detect_encoding_and_strip_bom(raw_bytes)?;
assert_eq!(encoding, Encoding::Utf8);

// Normalize CRLF to LF
let normalized = normalize_newlines(&content);
assert_eq!(normalized, "content\n");
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
```

---

## License

Licensed under the [MIT License](../../LICENSE).
