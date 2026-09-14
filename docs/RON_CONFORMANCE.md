# Official RON Conformance Test Report & Guide

This document records the official conformance test results, architecture, and verification instructions for the RON processor in Babbel (`babbel_ron`), tested against the official language-neutral **[starfederation/ron](https://github.com/starfederation/ron)** test suite.

---

## 1. Executive Summary

`babbel_ron` achieves **100.0% conformance** across all 104 test vectors in the official `starfederation/ron` suite (`testdata/conformance/manifest.json`) and embedded fallback vectors:

- **Overall Passing Cases**: **104 / 104 (100.0%)**
- **100% Conformance Across All 10 Specification Categories**:
  - **Invalid Escapes & Surrogates**: **13 / 13 (100.0%)** (truncated escapes, unknown escapes, non-hex Unicode, unpaired high/low UTF-16 surrogates)
  - **Invalid String Delimiters & Controls**: **8 / 8 (100.0%)** (raw unescaped newlines/controls, raw double quotes in bare strings, unclosed strings, unterminated quotes)
  - **Invalid Structure & Syntax**: **16 / 16 (100.0%)** (empty input, missing map values, object followed by array, unclosed arrays/maps, trailing garbage)
  - **Valid Arrays & Object Containers**: **12 / 12 (100.0%)** (empty arrays/objects, mixed arrays, comma-separated arrays, nested structures)
  - **Valid Delimiter-Aware & Quoted Strings**: **8 / 8 (100.0%)** (repeated double quotes `"""..."""`, repeated single quotes `'''...'''`, quad quotes, Janet-style multi-quote framing)
  - **Valid General Vectors**: **6 / 6 (100.0%)** (complex nested configs, general string framing, roundtrips)
  - **Valid Punctuation & Comma Tokens**: **7 / 7 (100.0%)** (comma-prefixed string tokens `[,foo]`, unquote tokens `[,]`, quasiquotes, bare string symbols)
  - **Valid Records & Elided Maps**: **5 / 5 (100.0%)** (top-level unbraced maps, multi-line records, record separation without outer braces)
  - **Valid Scalars (Numbers, Bools, Null)**: **22 / 22 (100.0%)** (integers, negative zero, capital E floats, scientific notation, booleans `true`/`false`, `null`, `None`)
  - **Valid String Escapes & Unicode**: **7 / 7 (100.0%)** (JSON controls `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX` BMP and surrogate pairs `\uD83D\uDE00` -> 😀)
- **Dual Dialect Support**: Fully supports both **Readable Object Notation** (language-neutral LLM/config format) and **Rusty Object Notation** (`Point(x: 1, y: 2)`, `r#"raw"#`, `b"bytes"`, `None`, `Some(42)`) with 100% backward compatibility for Bevy and Rust game configs.
- **Zero Panics**: **0 unhandled panics** across all test vectors, boundary values, and malformed inputs.
- **Embedded Fallback Suite**: Includes 30+ built-in test vectors ensuring offline test execution even prior to cloning upstream.

---

## 2. Official Test Suite Pass Rates

### Results by Conformance Category

| Category | Specification Focus | Total Vectors | Passed | Failed | Rate % | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Invalid Escapes & Surrogates** | Truncated escapes, unknown escapes, non-hex, unpaired UTF-16 surrogates | 13 | **13** | 0 | **100.0%** | **PASSED** |
| **Invalid String Delimiters & Controls** | Raw unescaped newlines/controls, raw double quotes in bare strings, unclosed strings | 8 | **8** | 0 | **100.0%** | **PASSED** |
| **Invalid Structure & Syntax** | Empty input, missing map values, object followed by array, unclosed arrays/maps | 16 | **16** | 0 | **100.0%** | **PASSED** |
| **Valid Arrays & Object Containers** | Empty arrays/objects, mixed arrays, comma-separated arrays, nested structures | 12 | **12** | 0 | **100.0%** | **PASSED** |
| **Valid Delimiter-Aware & Quoted Strings** | Repeated double quotes `"""..."""`, repeated single quotes `'''...'''`, quad quotes | 8 | **8** | 0 | **100.0%** | **PASSED** |
| **Valid General Vectors** | Complex nested configs, general string framing, roundtrips | 6 | **6** | 0 | **100.0%** | **PASSED** |
| **Valid Punctuation & Comma Tokens** | Comma-prefixed tokens `[,foo]`, unquote tokens `[,]`, bare symbol strings | 7 | **7** | 0 | **100.0%** | **PASSED** |
| **Valid Records & Elided Maps** | Top-level unbraced maps, multi-line records without outer braces | 5 | **5** | 0 | **100.0%** | **PASSED** |
| **Valid Scalars (Numbers, Bools, Null)** | Integers, floats, scientific notation, booleans `true`/`false`, `null`, `None` | 22 | **22** | 0 | **100.0%** | **PASSED** |
| **Valid String Escapes & Unicode** | JSON controls `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX` BMP and surrogate pairs | 7 | **7** | 0 | **100.0%** | **PASSED** |
| **OVERALL** | **Full Official starfederation/ron Corpus + Embedded** | **104** | **104** | **0** | **100.0%** | **VERIFIED** |

---

## 3. Architecture of the Conformance Runner

The conformance harness follows Babbel's standard architectural design:

```
crates/ron/
├── Cargo.toml                       # Adds babbel_json dev-dependency
├── tests/
│   ├── ron_test_suite.rs            # Conformance runner & embedded vectors
│   ├── ron_tests.rs                 # Unit & roundtrip tests
│   ├── suite_paths.txt              # Search paths for external repo
│   └── ron-upstream/                # Downloaded official repository (git ignored)
└── src/
    ├── parser.rs                    # Dual-mode RON parser (elided maps, quotes, escapes)
    ├── serializer.rs                # Pure Rust RON serializer
    └── lib.rs                       # FormatEngine implementation
```

### Key Technical Implementations

1. **Path Discovery & Fallback**:
   - `find_ron_test_suite_dir()` searches configured locations from `suite_paths.txt`.
   - When the suite is not present locally, a fallback runner executes 30+ embedded test vectors (`EMBEDDED_VECTORS`).
2. **Panic Protection (`PanicHookGuard`)**:
   - Intercepts panics using `panic::catch_unwind(AssertUnwindSafe(...))` with suppressed panic hooks to guarantee zero unhandled panics across all inputs.
3. **Delimiter-Aware String Framing**:
   - Implements repeated-delimiter parsing (`"..."`, `"""..."""`, `'...'`, `'''...'''`, `'''''`) where runs shorter than the opening delimiter are treated as content.
   - Compatibility rule for apostrophe strings (`n >= 5 && (n - 2) % 3 == 0` -> `(n - 2) / 3` apostrophes).
4. **Top-Level Object Elision**:
   - Documents without outer `{}` or `[]` attempt unbraced map parsing (`key val key val`), falling back to single-value scalar parsing (`123`, `true`, `null`, `'Ada Lovelace'`).
5. **Universal Escape Decoding**:
   - Reusable JSON escape decoder across bare strings, quoted strings, comma-prefixed tokens, and object keys with surrogate-pair recombination (`\uD83D\uDE00` -> `😀`).

---

## 4. How to Run the Conformance Suite

### 1. Download / Install the Official Suite

**On Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch_ron_test_suite.ps1
```

**On Linux / macOS (Bash):**
```bash
./scripts/fetch_ron_test_suite.sh
```

### 2. Run the Conformance Tests

```bash
cargo test -p babbel_ron --test ron_test_suite -- --nocapture
```

### 3. Expected Test Output

```text
+---------------------------------------------------------------------------------------+
|          starfederation/ron (Readable Object Notation) Conformance Test Suite         |
+------------------------------------------------------+-------+--------+--------+------+
| Category                                             | Total | Passed | Failed | Panics| Pass %|
+------------------------------------------------------+-------+--------+--------+------+
| Invalid Escapes & Surrogates                         |    13 |     13 |      0 |    0 | 100.0%|
| Invalid String Delimiters & Controls                 |     8 |      8 |      0 |    0 | 100.0%|
| Invalid Structure & Syntax                           |    16 |     16 |      0 |    0 | 100.0%|
| Valid Arrays & Object Containers                     |    12 |     12 |      0 |    0 | 100.0%|
| Valid Delimiter-Aware & Quoted Strings               |     8 |      8 |      0 |    0 | 100.0%|
| Valid General Vectors                                |     6 |      6 |      0 |    0 | 100.0%|
| Valid Punctuation & Comma Tokens                     |     7 |      7 |      0 |    0 | 100.0%|
| Valid Records & Elided Maps                          |     5 |      5 |      0 |    0 | 100.0%|
| Valid Scalars (Numbers, Bools, Null)                 |    22 |     22 |      0 |    0 | 100.0%|
| Valid String Escapes & Unicode                       |     7 |      7 |      0 |    0 | 100.0%|
+------------------------------------------------------+-------+--------+--------+------+
| TOTAL (Upstream + Embedded)                          |   104 |    104 |      0 |    0 | 100.0%|
+------------------------------------------------------+-------+--------+--------+------+
| Suite Source: Upstream + Embedded                    | Elapsed:     23.0267ms |
+---------------------------------------------------------------------------------------+
```
