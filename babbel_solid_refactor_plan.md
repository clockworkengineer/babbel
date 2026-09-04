# Babbel SOLID Refactoring Plan: Complete Architectural Design

## Executive Summary

Babbel is a unified, polyglot serialization and document manipulation workspace supporting JSON, YAML, XML, and Bencode. Following the DRY refactoring (which consolidated shared file I/O, Unicode handling, string escaping, and basic AST bridges into `babbel_core`), the codebase is significantly leaner. However, the architecture still exhibits significant structural coupling, interface bloat, and responsibilities that violate the **SOLID** design principles.

This document presents a comprehensive, actionable, and phased plan to refactor the entire Babbel ecosystem to be **completely SOLID**, while strictly preserving **100% backward compatibility**, maintaining all existing public APIs, and ensuring all ~3,000+ unit, integration, and doc tests continue to pass.

---

## SOLID Principles: Detailed Source Audit

### 1. S — Single Responsibility Principle (SRP)
*A class, struct, or module should have one, and only one, reason to change.*

#### Current Violations:
- **Monolithic AST Files**:
  - `crates/yaml/src/nodes/node.rs` (2,769 lines): Mixes AST node storage, mutation (`set`, `push`), hierarchical search (`find_all`, `find_first`), visitor pattern implementations, collection indexing, string representation formatting, and cross-format conversions.
  - `crates/json/src/nodes/node.rs` (330 lines) & `crates/json/src/nodes/accessors.rs` (356 lines): Couples DOM node storage with JSON Pointer (RFC 6901) resolution, JSON Merge Patch (RFC 7396) mutation, string formatting, and type coercions.
  - `crates/bencode/src/nodes/node.rs` (348 lines): Mixes raw bencode token parsing, byte-level serialization, dictionary key canonical sorting, and AST data structures.
  - `crates/xml/src/document.rs` (600 lines): Merges XML tree storage, namespace prefix scoping, compacting, serialization, and XPath 1.0 node bridging.
- **I/O Destinations Conflating Concerns**:
  - `babbel_core::io::Buffer` and crate-level `Buffer` structs act simultaneously as byte accumulators (`IDestination`), UTF-8 decoders (`to_string()`), and data storage vectors (`pub buffer: Vec<u8>`).
  - `FileDestination` mixes OS handle management, byte length tracking, filesystem file recreation (`clear()`), and formatting.
- **Parsers Coupling Scanning, Parsing, and AST Construction**:
  - `crates/yaml/src/parser/document/main_loop.rs`: A single loop mixes character streaming, indentation stack updates, anchor caching, tag resolution, and AST node construction.

---

### 2. O — Open/Closed Principle (OCP)
*Software entities should be open for extension, but closed for modification.*

#### Current Violations:
- **Hardcoded Format Serialization on `babbel_core::model::Value`**:
  - `Value` currently has inherent methods: `serialize_json`, `serialize_yaml`, `serialize_bencode`, `serialize_xml`. Adding a 5th format (such as TOML, MessagePack, or CBOR) forces developers to modify `Value` in `babbel_core`.
  - Serialization should instead be driven by an extensible `FormatSerializer` / `FormatEmitter` trait that external crates or new formats can implement without touching `Value`.
- **Combinatorial $O(N^2)$ Pairwise Conversions in `babbel::convert`**:
  - `crates/babbel/src/convert.rs` defines hardcoded pairs (`json_to_yaml`, `json_to_xml`, `yaml_to_json`, `bencode_to_json`, etc.). For $N=4$ formats, this requires 12 pairwise functions. For $N=5$, it requires 20 functions.
  - Instead, conversions must be orchestrated through an open-ended `FormatCodec` registry or pipeline: `convert(from_format, to_format, input, output)`, scaling as $O(N)$.
- **Closed Visitor Hierarchies**:
  - Each crate defines an ad-hoc, isolated visitor or walker (`BencodeVisitor` in bencode, `walk_node` in yaml, none in json). Adding a new lint, validator, or tree-transformer requires modifying or re-writing format-specific tree walkers.

---

### 3. L — Liskov Substitution Principle (LSP)
*Subtypes or implementations must be substitutable for their base types/traits without altering correctness.*

#### Current Violations:
- **Inconsistent `IDestination::clear()` Semantics**:
  - For `Buffer`: `clear()` resets the internal byte buffer to empty.
  - For `StringDestination`: `clear()` empties the string.
  - For `FileDestination`: In `babbel_core` it is a no-op comment (`// Files cannot easily be cleared without re-opening; no-op`), while in `crates/json/src/io/destinations/file.rs` it deletes and recreates the file from disk via `StdFile::create(&self.file_name)`.
  - A client consuming `&mut dyn IDestination` cannot rely on whether `clear()` actually clears the stream or silently ignores the command.
- **Inconsistent EOF and Reset Behavior Across `ISource` / `IByteStream`**:
  - Calling `next()` past EOF in `SliceSource` keeps returning `None` stably.
  - In `crates/yaml/src/io/sources/file.rs`, CRLF normalization attempts a read-ahead; if a seek fails or EOF is encountered, state behavior diverges from memory sources.
  - In `crates/json/src/io/sources/file.rs`, resetting a file after a seek error maintains stale characters.
- **LSP Solution**:
  - Segregate non-truncatable append-only sinks (`AppendOnlySink`) from clearable buffers (`TruncatableSink` / `Clearable`).
  - Standardize formal EOF, rewind, and error invariants across all `ISource` and `IByteStream` implementations.

---

### 4. I — Interface Segregation Principle (ISP)
*Clients should not be forced to depend upon interfaces that they do not use.*

#### Current Violations:
- **Bloated Stream Traits in `yaml_lib`**:
  - `IIndentationAware` forces implementations to handle tab detection, indentation stacks, and block scalar queries even for simple flat sources.
  - Parsers require monolithic trait bounds `T: ISource + IStatefulStream + IIndentationAware`.
- **Monolithic Node Traits / Bloated APIs**:
  - `Node` provides dozens of methods on one type (`as_bool`, `as_i32`, `as_slice`, `children`, `keys`, `contains_key`, `count_nodes`, `max_depth`, `visit`, `visit_mut`).
  - Clients wanting simple lookup must depend on a type that also bundles AST mutation, graph querying, and serialization.
- **Binary vs Character Stream Conflation**:
  - Binary formats (`bencode`) work with byte streams (`u8`), while text formats (`json`, `yaml`, `xml`) work with Unicode `char`. Coupling them into a single `ISource` creates awkward conversions and unnecessary UTF-8 decoding overhead.
- **ISP Solution**:
  - Granular trait hierarchy:
    - `IByteStream`: Minimal byte reader (`peek_byte`, `read_byte`, `advance`, `has_more`).
    - `ICharStream`: Minimal Unicode character reader (`current`, `next`, `more`).
    - `IPositionAware`: Optional capability for reporting byte offset, line, column.
    - `IRewindable`: Optional capability for seeking or resetting.
    - `IIndentTracker`: Optional capability for YAML-style indentation queries.

---

### 5. D — Dependency Inversion Principle (DIP)
*High-level modules should not depend on low-level modules. Both should depend on abstractions. Abstractions should not depend on details. Details should depend on abstractions.*

#### Current Violations:
- **Facade Depending on Concrete Crate Functions**:
  - `crates/babbel/src/convert.rs` imports and invokes concrete crates directly (`json_lib::from_str`, `yaml_lib::parse_string`, `bencode_lib::parse_bytes`).
  - High-level conversion workflows are directly bound to specific low-level crate functions rather than an abstract `FormatDriver` or `FormatCodec` interface.
- **Degradation of Structured Errors Across Boundaries**:
  - Whenever crates interoperate, they discard typed error enums (`JsonError`, `YamlError`, `BencodeError`, `XmlError`) and downgrade them to raw `String` (`Result<T, String>` or `e.to_string()`).
  - This is a direct violation of DIP: high-level orchestrators cannot inspect error codes, spans, or categories because they depend on arbitrary string representations.
- **DIP Solution**:
  - Introduce `FormatCodec`, `FormatParser`, and `FormatEmitter` abstractions in `babbel_core::codec`.
  - Introduce a unified `BabbelError` / `Diagnostic` abstraction in `babbel_core::error` that preserves line, column, byte offset, error code, and error cause across format boundaries.

---

## Target Architecture

```
                                  +-----------------------+
                                  |     babbel (Facade)   |
                                  | - Dynamic Pipeline    |
                                  | - Unified CLI / API   |
                                  +-----------+-----------+
                                              |
                          Depends on Abstractions, Not Details
                                              |
                                              v
+-----------------------------------------------------------------------------------+
|                                  babbel_core                                      |
|                                                                                   |
|  [codec]               [visitor]             [error]               [io]           |
|  - FormatCodec         - ValueVisitor        - BabbelError         - IByteStream  |
|  - FormatParser        - DynamicWalker       - Diagnostic          - ICharStream  |
|  - FormatEmitter       - TreeFold            - Span, Location      - IPosition    |
|  - CodecRegistry                                                   - IDestination |
|                                                                    - IClearable   |
|  [model]               [escape]              [encoding]            [num]          |
|  - Value (Open DOM)    - Fast Zero-Copy      - BOM Detection       - Fast itoa/   |
|  - ValueRef            - XML / JSON Escapes  - UTF-8 / UTF-16      - dtoa Formats |
+-----------------------------------------------------------------------------------+
         ^                         ^                       ^                   ^
         |                         |                       |                   |
    Implements                Implements              Implements          Implements
    Abstractions              Abstractions            Abstractions        Abstractions
         |                         |                       |                   |
+--------+--------+       +--------+--------+     +--------+-------+  +--------+--------+
|    json_lib     |       |    yaml_lib     |     |  bencode_lib   |  |  xml_lib_rust   |
| - JsonCodec     |       | - YamlCodec     |     | - BencodeCodec |  | - XmlCodec      |
| - JsonParser    |       | - YamlParser    |     | - BencodeParser|  | - XmlParser     |
| - JsonEmitter   |       | - YamlEmitter   |     | - BencodeEmitter  - XmlEmitter   |
| - Decomposed AST|       | - Decomposed AST|     | - Decomposed AST  - Validated DOM|
+-----------------+       +-----------------+     +----------------+  +-----------------+
```

---

## Phased Implementation Roadmap

### Phase 1: Interface Segregation (ISP) & Liskov Contracts (LSP) in `babbel_core::io`

#### Objectives:
1. Break down bloated I/O stream traits into single-purpose capabilities.
2. Formulate explicit contracts for EOF, rewinding, and stream truncation.
3. Segregate append-only sinks from clearable/truncatable destinations.

#### Changes:
- **`crates/babbel_core/src/io/traits.rs`**:
  - `IByteStream`: Minimal byte reading (`peek_byte() -> Option<u8>`, `read_byte() -> Option<u8>`, `advance()`, `has_more() -> bool`).
  - `ICharStream`: Minimal Unicode character reading (`current() -> Option<char>`, `next()`, `more() -> bool`).
  - `IPositionAware`: Optional interface for position reporting (`position() -> usize`, `location() -> Location`).
  - `IRewindable`: Optional interface for seeking/resetting (`reset()`).
  - `IDestination`: Core append-only interface (`add_byte(b: u8)`, `add_bytes(s: &str)`, `last() -> Option<u8>`).
  - `IClearable`: Separate interface for destinations that can be cleared (`clear()`).
- **LSP Guarantees**:
  - All stream implementations (`SliceSource`, `StringSource`, `BufferSource`, `FileSource`) must exhibit identical idempotence past EOF (subsequent calls return `None` without side-effects).
  - All sinks document and guarantee append-only behavior without implicit truncation.

---

### Phase 2: Dependency Inversion (DIP) & Universal Diagnostics in `babbel_core::error`

#### Objectives:
1. Replace brittle `Result<T, String>` boundaries with structured, high-fidelity diagnostics.
2. Invert error reporting dependencies so all formats speak a shared diagnostic vocabulary.

#### Changes:
- **`crates/babbel_core/src/error.rs`**:
  - Define `BabbelError`:
    ```rust
    #[derive(Debug, Clone, PartialEq)]
    pub struct BabbelError {
        pub code: ErrorCode,
        pub message: String,
        pub span: Option<Span>,
        pub source_format: Option<Format>,
        pub cause: Option<String>,
    }
    ```
  - Define `ErrorCode`: `SyntaxError`, `UnexpectedEof`, `InvalidEncoding`, `SchemaValidation`, `UnsupportedType`, `IoError`.
  - Implement `Display`, `std::error::Error`, and conversion from format-specific errors (`JsonError`, `YamlError`, `BencodeError`, `XmlError`).
  - Provide `From` implementations preserving source code `Span` and `Location`.

---

### Phase 3: Open/Closed Principle (OCP) - Pluggable Codec & Conversion Pipeline

#### Objectives:
1. Decouple format serialization from `babbel_core::model::Value`.
2. Convert $O(N^2)$ pairwise conversions in `babbel::convert` into an $O(N)$ extensible codec pipeline.
3. Allow adding new formats without modifying existing ASTs or converters.

#### Changes:
- **`crates/babbel_core/src/codec.rs`** (NEW):
  ```rust
  pub trait FormatParser {
      fn parse_str(&self, input: &str) -> Result<Value, BabbelError>;
      fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError>;
  }

  pub trait FormatEmitter {
      fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError>;
      fn emit_pretty(&self, value: &Value, dest: &mut dyn IDestination, indent: usize) -> Result<(), BabbelError>;
  }

  pub trait FormatCodec: FormatParser + FormatEmitter {
      fn format(&self) -> Format;
  }
  ```
- **`crates/babbel_core/src/model.rs`**:
  - Deprecate hardcoded format methods on `Value` (`serialize_json`, etc.) in favor of `val.emit(&serializer, dest)`.
  - Implement `Value::emit<E: FormatEmitter>(&self, emitter: &E, dest: &mut dyn IDestination)`.
- **`crates/babbel/src/convert.rs`**:
  - Implement a generic conversion function:
    ```rust
    pub fn convert<P: FormatParser, E: FormatEmitter>(
        input: &str,
        parser: &P,
        emitter: &E,
    ) -> Result<String, BabbelError> {
        let val = parser.parse_str(input)?;
        let mut dest = Buffer::new();
        emitter.emit(&val, &mut dest)?;
        Ok(dest.to_string())
    }
    ```
  - Maintain all existing helper functions (`json_to_yaml`, `yaml_to_json`, etc.) as thin, 1-line wrappers delegating to `convert()`.

---

### Phase 4: Single Responsibility Principle (SRP) - AST Module Decomposition

#### Objectives:
1. Decompose monolithic node files (`yaml/src/nodes/node.rs`, `json/src/nodes/node.rs`) into single-responsibility submodules.
2. Separate AST data structure definitions from querying, mutation, formatting, and conversion.

#### Changes:
- **`crates/yaml/src/nodes/`**:
  - `types.rs`: Pure AST enum and struct definitions (`Node`, `Numeric`, `QuoteStyle`, `ScalarStyle`).
  - `query.rs`: Hierarchy inspection and lookup (`get`, `get_key`, `children`, `keys`, `contains_key`, `find_first`, `find_all`).
  - `mutate.rs`: Tree modifications (`push`, `set`, `remove`, `merge`).
  - `convert.rs`: Conversions to and from `babbel_core::model::Value` and standard types.
  - `display.rs`: `Display` and `Debug` implementations.
  - `node.rs`: Clean re-exporting façade preserving 100% backward compatibility for all imports.
- **`crates/json/src/nodes/`**:
  - Maintain separation of `pointer.rs` (RFC 6901), `patch.rs` (RFC 7396), `indexing.rs`, and `query.rs`.
  - Clean up `node.rs` so it acts purely as the AST container.
- **`crates/bencode/src/nodes/`**:
  - Separate `sort.rs` (canonical dictionary key sorting) from `node.rs` (data structure).

---

### Phase 5: Extensible Visitor & Universal Serializer Framework

#### Objectives:
1. Provide a unified `FormatVisitor` and `TreeFold` trait in `babbel_core::visitor`.
2. Enable format-agnostic tree algorithms (e.g. depth calculation, node count, schema validation, key normalization) to be written once and run across JSON, YAML, Bencode, and XML.

#### Changes:
- **`crates/babbel_core/src/visitor.rs`** (NEW):
  - Define `ValueVisitor`:
    ```rust
    pub trait ValueVisitor<R = ()> {
        fn visit_null(&mut self) -> R;
        fn visit_bool(&mut self, value: bool) -> R;
        fn visit_integer(&mut self, value: i128) -> R;
        fn visit_float(&mut self, value: f64) -> R;
        fn visit_string(&mut self, value: &str) -> R;
        fn visit_array(&mut self, items: &[Value]) -> R;
        fn visit_object(&mut self, entries: &[(String, Value)]) -> R;
    }
    ```
  - Implement `walk_value<V: ValueVisitor>(val: &Value, visitor: &mut V)`.
- Re-export visitor utilities in crate stringifiers to eliminate redundant tree traversal logic.

---

## Concrete File-by-File Matrix

| File Path | SOLID Role | Refactoring Summary |
|---|---|---|
| `crates/babbel_core/src/io/traits.rs` | **ISP & LSP** | Segregate `IByteStream`, `ICharStream`, `IPositionAware`, `IDestination`, `IClearable`. |
| `crates/babbel_core/src/error.rs` | **DIP** | Introduce `BabbelError`, `ErrorCode`, structured diagnostic reporting and error bridges. |
| `crates/babbel_core/src/codec.rs` | **OCP & DIP** | Add `FormatParser`, `FormatEmitter`, and `FormatCodec` abstractions. |
| `crates/babbel_core/src/visitor.rs` | **SRP & OCP** | Add universal `ValueVisitor` and `walk_value` framework. |
| `crates/babbel_core/src/model.rs` | **OCP** | Transition hardcoded format methods on `Value` to open `FormatEmitter` delegation. |
| `crates/babbel/src/convert.rs` | **OCP & DIP** | Replace combinatorial pairwise match logic with generic `convert()` pipeline. |
| `crates/yaml/src/nodes/types.rs` | **SRP** | Extract pure AST types (`Node`, `Numeric`) from monolithic `node.rs`. |
| `crates/yaml/src/nodes/query.rs` | **SRP & ISP** | Extract tree querying and searching (`find_first`, `find_all`, `contains_key`). |
| `crates/yaml/src/nodes/mutate.rs` | **SRP** | Extract AST tree mutations (`set`, `push`, `merge`). |
| `crates/yaml/src/nodes/node.rs` | **SRP** | Re-export all types and methods preserving 100% public API compatibility. |
| `crates/json/src/nodes/node.rs` | **SRP** | Decouple node definition from pointer, patch, and conversion logic. |
| `crates/bencode/src/nodes/sort.rs` | **SRP** | Extract dictionary key sorting and validation from `node.rs`. |

---

## Verification & Non-Regression Protocol

1. **Zero Public API Breaking Changes**:
   - Every public struct, enum variant, function, and method signature remains accessible in its original namespace.
   - All trait re-exports ensure existing downstream code continues to compile unchanged.
2. **Automated Test Validation**:
   - `cargo test --workspace --jobs 2`: All ~3,000+ tests across `babbel`, `babbel_core`, `json_lib`, `yaml_lib`, `bencode_lib`, and `xml_lib_rust` must pass with 0 failures.
   - All doc tests must compile and pass cleanly.
3. **Benchmarking & Performance**:
   - Interface segregation and codec pipelines must introduce zero allocation overhead on hot parsing/serializing paths.
   - Static dispatch (`impl FormatEmitter`) is favored over dynamic dispatch (`dyn`) wherever performance-critical.
