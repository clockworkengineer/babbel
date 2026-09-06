# Babbel Architecture & Design Principles

Babbel is a high-performance, polyglot serialization, parsing, and document manipulation ecosystem in Rust. It unifies **JSON**, **YAML**, **Bencode**, and **XML** under a cohesive multi-crate architecture that adheres strictly to **DRY** (Don't Repeat Yourself) and **SOLID** engineering principles.

---

## 1. 3-Tier Layering Model

Babbel is architected across three distinct, decoupled tiers:

```mermaid
graph TD
    subgraph "Tier 2: Facade & Interoperability"
        Babbel["babbel (Master Facade)"]
        Convert["babbel::convert (O(N) Universal Conversion Pipeline)"]
    end

    subgraph "Tier 1: Domain Format Engines"
        JSON["json_lib (RFC 6901, RFC 7396, JSON5)"]
        YAML["yaml_lib (YAML 1.2, Anchors/Aliases, Custom Tags)"]
        XML["xml_lib_rust (W3C DOM, C14N, DTD, XSD, XPath 1.0)"]
        Bencode["bencode_lib (BitTorrent, Zero-Copy Slices)"]
    end

    subgraph "Tier 0: Foundational Kernel"
        Core["babbel_core"]
        CoreIO["babbel_core::io (IByteStream, ICharStream, ISource, IDestination)"]
        CoreAST["babbel_core::model (Universal Value AST)"]
        CoreCodec["babbel_core::codec (FormatCodec, FormatParser, FormatEmitter)"]
        CoreUtils["babbel_core (BOM Detection, Escaping, Diagnostic Errors)"]
    end

    Babbel --> Convert
    Convert --> CoreAST
    Convert --> CoreCodec
    Babbel --> JSON
    Babbel --> YAML
    Babbel --> XML
    Babbel --> Bencode

    JSON --> Core
    YAML --> Core
    XML --> Core
    Bencode --> Core
```

### Layer 0: The Core Kernel (`babbel_core`)
* **Independence**: Has zero dependencies on any format parser. Compiles for both standard library and `no_std` environments.
* **Responsibilities**:
  * **Streaming I/O**: Segregated interfaces for pull-based character streaming, binary byte reading, rewindability, location awareness, and buffered destinations.
  * **Universal AST (`Value`)**: Canonically represents arbitrary structured data (`Null`, `Bool`, `Integer`, `Float`, `String`, `Array`, `Object`, `Bytes`).
  * **Codec Interfaces**: Defines `FormatParser` and `FormatEmitter` abstractions.
  * **DRY Primitives**: Unicode BOM auto-detection (UTF-8, UTF-16 LE/BE, UTF-32 LE/BE), newline normalization, zero-allocation integer formatting via `itoa`, fast float formatting via `dtoa`, and canonical string escaping.

### Layer 1: Format Engines (`json_lib`, `yaml_lib`, `bencode_lib`, `xml_lib_rust`)
* **Grammar & Semantics**: Each engine implements parsing, syntax validation, document navigation, and serialization for its specific specification.
* **Abstractions**: All format engines depend on `babbel_core::io` abstractions rather than hardcoded OS files or buffers.
* **Autonomy**: Each crate can be consumed independently with minimal binary footprint.

### Layer 2: Facade & Interoperability (`babbel`)
* **Ergonomics**: Provides unified prelude imports (`use babbel::prelude::*;`) and re-exports all format engines under feature flags.
* **$O(N)$ Universal Conversion**: Cross-format conversion runs through the intermediate `Value` AST or streaming codecs without requiring $O(N^2)$ point-to-point converters.

---

## 2. SOLID Principles in Rust

### Single Responsibility Principle (SRP)
* **Transport vs. Syntax**: `FileSource`, `BufferSource`, and `FileDestination` manage strictly binary and character transfer. They contain zero knowledge of JSON brackets, YAML indentation, or XML tags.
* **Segregated Operations**: Reading (`IByteStream`, `ICharStream`), writing (`IByteWriter`), tail inspection (`ITailInspectable`), stream resetting (`IRewindable`), and storage synchronization (`IFlushable`) are decoupled into single-focus traits.

### Open/Closed Principle (OCP)
* **Format Extensibility**: Introducing a new format (e.g. TOML, MessagePack) requires only implementing `FormatParser` and `FormatEmitter` from `babbel_core::codec`. The conversion pipeline (`babbel::convert`) immediately supports the new format without any modifications.
* **Stream Extensibility**: Custom streams (e.g., memory-mapped files, compressed streams, network sockets) implement `ICharStream` or `IByteStream` and can be passed directly to all parsers.

### Liskov Substitution Principle (LSP)
* **Consistent Stream Invariants**:
  * Character streams (`ICharStream`) guarantee valid UTF-8 scalar decoding across all implementations (`SliceSource`, `BufferSource`, `FileSource`). Multi-byte UTF-8 sequences are never truncated by naive byte casts.
  * Destinations (`IDestination`) guarantee safe tail inspection (`last()`) without file system side-effects or file handle reopening locks on Windows.
* **Substitutability**: Any function expecting `&mut dyn ISource` behaves identically whether given a file on disk or an in-memory buffer.

### Interface Segregation Principle (ISP)
Clients bind only to the minimal interface required for their operation:

| Trait | Focus | Primary Implementor / Consumer |
| :--- | :--- | :--- |
| `IByteStream` | Forward byte reading (`read_byte`, `peek_byte`, `advance`) | Binary protocols (`bencode_lib`) |
| `IByteWriter` | Binary byte writing (`write_byte`, `write_bytes`) | Binary serialization |
| `ICharStream` | Minimal forward character pull (`next`, `current`, `more`) | Lightweight parsers |
| `IRewindable` | Stream position reset (`reset`) | Multi-pass parsers |
| `IPositionAware` | Absolute byte offset (`position`) | Token spans and diagnostics |
| `ILocationAware` | Line and column metrics (`line`, `column`) | Diagnostic error reporters |
| `ITailInspectable` | Inspect last written byte (`last_byte`) | Comma-separation formatters |
| `IClearable` | Truncate buffer/storage (`clear`) | Buffer pooling & recycling |
| `IFlushable` | Flush buffered bytes (`flush`) | File and network I/O |
| `IIndentationAware`| Current column / indent calculation | Whitespace-sensitive grammars (`yaml_lib`) |

### Dependency Inversion Principle (DIP)
* **High-level parsers** depend upon trait abstractions (`ISource`, `ICharStream`, `IByteStream`).
* **High-level serializers** depend upon `IDestination` and `IByteWriter`.
* Concrete OS file handles and heap buffers depend upon these abstractions via implementations in `babbel_core::io`.

---

## 3. Universal Data Model & Interoperability

The `Value` enum in `babbel_core::model` acts as the lingua franca of the ecosystem:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Bytes(Vec<u8>),
}
```

### Format Mapping Matrix

| `Value` Variant | JSON Representation | YAML Representation | XML Representation | Bencode Representation |
| :--- | :--- | :--- | :--- | :--- |
| `Null` | `null` | `null` / `~` | `<item nil="true"/>` / empty | *(omitted or empty string)* |
| `Bool(b)` | `true` / `false` | `true` / `false` | `<item>true</item>` | `i1e` / `i0e` |
| `Integer(i)` | Number (`42`) | Integer (`42`) | `<item>42</item>` | `i42e` |
| `Float(f)` | Number (`3.14`) | Float (`3.14`) | `<item>3.14</item>` | Byte string stringification |
| `String(s)` | String (`"hello"`) | String (`hello`) | Text node (`hello`) | `5:hello` |
| `Array(v)` | Array (`[...]`) | Sequence (`- ...`) | Elements (`<item>...</item>`) | `l...e` |
| `Object(o)` | Object (`{...}`) | Mapping (`key: val`) | Element tags / attributes | `d...e` |
| `Bytes(b)` | Base64 string | `!!binary` (Base64) | Base64 text node | Byte string (`len:bytes`) |

---

## 4. Performance & Memory Optimizations

1. **Zero-Copy Where Feasible**:
   - `bencode_lib` provides `BorrowedNode<'a>`, slicing directly from input buffers with zero heap allocations.
   - `xml_lib_rust` provides `slice_range(start, end)` directly over UTF-8 string buffers.
   - `SliceSource<'a>` reads directly from borrowed byte slices.
2. **Elimination of I/O System Call Overhead**:
   - `FileDestination` maintains in-memory tracking of written length and the last written byte, eliminating disk seeks and handle re-opening.
   - `FileSource` buffers disk reads in memory, preventing individual 1-byte system calls.
3. **Small-Vector & Inlining**:
   - Short string escaping utilizes stack-allocated buffers for ASCII sequences before falling back to heap allocations.
   - Numerical conversions use `itoa` and `dtoa` without allocating intermediate strings.
4. **String Interning**:
   - `yaml_lib` and `json_lib` provide optional string interners (`StringInterner`) to deduplicate dictionary and mapping keys across large datasets.
