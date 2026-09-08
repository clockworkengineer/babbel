# Babbel Developer & Contributor Guide

This guide covers developer workflows, toolchain prerequisites, build optimization, memory profiling, and testing practices across the **Babbel** workspace.

---

## 1. Prerequisites & Environment Setup

### 1.1 Rust Toolchain
Babbel uses Rust **2024 Edition** across all crates. A toolchain of Rust 1.88.0 or newer is required:

```bash
rustup update stable
rustup default stable
```

### 1.2 Git Configuration (Windows Long Paths)
On Windows, certain test fixture paths and deep directories may exceed the legacy 260-character MAX_PATH limit. Ensure long paths are enabled:

```bash
git config --global core.longpaths true
```

---

## 2. Workspace Layout & Architectural Boundaries

```
babbel/
├── crates/
│   ├── babbel/         # Tier 2: Master facade & conversion pipeline
│   ├── babbel_core/    # Tier 0: Kernel (I/O, AST, CSV, INI, embedded)
│   ├── json/           # Tier 1: babbel_json (DOM, pointer, patch, lines)
│   ├── yaml/           # Tier 1: babbel_yaml (YAML 1.2, anchors, tags)
│   ├── bencode/        # Tier 1: babbel_bencode (BitTorrent, borrowed DOM)
│   └── xml/            # Tier 1: babbel_xml (W3C DOM, C14N, DTD, XSD, XPath)
└── docs/               # Technical specifications, guides, and plans
```

### Architectural Dependency Invariant:
1. `babbel_core` must **never** depend on any format crate (`json`, `yaml`, `xml`, `bencode`).
2. Domain format crates must **never** depend on each other.
3. Cross-format conversions must flow through `babbel::convert` via `babbel_core::model::Value` or codecs.

---

## 3. Daily Development Workflow

### 3.1 Fast Compilation Check
```bash
# Verify entire workspace with all targets (benchmarks, examples, tests)
cargo check --workspace --all-targets
```

### 3.2 Running the Full Test Suite
Babbel features over 3,500 tests across its crates:

```bash
# Run all unit and integration tests
cargo test --workspace

# Run executable doctests
cargo test --workspace --doc
```

### 3.3 Verifying AST Memory Size Invariants
Babbel continuously enforces tight byte size bounds on AST nodes:

```bash
cargo test -p babbel --test size_checks
```

The size thresholds are:
- `babbel_core::Value` $\le$ **32 bytes**
- `babbel_yaml::Node` $\le$ **40 bytes**
- `babbel_xml::NodeKind` $\le$ **48 bytes**
- `babbel_json::Node` $\le$ **56 bytes**
- `babbel_bencode::Node` $\le$ **56 bytes**

Any modification that increases the byte size of an AST node will trigger a test failure in `size_checks.rs`.

### 3.4 Official Conformance Test Suites
Babbel supports running official specification test suites for XML and YAML:

```bash
# Download the official W3C XML Conformance Test Suite (XML TS 20130923)
powershell -ExecutionPolicy Bypass -File scripts/fetch_w3c_xmlts.ps1
# On Linux/macOS:
# ./scripts/fetch_w3c_xmlts.sh

# Run the 2,500+ test case W3C XML conformance runner
cargo test -p babbel_xml --test w3c_conformance -- --nocapture

# Run the official YAML 1.2 test suite (if cloned)
cargo test -p babbel_yaml --test yaml_test_suite -- --nocapture
```

---

## 4. Linting & Formatting

Before opening a pull request, verify that code meets formatting and linting standards:

```bash
# Format check
cargo fmt --all -- --check

# Clippy lints
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 5. Release Optimization & LTO Profiling

For performance benchmarking and minimal binary footprint, build with Link Time Optimization (LTO):

```bash
# Build release binaries with thin LTO
cargo build --release --workspace

# Run benchmarks
cargo bench --workspace
```
