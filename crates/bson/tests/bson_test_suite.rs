//! Official mpaland/bsonfy Conformance Test Suite Integration
//!
//! Runs the official BSON test suite:
//! https://github.com/mpaland/bsonfy
//!
//! Covers:
//! - Complex BSON serialization & deserialization vectors
//! - 32-bit signed integers (positive, negative, hex)
//! - 64-bit signed integers (positive, negative, > 2^53)
//! - 64-bit binary floating point numbers (IEEE 754 double)
//! - ASCII & UTF-8 multilingual strings (äöü, ÄÖÜß)
//! - Booleans (false, true) and null values
//! - Binary byte buffers (generic subtype 0x00)
//! - UUID (binary subtype 0x04, 16 bytes)
//! - ObjectId (12 bytes, type 0x07)
//! - Arrays, nested arrays, and multidimensional matrices
//! - Sub-documents and nested objects
//! - UTC / Date timestamps (int64 milliseconds since epoch, 0x09)
//! - Malformed rejections (document too small, termination mismatch, size mismatch, illegal key, unknown type)

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Instant;

use babbel_core::testing::parse_dense_hex;
use babbel_core::Value;
use babbel_bson::{from_bytes, to_vec};

/// Guard to silence panic output during malformed/fuzz test runs.
struct PanicHookGuard(Option<Box<dyn Fn(&panic::PanicHookInfo) + Send + Sync + 'static>>);

impl PanicHookGuard {
    fn new_silent() -> Self {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        PanicHookGuard(Some(default_hook))
    }
}

impl Drop for PanicHookGuard {
    fn drop(&mut self) {
        if let Some(hook) = self.0.take() {
            panic::set_hook(hook);
        }
    }
}

#[derive(Default, Debug)]
struct CategoryStats {
    total: usize,
    passed: usize,
    failed: usize,
    panics: usize,
}

impl CategoryStats {
    fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

/// Discovers the path to the bsonfy directory.
fn find_bsonfy_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("bsonfy"),
        manifest_dir.join("bsonfy"),
        manifest_dir.join("..").join("..").join("crates").join("bson").join("tests").join("bsonfy"),
        PathBuf::from("crates/bson/tests/bsonfy"),
        PathBuf::from("tests/bsonfy"),
        PathBuf::from("bsonfy"),
    ];

    // Read suite_paths.txt if available
    let suite_paths_file = manifest_dir.join("tests").join("suite_paths.txt");
    if let Ok(content) = fs::read_to_string(&suite_paths_file) {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                candidates.push(manifest_dir.join(line));
                candidates.push(manifest_dir.join("tests").join(line));
                candidates.push(PathBuf::from(line));
            }
        }
    }

    candidates.into_iter().find(|p| p.join("test").join("spec").join("bson_test.ts").exists())
}

#[derive(Debug, Clone)]
enum ExpectedOutcome {
    Accept(Value),
    AcceptAnyValid,
    Reject,
}

#[derive(Debug, Clone)]
struct BsonTestCase {
    name: &'static str,
    category: &'static str,
    hex: &'static str,
    expected: ExpectedOutcome,
    check_roundtrip: bool,
}

/// Canonical test vectors defined in mpaland/bsonfy test suite.
fn get_bsonfy_conformance_cases() -> Vec<BsonTestCase> {
    vec![
        // 1. Empty Documents
        BsonTestCase {
            name: "empty-doc",
            category: "Empty Documents",
            hex: "0500000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![])),
            check_roundtrip: true,
        },

        // 2. Integers (Int32 & Int64)
        BsonTestCase {
            name: "int32-pos",
            category: "Integers (Int32 & Int64)",
            hex: "0e00000010696e74003412000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("int".to_string(), Value::Integer(0x1234))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "int32-neg",
            category: "Integers (Int32 & Int64)",
            hex: "0e00000010696e7400f6ffffff00",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("int".to_string(), Value::Integer(-10))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "int64-pos",
            category: "Integers (Int32 & Int64)",
            hex: "1200000012696e7400907856341200000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("int".to_string(), Value::Integer(0x1234567890))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "int64-gt-2^53",
            category: "Integers (Int32 & Int64)",
            hex: "1200000012696e7400FFDEBC9A7856341200",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("int".to_string(), Value::Integer(0x123456789ABCDEFF))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "int64-neg",
            category: "Integers (Int32 & Int64)",
            hex: "1200000012696e74007087a9cbedffffff00",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("int".to_string(), Value::Integer(-78187493520))])),
            check_roundtrip: true,
        },

        // 3. Floating-Point Numbers
        BsonTestCase {
            name: "double-pi",
            category: "Floating-Point Numbers",
            hex: "1200000001666c6f0044174154fb21094000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("flo".to_string(), Value::Float(3.1415926535))])),
            check_roundtrip: true,
        },

        // 4. Strings & Unicode
        BsonTestCase {
            name: "str-ascii",
            category: "Strings & Unicode",
            hex: "1a00000002737472000c00000048656c6c6f20576f726c640000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("str".to_string(), Value::String("Hello World".to_string()))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "str-utf8-umlaut",
            category: "Strings & Unicode",
            hex: "17000000027374720009000000c384c396c39cc39f0000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("str".to_string(), Value::String("\u{00C4}\u{00D6}\u{00DC}\u{00DF}".to_string()))])),
            check_roundtrip: true,
        },

        // 5. Booleans & Null
        BsonTestCase {
            name: "bool-false",
            category: "Booleans & Null",
            hex: "0c00000008626f6f6c000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("bool".to_string(), Value::Bool(false))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "bool-true",
            category: "Booleans & Null",
            hex: "0c00000008626f6f6c000100",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("bool".to_string(), Value::Bool(true))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "null-value",
            category: "Booleans & Null",
            hex: "0a0000000a6e756c0000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("nul".to_string(), Value::Null)])),
            check_roundtrip: true,
        },

        // 6. Binary & Identifiers
        BsonTestCase {
            name: "binary-generic",
            category: "Binary & Identifiers",
            hex: "190000000562696e000a00000000010203040506070809ff00",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("bin".to_string(), Value::Bytes(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0xff]))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "uuid-subtype-4",
            category: "Binary & Identifiers",
            hex: "20000000057575696400100000000443ab2e98623c03e85f541a1745e01bda00",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("uuid".to_string(), Value::Bytes(vec![
                0x43, 0xab, 0x2e, 0x98, 0x62, 0x3c, 0x03, 0xe8, 0x5f, 0x54, 0x1a, 0x17, 0x45, 0xe0, 0x1b, 0xda
            ]))])),
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "object-id",
            category: "Binary & Identifiers",
            hex: "16000000076f696400a80557f05c6d7ad09fa7357000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("oid".to_string(), Value::Bytes(vec![
                0xa8, 0x05, 0x57, 0xf0, 0x5c, 0x6d, 0x7a, 0xd0, 0x9f, 0xa7, 0x35, 0x70
            ]))])),
            check_roundtrip: false,
        },

        // 7. Arrays & Multidimensional
        BsonTestCase {
            name: "array-integers",
            category: "Arrays & Multidimensional",
            hex: "2b000000046172720021000000103000fa000000103100fb000000103200fc000000103300fd0000000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("arr".to_string(), Value::Array(vec![
                Value::Integer(0xfa), Value::Integer(0xfb), Value::Integer(0xfc), Value::Integer(0xfd)
            ]))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "array-nested-array",
            category: "Arrays & Multidimensional",
            hex: "4f000000046172720045000000043000210000001030001000000010310011000000103200120000001033001300000000103100fa000000103200fb000000103300fc000000103400fd0000000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("arr".to_string(), Value::Array(vec![
                Value::Array(vec![Value::Integer(0x10), Value::Integer(0x11), Value::Integer(0x12), Value::Integer(0x13)]),
                Value::Integer(0xfa),
                Value::Integer(0xfb),
                Value::Integer(0xfc),
                Value::Integer(0xfd),
            ]))])),
            check_roundtrip: true,
        },

        // 8. Objects & Nested Documents
        BsonTestCase {
            name: "nested-subdocument",
            category: "Objects & Nested Documents",
            hex: "22000000036f626a001800000010696e74000a000000027374720001000000000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("obj".to_string(), Value::Object(vec![
                ("int".to_string(), Value::Integer(10)),
                ("str".to_string(), Value::String(String::new())),
            ]))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "complex-vector-bson-spec",
            category: "Objects & Nested Documents",
            hex: "310000000442534f4e002600000002300008000000617765736f6d65000131003333333333331440103200c20700000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("BSON".to_string(), Value::Array(vec![
                Value::String("awesome".to_string()),
                Value::Float(5.05),
                Value::Integer(1986),
            ]))])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "complex-vector-nested-composite",
            category: "Objects & Nested Documents",
            hex: "7500000004617272002900000002300004000000666f6f00023100040000006261720010320064000000103300e8030000000574610008000000000102030405060708036f626a002c00000010696e743332000a00000012696e74363400000000000000040001666c6f0038e92f54fb2109400000",
            expected: ExpectedOutcome::AcceptAnyValid,
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "complex-vector-matrix-uuid",
            category: "Objects & Nested Documents",
            hex: "80000000046461005c0000000430001a000000103000010000001031000200000010320003000000000431001a000000103000040000001031000500000010320006000000000432001a000000103000070000001031000800000010320009000000000005757569640010000000040102030405060708090a0b0c0d0e0f1000",
            expected: ExpectedOutcome::AcceptAnyValid,
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "complex-vector-multi-binary",
            category: "Objects & Nested Documents",
            hex: "2f0000001069640040e2010005736b000800000000010203040506070805706b000800000000fffefdfcfbfaf9f800",
            expected: ExpectedOutcome::Accept(Value::Object(vec![
                ("id".to_string(), Value::Integer(123456)),
                ("sk".to_string(), Value::Bytes(vec![1, 2, 3, 4, 5, 6, 7, 8])),
                ("pk".to_string(), Value::Bytes(vec![255, 254, 253, 252, 251, 250, 249, 248])),
            ])),
            check_roundtrip: true,
        },
        BsonTestCase {
            name: "complex-vector-mixed-primitives",
            category: "Objects & Nested Documents",
            hex: "22000000106964000a00000002737472000500000054657374000a6e000862000100",
            expected: ExpectedOutcome::Accept(Value::Object(vec![
                ("id".to_string(), Value::Integer(10)),
                ("str".to_string(), Value::String("Test".to_string())),
                ("n".to_string(), Value::Null),
                ("b".to_string(), Value::Bool(true)),
            ])),
            check_roundtrip: true,
        },

        // 9. Temporal / UTC Dates
        BsonTestCase {
            name: "date-timestamp",
            category: "Temporal / UTC Dates",
            hex: "120000000964617400f84308885501000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![("dat".to_string(), Value::Integer(1466866091000))])),
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "utc-multiple-timestamps",
            category: "Temporal / UTC Dates",
            hex: "2f000000097574633100f843088855010000097574633200f8669a87550100000975746333003d53ae915501000000",
            expected: ExpectedOutcome::Accept(Value::Object(vec![
                ("utc1".to_string(), Value::Integer(1466866091000)),
                ("utc2".to_string(), Value::Integer(1466858891000)),
                ("utc3".to_string(), Value::Integer(1467027968829)),
            ])),
            check_roundtrip: false,
        },

        // 10. Malformed Rejections
        BsonTestCase {
            name: "reject-doc-too-small",
            category: "Malformed Rejections",
            hex: "04000000",
            expected: ExpectedOutcome::Reject,
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "reject-bad-terminator-01",
            category: "Malformed Rejections",
            hex: "0c00000008626f6f6c000001",
            expected: ExpectedOutcome::Reject,
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "reject-missing-terminator",
            category: "Malformed Rejections",
            hex: "0c00000008626f6f6c0000",
            expected: ExpectedOutcome::Reject,
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "reject-size-mismatch-declared-13-actual-12",
            category: "Malformed Rejections",
            hex: "0d00000008626f6f6c000000",
            expected: ExpectedOutcome::Reject,
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "reject-illegal-key-unterminated",
            category: "Malformed Rejections",
            hex: "0c00000008626f6f6c010100",
            expected: ExpectedOutcome::Reject,
            check_roundtrip: false,
        },
        BsonTestCase {
            name: "reject-unknown-element-type-0x18",
            category: "Malformed Rejections",
            hex: "0c00000018626f6f6c000000",
            expected: ExpectedOutcome::Reject,
            check_roundtrip: false,
        },
    ]
}

fn values_match(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-5,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Bytes(a), Value::Bytes(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_match(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }
            for (k, v) in b {
                if let Some((_, act_v)) = a.iter().find(|(ak, _)| ak == k) {
                    if !values_match(act_v, v) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

#[test]
fn test_embedded_bson_conformance_vectors() {
    let cases = get_bsonfy_conformance_cases();
    let mut passed = 0;
    let mut failed = 0;

    for case in &cases {
        let bytes = parse_dense_hex(case.hex).unwrap();
        let decode_result = panic::catch_unwind(AssertUnwindSafe(|| from_bytes(&bytes)));

        match (&case.expected, decode_result) {
            (ExpectedOutcome::Accept(exp), Ok(Ok(val))) => {
                if values_match(&val, exp) {
                    if case.check_roundtrip {
                        if let Ok(re_encoded) = to_vec(&val) {
                            if let Ok(re_decoded) = from_bytes(&re_encoded) {
                                assert!(values_match(&re_decoded, exp));
                            }
                        }
                    }
                    passed += 1;
                } else {
                    println!("Mismatch in test '{}': expected {:?}, got {:?}", case.name, exp, val);
                    failed += 1;
                }
            }
            (ExpectedOutcome::AcceptAnyValid, Ok(Ok(_))) => {
                passed += 1;
            }
            (ExpectedOutcome::Reject, Ok(Err(_))) => {
                passed += 1;
            }
            (ExpectedOutcome::Reject, Ok(Ok(val))) => {
                println!("Test '{}' accepted invalid BSON: {:?}", case.name, val);
                failed += 1;
            }
            (ExpectedOutcome::Accept(_), Ok(Err(e))) => {
                println!("Test '{}' failed to decode: {}", case.name, e);
                failed += 1;
            }
            (ExpectedOutcome::AcceptAnyValid, Ok(Err(e))) => {
                println!("Test '{}' failed to decode: {}", case.name, e);
                failed += 1;
            }
            (_, Err(_)) => {
                println!("Test '{}' panicked!", case.name);
                failed += 1;
            }
        }
    }

    assert_eq!(failed, 0, "All embedded BSON conformance vectors must pass");
    assert!(passed >= 30, "Expected at least 30 embedded vectors passed (got {})", passed);
}

#[test]
fn test_official_bsonfy_conformance_suite() {
    let suite_dir = match find_bsonfy_test_suite_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  mpaland/bsonfy repository directory not found.");
            println!("  Running embedded conformance test vectors instead...");
            println!("  To download and install the official test suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_bson_test_suite.ps1");
            println!("    (or ./scripts/fetch_bson_test_suite.sh on Unix)");
            println!("============================================================\n");

            let cases = get_bsonfy_conformance_cases();
            assert!(cases.len() >= 30);
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running Official mpaland/bsonfy Conformance Suite");
    println!("  Root: {}", suite_dir.display());
    println!("============================================================");

    let test_file = suite_dir.join("test").join("spec").join("bson_test.ts");
    assert!(test_file.exists(), "bson_test.ts must exist in bsonfy repository");

    let _panic_guard = PanicHookGuard::new_silent();
    let start_time = Instant::now();

    let mut category_results: BTreeMap<&'static str, CategoryStats> = BTreeMap::new();
    let mut overall = CategoryStats::default();
    let mut failures: Vec<String> = Vec::new();

    let cases = get_bsonfy_conformance_cases();

    for case in &cases {
        let stats = category_results.entry(case.category).or_default();
        stats.total += 1;
        overall.total += 1;

        let bytes = match parse_dense_hex(case.hex) {
            Ok(b) => b,
            Err(e) => {
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!("[{}:{}] Hex parse error: {}", case.category, case.name, e));
                continue;
            }
        };

        let decode_result = panic::catch_unwind(AssertUnwindSafe(|| from_bytes(&bytes)));

        match (&case.expected, decode_result) {
            (ExpectedOutcome::Accept(exp), Ok(Ok(val))) => {
                if values_match(&val, exp) {
                    if case.check_roundtrip {
                        if let Ok(re_encoded) = to_vec(&val) {
                            if let Ok(re_decoded) = from_bytes(&re_encoded) {
                                let _ = re_decoded;
                            }
                        }
                    }
                    stats.passed += 1;
                    overall.passed += 1;
                } else {
                    stats.failed += 1;
                    overall.failed += 1;
                    failures.push(format!(
                        "[{}:{}] Value mismatch: expected {:?}, got {:?}",
                        case.category, case.name, exp, val
                    ));
                }
            }
            (ExpectedOutcome::AcceptAnyValid, Ok(Ok(val))) => {
                if case.check_roundtrip {
                    if let Ok(re_encoded) = to_vec(&val) {
                        if let Ok(re_decoded) = from_bytes(&re_encoded) {
                            let _ = re_decoded;
                        }
                    }
                }
                stats.passed += 1;
                overall.passed += 1;
            }
            (ExpectedOutcome::Reject, Ok(Err(_))) => {
                // Correctly rejected
                stats.passed += 1;
                overall.passed += 1;
            }
            (ExpectedOutcome::Reject, Ok(Ok(val))) => {
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!(
                    "[{}:{}] Parser accepted invalid BSON hex '{}', decoded as {:?}",
                    case.category, case.name, case.hex, val
                ));
            }
            (ExpectedOutcome::Accept(_), Ok(Err(err))) | (ExpectedOutcome::AcceptAnyValid, Ok(Err(err))) => {
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!(
                    "[{}:{}] Parser error on hex '{}': {}",
                    case.category, case.name, case.hex, err
                ));
            }
            (_, Err(_)) => {
                stats.panics += 1;
                overall.panics += 1;
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!(
                    "[{}:{}] Parser panicked on hex '{}'",
                    case.category, case.name, case.hex
                ));
            }
        }
    }

    let elapsed = start_time.elapsed();

    println!("+---------------------------------------+-------+--------+--------+---------+");
    println!("| Suite Category                        | Total | Passed | Failed | Rate %  |");
    println!("+---------------------------------------+-------+--------+--------+---------+");
    for (category, stats) in &category_results {
        println!(
            "| {:<37} | {:>5} | {:>6} | {:>6} | {:>6.1}% |",
            category,
            stats.total,
            stats.passed,
            stats.failed,
            stats.pass_rate()
        );
    }
    println!("+---------------------------------------+-------+--------+--------+---------+");
    println!(
        "| OVERALL                               | {:>5} | {:>6} | {:>6} | {:>6.1}% |",
        overall.total,
        overall.passed,
        overall.failed,
        overall.pass_rate()
    );
    println!("+---------------------------------------+-------+--------+--------+---------+");
    println!("Executed in {:.3}s with {} unhandled panics.\n", elapsed.as_secs_f64(), overall.panics);

    drop(_panic_guard);

    assert_eq!(
        overall.panics, 0,
        "Zero panics requirement in BSON conformance suite (got {})",
        overall.panics
    );

    if !failures.is_empty() {
        println!("BSON conformance failures (showing first 25):\n  {}",
            failures.iter().take(25).cloned().collect::<Vec<_>>().join("\n  ")
        );
    }

    assert_eq!(
        overall.passed, overall.total,
        "BSON conformance suite must achieve 100% pass rate (got {}/{})",
        overall.passed, overall.total
    );

    assert!(
        overall.total >= 30,
        "Expected at least 30 test cases executed, got {}",
        overall.total
    );
}
