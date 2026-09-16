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

Across **2,228 native HCL specification test cases**, Babbel achieves **100.0% conformance** across **all 17 categories** with **0 unhandled panics**:

| Suite Category | Total Cases | Passed | Failed | Pass Rate % | Panics |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **analysis** | 95 | 95 | 0 | **100.0%** | **0** |
| **collections** | 128 | 128 | 0 | **100.0%** | **0** |
| **for** | 154 | 154 | 0 | **100.0%** | **0** |
| **functions** | 111 | 111 | 0 | **100.0%** | **0** |
| **heredocs** | 78 | 78 | 0 | **100.0%** | **0** |
| **lexical** | 133 | 133 | 0 | **100.0%** | **0** |
| **numbers** | 52 | 52 | 0 | **100.0%** | **0** |
| **operators** | 301 | 301 | 0 | **100.0%** | **0** |
| **splat** | 71 | 71 | 0 | **100.0%** | **0** |
| **strings** | 74 | 74 | 0 | **100.0%** | **0** |
| **structure** | 161 | 161 | 0 | **100.0%** | **0** |
| **templates** | 252 | 252 | 0 | **100.0%** | **0** |
| **traversals** | 142 | 142 | 0 | **100.0%** | **0** |
| **try-functions** | 83 | 83 | 0 | **100.0%** | **0** |
| **type-expressions** | 149 | 149 | 0 | **100.0%** | **0** |
| **types** | 102 | 102 | 0 | **100.0%** | **0** |
| **unknowns** | 142 | 142 | 0 | **100.0%** | **0** |
| **OVERALL** | **2,228** | **2,228** | **0** | **100.0%** | **0** |

*Execution time: ~0.39s on AMD64/Windows.*

---

## 3. Key Architectural Features

1. **Pure Rust Parser & Emitter**: Zero external C/Go dependencies, 100% safe Rust.
2. **Universal AST Mapping**: Maps HCL blocks, labels, attributes, and expressions directly to universal `babbel_core::Value` (`Value::Object`, `Value::Array`, `Value::String`, `Value::Integer`, `Value::Float`, `Value::Bool`).
3. **Compound Expression Support**: Full support for binary arithmetic, comparisons, logical operations, unary prefixes, ternary conditionals (`cond ? a : b`), function call syntax, dot navigation (`foo.bar`), indexing, and splat expressions (`[*]`, `.*`).
4. **Full Template & Directive Engine**: Validates all `${...}` interpolations and `%{if ...}`, `%{for ...}`, `%{else}`, `%{endif}`, `%{endfor}` directives, with nesting depth checks, strip markers (`~`), and escape sequences (`$${`, `%%{`).
5. **Heredoc Stripping & Validation**: Handles standard (`<<EOF`), indented (`<<-EOF`), CRLF line endings, and template expressions within heredocs.
6. **UAX #31 & Unicode Compliance**: Correctly validates identifiers per Unicode Annex #31 (including Roman numerals, Arabic digits, and excluding Pattern_Syntax / Other Numbers).
7. **Strict Structure Validation**: Enforces one-line and multi-line block syntax invariants, attribute redefinition guards, and terminator restrictions.
8. **Zero Panics Requirement**: All inputs—valid or malformed—are processed safely with error propagation without panicking.

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
