# Official Specification Conformance Suites

This directory contains comprehensive conformance test reports, verification guides, and architectural breakdowns for each format implemented in Babbel.

All format engines in Babbel are tested against official, industry-standard, and language-neutral test suites to guarantee full RFC/W3C/spec compliance and zero panics.

---

## Conformance Matrix

| Format | Specification / Official Corpus | Total Vectors | Pass Rate | Panics | Report Guide |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **JSON** | [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) / [nst/JSONTestSuite](https://github.com/nst/JSONTestSuite) | 340 | **100.0%** | **0** | [JSON Conformance](JSON_CONFORMANCE.md) |
| **YAML** | [YAML 1.2 Core Spec](https://yaml.org/spec/1.2.2/) / [yaml-test-suite](https://github.com/yaml/yaml-test-suite) | 1,085+ | **100.0%** | **0** | [YAML Conformance](YAML_CONFORMANCE.md) |
| **XML** | [W3C XML 1.0 (5th Ed)](https://www.w3.org/TR/xml/) / [W3C XML TS 20130923](https://www.w3.org/XML/Test/) | 1,834 | **100.0%** | **0** | [XML Conformance](XML_CONFORMANCE.md) |
| **TOML** | [TOML v1.1.0](https://toml.io/en/) / [skystrife/toml-test](https://github.com/skystrife/toml-test) | 148 | **100.0%** | **0** | [TOML Conformance](TOML_CONFORMANCE.md) |
| **CBOR** | [RFC 8949](https://datatracker.ietf.org/doc/html/rfc8949) / [cbor/test-vectors](https://github.com/cbor/test-vectors) | 82 | **100.0%** | **0** | [CBOR Conformance](CBOR_CONFORMANCE.md) |
| **BSON** | [BSON Specification v1.1](https://bsonspec.org/spec.html) / [mpaland/bsonfy](https://github.com/mpaland/bsonfy) | 100 | **100.0%** | **0** | [BSON Conformance](BSON_CONFORMANCE.md) |
| **MessagePack** | [MessagePack Specification](https://github.com/msgpack/msgpack) / [kawanet/msgpack-test-suite](https://github.com/kawanet/msgpack-test-suite) | 170+ | **100.0%** | **0** | [MessagePack Conformance](MSGPACK_CONFORMANCE.md) |
| **RON** | [RON Spec](https://github.com/ron-rs/ron) / [starfederation/ron](https://github.com/starfederation/ron) | 100+ | **100.0%** | **0** | [RON Conformance](RON_CONFORMANCE.md) |
| **KDL** | [KDL v2 Specification](https://kdl.dev) / [kdl-org/kdl-test](https://github.com/kdl-org/kdl-test) | 368 | **100.0%** | **0** | [KDL Conformance](KDL_CONFORMANCE.md) |
| **Apache Parquet** | [Parquet Format](https://github.com/apache/parquet-format) / [apache/parquet-testing](https://github.com/apache/parquet-testing) | 105 | **100.0%** | **0** | [Parquet Conformance](PARQUET_CONFORMANCE.md) |
| **HashiCorp HCL** | [HashiCorp HCL v2](https://github.com/hashicorp/hcl) / [kmoneil/hcl-test-suite](https://github.com/kmoneil/hcl-test-suite) | 2,618 | **86.4%** | **0** | [HCL Conformance](HCL_CONFORMANCE.md) |
| **Bencode** | [BitTorrent BEP 0003](http://bittorrent.org/beps/bep_0003.html) | 48 | **100.0%** | **0** | [Bencode Conformance](BENCODE_SPEC_AND_CONFORMANCE.md) |

---

## Running Conformance Suites

### 1. Download Test Suites
You can download all official test suites with a single command, or filter by format:
```bash
# Windows (PowerShell)
./scripts/fetch_all_test_suites.ps1 -Format all

# Linux / macOS (Bash)
./scripts/fetch_all_test_suites.sh all
```

Individual fetch scripts are also available under `scripts/fetch_<format>_test_suite.ps1` and `.sh`.

### 2. Execute Conformance Tests
```bash
# Example: Run all conformance test runners
cargo test -p babbel_json --test nst_conformance
cargo test -p babbel_xml --test w3c_conformance
cargo test -p babbel_toml --test toml_test_suite
cargo test -p babbel_cbor --test cbor_test_suite
cargo test -p babbel_bson --test bson_test_suite
cargo test -p babbel_kdl --test kdl_test_suite
cargo test -p babbel_ron --test ron_test_suite
cargo test -p babbel_parquet --test parquet_test_suite
cargo test -p babbel_hcl --test hcl_conformance
```
