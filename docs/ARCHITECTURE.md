# Babbel Architecture & Design Principles

Babbel is a high-performance, polyglot serialization, parsing, and document manipulation ecosystem in Rust. It unifies **JSON**, **YAML**, **Bencode**, **XML**, **CSV / TSV**, **INI / Properties**, and **JSON Lines** under a cohesive multi-crate architecture adhering strictly to **DRY** (Don't Repeat Yourself) and **SOLID** engineering principles.

---

## 1. 3-Tier Layering Model

Babbel is architected across three distinct, decoupled tiers:

```mermaid
graph TD
    subgraph "Tier 2: Facade & Interoperability"
        Babbel["babbel (Master Facade & Prelude)"]
        Convert["babbel::convert (convert_format / convert_format_bytes)"]
        Registry["babbel::default_registry() (Dynamic FormatRegistry)"]
    end

    subgraph "Tier 1: Domain Format Engines"
        JSON["babbel_json::JsonEngine (RFC 6901, RFC 7396, JSON5, JSON Lines)"]
        YAML["babbel_yaml::YamlEngine (YAML 1.2, Anchors/Aliases, Custom Tags)"]
        XML["babbel_xml::XmlEngine (W3C DOM, C14N, DTD, XSD, XPath 1.0)"]
        Bencode["babbel_bencode::BencodeEngine (BitTorrent, Zero-Copy Slices)"]
        TOML["babbel_toml::TomlEngine (TOML v1.1.0, Streaming Pull Parser)"]
    end

    subgraph "Tier 0: Foundational Kernel"
        Core["babbel_core"]
        CoreIO["babbel_core::io (ILineReader, IByteStream, ICharStream, ISource, IDestination)"]
        CoreAST["babbel_core::model (Universal Value AST)"]
        CoreText["babbel_core (RFC 4180 CSV/TSV, INI/.env, Frontmatter)"]
        CoreCodec["babbel_core::codec (FormatEngine, FormatRegistry, FormatParser, FormatEmitter)"]
        CoreUtils["babbel_core (BOM Detection, Escaping, Diagnostic Errors)"]
    end

    Babbel --> Convert
    Babbel --> Registry
    Convert --> CoreAST
    Convert --> CoreCodec
    Registry --> CoreCodec

    Babbel --> JSON
    Babbel --> YAML
    Babbel --> XML
    Babbel --> Bencode
    Babbel --> TOML
    Babbel --> CoreText

    JSON -.->|implements FormatEngine| CoreCodec
    YAML -.->|implements FormatEngine| CoreCodec
    XML -.->|implements FormatEngine| CoreCodec
    Bencode -.->|implements FormatEngine| CoreCodec
    TOML -.->|implements FormatEngine| CoreCodec

    JSON --> CoreIO
    YAML --> CoreIO
    XML --> CoreIO
    Bencode --> CoreIO
    TOML --> CoreIO
```

### Layer 0: The Core Kernel (`babbel_core`)
* **Independence**: Zero dependencies on foreign format parsers. Compiles cleanly for standard library and `no_std` + `alloc` environments.
* **Responsibilities**:
  * **Streaming I/O**: Segregated interfaces for pull-based character streaming (`ICharStream`), binary byte reading (`IByteStream`), line-by-line streaming across mixed CRLF/LF/CR (`ILineReader`), rewindability (`IRewindable`), location awareness (`ILocationAware`), and buffered destinations (`IDestination`).
  * **Universal AST (`Value`)**: Canonically represents arbitrary structured data (`Null`, `Bool`, `Integer(i128)`, `Float`, `String`, `Array`, `Object`, `Bytes`) in a cache-aligned 32-byte layout.
  * **Format Engine Abstractions (`babbel_core::codec`)**: Defines `FormatEngine`, `FormatParser`, `FormatEmitter`, `FormatOptions`, and `FormatRegistry` with dynamic registration and static slice lookup (`find_engine*`).
  * **Text Engines**:
    * **RFC 4180 Delimited Text (`csv`)**: High-performance CSV & TSV parsing and emission, delimiter auto-sniffing, multi-line quoted fields, and scalar type inference.
    * **Configuration Text (`ini`)**: Section-based INI (`[section]`), `.properties`, and `.env` parsing, comment handling (`#`, `;`, `!`), and emission.
    * **Frontmatter & Line Utilities (`text`)**: Extracting YAML (`---`) and TOML (`+++`) metadata blocks (`split_frontmatter`), indentation trimming (`indent`, `dedent`, `trim_lines`).
  * **Embedded Systems & Zero-Allocation (`embedded`)**:
    * **Stack Destinations**: `SliceDestination<'a>` and `ArrayVecDestination<const N>` allow writing directly into caller-provided stack memory with zero dynamic allocations.
    * **Streaming Pull Parsers**: `JsonPullParser`, `XmlPullParser`, `CsvPullParser`, and `IniPullParser` enable microcontrollers with 16–64 KB RAM to parse documents of arbitrary size with $O(1)$ stack memory.
    * **Memory Control**: `StackBuffer<const N>`, `MemoryTracker`, `EmbeddedLimits`, and 8-byte `CompactError` ensure deterministic execution without heap fragmentation.
  * **DRY Primitives**: Unicode BOM auto-detection (UTF-8, UTF-16 LE/BE, UTF-32 LE/BE), newline normalization, zero-allocation integer formatting via `itoa`, fast float formatting via `dtoa`, and canonical string escaping.

### Layer 1: Domain Format Engines (`babbel_json`, `babbel_yaml`, `babbel_bencode`, `babbel_xml`, `babbel_toml`)
* **Grammar & Semantics**: Each engine implements parsing, syntax validation, document navigation, and serialization for its specific specification.
* **FormatEngine Implementation**: Each crate exposes an autonomous engine struct (`JsonEngine`, `YamlEngine`, `XmlEngine`, `BencodeEngine`, `TomlEngine`) implementing `babbel_core::FormatEngine`.
* **JSON Lines Streaming**: `babbel_json::lines` provides streaming `JsonLinesReader` and `to_json_lines` over `ILineReader`.
* **Abstractions**: All format engines depend on `babbel_core::io` abstractions rather than hardcoded OS files or heap buffers.
* **Autonomy**: Each crate can be consumed independently with minimal binary footprint.

### Layer 2: Facade & Interoperability (`babbel`)
* **Ergonomics**: Provides unified prelude imports and re-exports all format engines and core text engines under feature flags.
* **$O(N)$ Universal Conversion**: Cross-format conversion pipelines (`babbel::convert::convert_format`, `convert_format_bytes`, `convert_format_bytes_to_str`) between JSON, YAML, XML, Bencode, TOML, CSV, TSV, INI, and JSON Lines route through abstract `FormatEngine` and `Value` AST with zero circular dependencies.

---

## 2. SOLID Principles in Rust

### Single Responsibility Principle (SRP)
* **Decomposed AST Submodules**: Monolithic files have been broken down into single-responsibility submodules (e.g. YAML's `nodes/` split into `access.rs` for scalar/map queries, `search.rs` for traversal, `convert.rs` for `Value` bridging, and `scalar.rs` for numeric conversions).
* **Transport vs. Syntax**: `FileSource`, `BufferSource`, and `FileDestination` manage strictly binary and character transfer. They contain zero knowledge of JSON brackets, YAML indentation, CSV quotes, or XML tags.
* **Segregated Operations**: Reading (`IByteStream`, `ICharStream`, `ILineReader`), writing (`IByteWriter`), tail inspection (`ITailInspectable`), stream resetting (`IRewindable`), and storage synchronization (`IFlushable`) are decoupled into single-focus traits.

### Open/Closed Principle (OCP)
* **Pluggable Format Engines**: Introducing a new format (e.g. MessagePack, CBOR) requires only implementing `FormatEngine` from `babbel_core::codec`. The conversion pipeline (`babbel::convert`) immediately supports the new format without modifying existing crates.
* **Dynamic & Static Registry**: `FormatRegistry` allows runtime engine registration and lookup by MIME type or file extension, while `find_engine*` enables compile-time slice lookup.
* **Stream Extensibility**: Custom streams (e.g., memory-mapped files, compressed streams, network sockets) implement `ICharStream`, `IByteStream`, or `ILineReader` and can be passed directly to all parsers.

### Liskov Substitution Principle (LSP)
* **Infallible Document Navigation**: Tree accessors (`get()`, `at()`, `pointer()`) return `Option<&Node>` or `Result`, never panicking on out-of-bounds indices or type mismatches.
* **Error Type Unification**: All format errors implement `std::error::Error + Send + Sync + 'static` and convert into `BabbelError`, ensuring standardized error handling across all formats.
* **Consistent Stream Invariants**:
  * Character streams (`ICharStream`) guarantee valid UTF-8 scalar decoding across all implementations (`SliceSource`, `BufferSource`, `FileSource`). Multi-byte UTF-8 sequences are never truncated by naive byte casts.
  * Line readers (`ILineReader`) correctly handle `\r\n`, `\n`, and `\r` across all sources.
  * Destinations (`IDestination`) guarantee safe tail inspection (`last()`) without file system side-effects or file handle reopening locks on Windows.

### Interface Segregation Principle (ISP)
Clients bind only to the minimal interface required for their operation:
* **Fine-Grained Streaming Traits**: Clients bind only to the minimal interface required for their operation (`ILineReader`, `IByteStream`, `ICharStream`, etc.).
* **Visitor Default Methods**: `ValueVisitor` and `NodeVisitor` provide sensible default no-op methods, removing mandatory stub boilerplate.

| Trait | Focus | Primary Implementor / Consumer |
| :--- | :--- | :--- |
| `ILineReader` | Line-by-line text reading across CRLF/LF/CR (`read_line`, `read_line_into`, `lines`) | Text engines, JSON Lines, CSV |
| `IByteStream` | Forward byte reading (`read_byte`, `peek_byte`, `advance`) | Binary protocols (`babbel_bencode`) |
| `IByteWriter` | Binary byte writing (`write_byte`, `write_bytes`) | Binary serialization |
| `ICharStream` | Minimal forward character pull (`next`, `current`, `more`) | Lightweight parsers |
| `IRewindable` | Stream position reset (`reset`) | Multi-pass parsers |
| `IPositionAware` | Absolute byte offset (`position`) | Token spans and diagnostics |
| `ILocationAware` | Line and column metrics (`line`, `column`) | Diagnostic error reporters |
| `ITailInspectable` | Inspect last written byte (`last_byte`) | Comma-separation formatters |
| `IClearable` | Truncate buffer/storage (`clear`) | Buffer pooling & recycling |
| `IFlushable` | Flush buffered bytes (`flush`) | File and network I/O |
| `IIndentationAware`| Current column / indent calculation | Whitespace-sensitive grammars (`babbel_yaml`) |

### Dependency Inversion Principle (DIP)
* **High-level conversion coordinators** (`babbel::convert`) depend upon abstract traits (`FormatEngine`, `Value`), never concrete format parser functions.
* **High-level parsers** depend upon trait abstractions (`ISource`, `ICharStream`, `IByteStream`, `ILineReader`).
* **High-level serializers** depend upon `IDestination` and `IByteWriter`.
* Concrete OS file handles and heap buffers depend upon these abstractions via implementations in `babbel_core::io`.

---

## 3. Universal Data Model & Interoperability

The `Value` enum in `babbel_core::model` acts as the lingua franca of the ecosystem:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}
```

### Format Mapping Matrix

| `Value` Variant | JSON | YAML | XML | Bencode | TOML | CSV / TSV | INI / .env | JSON Lines |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `Null` | `null` | `null` / `~` | `<item nil="true"/>` / empty | *(omitted/empty)* | `""` / *(omitted)* | empty cell | empty / omitted | `null` |
| `Bool(b)` | `true` / `false` | `true` / `false` | `<item>true</item>` | `i1e` / `i0e` | `true` / `false` | `true` / `false` | `true` / `false` | `true` / `false` |
| `Integer(i)`| Number (`42`) | Integer (`42`) | `<item>42</item>` | `i42e` | Integer (`42`) | Integer string | Integer string | Number (`42`) |
| `Float(f)` | Number (`3.14`) | Float (`3.14`) | `<item>3.14</item>` | Stringified float | Float (`3.14`) | Float string | Float string | Number (`3.14`) |
| `String(s)` | String (`"hi"`) | String (`hi`) | Text node (`hi`) | `2:hi` | String (`"hi"`) | Escaped cell | Quoted / raw string| String (`"hi"`) |
| `Array(v)` | Array (`[...]`) | Sequence (`- ...`)| Elements (`<item>...`) | `l...e` | Array (`[...]`) | Row cells or rows | `<array>` fallback | Single-line arrays |
| `Object(o)` | Object (`{...}`) | Mapping (`k: v`) | Element tags / attrs | `d...e` | Table / inline `{}` | Header $\rightarrow$ cell map| `[section]` & `k = v`| Record per line |
| `Bytes(b)` | Base64 string | `!!binary` (Base64) | Base64 text node | `len:bytes` | Escaped string | `<bytes>` fallback | `<bytes>` fallback | Base64 string |

---

## 4. Performance & Memory Optimizations

1. **Memory Compacted Node Layouts**:
   - `babbel_core::Value`: compacted to **32 bytes**.
   - `babbel_toml::Node`: compacted to **48 bytes**.
   - `babbel_json::Node`: compacted to **56 bytes**.
   - `babbel_xml::NodeKind`: compacted from 72 bytes to **48 bytes** via targeted boxing.
   - `babbel_xml::NodeData`: compacted from 112 bytes to **88 bytes**.
   - `babbel_yaml::Node`: compacted to **40 bytes**.
   - `babbel_bencode::Node`: compacted to **56 bytes**.
   - Validated continuously via automated assertion tests in `crates/babbel/tests/size_checks.rs`.
2. **Zero-Copy Where Feasible**:
   - `babbel_bencode` provides `BorrowedNode<'a>`, slicing directly from input buffers with zero heap allocations.
   - `babbel_xml` provides `slice_range(start, end)` directly over UTF-8 string buffers.
   - `SliceSource<'a>` reads directly from borrowed byte slices and yields line slices with `read_line_slice()`.
3. **Elimination of I/O System Call Overhead**:
   - `FileDestination` maintains in-memory tracking of written length and the last written byte, eliminating disk seeks and handle re-opening.
   - `FileSource` buffers disk reads in memory, preventing individual 1-byte system calls.
4. **Small-Vector & Inlining**:
   - Short string escaping utilizes stack-allocated buffers for ASCII sequences before falling back to heap allocations.
   - Numerical conversions use `itoa` and `dtoa` without allocating intermediate strings.
