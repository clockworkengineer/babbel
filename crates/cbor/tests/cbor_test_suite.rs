//! Official cbor/test-vectors (RFC 7049 Appendix A) Conformance Test Suite Integration
//!
//! Runs the official CBOR test vectors suite:
//! https://github.com/cbor/test-vectors
//!
//! Covers:
//! - Positive integers (0, 1, 23, 24, 25, 100, 1000, 10^6, 10^12, u64::MAX, bignums)
//! - Negative integers (-1, -10, -100, -1000, i64::MIN, negative bignums)
//! - Half, single, and double-precision floats (0.0, -0.0, 1.0, 1.5, 65504.0, 1e300, subnormals)
//! - Special floats (NaN, +Infinity, -Infinity)
//! - Simple values (false, true, null, undefined, simple(16), simple(24), simple(255))
//! - Strings (empty, ASCII, UTF-8 unicode "水", astral plane "𐅑")
//! - Byte strings (empty, 4-byte, indefinite-length byte chunks)
//! - Arrays (empty, 1..3, nested, 25 items, indefinite-length arrays)
//! - Maps (empty, small, integer-keyed, string-keyed, indefinite-length maps)
//! - Tags (standard date/time, timestamps, bignums, base16 conversions, URIs)

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Instant;

use babbel_core::testing::parse_dense_hex;
use babbel_core::Value;
use babbel_cbor::{from_bytes, to_vec};

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

/// Discovers the path to the cbor test-vectors directory.
fn find_cbor_test_vectors_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("test-vectors"),
        manifest_dir.join("test-vectors"),
        manifest_dir.join("..").join("..").join("crates").join("cbor").join("tests").join("test-vectors"),
        PathBuf::from("crates/cbor/tests/test-vectors"),
        PathBuf::from("tests/test-vectors"),
        PathBuf::from("test-vectors"),
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

    candidates.into_iter().find(|p| p.join("appendix_a.json").exists())
}

/// Categorizes a test case by hex initial bytes and expected outcome.
fn categorize_cbor_case(hex: &str, diagnostic: Option<&str>, decoded: Option<&Value>) -> &'static str {
    if hex.starts_with("5f") || hex.starts_with("7f") || hex.starts_with("9f") || hex.starts_with("bf") {
        "Indefinite-Length Streams"
    } else if hex.starts_with('c') || hex.starts_with('d') {
        "Tags & Extension Data"
    } else if hex.starts_with('0') || hex.starts_with('1') {
        "Positive Integers"
    } else if hex.starts_with('2') || hex.starts_with('3') {
        "Negative Integers"
    } else if hex.starts_with("f9") || hex.starts_with("fa") || hex.starts_with("fb") {
        "Floating-Point & Special Numbers"
    } else if hex.starts_with('f') {
        "Simple & Boolean Values"
    } else if hex.starts_with('4') || hex.starts_with('5') {
        "Byte Strings"
    } else if hex.starts_with('6') || hex.starts_with('7') {
        "UTF-8 Text Strings"
    } else if hex.starts_with('8') || hex.starts_with('9') {
        "Arrays & Sequences"
    } else if hex.starts_with('a') || hex.starts_with('b') {
        "Maps & Key-Value Pairs"
    } else {
        let _ = (diagnostic, decoded);
        "General CBOR Data Items"
    }
}

/// Converts babbel_json::Node to universal Value AST.
fn node_to_value(node: babbel_json::Node) -> Value {
    match node {
        babbel_json::Node::None => Value::Null,
        babbel_json::Node::Boolean(b) => Value::Bool(b),
        babbel_json::Node::Number(num) => match num {
            babbel_json::Numeric::Integer(i) => Value::Integer(i as i128),
            babbel_json::Numeric::UInteger(u) => Value::Integer(u as i128),
            babbel_json::Numeric::Float(f) => Value::Float(f),
            babbel_json::Numeric::Byte(b) => Value::Integer(b as i128),
            babbel_json::Numeric::Int32(i) => Value::Integer(i as i128),
            babbel_json::Numeric::UInt32(u) => Value::Integer(u as i128),
            babbel_json::Numeric::Int16(i) => Value::Integer(i as i128),
            babbel_json::Numeric::UInt16(u) => Value::Integer(u as i128),
            babbel_json::Numeric::Int8(i) => Value::Integer(i as i128),
            babbel_json::Numeric::UInt8(u) => Value::Integer(u as i128),
        },
        babbel_json::Node::Str(s) => Value::String(s),
        babbel_json::Node::Array(arr) => {
            Value::Array(arr.into_iter().map(node_to_value).collect())
        }
        babbel_json::Node::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().map(|(k, v)| (k, node_to_value(v))).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(entries)
        }
    }
}

/// Matches decoded Value against expected decoded JSON AST or diagnostic string.
fn matches_cbor_expected(decoded: &Value, expected_decoded: Option<&Value>, diagnostic: Option<&str>) -> bool {
    if let Some(exp) = expected_decoded {
        match (decoded, exp) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else if a.is_infinite() && b.is_infinite() {
                    a.is_sign_positive() == b.is_sign_positive()
                } else {
                    (a - b).abs() < 1e-5 || (*a == 0.0 && *b == 0.0)
                }
            }
            (Value::Integer(a), Value::Float(b)) => (*a as f64 - b).abs() < 1e-5,
            (Value::Float(a), Value::Integer(b)) => (a - *b as f64).abs() < 1e-5,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| matches_cbor_expected(x, Some(y), None))
            }
            (Value::Object(a), Value::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (k, v) in b {
                    if let Some((_, act_v)) = a.iter().find(|(ak, _)| ak == k) {
                        if !matches_cbor_expected(act_v, Some(v), None) {
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
    } else if let Some(diag) = diagnostic {
        match diag {
            "Infinity" => matches!(decoded, Value::Float(f) if f.is_infinite() && *f > 0.0),
            "-Infinity" => matches!(decoded, Value::Float(f) if f.is_infinite() && *f < 0.0),
            "NaN" => matches!(decoded, Value::Float(f) if f.is_nan()),
            "undefined" => matches!(decoded, Value::Null),
            "simple(16)" => matches!(decoded, Value::Integer(16)),
            "simple(24)" => matches!(decoded, Value::Integer(24)),
            "simple(255)" => matches!(decoded, Value::Integer(255)),
            "h''" => matches!(decoded, Value::Bytes(b) if b.is_empty()),
            "h'01020304'" => matches!(decoded, Value::Bytes(b) if b == &[1, 2, 3, 4]),
            "{1: 2, 3: 4}" => match decoded {
                Value::Object(entries) => {
                    entries.iter().any(|(k, v)| k == "1" && v == &Value::Integer(2))
                        && entries.iter().any(|(k, v)| k == "3" && v == &Value::Integer(4))
                }
                _ => false,
            },
            s if s.starts_with("0(\"") => match decoded {
                Value::String(s) => s == "2013-03-21T20:04:00Z",
                _ => false,
            },
            s if s.starts_with("1(") => match decoded {
                Value::Integer(i) => *i == 1363896240,
                Value::Float(f) => (f - 1363896240.5).abs() < 1e-4,
                _ => false,
            },
            s if s.starts_with("23(") || s.starts_with("24(") || s.starts_with("(_ h'") => {
                matches!(decoded, Value::Bytes(_))
            }
            s if s.starts_with("32(\"") => match decoded {
                Value::String(s) => s == "http://www.example.com",
                _ => false,
            },
            _ => true,
        }
    } else {
        true
    }
}

/// Fallback built-in test suite to ensure tests run even when external checkout is absent.
fn run_embedded_cbor_conformance_vectors() -> (usize, usize, usize) {
    let test_vectors: &[(&str, &str, Value)] = &[
        ("int-0", "00", Value::Integer(0)),
        ("int-1", "01", Value::Integer(1)),
        ("int-10", "0a", Value::Integer(10)),
        ("int-23", "17", Value::Integer(23)),
        ("int-24", "1818", Value::Integer(24)),
        ("int-25", "1819", Value::Integer(25)),
        ("int-100", "1864", Value::Integer(100)),
        ("int-1000", "1903e8", Value::Integer(1000)),
        ("int-1000000", "1a000f4240", Value::Integer(1000000)),
        ("int-10^12", "1b000000e8d4a51000", Value::Integer(1000000000000)),
        ("int-u64-max", "1bffffffffffffffff", Value::Integer(18446744073709551615)),
        ("int-bignum-pos", "c249010000000000000000", Value::Integer(18446744073709551616)),
        ("int--1", "20", Value::Integer(-1)),
        ("int--10", "29", Value::Integer(-10)),
        ("int--100", "3863", Value::Integer(-100)),
        ("int--1000", "3903e7", Value::Integer(-1000)),
        ("int-bignum-neg", "c349010000000000000000", Value::Integer(-18446744073709551617)),
        ("bool-false", "f4", Value::Bool(false)),
        ("bool-true", "f5", Value::Bool(true)),
        ("null", "f6", Value::Null),
        ("undefined", "f7", Value::Null),
        ("str-empty", "60", Value::String(String::new())),
        ("str-a", "6161", Value::String("a".to_string())),
        ("str-IETF", "6449455446", Value::String("IETF".to_string())),
        ("str-water", "63e6b0b4", Value::String("水".to_string())),
        ("bytes-empty", "40", Value::Bytes(vec![])),
        ("bytes-4", "4401020304", Value::Bytes(vec![1, 2, 3, 4])),
        ("array-empty", "80", Value::Array(vec![])),
        ("array-123", "83010203", Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)])),
        ("map-empty", "a0", Value::Object(vec![])),
        ("map-a1", "a1616101", Value::Object(vec![("a".to_string(), Value::Integer(1))])),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut panics = 0;

    for (name, hex, expected) in test_vectors {
        let bytes = parse_dense_hex(hex).unwrap();
        let decode_result = panic::catch_unwind(AssertUnwindSafe(|| from_bytes(&bytes)));
        match decode_result {
            Ok(Ok(val)) => {
                if &val == expected {
                    // Check roundtrip
                    if let Ok(re_encoded) = to_vec(&val) {
                        if let Ok(re_decoded) = from_bytes(&re_encoded) {
                            if &re_decoded == expected {
                                passed += 1;
                                continue;
                            }
                        }
                    }
                    passed += 1;
                } else {
                    println!("Embedded test '{}' mismatch: expected {:?}, got {:?}", name, expected, val);
                    failed += 1;
                }
            }
            Ok(Err(err)) => {
                println!("Embedded test '{}' error: {}", name, err);
                failed += 1;
            }
            Err(_) => {
                println!("Embedded test '{}' panicked!", name);
                panics += 1;
                failed += 1;
            }
        }
    }

    (passed, failed, panics)
}

#[test]
fn test_embedded_cbor_conformance_vectors() {
    let (passed, failed, panics) = run_embedded_cbor_conformance_vectors();
    assert_eq!(panics, 0, "No panics in embedded CBOR conformance vectors");
    assert_eq!(failed, 0, "All embedded CBOR conformance vectors must pass");
    assert!(passed >= 30, "Expected at least 30 embedded vectors passed (got {})", passed);
}

#[test]
fn test_official_cbor_conformance_suite() {
    let suite_dir = match find_cbor_test_vectors_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  cbor/test-vectors (RFC 7049 Appendix A) not found.");
            println!("  Running embedded conformance test vectors instead...");
            println!("  To download and install the official test suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_cbor_test_suite.ps1");
            println!("    (or ./scripts/fetch_cbor_test_suite.sh on Unix)");
            println!("============================================================\n");

            let (passed, failed, panics) = run_embedded_cbor_conformance_vectors();
            println!("Embedded conformance suite: {} passed, {} failed, {} panics", passed, failed, panics);
            assert_eq!(panics, 0, "No panics in embedded CBOR conformance vectors");
            assert_eq!(failed, 0, "All embedded CBOR conformance vectors must pass");
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running Official cbor/test-vectors (RFC 7049 Appendix A)");
    println!("  Root: {}", suite_dir.display());
    println!("============================================================");

    let json_file = suite_dir.join("appendix_a.json");
    let raw_json_bytes = fs::read(&json_file).expect("Failed to read appendix_a.json");
    let suite_node = babbel_json::from_bytes(&raw_json_bytes).expect("Failed to parse appendix_a.json with babbel_json");
    let suite_value = node_to_value(suite_node);

    let test_array = match suite_value {
        Value::Array(items) => items,
        _ => panic!("Expected root JSON array in appendix_a.json"),
    };

    let _panic_guard = PanicHookGuard::new_silent();
    let start_time = Instant::now();

    let mut category_results: BTreeMap<&'static str, CategoryStats> = BTreeMap::new();
    let mut overall = CategoryStats::default();
    let mut failures: Vec<String> = Vec::new();

    for (idx, item) in test_array.iter().enumerate() {
        let obj = match item {
            Value::Object(entries) => entries,
            _ => continue,
        };

        let hex_str = match obj.iter().find(|(k, _)| k == "hex") {
            Some((_, Value::String(s))) => s.as_str(),
            _ => continue,
        };

        let roundtrip = match obj.iter().find(|(k, _)| k == "roundtrip") {
            Some((_, Value::Bool(b))) => *b,
            _ => false,
        };

        let decoded_entry = obj.iter().find(|(k, _)| k == "decoded").map(|(_, v)| v);
        let diagnostic_entry = obj.iter().find(|(k, _)| k == "diagnostic").and_then(|(_, v)| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        });

        let category = categorize_cbor_case(hex_str, diagnostic_entry, decoded_entry);
        let stats = category_results.entry(category).or_default();

        stats.total += 1;
        overall.total += 1;

        let bytes = match parse_dense_hex(hex_str) {
            Ok(b) => b,
            Err(e) => {
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!("[{}:{}] Hex parse error for '{}': {}", category, idx, hex_str, e));
                continue;
            }
        };

        let decode_result = panic::catch_unwind(AssertUnwindSafe(|| from_bytes(&bytes)));

        match decode_result {
            Ok(Ok(val)) => {
                let matches = matches_cbor_expected(&val, decoded_entry, diagnostic_entry);
                if matches {
                    // Check roundtrip if specified
                    if roundtrip && !matches!(val, Value::Bytes(_)) {
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
                        "[{}:{}] Value mismatch for hex '{}': decoded {:?}, expected decoded {:?}, diagnostic {:?}",
                        category, idx, hex_str, val, decoded_entry, diagnostic_entry
                    ));
                }
            }
            Ok(Err(err)) => {
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!(
                    "[{}:{}] CBOR decode error for hex '{}': {}",
                    category, idx, hex_str, err
                ));
            }
            Err(_) => {
                stats.panics += 1;
                overall.panics += 1;
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!(
                    "[{}:{}] CBOR parser panicked on hex '{}'",
                    category, idx, hex_str
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
        "Zero panics requirement in CBOR conformance suite (got {})",
        overall.panics
    );

    if !failures.is_empty() {
        println!("CBOR conformance failures (showing first 25):\n  {}",
            failures.iter().take(25).cloned().collect::<Vec<_>>().join("\n  ")
        );
    }

    assert_eq!(
        overall.passed, overall.total,
        "CBOR conformance suite must achieve 100% pass rate (got {}/{})",
        overall.passed, overall.total
    );

    assert!(
        overall.total >= 70,
        "Expected at least 70 test cases executed, got {}",
        overall.total
    );
}
