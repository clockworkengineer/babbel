# Concrete Refactor Plan: Input Sources Subsystem & Quality Attributes

## Executive Summary

This document presents a concrete, in-depth architectural refactoring plan for the **Input Sources Subsystem** across the entire Babbel workspace (`crates/babbel_core`, `crates/babbel_json`, `crates/babbel_yaml`, `crates/babbel_bencode`, and `crates/babbel_xml`). 

The analysis cross-references every input source implementation against the **10 Quality Attributes** established in [`notes/attributes.md`](../notes/attributes.md):
1. **Intuitive API Design**
2. **Comprehensive Documentation**
3. **High Reliability**
4. **Performance and Efficiency**
5. **Maintainability**
6. **Flexibility and Customization**
7. **Strong Security**
8. **High Testability**
9. **Compatibility and Portability**
10. **Low Dependency Footprint**

---

## 1. Current State Assessment of Sources

The workspace currently contains fractured, redundant, and divergent implementations of input sources:

```
crates/
├── babbel_core/src/io/
│   ├── traits.rs           (ISource, ICharStream, IByteStream, IPositionAware, ILocationAware, IRewindable)
│   └── sources.rs          (SliceSource<'a>, BufferSource, FileSource, LineReader, PeekableCharStream)
├── bencode/src/io/sources/
│   ├── mod.rs              (Re-exports buffer, file)
│   ├── buffer.rs           (Thin re-export of BufferSource + RewindableRead)
│   └── file.rs             (Thin re-export of FileSource + tests writing to root)
├── json/src/io/sources/
│   ├── mod.rs              (Re-exports buffer, file)
│   ├── buffer.rs           (Thin re-export of BufferSource)
│   └── file.rs             (Thin re-export of FileSource + tests writing to root)
├── yaml/src/io/sources/
│   ├── mod.rs              (Docstrings claim to parse JSON instead of YAML)
│   ├── buffer.rs           (358 lines! Independent Buffer struct, clones entire &[u8] into Vec<u8>)
│   └── file.rs             (481 lines! Reads 1 byte per syscall, panics on unseekable files via unwrap)
└── xml/src/io/
    └── source.rs           (300 lines! XmlSource: allocates String, replaces CRLF with String::replace)
```

---

## 2. Gap Analysis by Quality Attribute

### Attribute 1: Intuitive API Design
- **Issue**: Each format crate exposes different names and paradigms for input sources:
  - `yaml::io::sources::Buffer::new(&[u8])` vs `babbel_core::io::SliceSource::new(&str)` vs `xml::io::source::XmlSource::from_string(&str)`.
  - `xml` exposes `from_string`, `from_bytes`, and `from_bytes_with_encoding`, but does not implement `babbel_core::io::ISource` or `ICharStream`.
  - `yaml` implements local traits `ICharStream`, `IIndentationAware`, and `IStatefulStream` rather than workspace-standard traits.
- **Refactor Goal**:
  - Unify all sources behind common constructor conventions: `from_str(&'a str)`, `from_bytes(&'a [u8])`, `from_reader(R: Read)`, and `from_file(P: AsRef<Path>)`.
  - All character sources implement `babbel_core::io::traits::ICharStream` + `ILocationAware` + `IPositionAware`.
  - All byte sources implement `babbel_core::io::traits::IByteStream` + `IPositionAware`.

### Attribute 2: Comprehensive Documentation
- **Issue**:
  - `crates/yaml/src/io/sources/mod.rs` lines 4, 9, 11 contain copy-pasted comments stating they read "JSON data from memory/disk".
  - Doctests are absent for `SliceSource`, `BufferSource`, `FileSource`, and `XmlSource`.
  - Inconsistent error documentation for I/O failure conditions.
- **Refactor Goal**:
  - Correct all crate-specific doc comments.
  - Add runnable doctests with assertions for all source constructors and streaming methods.
  - Add `# Errors` sections documenting when `FileSource::new` fails or when invalid encodings are encountered.

### Attribute 3: High Reliability
- **Issue**:
  - `crates/yaml/src/io/sources/file.rs` line 37 executes `self.file.seek(SeekFrom::Current(-1)).unwrap()`. If the input stream is non-seekable (e.g. standard input, pipe, socket), the thread panics.
  - `crates/yaml/src/io/sources/file.rs` line 24 executes `self.file.read(&mut byte1).unwrap_or(0)` which silently masks I/O read errors as EOF.
  - `XmlSource::from_string` relies on multiple passes of `String::replace` which allocates repeatedly and can trigger OOM panics on large inputs.
- **Refactor Goal**:
  - Eliminate all `.unwrap()` and `.expect()` calls in `sources` implementations.
  - Propagate I/O errors cleanly via `std::io::Result` or `babbel_core::BabbelError`.
  - Implement buffered peek without reverse seeking (`SeekFrom::Current(-1)`), using an internal 1-to-2 byte lookahead buffer.

### Attribute 4: Performance and Efficiency
- **Issue**:
  - **Single-byte Syscalls**: `crates/yaml/src/io/sources/file.rs` reads exactly one byte per `read(&mut [0u8; 1])` syscall. For a 1 MB file, this invokes 1,000,000 OS system calls!
  - **Eager Heap Allocations**:
    - `crates/yaml/src/io/sources/buffer.rs` takes `&[u8]` and immediately calls `.to_vec()`, cloning the entire buffer into memory.
    - `crates/xml/src/io/source.rs` clones strings and performs full-text string replaces for newline normalization before parsing begins.
- **Refactor Goal**:
  - Wrap all file sources in `std::io::BufReader` with a configurable 8 KB buffer, reducing OS syscall overhead by 99.9%.
  - Make `SliceSource<'a>` the zero-copy standard across YAML, JSON, Bencode, and XML for all in-memory string parsing.
  - Eliminate eager newline normalization by normalizing lazily during character iteration in `SliceSource` and `BufferSource`.

### Attribute 5: Maintainability
- **Issue**:
  - Extreme duplication: 1,139 lines of code across format crates (`yaml`: 839 lines, `xml`: 300 lines) re-implement character position tracking, line/column counting, and buffer indexing that already exist in `babbel_core/src/io/sources.rs` (770 lines).
- **Refactor Goal**:
  - Consolidate all stream tracking logic into `babbel_core::io::sources`:
    - Port `IIndentationAware` and `IStatefulStream` (state save/restore) to `babbel_core::io::traits`.
    - Update `babbel_core::io::sources::BufferSource` and `SliceSource` to implement `IIndentationAware` and `IStatefulStream`.
    - Replace the 839 lines in `crates/yaml/src/io/sources/{buffer.rs, file.rs}` with thin wrappers/re-exports of `babbel_core::io`.
    - Refactor `crates/xml/src/io/source.rs` to wrap `babbel_core::io::SliceSource`.

### Attribute 6: Flexibility and Customization
- **Issue**:
  - Sources only support raw `&[u8]` or file paths.
  - No generic `ReadSource<R: Read>` for streaming from network sockets, stdin, or decompression streams.
  - No configurable line-ending handling (strict LF vs lenient CRLF/CR normalization).
- **Refactor Goal**:
  - Introduce `ReaderSource<R: Read>` in `babbel_core::io::sources` allowing streaming from any `std::io::Read` implementor.
  - Add `SourceOptions` allowing callers to configure:
    - Line-break normalization mode (`Preserve`, `NormalizeLf`).
    - Tab width for column calculation (default: 4 or 8 spaces).
    - Maximum input size limits to avoid unbounded memory consumption.

### Attribute 7: Strong Security
- **Issue**:
  - No bounds or limits on file reading. Calling `FileSource::new` on `/dev/urandom` or an infinite stream/huge file can exhaust host RAM.
  - No path canonicalization or traversal guardrails on file sources.
- **Refactor Goal**:
  - Introduce `with_max_bytes(limit: usize)` on `FileSource` and `ReaderSource`.
  - Fail with `BabbelError::ResourceExhausted` when input exceeds configured security limits (default limit: 64 MB, configurable).
  - Add safe path handling ensuring file paths are validated.

### Attribute 8: High Testability
- **Issue**:
  - `crates/json/src/io/sources/file.rs` and `crates/bencode/src/io/sources/file.rs` create files like `test_json_file_src_...txt` in the root working directory.
  - In concurrent test runs or aborted tests, orphan files pollute the workspace.
- **Refactor Goal**:
  - Rewrite all file source tests to use `std::env::temp_dir()`.
  - Introduce `MockSource` / in-memory `Cursor` tests to test I/O failures (e.g., simulated read errors, partial reads) without touching the filesystem.

### Attribute 9: Compatibility and Portability
- **Issue**:
  - Newline handling differs between `yaml`, `xml`, and `babbel_core`:
    - `babbel_core::SliceSource` advances through CRLF in `read_line_into`.
    - `yaml::Buffer` checks for CRLF in `next()` but increments column unevenly.
    - `xml::XmlSource` replaces CRLF up front with `.replace("\r\n", "\n")`.
  - Windows file paths with backslashes vs Unix slashes.
  - `no_std` environments: `yaml` and `xml` sources force `std` requirement even for in-memory byte/string decoding.
- **Refactor Goal**:
  - Standardize CRLF, LF, and CR recognition across all sources in `babbel_core::io`.
  - Maintain full `no_std + alloc` support for `SliceSource` and `BufferSource`.
  - Ensure path handling works seamlessly on Windows, Linux, and macOS.

### Attribute 10: Low Dependency Footprint
- **Issue**:
  - While sources currently use `std`, care must be taken not to introduce external crates (e.g. `tempfile`, `regex`, `encoding_rs`) when standard library primitives and core abstractions suffice.
- **Refactor Goal**:
  - Keep the entire `sources` subsystem dependency-free (0 external dependencies in `babbel_core`).
  - Rely exclusively on `core`, `alloc`, and `std::io`.

---

## 3. Concrete Step-by-Step Refactor Roadmap

### Phase 1: Core Trait Enhancements in `babbel_core::io`
1. **Extend `babbel_core::io::traits`**:
   - Add `IIndentationAware`:
     ```rust
     pub trait IIndentationAware {
         fn current_indent_level(&self) -> usize;
     }
     ```
   - Add `IStatefulStream`:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub struct StreamState {
         pub pos: usize,
         pub line: usize,
         pub column: usize,
     }

     pub trait IStatefulStream {
         fn save_state(&mut self) -> StreamState;
         fn restore_state(&mut self, state: StreamState);
     }
     ```
   - Standardize `ILocationAware` (1-based line and column).

### Phase 2: High-Performance Primitives in `babbel_core::io::sources`
1. **Enhance `SliceSource<'a>`**:
   - Implement `IIndentationAware`, `IStatefulStream`, `ILocationAware`, `IPositionAware`, `ICharStream`, and `IRewindable`.
   - Add zero-allocation, zero-copy support for UTF-8 slices.
2. **Enhance `BufferSource`**:
   - Implement `IIndentationAware` and `IStatefulStream`.
   - Provide `from_vec(Vec<u8>)` and `from_slice(&[u8])`.
3. **Upgrade `FileSource`**:
   - Wrap internal file with `std::io::BufReader`.
   - Maintain a 2-byte lookahead ring buffer for peeking without `seek(SeekFrom::Current(-1))`.
   - Add safe error reporting (`std::io::Result`) rather than panic or swallowing errors.
   - Add `max_bytes` guard (default 64 MB).
4. **Add `ReaderSource<R: Read>`**:
   - General-purpose streaming source over any `std::io::Read`.

### Phase 3: Unify Format Crates (`yaml`, `json`, `bencode`, `xml`)
1. **Refactor `crates/yaml/src/io/sources/`**:
   - Replace 358 lines of `buffer.rs` with `pub use babbel_core::io::BufferSource as Buffer;` plus zero-copy `SliceSource` adapter.
   - Replace 481 lines of `file.rs` with `pub use babbel_core::io::FileSource as File;`.
   - Correct docstrings in `crates/yaml/src/io/sources/mod.rs` (remove erroneous "JSON" references).
   - Deprecate local `IIndentationAware` and `IStatefulStream` in favor of re-exports from `babbel_core::io`.
2. **Refactor `crates/xml/src/io/source.rs`**:
   - Re-implement `XmlSource` on top of `babbel_core::io::SliceSource` or `BufferSource`.
   - Eliminate upfront `String::replace` allocations for line breaks; normalize on-the-fly during iteration.
   - Implement `ICharStream`, `ILocationAware`, and `IPositionAware` on `XmlSource`.
3. **Refactor `crates/json/src/io/sources/` & `crates/bencode/src/io/sources/`**:
   - Verify re-exports and API consistency.

### Phase 4: Test Suite Isolation & Resilience
1. **Fix File Pollution**:
   - Update `crates/json/src/io/sources/file.rs` to generate test files in `std::env::temp_dir()`.
   - Update `crates/bencode/src/io/sources/file.rs` to generate test files in `std::env::temp_dir()`.
2. **Add Unit & Integration Tests**:
   - Test CRLF, LF, and mixed newline handling across all sources.
   - Test non-seekable streams (pipes/cursors) to verify no panics occur.
   - Test security limit triggers (`max_bytes` limit violation returns error).
   - Test zero-copy behavior (no heap allocation during `SliceSource` iteration).

---

## 4. Verification & Acceptance Criteria

1. **Compilation & Invariants**:
   - `cargo check --workspace --all-targets` passes with 0 warnings.
   - All AST node size assertions in `crates/babbel/tests/size_checks.rs` remain completely unaffected.
2. **Backward Compatibility**:
   - Existing caller syntax (`Buffer::new(...)`, `File::new(...)`, `XmlSource::from_string(...)`) continues to compile with zero breaking changes.
3. **Workspace Cleanliness**:
   - `cargo test --workspace` executes without leaving any temporary files in the repository directory.
4. **Performance Benchmark**:
   - Reading YAML files via `File` shows >10x speedup due to replacing 1-byte syscall reads with `BufReader`.
