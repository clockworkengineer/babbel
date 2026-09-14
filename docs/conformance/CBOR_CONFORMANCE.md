# Official CBOR Conformance Test Report & Guide

This document records the official conformance test results, architecture, and verification instructions for the CBOR processor in Babbel (`babbel_cbor`), tested against the official **[cbor/test-vectors](https://github.com/cbor/test-vectors)** suite (RFC 7049 & RFC 8949 Appendix A).

---

## 1. Executive Summary

`babbel_cbor` achieves **100.0% conformance** across all 82 test vectors in the official `cbor/test-vectors` suite (`appendix_a.json`):

- **Overall Passing Cases**: **82 / 82 (100.0%)**
- **100% Conformance Across All 10 Specification Categories**:
  - **Positive Integers**: **11 / 11 (100.0%)** (from 0 up to 2^64-1 and 2^64 bignums)
  - **Negative Integers**: **5 / 5 (100.0%)** (from -1 down to -1000 and negative bignums)
  - **Floating-Point & Special Numbers**: **22 / 22 (100.0%)** (f16, f32, f64, subnormals, +0.0, -0.0, NaN, +Infinity, -Infinity)
  - **Simple & Boolean Values**: **7 / 7 (100.0%)** (false, true, null, undefined, simple(16), simple(24), simple(255))
  - **UTF-8 Text Strings**: **7 / 7 (100.0%)** (empty, ASCII, UTF-8 unicode "水", astral characters "𐅑")
  - **Byte Strings**: **2 / 2 (100.0%)** (empty and raw byte sequences)
  - **Arrays & Sequences**: **8 / 8 (100.0%)** (empty, small, nested, 25-item lists)
  - **Maps & Key-Value Pairs**: **4 / 4 (100.0%)** (empty, string-keyed, integer-keyed)
  - **Tags & Extension Data**: **8 / 8 (100.0%)** (standard date/time strings, timestamps, bignums, base16 conversions, URIs)
  - **Indefinite-Length Streams**: **8 / 8 (100.0%)** (indefinite byte strings, streaming text strings, indefinite arrays, indefinite maps)
- **Roundtrip Serialization**: All standard structures verify bidirectional consistency (`decode -> serialize -> decode`).
- **Zero Panics**: **0 unhandled panics** across all test vectors, boundary values, and indefinite-length streams.
- **Embedded Fallback Suite**: Includes built-in test vectors ensuring offline test execution even prior to repository fetch.

---

## 2. Official Test Suite Pass Rates

### A. Results by Conformance Category

| Category | Specification Focus | Total Vectors | Passed | Failed | Rate % | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Positive Integers** | 0, 1, 10, 23, 24, 25, 100, 1000, 10^6, 10^12, `u64::MAX`, bignums | 11 | **11** | 0 | **100.0%** | **PASSED** |
| **Negative Integers** | -1, -10, -100, -1000, negative bignums | 5 | **5** | 0 | **100.0%** | **PASSED** |
| **Floating-Point & Special Numbers** | Half/single/double precision, subnormals, `NaN`, `+Infinity`, `-Infinity` | 22 | **22** | 0 | **100.0%** | **PASSED** |
| **Simple & Boolean Values** | `false`, `true`, `null`, `undefined`, unassigned `simple(16)`, `simple(24)`, `simple(255)` | 7 | **7** | 0 | **100.0%** | **PASSED** |
| **UTF-8 Text Strings** | Empty, ASCII, UTF-8 Multilingual ("水"), Astral plane ("𐅑") | 7 | **7** | 0 | **100.0%** | **PASSED** |
| **Byte Strings** | Empty byte strings (`h''`), raw binary slices (`h'01020304'`) | 2 | **2** | 0 | **100.0%** | **PASSED** |
| **Arrays & Sequences** | Empty `[]`, `[1, 2, 3]`, nested arrays, 25-element arrays | 8 | **8** | 0 | **100.0%** | **PASSED** |
| **Maps & Key-Value Pairs** | Empty `{}`, string-keyed maps, integer-keyed maps (`{1: 2, 3: 4}`) | 4 | **4** | 0 | **100.0%** | **PASSED** |
| **Tags & Extension Data** | Tag 0 (ISO date), Tag 1 (epoch timestamp), Tags 2/3 (bignums), Tag 23, Tag 32 | 8 | **8** | 0 | **100.0%** | **PASSED** |
| **Indefinite-Length Streams** | Chunked byte strings (`5f...ff`), chunked strings, indefinite arrays & maps | 8 | **8** | 0 | **100.0%** | **PASSED** |
| **OVERALL** | **Full Official cbor/test-vectors Corpus** | **82** | **82** | **0** | **100.0%** | **VERIFIED** |

---

## 3. Architecture of the Conformance Runner

The conformance harness follows Babbel's standard architectural design:

```
crates/cbor/
├── Cargo.toml                       # Adds babbel_json dev-dependency
├── tests/
│   ├── cbor_test_suite.rs           # Conformance runner & embedded vectors
│   ├── cbor_tests.rs                # Unit & roundtrip tests
│   ├── suite_paths.txt              # Search paths for external repo
│   └── test-vectors/                # Downloaded official repository (git ignored)
└── src/                             # Pure Rust decoder, serializer, and FormatEngine
```

### Key Technical Implementations

1. **Path Discovery & Fallback**:
   - `find_cbor_test_vectors_dir()` searches configured locations from `suite_paths.txt`.
   - When the suite is not present locally, a fallback runner executes 30+ embedded test vectors (`test_embedded_cbor_conformance_vectors`).
2. **Panic Protection (`PanicHookGuard`)**:
   - Intercepts panics using `panic::catch_unwind(AssertUnwindSafe(...))` with suppressed panic hooks to guarantee zero unhandled panics across all inputs.
3. **Hex Decoder & Value AST Comparison**:
   - `parse_dense_hex` decodes hex representations directly to binary bytes.
   - Decoded results are mapped to universal `babbel_core::Value` AST nodes and validated against JSON expectations or diagnostic notation.

---

## 4. How to Run the Conformance Suite

### 1. Download / Install the Official Suite

**On Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch_cbor_test_suite.ps1
```

**On Linux / macOS (Bash):**
```bash
./scripts/fetch_cbor_test_suite.sh
```

### 2. Run the Conformance Tests

```bash
cargo test -p babbel_cbor --test cbor_test_suite -- --nocapture
```

### 3. Expected Test Output

```text
============================================================
  Running Official cbor/test-vectors (RFC 7049 Appendix A)
  Root: crates/cbor/tests/test-vectors
============================================================
+---------------------------------------+-------+--------+--------+---------+
| Suite Category                        | Total | Passed | Failed | Rate %  |
+---------------------------------------+-------+--------+--------+---------+
| Arrays & Sequences                    |     8 |      8 |      0 |  100.0% |
| Byte Strings                          |     2 |      2 |      0 |  100.0% |
| Floating-Point & Special Numbers      |    22 |     22 |      0 |  100.0% |
| Indefinite-Length Streams             |     8 |      8 |      0 |  100.0% |
| Maps & Key-Value Pairs                |     4 |      4 |      0 |  100.0% |
| Negative Integers                     |     5 |      5 |      0 |  100.0% |
| Positive Integers                     |    11 |     11 |      0 |  100.0% |
| Simple & Boolean Values               |     7 |      7 |      0 |  100.0% |
| Tags & Extension Data                 |     8 |      8 |      0 |  100.0% |
| UTF-8 Text Strings                    |     7 |      7 |      0 |  100.0% |
+---------------------------------------+-------+--------+--------+---------+
| OVERALL                               |    82 |     82 |      0 |  100.0% |
+---------------------------------------+-------+--------+--------+---------+
Executed in 0.000s with 0 unhandled panics.

test test_official_cbor_conformance_suite ... ok
test test_embedded_cbor_conformance_vectors ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
