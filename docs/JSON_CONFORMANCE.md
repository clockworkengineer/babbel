# Official JSONTestSuite (RFC 8259) Conformance Test Report & Guide

This document records the official conformance test results, architecture, and verification instructions for the JSON processor in Babbel (`babbel_json`), tested against the official **[JSONTestSuite](https://github.com/nst/JSONTestSuite)** (created by Nicolas Seriot, author of *Parsing JSON is a Minefield*).

---

## 1. Executive Summary

`babbel_json` achieves **100.0% conformance** across all 340 active test cases of the official JSONTestSuite:

- **Overall Passing Cases**: **340 / 340 (100.0%)**
- **100% Conformance Across All Categories**:
  - `y_` (Must Accept): **95 / 95 (100.0%)**
  - `n_` (Must Reject): **188 / 188 (100.0%)**
  - `i_` (Implementation Defined): **35 / 35 (100.0%)**
  - `test_transform` (Edge Cases & Normalization): **22 / 22 (100.0%)**
- **Zero Panics**: **0 unhandled panics** across all valid, invalid, malformed, and adversarial inputs
- **Zero Memory Leaks & Stack Safety**: Safe recursion depth bounds prevent stack overflow on deeply nested payloads (e.g. 100,000 opening arrays)
- **Execution Time**: Entire 340-case corpus executes in **~2.5 seconds**

---

## 2. Official Test Suite Pass Rates

| Suite Category | Focus Area | Total Cases | Passed | Failed | Rate % | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Parsing (Must Accept - `y_`)** | Valid RFC 8259 documents, unicode surrogates, exponents | 95 | **95** | 0 | **100.0%** | **PASSED** |
| **Parsing (Must Reject - `n_`)** | Malformed tokens, trailing garbage, unescaped control chars | 188 | **188** | 0 | **100.0%** | **PASSED** |
| **Parsing (Implementation Defined - `i_`)** | Ambiguous edge cases (extreme numbers, lone surrogates) | 35 | **35** | 0 | **100.0%** | **PASSED** |
| **Transform (`test_transform/`)** | Key duplicates, escaped nulls, huge numbers | 22 | **22** | 0 | **100.0%** | **PASSED** |
| **OVERALL** | **Full Official JSONTestSuite Corpus** | **340** | **340** | **0** | **100.0%** | **VERIFIED** |

---

## 3. Sub-Suite Details & Standards

### A. Valid Document Acceptance (`test_parsing/y_*.json` - 95/95, 100.0%)
Validates strict compliance with all allowable JSON productions defined in RFC 8259:
- **Unicode Surrogate Pairs**: RFC 8259 §7 compliant decoding of UTF-16 surrogate pairs (`\uD834\uDD1E` -> `𝄞`) into valid UTF-8 scalar values.
- **Scientific Notation & Numbers**: Positive and negative exponents (`0e1`, `0e+1`, `1.0e-5`, `123.456e78`), negative zero (`-0`), and high-precision fractional numbers (`y_number_double_close_to_zero.json`).
- **Whitespace Flexibility**: Allows ASCII whitespace (`0x20`, `0x09`, `0x0A`, `0x0D`) surrounding tokens and inside containers.
- **Allowed String Escapes**: Quotes (`\"`), reverse solidus (`\\`), solidus (`\/`), backspace (`\b`), formfeed (`\f`), newline (`\n`), carriage return (`\r`), tab (`\t`), and hex escapes (`\uXXXX`).

### B. Invalid Document Rejection (`test_parsing/n_*.json` - 188/188, 100.0%)
Rigorous rejection of malformed syntax and grammar violations:
- **Trailing Characters & Garbage**: Disallows trailing tokens or unparsed characters after the root value (`[1]x`, `{}//comment`, `1#`).
- **Number Syntax Constraints**:
  - Rejects leading zeros in integer part (`01`, `-01`).
  - Rejects naked decimals without integer or fractional digits (`2.`, `2.e3`, `-.1`).
  - Rejects non-JSON numeric literals (`+1`, `NaN`, `Infinity`, `0x1F`).
- **String Control Characters**: Rejects unescaped ASCII control codes (`0x00..=0x1F`) inside string literals per RFC 8259 §7.
- **Unpaired / Malformed Surrogates**: Rejects invalid high/low surrogate sequences.
- **Structural Integrity**: Rejects trailing commas (`[1,]`, `{"a":1,}`), unclosed delimiters (`[1, 2`), and missing colons.

### C. Implementation-Defined Handling (`test_parsing/i_*.json` - 35/35, 100.0%)
Exercises implementation-defined areas under RFC 8259:
- **Large Integers & Exponents**: Seamlessly handles numbers exceeding 64-bit integer limits by falling back to floating point representation without memory exhaustion or panics.
- **Deep Nesting**: Configurable recursion limit prevents call stack exhaustion on malicious inputs (e.g. 100,000 opening brackets).

### D. Transformation & Normalization (`test_transform/*.json` - 22/22, 100.0%)
Covers edge cases in object keys, escaped nulls (`\u0000`), duplicate dictionary keys, and large numbers.

---

## 4. Conformance Architecture & Implementation

### A. Surrogate Pair & Unicode Decoder
Implemented within [`crates/json/src/parser/default.rs`](../crates/json/src/parser/default.rs) and [`crates/json/src/parser/validate.rs`](../crates/json/src/parser/validate.rs):
- Detects high surrogate range (`0xD800..=0xDBFF`).
- Expects and validates following `\u` escape with low surrogate (`0xDC00..=0xDFFF`).
- Decodes full scalar value: `0x10000 + (((high - 0xD800) << 10) | (low - 0xDC00))`.
- Correctly rejects unpaired or misplaced surrogates.

### B. RFC 8259 Number Grammar
- Buffers numeric characters in an `ArrayString<256>` with zero heap allocation.
- Enforces grammar:
  ```
  number = [ minus ] int [ frac ] [ exp ]
  int    = zero / ( digit1-9 *DIGIT )
  frac   = decimal-point 1*DIGIT
  exp    = e [ minus / plus ] 1*DIGIT
  ```
- Prevents buffer overflow and panics via bounded `try_push`.

### C. Security & Memory Guarantees
- **Nesting Limits**: Safe default maximum recursion depth (256 levels) preventing stack overflow attacks.
- **UTF-8 Enforcement**: Validates strict UTF-8 input streams per RFC 8259 §8.1.
- **Zero-Allocation Validation**: [`validate_json`](../crates/json/src/parser/validate.rs) verifies document syntax without heap allocations.

---

## 5. How to Run the Conformance Test Suite

### Step 1: Fetch the Official Test Cases

The test fixtures are fetched on demand via git or zip download:

**On Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch_json_test_suite.ps1
```

**On Linux / macOS (Bash):**
```bash
chmod +x scripts/fetch_json_test_suite.sh
./scripts/fetch_json_test_suite.sh
```

### Step 2: Run the Conformance Runner

Execute the automated test harness:

```bash
# Run all 340 test cases with summary table output
cargo test -p babbel_json --test nst_conformance -- --nocapture
```

### Step 3: Run Full Workspace Verification

```bash
# Run all JSON crate tests
cargo test -p babbel_json

# Run all XML conformance tests
cargo test -p babbel_xml --test w3c_conformance -- --nocapture

# Run full workspace validation
cargo test --workspace
```
