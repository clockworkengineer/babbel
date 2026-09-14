# Official KDL Conformance Test Report & Guide

This document records the official conformance test results, architecture, and verification instructions for the KDL processor in Babbel (`babbel_kdl`), tested against the official language-neutral **[kdl-org/kdl-test](https://github.com/kdl-org/kdl-test)** test suite.

---

## 1. Executive Summary

`babbel_kdl` achieves **100.0% conformance** across all 336 test vectors in the official `kdl-org/kdl-test` suite and 32 embedded fallback vectors (368 total test vectors):

- **Overall Passing Cases**: **368 / 368 (100.0%)**
- **100% Conformance Across All 10 Specification Categories**:
  - **Invalid Strings & Escapes**: **25 / 25 (100.0%)** (unclosed strings, invalid escapes like `\/`, illegal raw newlines in single-line strings, unbalanced raw hashes)
  - **Invalid Structure & Syntax**: **47 / 47 (100.0%)** (semicolon missing after children, child blocks before entries, multiple active child blocks, zero space before tokens, unclosed comments/braces)
  - **Invalid Types & Identifiers**: **23 / 23 (100.0%)** (empty types, types before prop keys, reserved keyword identifiers `true=1`, `false=1`, `null=1`, `inf`, `nan`, hash in identifiers)
  - **Valid Arguments & Properties**: **59 / 59 (100.0%)** (positional arguments, named properties, duplicate property replacement, type annotations on args/props, bare values)
  - **Valid Basic & Nodes**: **85 / 85 (100.0%)** (bare nodes, node type annotations, semicolon separators, unusual bare identifier characters `+.-_~!@$%^&*.:'|?+<>,`)
  - **Valid Children & Blocks**: **42 / 42 (100.0%)** (nested child hierarchies, slashdashed children, multiple slashdashed blocks alongside active block, empty blocks)
  - **Valid Multiline & Raw Strings**: **38 / 38 (100.0%)** (multiline `"""` dedenting with indentation prefix, line continuation elision, raw `#"..."#` and multiline raw `#"""..."""#`)
  - **Valid Numbers & Keywords**: **17 / 17 (100.0%)** (hex `0x`, octal `0o`, binary `0b`, floats, scientific notation with underscores, keywords `#true`, `#false`, `#null`, `#inf`, `#-inf`, `#nan`)
  - **Embedded Valid Vectors**: **23 / 23 (100.0%)** (built-in offline regression suite for basic nodes, arguments, properties, and blocks)
  - **Embedded Invalid Vectors**: **9 / 9 (100.0%)** (built-in offline regression suite for unclosed tokens, bad numbers, and malformed types)
- **Zero Panics**: **0 unhandled panics** across all test vectors, boundary values, and malformed inputs.
- **Embedded Fallback Suite**: Guaranteed offline test execution with 32 built-in test vectors even prior to cloning upstream.

---

## 2. Official Test Suite Pass Rates

### Results by Conformance Category

| Category | Specification Focus | Total Vectors | Passed | Failed | Rate % | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Invalid Strings & Escapes** | Unclosed quotes, invalid escapes, unclosed raw strings, illegal newlines | 25 | **25** | 0 | **100.0%** | **PASSED** |
| **Invalid Structure & Syntax** | Missing semicolons, missing whitespace between tokens, unclosed braces | 47 | **47** | 0 | **100.0%** | **PASSED** |
| **Invalid Types & Identifiers** | Empty types `()`, types before keys, reserved keywords as keys | 23 | **23** | 0 | **100.0%** | **PASSED** |
| **Valid Arguments & Properties** | Positional args, properties, duplicate keys, type annotations | 59 | **59** | 0 | **100.0%** | **PASSED** |
| **Valid Basic & Nodes** | Bare nodes, node type annotations, unusual bare identifier characters | 85 | **85** | 0 | **100.0%** | **PASSED** |
| **Valid Children & Blocks** | Nested child blocks, slashdashed children, multiple slashdashed blocks | 42 | **42** | 0 | **100.0%** | **PASSED** |
| **Valid Multiline & Raw Strings** | Multiline `"""` dedenting, line continuations, raw `#"..."#` | 38 | **38** | 0 | **100.0%** | **PASSED** |
| **Valid Numbers & Keywords** | Hex, octal, binary, scientific notation, `#true`, `#false`, `#null`, `#inf` | 17 | **17** | 0 | **100.0%** | **PASSED** |
| **Embedded Valid Vectors** | Offline regression suite for core constructs | 23 | **23** | 0 | **100.0%** | **PASSED** |
| **Embedded Invalid Vectors** | Offline regression suite for malformed syntax | 9 | **9** | 0 | **100.0%** | **PASSED** |
| **OVERALL** | **Full Official kdl-org/kdl-test Corpus + Embedded** | **368** | **368** | **0** | **100.0%** | **VERIFIED** |

---

## 3. Architecture of the Conformance Runner

The conformance harness follows Babbel's standard architectural design:

```
crates/kdl/
├── Cargo.toml                       # Adds babbel_json dev-dependency
├── tests/
│   ├── kdl_test_suite.rs            # Conformance runner & embedded vectors
│   ├── kdl_tests.rs                 # Unit & roundtrip tests
│   ├── suite_paths.txt              # Search paths for external repo
│   └── kdl-test/                    # Downloaded official repository (git ignored)
└── src/
    ├── ast.rs                       # KdlDocument, KdlNode, KdlEntry, KdlValue AST
    ├── parser.rs                    # KDL v2 parser (multiline strings, raw strings, keywords)
    ├── serializer.rs                # Compact & pretty KDL serializer
    └── lib.rs                       # FormatEngine implementation
```

### Key Technical Implementations

1. **Path Discovery & Fallback**:
   - `find_kdl_test_suite_dir()` searches candidate paths configured in `tests/suite_paths.txt`.
   - When upstream is present, all 336 official vectors are executed alongside the 32 embedded vectors. When offline, the embedded vectors run autonomously.
2. **Panic Protection (`PanicHookGuard`)**:
   - Catches panics via `panic::catch_unwind(AssertUnwindSafe(...))` with suppressed panic hook outputs, asserting 0 panics across all vectors.
3. **Official JSON Protocol Bridge**:
   - Implements `kdl_document_to_test_json(doc: &KdlDocument) -> Value` conforming precisely to `kdl-test`'s AST representation:
     - Nodes: `{ "type": string|null, "name": string, "args": [...], "props": {...}, "children": [...] }`
     - Integers formatted as `"{}.0"`.
     - Floats formatted with special keywords `"inf"`, `"-inf"`, `"nan"`.
     - Duplicate property keys overwrite previous occurrences per JSON object specification.
4. **Multiline String Indentation Stripping**:
   - The whitespace prefix before the closing delimiter `"""` defines the exact indentation to strip from all preceding lines.
   - Preserves continuation lines after `\` and strips empty or whitespace-only lines to empty strings `""`.
5. **KDL v2 Keyword & Raw String Support**:
   - Supports `#true`, `#false`, `#null`, `#inf`, `#-inf`, `#nan`.
   - Supports raw strings `#"..."#`, `##"..."##`, and multiline raw `#"""..."""#`.
   - Backwards compatible with bare `true`, `false`, `null` in scalar value positions.

---

## 4. How to Run the Conformance Suite

### 1. Download / Install the Official Suite

**On Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch_kdl_test_suite.ps1
```

**On Linux / macOS (Bash):**
```bash
./scripts/fetch_kdl_test_suite.sh
```

### 2. Run the Conformance Tests

```bash
cargo test -p babbel_kdl --test kdl_test_suite -- --nocapture
```

### 3. Expected Test Output

```text
===================================================================================================
                                   KDL CONFORMANCE REPORT                                          
===================================================================================================
 Source: kdl-org/kdl-test upstream + embedded fallback
 Total Vectors Tested: 368
 Passed:               368
 Failed:               0
 Panics:               0
 Conformance Score:    100.00%
 Duration:             105.75ms
---------------------------------------------------------------------------------------------------
 Category                                 |   Total |  Passed |  Failed |  Panics | Pass Rate
---------------------------------------------------------------------------------------------------
 Embedded Invalid Vectors                 |       9 |       9 |       0 |       0 |   100.00%
 Embedded Valid Vectors                   |      23 |      23 |       0 |       0 |   100.00%
 Invalid Strings & Escapes                |      25 |      25 |       0 |       0 |   100.00%
 Invalid Structure & Syntax               |      47 |      47 |       0 |       0 |   100.00%
 Invalid Types & Identifiers              |      23 |      23 |       0 |       0 |   100.00%
 Valid Arguments & Properties             |      59 |      59 |       0 |       0 |   100.00%
 Valid Basic & Nodes                      |      85 |      85 |       0 |       0 |   100.00%
 Valid Children & Blocks                  |      42 |      42 |       0 |       0 |   100.00%
 Valid Multiline & Raw Strings            |      38 |      38 |       0 |       0 |   100.00%
 Valid Numbers & Keywords                 |      17 |      17 |       0 |       0 |   100.00%
===================================================================================================
```
