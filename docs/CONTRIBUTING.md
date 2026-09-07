# Contributing to Babbel

Thank you for your interest in contributing to **Babbel**! Babbel is a high-performance, polyglot serialization, parsing, and document manipulation ecosystem in Rust.

To maintain our high standards of software quality, safety, and performance, all contributions must adhere to the engineering guidelines outlined below.

---

## 1. Architecture & Design Principles

Babbel strictly enforces **DRY** (Don't Repeat Yourself) and **SOLID** engineering principles across its multi-crate workspace.

### Clean Layering Model
1. **Tier 0: Foundational Kernel (`babbel_core`)**:
   - Contains streaming abstractions (`ICharStream`, `IByteStream`, `ISource`, `IDestination`, `ILineReader`), universal `Value` AST, BOM detection, and numeric/error utilities.
   - Must have **zero** dependencies on any format crate (`json_lib`, `yaml_lib`, etc.).
   - Must remain `no_std` + `alloc` compatible when the `std` feature is disabled.
2. **Tier 1: Domain Format Engines (`json_lib`, `yaml_lib`, `xml_lib`, `bencode_lib`)**:
   - Each engine is autonomous and depends on `babbel_core`.
   - Format engines must **never** depend on one another.
3. **Tier 2: Master Facade & Conversion (`babbel`)**:
   - Re-exports format engines and provides the $O(N)$ cross-conversion matrix (`babbel::convert`).

### SOLID Rules
- **Single Responsibility (SRP)**: Keep I/O transport completely decoupled from syntax parsing.
- **Open/Closed (OCP)**: Format additions and custom stream sources/destinations should be open for extension via `FormatParser`, `FormatEmitter`, `ISource`, and `IDestination` without altering existing converters.
- **Liskov Substitution (LSP)**: Stream implementations (`SliceSource`, `BufferSource`, `FileSource`) must preserve UTF-8 scalar decoding invariants and safe tail inspection (`last()`).
- **Interface Segregation (ISP)**: Consume only the minimal trait needed (e.g. `ILineReader`, `IByteStream`, `ITailInspectable`) rather than monolithic interfaces.
- **Dependency Inversion (DIP)**: High-level algorithms depend upon trait abstractions rather than concrete files or memory buffers.

---

## 2. Development Setup & Prerequisites

- **Rust Toolchain**: Rust 1.88.0 or newer (2024 Edition).
  ```bash
  rustup update stable
  ```
- **Git**: Ensure long paths are enabled on Windows if applicable (`git config --global core.longpaths true`).

---

## 3. Building and Testing

Babbel contains extensive test suites with over 3,500 tests.

### Running Tests
> [!IMPORTANT]
> **Windows File Concurrency**: When running tests on Windows, always pass `--jobs 2` to prevent OS file-locking conflicts when cleaning and creating temporary test fixtures.

```bash
# Check all workspace crates
cargo check --workspace

# Run all tests across the workspace
cargo test --workspace --jobs 2

# Run documentation tests
cargo test --workspace --doc --jobs 2

# Run memory struct size assertion benchmarks
cargo test -p babbel --test size_checks

# Test specific crate
cargo test -p babbel_core --jobs 2
cargo test -p json_lib --jobs 2
cargo test -p yaml_lib --jobs 2
cargo test -p xml_lib --jobs 2
cargo test -p bencode_lib --jobs 2
cargo test -p babbel --jobs 2
```

---

## 4. Coding Standards

### Memory & Performance Guidelines
- **Zero-Allocation**: Prefer borrowed slices (`SliceSource`, `read_line_slice`) and in-place buffers (`dtoa`, `itoa`, `SmallVec`) wherever possible.
- **Struct Compaction**: Keep node representation compact. Adhere to the established size bounds:
  - `babbel_core::Value` $\le$ 32 bytes
  - `json_lib::Node` $\le$ 56 bytes
  - `xml_lib::NodeKind` $\le$ 48 bytes
  - `yaml_lib::Node` $\le$ 40 bytes
  - `bencode_lib::Node` $\le$ 56 bytes

### Error Handling
- Return `Result<T, BabbelError>` or format-specific error types that implement `Into<BabbelError>`.
- Always provide structured `ErrorCode` (e.g. `ErrorCode::SyntaxError`, `ErrorCode::UnexpectedEof`, `ErrorCode::UnsupportedType`) and source `Span`/`Location` where available.

### Documentation & Comments
- Maintain documentation integrity: preserve all existing comments and docstrings unless intentionally modifying the underlying behavior.
- Every public function, struct, enum, and trait must have doc-comments with clear markdown explanations and executable `# Examples` doctests.

---

## 5. Submitting Pull Requests

1. **Create a Feature Branch**:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. **Commit Messages**: Follow Conventional Commits:
   - `feat: add RFC 4180 CSV streaming emitter`
   - `fix: correctly handle CRLF newlines in line reader`
   - `perf: compact AST node memory layout`
   - `docs: update conversion matrix guide`
3. **Verify Locally**:
   Ensure `cargo check --workspace`, `cargo test --workspace --jobs 2`, and `cargo test --workspace --doc --jobs 2` pass with **zero warnings** and **zero errors**.
4. **Submit PR**: Open a pull request against the `master` branch with a clear description of the changes and test results.
