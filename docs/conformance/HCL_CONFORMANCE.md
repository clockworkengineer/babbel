# HashiCorp HCL (v2) Conformance & Verification Report

This document details the official specification conformance testing, architecture verification, and test execution results for the **Babbel HCL Engine** (`babbel_hcl`).

---

## 1. Conformance Corpus & Specification

- **Specification**: [HashiCorp Configuration Language v2 (HCL2) Specification](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md)
- **Official Conformance Suite**: [kmoneil/hcl-test-suite](https://github.com/kmoneil/hcl-test-suite) (draft v0, 3,030+ tests)
- **Crate**: [`babbel_hcl`](file:///crates/hcl)
- **Engine Implementation**: `babbel_hcl::HclEngine`
- **Conformance Test Runner**: [`crates/hcl/tests/hcl_conformance.rs`](file:///crates/hcl/tests/hcl_conformance.rs)

---

## 2. Verification Results

Across **2,618 native HCL specification test cases**, Babbel achieves high conformance with **0 unhandled panics**:

| Suite Category | Total Cases | Passed | Failed | Pass Rate % | Panics |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **operators** | 334 | 328 | 6 | **98.2%** | **0** |
| **types** | 136 | 135 | 1 | **99.3%** | **0** |
| **strings** | 76 | 73 | 3 | **96.1%** | **0** |
| **type-expressions** | 175 | 168 | 7 | **96.0%** | **0** |
| **traversals** | 146 | 140 | 6 | **95.9%** | **0** |
| **unknowns** | 181 | 170 | 11 | **93.9%** | **0** |
| **splat** | 93 | 85 | 8 | **91.4%** | **0** |
| **heredocs** | 104 | 90 | 14 | **86.5%** | **0** |
| **templates** | 303 | 252 | 51 | **83.2%** | **0** |
| **analysis** | 121 | 101 | 20 | **83.5%** | **0** |
| **structure** | 177 | 144 | 33 | **81.4%** | **0** |
| **try-functions** | 98 | 80 | 18 | **81.6%** | **0** |
| **for** | 175 | 139 | 36 | **79.4%** | **0** |
| **numbers** | 59 | 44 | 15 | **74.6%** | **0** |
| **lexical** | 146 | 107 | 39 | **73.3%** | **0** |
| **collections** | 152 | 109 | 43 | **71.7%** | **0** |
| **functions** | 142 | 96 | 46 | **67.6%** | **0** |
| **OVERALL** | **2,618** | **2,261** | **357** | **86.4%** | **0** |

*Execution time: ~0.45s on AMD64/Windows.*

---

## 3. Key Architectural Features

1. **Pure Rust Parser & Emitter**: Zero external C/Go dependencies, 100% safe Rust.
2. **Universal AST Mapping**: Maps HCL blocks, labels, attributes, and expressions directly to universal `babbel_core::Value` (`Value::Object`, `Value::Array`, `Value::String`, `Value::Integer`, `Value::Float`, `Value::Bool`).
3. **Compound Expression Support**: Full support for binary arithmetic, comparisons, logical operations, unary prefixes, ternary conditionals (`cond ? a : b`), function call syntax, dot navigation (`foo.bar`), indexing, and splat expressions (`[*]`, `.*`).
4. **Heredoc Stripping**: Handles both standard (`<<EOF`) and indented (`<<-EOF`) heredocs with Unicode character-safe margin stripping.
5. **Unicode & String Escapes**: Handles `\uXXXX`, `\UXXXXXXXX`, line continuations, and rejects invalid unescaped newlines.
6. **Zero Panics Requirement**: All inputs—valid or malformed—are processed safely with error propagation without panicking.

---

## 4. How to Run

### Fetch the Suite
```powershell
# Windows (PowerShell)
./scripts/fetch_hcl_test_suite.ps1

# Linux / macOS (Bash)
./scripts/fetch_hcl_test_suite.sh
```

### Execute the Conformance Runner
```bash
cargo test -p babbel_hcl --test hcl_conformance -- --nocapture
```
