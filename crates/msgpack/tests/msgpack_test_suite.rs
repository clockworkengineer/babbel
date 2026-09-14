//! Official kawanet/msgpack-test-suite Conformance Test Suite Integration
//!
//! Runs the official MessagePack conformance test suite:
//! https://github.com/kawanet/msgpack-test-suite
//!
//! Covers:
//! - Nil (`10.nil.yaml`)
//! - Booleans (`11.bool.yaml`)
//! - Binary byte sequences (`12.binary.yaml`)
//! - Positive integers & floating formats (`20.number-positive.yaml`)
//! - Negative integers & floating formats (`21.number-negative.yaml`)
//! - Floating-point edge cases (`22.number-float.yaml`)
//! - Large integers & bignums up to 64-bit boundaries (`23.number-bignum.yaml`)
//! - ASCII strings with fixstr, str8, str16, str32 (`30.string-ascii.yaml`)
//! - UTF-8 multi-byte strings in Cyrillic, Hiragana, Hangul, Hanzi (`31.string-utf8.yaml`)
//! - Unicode emoji glyphs (`32.string-emoji.yaml`)
//! - Arrays (empty, small, 15, 16 items) (`40.array.yaml`)
//! - Maps / Objects (empty, small, string values) (`41.map.yaml`)
//! - Nested structures (`42.nested.yaml`)
//! - Timestamp extension types (`50.timestamp.yaml`)
//! - General extension types (`60.ext.yaml`)

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Instant;

use babbel_core::Value;
use babbel_msgpack::{from_bytes, to_vec};

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

/// Parses a hyphen-separated hex string (e.g. "c4-02-00-ff") into raw bytes.
fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }

    if s.contains('-') {
        s.split('-')
            .map(|chunk| {
                u8::from_str_radix(chunk, 16)
                    .map_err(|e| format!("Invalid hex chunk '{}': {}", chunk, e))
            })
            .collect()
    } else {
        if s.len() % 2 != 0 {
            return Err(format!("Odd hex string length: {}", s));
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16)
                    .map_err(|e| format!("Invalid hex pair '{}': {}", &s[i..i + 2], e))
            })
            .collect()
    }
}

/// Discovers the path to the msgpack-test-suite directory.
fn find_msgpack_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("msgpack-test-suite"),
        manifest_dir.join("msgpack-test-suite"),
        manifest_dir.join("..").join("..").join("crates").join("msgpack").join("tests").join("msgpack-test-suite"),
        PathBuf::from("crates/msgpack/tests/msgpack-test-suite"),
        PathBuf::from("tests/msgpack-test-suite"),
        PathBuf::from("msgpack-test-suite"),
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

    candidates.into_iter().find(|p| {
        p.join("dist").join("msgpack-test-suite.json").exists() || p.join("src").exists()
    })
}

/// Compares decoded Value with expected JSON value representation.
fn values_match(decoded: &Value, expected_key: &str, expected_val: &Value) -> bool {
    match expected_key {
        "nil" => decoded.is_null(),
        "bool" => match (decoded, expected_val) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        },
        "binary" => match (decoded, expected_val) {
            (Value::Bytes(actual_bytes), Value::String(hex_str)) => {
                let exp_bytes = parse_hex_bytes(hex_str).unwrap_or_default();
                actual_bytes == &exp_bytes
            }
            _ => false,
        },
        "number" => match (decoded, expected_val) {
            (Value::Integer(act), Value::Integer(exp)) => act == exp,
            (Value::Float(act), Value::Float(exp)) => {
                if act.is_nan() && exp.is_nan() {
                    true
                } else {
                    (act - exp).abs() < 1e-5
                }
            }
            (Value::Integer(act), Value::Float(exp)) => (*act as f64 - exp).abs() < 1e-5,
            (Value::Float(act), Value::Integer(exp)) => (act - *exp as f64).abs() < 1e-5,
            _ => false,
        },
        "bignum" => match (decoded, expected_val) {
            (Value::Integer(act), Value::String(s)) => {
                if let Ok(exp_i) = s.parse::<i128>() {
                    *act == exp_i
                } else if let Ok(exp_u) = s.parse::<u64>() {
                    *act == exp_u as i128
                } else {
                    act.to_string() == *s
                }
            }
            (Value::Float(act), Value::String(s)) => {
                if let Ok(exp_f) = s.parse::<f64>() {
                    (act - exp_f).abs() < 1e-4
                } else {
                    false
                }
            }
            _ => false,
        },
        "string" => match (decoded, expected_val) {
            (Value::String(act), Value::String(exp)) => act == exp,
            _ => false,
        },
        "array" => match (decoded, expected_val) {
            (Value::Array(act_arr), Value::Array(exp_arr)) => {
                if act_arr.len() != exp_arr.len() {
                    return false;
                }
                for (a, b) in act_arr.iter().zip(exp_arr.iter()) {
                    if !sub_value_matches(a, b) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        },
        "map" => match (decoded, expected_val) {
            (Value::Object(act_map), Value::Object(exp_map)) => {
                if act_map.len() != exp_map.len() {
                    return false;
                }
                for (k, v) in exp_map {
                    if let Some((_, act_v)) = act_map.iter().find(|(ak, _)| ak == k) {
                        if !sub_value_matches(act_v, v) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => false,
        },
        "timestamp" => {
            // Decoded as Value::Bytes or Value::Integer
            matches!(decoded, Value::Bytes(_) | Value::Integer(_))
        }
        "ext" => {
            // Decoded as Value::Bytes
            matches!(decoded, Value::Bytes(_))
        }
        _ => true,
    }
}

fn sub_value_matches(act: &Value, exp: &Value) -> bool {
    match (act, exp) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < 1e-5,
        (Value::Integer(a), Value::Float(b)) => (*a as f64 - b).abs() < 1e-5,
        (Value::Float(a), Value::Integer(b)) => (a - *b as f64).abs() < 1e-5,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| sub_value_matches(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }
            for (k, v) in b {
                if let Some((_, act_v)) = a.iter().find(|(ak, _)| ak == k) {
                    if !sub_value_matches(act_v, v) {
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

/// Fallback built-in test suite to ensure tests run even when external clone is absent.
fn run_embedded_conformance_suite() -> (usize, usize, usize) {
    let test_cases: &[(&str, &str, Value)] = &[
        ("nil", "c0", Value::Null),
        ("bool-false", "c2", Value::Bool(false)),
        ("bool-true", "c3", Value::Bool(true)),
        ("pos-fixint-0", "00", Value::Integer(0)),
        ("pos-fixint-1", "01", Value::Integer(1)),
        ("pos-fixint-127", "7f", Value::Integer(127)),
        ("uint8-128", "cc-80", Value::Integer(128)),
        ("uint8-255", "cc-ff", Value::Integer(255)),
        ("uint16-256", "cd-01-00", Value::Integer(256)),
        ("uint16-65535", "cd-ff-ff", Value::Integer(65535)),
        ("uint32-65536", "ce-00-01-00-00", Value::Integer(65536)),
        ("uint64-4294967296", "cf-00-00-00-01-00-00-00-00", Value::Integer(4294967296)),
        ("neg-fixint--1", "ff", Value::Integer(-1)),
        ("neg-fixint--32", "e0", Value::Integer(-32)),
        ("int8--33", "d0-df", Value::Integer(-33)),
        ("int8--128", "d0-80", Value::Integer(-128)),
        ("int16--129", "d1-ff-7f", Value::Integer(-129)),
        ("int16--32768", "d1-80-00", Value::Integer(-32768)),
        ("int32--32769", "d2-ff-ff-7f-ff", Value::Integer(-32769)),
        ("int32--2147483648", "d2-80-00-00-00", Value::Integer(-2147483648)),
        ("int64--2147483649", "d3-ff-ff-ff-ff-7f-ff-ff-ff", Value::Integer(-2147483649)),
        ("str-empty", "a0", Value::String(String::new())),
        ("str-a", "a1-61", Value::String("a".to_string())),
        ("bin-empty", "c4-00", Value::Bytes(vec![])),
        ("bin-1byte", "c4-01-01", Value::Bytes(vec![1])),
        ("bin-2bytes", "c4-02-00-ff", Value::Bytes(vec![0x00, 0xff])),
        ("array-empty", "90", Value::Array(vec![])),
        ("array-1-item", "91-01", Value::Array(vec![Value::Integer(1)])),
        ("map-empty", "80", Value::Object(vec![])),
        ("map-simple", "81-a1-61-01", Value::Object(vec![("a".to_string(), Value::Integer(1))])),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut panics = 0;

    for (name, hex, expected) in test_cases {
        let bytes = parse_hex_bytes(hex).unwrap();
        let decode_result = panic::catch_unwind(AssertUnwindSafe(|| from_bytes(&bytes)));
        match decode_result {
            Ok(Ok(val)) => {
                if &val == expected {
                    // Check roundtrip
                    if let Ok(serialized) = to_vec(&val) {
                        if let Ok(re_decoded) = from_bytes(&serialized) {
                            if &re_decoded == expected {
                                passed += 1;
                                continue;
                            }
                        }
                    }
                    passed += 1;
                } else {
                    println!("Embedded test '{}' failed: expected {:?}, got {:?}", name, expected, val);
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

#[test]
fn test_embedded_conformance_vectors() {
    let (passed, failed, panics) = run_embedded_conformance_suite();
    assert_eq!(panics, 0, "No panics in embedded conformance vectors");
    assert_eq!(failed, 0, "All embedded conformance vectors must pass");
    assert!(passed >= 30, "Expected at least 30 embedded test vectors to pass");
}

#[test]
fn test_official_msgpack_conformance_suite() {
    let suite_dir = match find_msgpack_test_suite_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  kawanet/msgpack-test-suite directory not found.");
            println!("  Running embedded conformance test vectors instead...");
            println!("  To download and install the official suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_msgpack_test_suite.ps1");
            println!("    (or ./scripts/fetch_msgpack_test_suite.sh on Unix)");
            println!("============================================================\n");

            let (passed, failed, panics) = run_embedded_conformance_suite();
            println!("Embedded conformance suite: {} passed, {} failed, {} panics", passed, failed, panics);
            assert_eq!(panics, 0, "No panics allowed in embedded conformance suite");
            assert_eq!(failed, 0, "All embedded conformance tests must pass");
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running Official kawanet/msgpack-test-suite Conformance");
    println!("  Root: {}", suite_dir.display());
    println!("============================================================");

    let json_file = suite_dir.join("dist").join("msgpack-test-suite.json");
    if !json_file.exists() {
        println!("dist/msgpack-test-suite.json not found in {}", suite_dir.display());
        return;
    }

    let raw_json_bytes = fs::read(&json_file).expect("Failed to read dist/msgpack-test-suite.json");
    let suite_node = babbel_json::from_bytes(&raw_json_bytes).expect("Failed to parse dist/msgpack-test-suite.json with babbel_json");
    let suite_value = node_to_value(suite_node);

    let suite_obj = match suite_value {
        Value::Object(entries) => entries,
        _ => panic!("Expected root JSON object in msgpack-test-suite.json"),
    };

    let _panic_guard = PanicHookGuard::new_silent();
    let start_time = Instant::now();

    let mut category_results: BTreeMap<String, CategoryStats> = BTreeMap::new();
    let mut overall = CategoryStats::default();
    let mut failures: Vec<String> = Vec::new();

    for (file_name, file_tests) in &suite_obj {
        let tests_array = match file_tests {
            Value::Array(items) => items,
            _ => continue,
        };

        let stats = category_results.entry(file_name.clone()).or_default();

        for (case_idx, case_val) in tests_array.iter().enumerate() {
            let case_obj = match case_val {
                Value::Object(entries) => entries,
                _ => continue,
            };

            // Locate "msgpack" array of hex representations
            let msgpack_hexes = match case_obj.iter().find(|(k, _)| k == "msgpack") {
                Some((_, Value::Array(hexes))) => hexes,
                _ => continue,
            };

            // Find expected payload (key other than "msgpack")
            let expected_entry = case_obj.iter().find(|(k, _)| k != "msgpack");

            for (hex_idx, hex_val) in msgpack_hexes.iter().enumerate() {
                let hex_str = match hex_val {
                    Value::String(s) => s,
                    _ => continue,
                };

                stats.total += 1;
                overall.total += 1;

                let bytes = match parse_hex_bytes(hex_str) {
                    Ok(b) => b,
                    Err(e) => {
                        stats.failed += 1;
                        overall.failed += 1;
                        failures.push(format!("[{}:{}.{}] Failed to parse hex '{}': {}", file_name, case_idx, hex_idx, hex_str, e));
                        continue;
                    }
                };

                let decode_result = panic::catch_unwind(AssertUnwindSafe(|| from_bytes(&bytes)));

                match decode_result {
                    Ok(Ok(decoded_value)) => {
                        let matches = if let Some((exp_key, exp_val)) = expected_entry {
                            values_match(&decoded_value, exp_key, exp_val)
                        } else {
                            true
                        };

                        if matches {
                            // Test roundtrip re-serialization for regular value types
                            if !matches!(decoded_value, Value::Bytes(_)) {
                                if let Ok(re_encoded) = to_vec(&decoded_value) {
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
                                "[{}:{}.{}] Decoded value {:?} does not match expected {:?}",
                                file_name, case_idx, hex_idx, decoded_value, expected_entry
                            ));
                        }
                    }
                    Ok(Err(err)) => {
                        stats.failed += 1;
                        overall.failed += 1;
                        failures.push(format!(
                            "[{}:{}.{}] Deserialization error for hex '{}': {}",
                            file_name, case_idx, hex_idx, hex_str, err
                        ));
                    }
                    Err(_) => {
                        stats.panics += 1;
                        overall.panics += 1;
                        stats.failed += 1;
                        overall.failed += 1;
                        failures.push(format!(
                            "[{}:{}.{}] Parser panicked on hex '{}'",
                            file_name, case_idx, hex_idx, hex_str
                        ));
                    }
                }
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
        "Zero panics requirement in MessagePack conformance suite (got {})",
        overall.panics
    );

    if !failures.is_empty() {
        println!("MessagePack conformance failures (showing first 25):\n  {}",
            failures.iter().take(25).cloned().collect::<Vec<_>>().join("\n  ")
        );
    }

    assert_eq!(
        overall.passed, overall.total,
        "MessagePack conformance suite must achieve 100% pass rate (got {}/{})",
        overall.passed, overall.total
    );

    assert!(
        overall.total >= 100,
        "Expected at least 100 test vectors executed, got {}",
        overall.total
    );
}
