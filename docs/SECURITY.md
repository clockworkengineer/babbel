# Security Policy & Defensive Engineering

The Babbel project takes security and memory safety seriously. As a multi-format serialization library intended to ingest untrusted inputs across networks and filesystems, Babbel employs multi-layered defense-in-depth controls to prevent Denial of Service (DoS), memory exhaustion, and algorithmic complexity attacks.

---

## 1. Threat Model & Mitigations

### 1.1 Memory Exhaustion via Unbounded Length Prefixes
- **Threat**: Binary or length-prefixed formats (such as BitTorrent Bencode: `2147483648:data...`) can specify astronomical allocation lengths, causing immediate Out-of-Memory (OOM) aborts.
- **Defense**: [`babbel_bencode`](../crates/bencode/src/parser/default.rs) enforces a hard **64 MB** ceiling on length-prefixed strings and byte arrays (`MAX_BENCODE_STR_LEN = 64 * 1024 * 1024`). Any length prefix exceeding this bound is rejected immediately prior to buffer allocation with a syntax error.

### 1.2 Billion Laughs & Exponential Entity Expansion
- **Threat**: XML document parsing is vulnerable to exponential entity expansion (Billion Laughs attack) and quadratic blowup through deeply nested parameter entities.
- **Defense**: [`babbel_xml`](../crates/xml/src/parser/mod.rs) imposes strict recursion limits on entity expansion, tracking total expanded characters and bounding entity depth. Documents violating these limits are aborted with `XmlError::SecurityLimitExceeded`.

### 1.3 Unbounded File System Streaming
- **Threat**: Opening infinite streams (such as `/dev/urandom` or multi-gigabyte files) via convenience loaders (`FileSource::open`) can exhaust host RAM.
- **Defense**: [`FileSource`](../crates/babbel_core/src/io/sources.rs) enforces a **64 MB default size limit** via `FileSource::open` and provides `FileSource::open_with_limit(path, max_bytes)` to allow callers to configure custom application thresholds.

### 1.4 Deeply Nested Structures & Stack Overflow
- **Threat**: Maliciously crafted JSON, YAML, or XML payloads with thousands of nested brackets/tags (`[[[[[[[[...]]]]]]]]`) can overflow the operating system call stack.
- **Defense**:
  - `babbel_bencode` provides an iterative stack-based parser (`parse_iterative`) avoiding recursion entirely.
  - `babbel_core::embedded::EmbeddedLimits` allows callers to enforce strict nesting depth checks (default: 16 in embedded mode).
  - All recursive format parsers enforce maximum parse depth checks.

### 1.5 Windows File Handle Contention & Path Traversal
- **Threat**: In concurrent or multi-threaded scenarios, improper file handle management can cause file-locking contention or race conditions.
- **Defense**: File destinations maintain in-memory byte counts and tail inspection, eliminating reverse seeking on file handles. Test suites utilize isolated temporary directories via `std::env::temp_dir()`.

---

## 2. Memory Safety & `#![forbid(unsafe_code)]`

- Core DOM nodes, parsers, and public APIs are implemented in pure, safe Rust.
- Unsafe code is strictly prohibited throughout format parsers and text engines, except for thoroughly reviewed, verified UTF-8 slice conversions on validated byte boundaries in internal zero-copy primitives.
- All node sizes and alignments are continuously validated against regression bounds in `crates/babbel/tests/size_checks.rs`.

---

## 3. Supported Versions

Security updates are actively applied to the latest release of Babbel:

| Version | Supported |
| :--- | :--- |
| `0.1.x` | ✅ Yes |
| `< 0.1.0` | ❌ No |

---

## 4. Reporting a Vulnerability

If you discover a security vulnerability in Babbel, please report it privately rather than opening a public issue on GitHub.

### Reporting Procedure
1. Send an email to the project maintainers or open a [GitHub Security Advisory](https://github.com/clockworkengineer/babbel/security/advisories).
2. Include a detailed description of the vulnerability, steps to reproduce, and a Minimal Working Example (MWE) or payload if available.
3. The project maintainers will acknowledge receipt within 48 hours and coordinate a fix and release timeline before public disclosure.
