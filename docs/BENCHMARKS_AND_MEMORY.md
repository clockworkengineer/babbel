# Babbel Memory & Performance Architecture

Babbel is engineered from the ground up for high-throughput, low-latency, and memory-constrained environments, including serverless workloads, streaming pipelines, and embedded `no_std` systems.

---

## 1. Compact Memory Footprint (Struct Size Bounds)

In Rust, deeply nested AST nodes and enum variants can quickly balloon in memory due to discriminant alignment and wide enum variants. Large nodes degrade CPU L1/L2 cache locality and amplify heap allocation overhead.

Babbel strictly enforces compact struct sizes, verified by continuous automated regression tests in [`crates/babbel/tests/size_checks.rs`](../crates/babbel/tests/size_checks.rs):

| Type | Maximum Size | Before Optimization | Strategy |
| :--- | :--- | :--- | :--- |
| **`babbel_core::Value`** | **32 bytes** | 48 bytes | Compacted discriminants, string/vec pairing, and boxed complex variants. |
| **`xml_lib::NodeKind`** | **48 bytes** | 72 bytes | Boxed large element payload (`Box<ElementData>`) and processing instructions. |
| **`xml_lib::NodeData`** | **88 bytes** | 112 bytes | Shrunk inner attribute mapping and compacted node kinds. |
| **`yaml_lib::Node`** | **40 bytes** | 56 bytes | Inlined small scalar storage and pointer tagging for aliases. |
| **`json_lib::Node`** | **56 bytes** | 64 bytes | Small string optimization (`sso`) and unioned number discriminants. |
| **`bencode_lib::Node`** | **56 bytes** | 64 bytes | Compacted dictionary maps and zero-copy byte slices. |

### Size Check Regression Suite
To run the automated memory size verification:
```bash
cargo test -p babbel --test size_checks
```

---

## 2. Zero-Allocation Primitives

### 2.1 Integer and Float Formatting
Standard formatting with `format!("{}", number)` allocates an intermediate `String` on the heap for every single formatted number.

Babbel replaces all dynamic number allocations with zero-allocation stack buffers:
- **`babbel_core::num::format_integer`**: Backed by [`itoa`](https://crates.io/crates/itoa), formatting directly into an in-memory stack buffer without heap allocation.
- **`babbel_core::num::format_float`**: Backed by [`dtoa`](https://crates.io/crates/dtoa), writing IEEE 754 floating point strings into stack buffers in zero allocations.

### 2.2 Zero-Copy Slicing with `read_line_slice`
While `read_line` returns an owned `String`, `SliceSource` provides zero-copy borrowing:
```rust
use babbel_core::io::SliceSource;

let data = "header1,header2\nval1,val2\n";
let mut source = SliceSource::new(data);

// Yields borrowed &'a str slices directly from source memory
while let Some(line) = source.read_line_slice() {
    println!("Line: {}", line);
}
```

### 2.3 Short-String Escaping
When writing JSON or XML escaped strings (`write_json_escaped_string`, `write_xml_escaped_string`), strings that do not contain special characters pass through directly without copying or allocating intermediary memory.

---

## 3. Streaming I/O Optimization

### 3.1 Eliminating Windows File Handle Contention
Traditional implementations that track written length or check trailing bytes reopen file handles or perform expensive disk seek system calls (`lseek`). On Windows, concurrent file handle reopening triggers OS file locks.

`babbel_core::io::FileDestination` maintains in-memory tracking:
- `last_byte`: Stores the most recently written byte in memory.
- `length`: Tracks written stream offset in memory.
- Result: Fast comma suppression and delimiter tracking with **zero file system seeks**.

### 3.2 Buffered File I/O
`FileSource` and `BufferSource` buffer disk reads into a configurable internal buffer, avoiding individual 1-byte read system calls while preserving accurate byte-offset and location metrics.

---

## 4. `no_std` and Embedded Targets

All primitives in `babbel_core` compile cleanly in `no_std` environments:
```toml
[dependencies]
babbel_core = { version = "0.1", default-features = false, features = ["alloc"] }
```
When `std` is disabled, `babbel_core` utilizes `alloc::string::String`, `alloc::vec::Vec`, and `core::*` primitives, making it suitable for WebAssembly, microcontrollers, and kernel modules.
