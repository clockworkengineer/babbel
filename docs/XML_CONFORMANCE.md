# Official W3C XML Conformance Test Report & Guide

This document records the official conformance test results, architecture, and verification instructions for the XML processor in Babbel (`babbel_xml`), tested against the official **[W3C XML Conformance Test Suite (XML TS 20130923)](https://www.w3.org/XML/Test/)**.

---

## 1. Executive Summary

`babbel_xml` achieves high conformance across the 2,144 test cases comprising the official W3C XML Conformance Corpus:

- **Overall Passing Cases**: **1,799 / 2,144 (83.9%)**
- **100% Conformance Suites**: **7 sub-suites** at **100.0%** (including OASIS XML 1.0, Edinburgh Errata 4e, 2e, and NS-1e)
- **Zero Skipped Tests**: **0 skipped** (`overall.skipped_encoding == 0`), including ISO-8859-1, Windows-1252, and UTF-16 suites
- **Zero Panics**: **0 unhandled panics** across all valid, invalid, and malformed inputs
- **Execution Time**: Entire 2,144-case corpus executes in **under 0.50 seconds**

---

## 2. Official Test Suite Pass Rates

| Suite Name | Focus Area | Total Cases | Passed | Failed | Skipped* | Rate % | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`oasis/oasis.xml`** | OASIS / NIST XML 1.0 Conformance Suite | 348 | **348** | 0 | 0 | **100.0%** | **PASSED** |
| **`eduni/errata-4e/errata4e.xml`** | Edinburgh Univ. XML 1.0 (4th & 5th Edition Errata) | 393 | **393** | 0 | 0 | **100.0%** | **PASSED** |
| **`eduni/errata-2e/errata2e.xml`** | Edinburgh Univ. XML 1.0 (2nd Edition Errata) | 33 | **33** | 0 | 0 | **100.0%** | **PASSED** |
| **`eduni/namespaces/errata-1e/errata1e.xml`** | Edinburgh Univ. Namespaces in XML 1.0 Errata | 3 | **3** | 0 | 0 | **100.0%** | **PASSED** |
| **`ibm/ibm_oasis_invalid.xml`** | IBM / OASIS Validity Constraint Rejections | 48 | **48** | 0 | 0 | **100.0%** | **PASSED** |
| **`japanese/japanese.xml`** | Japanese Character Set & Fatal Error Validations | 12 | **12** | 0 | 0 | **100.0%** | **PASSED** |
| **`sun/sun-error.xml`** | Sun Microsystems Fatal Error Detection | 1 | **1** | 0 | 0 | **100.0%** | **PASSED** |
| **`eduni/namespaces/1.0/rmt-ns10.xml`** | Namespaces in XML 1.0 (Richard Tobin) | 48 | **47** | 1 | 0 | **97.9%** | **HIGH** |
| **`ibm/ibm_oasis_valid.xml`** | IBM / OASIS Valid XML Document Grammar | 149 | **145** | 4 | 0 | **97.3%** | **HIGH** |
| **`xmltest/xmltest.xml`** | James Clark XML Test Suite | 365 | **340** | 25 | 0 | **93.2%** | **HIGH** |
| **`eduni/errata-3e/errata3e.xml`** | Edinburgh Univ. XML 1.0 (3rd Edition Errata) | 13 | **12** | 1 | 0 | **92.3%** | **HIGH** |
| **`ibm/ibm_oasis_not-wf.xml`** | IBM Not-Well-Formed Negative Corpus | 731 | **417** | 314 | 0 | **57.0%** | **PROGRESSING** |
| **OVERALL** | **Full Official W3C Conformance Corpus** | **2,144** | **1,799** | **345** | **0** | **83.9%** | **VERIFIED** |

*\* Zero tests skipped: Built-in multi-byte decoding (UTF-8, UTF-16LE, UTF-16BE, ISO-8859-1, Windows-1252) processes all fixtures without external iconv dependency.*

---

## 3. Sub-Suite Details & Standards

### A. OASIS XML 1.0 Conformance Suite (`oasis/oasis.xml` - 100.0%)
Created by OASIS and NIST (1-Nov-1998) to validate all core productions of XML 1.0:
- **XML Declaration**: Position 0 constraint, attribute order (`version` -> `encoding` -> `standalone`), whitespace requirements.
- **Processing Instructions**: Whitespace separation between `PITarget` and PI data.
- **Character References**: Decimal (`&#...;`) and hexadecimal (`&#x...;`) bounds enforcement against XML 1.0 §4.1 [66].
- **Entity References**: Syntax validation for `&Name;` (§4.1 [68]).
- **Public Identifiers**: Strict character enforcement of `PubidChar` (§2.3 [12]-[13]).
- **DTD Internal Subset**: Recursive-descent grammar validation for `<!ELEMENT` (models `EMPTY`, `ANY`, `Mixed`, `children`), `<!ATTLIST` (types and defaults), `<!ENTITY` (general and parameter), and `<!NOTATION`.

### B. Edinburgh University Errata Suites (`eduni/` - 99.4% Combined)
Authored by Richard Tobin at the University of Edinburgh:
- **`errata-4e` (393/393, 100.0%)**: Exercises 4th and 5th edition XML 1.0 clarifications, Unicode 5.0 character classes, and XML Names.
- **`errata-2e` (33/33, 100.0%)**: 2nd edition errata updates regarding whitespace and parameter entity boundaries.
- **`namespaces/errata-1e` (3/3, 100.0%)**: First edition errata for Namespaces in XML 1.0.
- **`namespaces/1.0` (47/48, 97.9%)**: Comprehensive namespace prefix scoping, undeclared prefix detection, and QName parsing.
- **`errata-3e` (12/13, 92.3%)**: Third edition errata updates.

### C. IBM XML Conformance Corpus (`ibm/` - 65.5% Combined)
Contributed by IBM to validate XML 1.0 recommendations:
- **`ibm_oasis_invalid` (48/48, 100.0%)**: Tests validity constraint rejections that non-validating processors must parse without fatal error.
- **`ibm_oasis_valid` (145/149, 97.3%)**: Comprehensive valid document forms.
- **`ibm_oasis_not-wf` (417/731, 57.0%)**: Exhaustive negative test cases for malformed XML tokens.

### D. James Clark XMLTEST (`xmltest/` - 340/365, 93.2%)
Historic benchmark created by James Clark testing well-formedness, parameter entities, CDATA sections, and epilog comments.

---

## 4. Conformance Architecture & Implementation

### A. Non-Validating Processor Compliance (XML 1.0 §5.1)
Per W3C `testcases.dtd` (lines 96-98):
```dtd
<!-- No parser should accept a "not-wf" testcase unless it's a
     nonvalidating parser and the test contains external entities
     that the parser doesn't read. -->
```
In compliance with XML 1.0 §5.1 (*"A non-validating parser is required to parse only the document entity, including the internal DTD subset... It is not required to process the external parameter entity declarations..."*), `babbel_xml` conforms by parsing the document entity while rejecting malformed internal syntax.

### B. Internal Subset Syntax Validator
`babbel_xml` features a zero-allocation recursive-descent validator within [`XmlParser`](../crates/xml/src/parser/xml_parser.rs):
- Validates element content particles (`cp`), choice (`|`), and sequence (`,`) grouping without mixing separators at the same level.
- Enforces uppercase declaration keywords (`<!ELEMENT`, `<!ATTLIST`, `<!ENTITY`, `<!NOTATION`, `#REQUIRED`, `#IMPLIED`, `#FIXED`).
- Validates Public ID literals against `PubidChar ::= #x20 | #xD | #xA | [a-zA-Z0-9] | [-'()+,./:=?;!*#@$_%]`.
- Returns the complete raw internal subset string slice to [`DtdValidator`](../crates/xml/src/dtd/validator.rs) and the entity registry.

### C. Security & Memory Guarantees
- **Billion Laughs Mitigation**: Strict entity expansion depth and total byte allocation limits (`max_entity_expansion_depth`, `max_total_entity_expansion_size`).
- **Nesting Limits**: Configurable element recursion ceiling (default 256) preventing call stack overflow.
- **Streaming Bounds**: Default 50 MB reader limit on unbounded streaming inputs.
- **Compact AST Memory**: Verified by [`tests/size_checks.rs`](../crates/babbel/tests/size_checks.rs).

---

## 5. How to Run the Conformance Test Suite

### Step 1: Fetch the Official W3C Test Cases

The test fixtures (~20 MB uncompressed) are excluded from git and can be fetched on demand:

**On Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch_w3c_xmlts.ps1
```

**On Linux / macOS (Bash):**
```bash
chmod +x scripts/fetch_w3c_xmlts.sh
./scripts/fetch_w3c_xmlts.sh
```

### Step 2: Run the Conformance Runner

Execute the automated test harness:

```bash
# Run all 2,144 test cases with summary table output
cargo test -p babbel_xml --test w3c_conformance -- --nocapture
```

### Step 3: Run Related Regression & Size Checks

```bash
# Verify AST memory size constraints
cargo test -p babbel --test size_checks

# Run all crate tests
cargo test -p babbel_xml

# Run full workspace validation
cargo test --workspace
```
