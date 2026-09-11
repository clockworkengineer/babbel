# Babbel Embedded Systems & Zero-Allocation Guide

Babbel is engineered to run in resource-constrained environments, including microcontrollers (ARM Cortex-M, RISC-V, ESP32), Real-Time Operating Systems (RTOS), WebAssembly, and bare-metal environments (`no_std`).

This guide details how to leverage Babbel's embedded primitives and streaming pull parsers to process, transform, and serialize structured documents with minimal or **zero heap allocations**.

---

## 1. `no_std` Support & Cargo Configuration

Every core component of Babbel supports `no_std` execution with the `alloc` feature enabled.

### Cargo.toml Setup
```toml
[dependencies]
babbel_core = { version = "0.2.0", default-features = false, features = ["alloc"] }
babbel_json = { version = "0.2.0", default-features = false, features = ["alloc"] }
babbel_bencode = { version = "0.2.0", default-features = false, features = ["alloc"] }
babbel_xml = { version = "0.2.0", default-features = false, features = ["alloc"] }
babbel_yaml = { version = "0.2.0", default-features = false, features = ["alloc"] }
babbel_toml = { version = "0.2.0", default-features = false, features = ["alloc"] }
```

When building for bare-metal targets (e.g. `thumbv7em-none-eabihf`):
```bash
cargo build --target thumbv7em-none-eabihf --no-default-features --features alloc
```

---

## 2. Stack Memory Primitives (`StackBuffer`)

Dynamic heap allocations can cause memory fragmentation and unpredictable execution times in RTOS environments. Babbel provides stack-allocated primitives in [`babbel_core::embedded`](../crates/babbel_core/src/embedded/mod.rs) and via `babbel::embedded`:

### `StackBuffer<const N: usize>`
A fixed-size byte buffer stored entirely on the call stack using const generics:

```rust
use babbel::embedded::StackBuffer;

// Allocate a 128-byte buffer on the stack
let mut buf = StackBuffer::<128>::new();

assert_eq!(buf.capacity(), 128);
buf.push(b'{');
buf.extend_from_slice(b"\"status\":200");
buf.push(b'}');

assert_eq!(buf.as_str().unwrap(), "{\"status\":200}");
```

---

## 3. Zero-Allocation Output Destinations

Traditional serialization emits into a heap-allocated `String` or `Vec<u8>`. Babbel's [`IDestination`](../crates/babbel_core/src/io/traits.rs) trait allows emitting directly into stack memory or fixed buffers without touching the heap allocator.

### 3.1 `SliceDestination<'a>`
Emits output sequentially into a caller-provided byte slice:

```rust
use babbel_core::io::IDestination;
use babbel::embedded::SliceDestination;

let mut storage = [0u8; 64];
let mut dest = SliceDestination::new(&mut storage);

dest.add_bytes("{\"sensor\": \"temp\", \"val\": 23.4}");

// Check written output without any heap allocation:
assert_eq!(dest.as_str().unwrap(), "{\"sensor\": \"temp\", \"val\": 23.4}");
assert!(!dest.is_truncated());
```

If the document exceeds the slice size, `dest.is_truncated()` returns `true` rather than panicking or overflowing memory.

### 3.2 `ArrayVecDestination<const N: usize>`
Self-contained stack destination holding an inline array:

```rust
use babbel_core::io::IDestination;
use babbel::embedded::ArrayVecDestination;

let mut dest = ArrayVecDestination::<256>::new();
dest.add_bytes("<reading>1024</reading>");

assert_eq!(dest.as_str().unwrap(), "<reading>1024</reading>");
assert_eq!(dest.remaining(), 256 - 24);
```

---

## 4. Zero-Copy Input Streaming (`SliceSource`)

When parsing input in memory (from a flash memory address or network receive packet), [`SliceSource<'a>`](../crates/babbel_core/src/io/sources.rs) operates directly over the borrowed input slice without allocating memory:

```rust
use babbel_core::io::{ILineReader, SliceSource};

let packet = "line1\r\nline2\r\nline3\r\n";
let mut source = SliceSource::new(packet);

// Zero-copy borrowed line slices:
while let Some(line) = source.read_line_slice() {
    // `line` is a &'a str directly referencing `packet` memory
    println!("Line slice: {}", line);
}
```

---

## 5. Streaming Event Pull Parsers

For memory-critical systems where building a full AST tree into heap RAM is prohibitive, Babbel exposes zero-allocation pull parsers in `babbel::embedded`:

### 5.1 JSON Pull Parser (`JsonPullParser`)
Processes JSON streams token-by-token with zero AST overhead:

```rust
use babbel::embedded::{JsonPullParser, JsonPullEvent, JsonScalar};

let json = r#"{"sensor": "temp", "val": 23.4}"#;
let mut parser = JsonPullParser::new(json);

while let Some(event) = parser.next_event()? {
    match event {
        JsonPullEvent::ObjectStart => println!("Object started"),
        JsonPullEvent::Key(key) => println!("Key: {}", key),
        JsonPullEvent::Scalar(JsonScalar::Float(val)) => println!("Float: {}", val),
        JsonPullEvent::ObjectEnd => println!("Object finished"),
        _ => {}
    }
}
```

### 5.2 TOML Pull Parser (`TomlPullParser`)
Streams TOML keys, tables, and values without allocating an AST:

```rust
use babbel::embedded::{TomlPullParser, TomlPullEvent};

let toml = "name = 'device-1'\ncount = 42\n";
let mut parser = TomlPullParser::new(toml);

while let Some(event) = parser.next_event()? {
    match event {
        TomlPullEvent::KeyValue { key, value } => {
            println!("Key: {}, Value: {:?}", key, value);
        }
        _ => {}
    }
}
```

### 5.3 XML Pull Parser (`XmlPullParser`)
Streams XML tags and attributes with zero allocations:

```rust
use babbel::embedded::{XmlPullParser, XmlPullEvent};

let xml = "<sensor id='101'>active</sensor>";
let mut parser = XmlPullParser::new(xml);

while let Some(event) = parser.next_event()? {
    match event {
        XmlPullEvent::StartTag { name, attributes } => {
            println!("Start tag: {}", name);
        }
        XmlPullEvent::Text(t) => println!("Text: {}", t),
        XmlPullEvent::EndTag { name } => println!("End tag: {}", name),
        _ => {}
    }
}
```

### 5.4 CSV & INI Pull Parsers
- `CsvPullParser`: Iterates over `CsvRecord` without collecting all rows into memory.
- `IniPullParser`: Streams `IniEvent::Section` and `IniEvent::Property` key-value pairs.

---

## 6. Memory Tracking & Allocation Guards (`MemoryTracker`)

For embedded systems with strict RAM budgets, [`MemoryTracker`](../crates/babbel_core/src/embedded/mod.rs) monitors dynamic allocations and enforces peak thresholds:

```rust
use babbel::embedded::MemoryTracker;

// Enforce a hard ceiling of 4096 bytes:
let tracker = MemoryTracker::with_limit(4096);

// Record allocations
tracker.allocate(1024).expect("Within budget");
assert_eq!(tracker.current(), 1024);
assert_eq!(tracker.peak(), 1024);

// Attempting to exceed budget fails safely:
assert!(tracker.allocate(4000).is_err());
```

---

## 7. Embedded Safety Limits (`EmbeddedLimits`)

Untrusted or malformed inputs can trigger stack overflow via deeply nested structures. [`EmbeddedLimits`](../crates/babbel_core/src/embedded/mod.rs) prevents unbounded recursion:

```rust
use babbel::embedded::EmbeddedLimits;

// Pre-tuned conservative profile for 32-bit MCUs:
let limits = EmbeddedLimits::CONSERVATIVE;
assert_eq!(limits.max_depth, 16);
assert_eq!(limits.max_token_length, 256);
assert_eq!(limits.max_container_items, 64);

// Ultra-constrained profile for 8-bit MCUs (AVR / Cortex-M0):
let minimal = EmbeddedLimits::MINIMAL;
assert_eq!(minimal.max_depth, 8);
```

---

## 8. Zero-Allocation Error Reporting (`CompactError`)

In bare-metal targets, formatted string errors (`format!("Error at line {}: {}", line, msg)`) force heap allocation. Babbel provides [`CompactError`](../crates/babbel_core/src/embedded/mod.rs), an **8-byte** error descriptor:

```rust
use babbel::embedded::CompactError;
use babbel_core::error::ErrorCode;

let err = CompactError::new(ErrorCode::UnexpectedEof, 128);

assert_eq!(core::mem::size_of::<CompactError>(), 8);
assert_eq!(err.code, ErrorCode::UnexpectedEof);
assert_eq!(err.offset, 128);
```

---

## 9. Embedded Formats Summary

| Format | Zero-Copy Borrowed Parsing | Pull Parser Stream | Stack Buffering Support | Recommended Target |
| :--- | :---: | :---: | :---: | :--- |
| **Bencode** | ✅ `BorrowedNode<'a>` | N/A (linear tokenization) | ✅ `FixedSizeBuffer` | BitTorrent, IoT telemetry, low-overhead binary RPC |
| **JSON** | ✅ `SliceSource` | ✅ `JsonPullParser` | ✅ `SliceDestination` | Web services, sensors, telemetry |
| **TOML** | ✅ `SliceSource` | ✅ `TomlPullParser` | ✅ `ArrayVecDestination` | Device configuration, application manifests |
| **CSV / TSV** | ✅ `read_line_slice` | ✅ `CsvPullParser` | ✅ `ArrayVecDestination` | High-frequency sensor logging, tabular flash storage |
| **INI / .env** | ✅ `SliceSource` | ✅ `IniPullParser` | ✅ `SliceDestination` | Hardware configuration, non-volatile parameters |
| **XML** | ✅ String slicing | ✅ `XmlPullParser` | ✅ `SliceDestination` | Standard device interchange, SOAP/XML RPC |
