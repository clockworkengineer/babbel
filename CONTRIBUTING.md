# Contributing to Babbel

Thank you for your interest in contributing to Babbel! This document outlines our development process, testing workflows, coding standards, and architectural guidelines.

---

## 1. Development Workflow

### Prerequisites
- **Rust Toolchain**: Stable Rust 1.88.0+ or Rust 2024 edition.
- **Cargo**: Standard package manager included with Rust.

### Cloning & Building
```bash
# Clone the repository
git clone https://github.com/clockworkengineer/babbel.git
cd babbel

# Build all workspace crates
cargo check --workspace

# Build all examples
cargo build --examples
```

---

## 2. Testing Standards

Babbel maintains over **3,500 unit, integration, and doc-tests** across all packages.

### Running Workspace Tests
> [!IMPORTANT]
> **Windows File Handling**: When running tests on Windows, always pass `--jobs 2` (e.g., `cargo test --workspace --jobs 2`). This prevents parallel test binaries from competing for OS file locks on temporary test files.

```bash
# Run all workspace tests (unit, integration, and doc-tests)
cargo test --workspace --jobs 2

# Test a specific package
cargo test -p babbel_core --jobs 2
cargo test -p json_lib --jobs 2
cargo test -p bencode_lib --jobs 2
cargo test -p xml_lib_rust --jobs 2
cargo test -p yaml_lib --jobs 2
cargo test -p babbel --jobs 2
```

### Doc-Tests
Every public code snippet in documentation must be valid, compiling Rust code:
```bash
cargo test --workspace --doc --jobs 2
```

---

## 3. Architectural Rules

All contributions must adhere to the design principles outlined in [`ARCHITECTURE.md`](ARCHITECTURE.md):

1. **Strictly DRY I/O Transport**:
   - **Never** write crate-local file readers, file writers, or buffer abstractions.
   - Always reuse and extend `babbel_core::io::traits` (`IByteStream`, `ICharStream`, `ISource`, `IDestination`) and `babbel_core::io` implementations (`BufferSource`, `FileSource`, `Buffer`, `FileDestination`).
2. **Interface Segregation (ISP)**:
   - When writing a parser or validator, bind to the narrowest trait that satisfies your requirement (e.g., accept `&mut impl ICharStream` if you only read forward characters; accept `&mut impl IByteStream` for binary data).
3. **Liskov Substitutability (LSP)**:
   - Do not make assumptions about underlying storage in trait implementors.
   - Character streams must always decode UTF-8 scalars correctly. Multi-byte UTF-8 sequences must never be cast as `b as char`.
   - Destinations must never reopen files from disk during tail inspection (`last()`).
4. **Centralized Diagnostic Errors**:
   - Return structured error types (`BabbelError`, `ErrorCode`) with span and location information.
   - Avoid returning raw error strings or untyped errors.
5. **Zero Unnecessary Allocations**:
   - Use borrowed slices (`&str`, `&[u8]`) where feasible.
   - Use `itoa` and `dtoa` for numeric formatting rather than `format!("{val}")`.

---

## 4. Documentation Standards

1. **Crate READMEs**:
   - Every crate in `crates/` must maintain its own up-to-date `README.md` with:
     - Clear crate overview and target use-cases.
     - `Cargo.toml` dependency snippet with available feature flags.
     - A self-contained, working quickstart example.
2. **Rustdoc Comments**:
   - Public items (`struct`, `enum`, `trait`, `fn`) must have `///` documentation comments.
   - Module and crate roots must have `//!` high-level documentation.
   - Include compile-checked examples where applicable.
3. **Verify Documentation Build**:
   ```bash
   cargo doc --workspace --no-deps --jobs 2
   ```

---

## 5. Pull Request Checklist

Before submitting a pull request, ensure:
- [ ] `cargo check --workspace` succeeds with 0 errors.
- [ ] `cargo test --workspace --jobs 2` passes completely (all ~3,500 tests).
- [ ] `cargo build --examples` builds all example binaries cleanly.
- [ ] `cargo doc --workspace --no-deps --jobs 2` generates without warnings.
- [ ] New features include corresponding unit tests and documentation examples.
- [ ] No duplicated I/O or buffer logic was introduced.
