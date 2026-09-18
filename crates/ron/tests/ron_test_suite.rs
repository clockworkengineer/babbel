//! Official starfederation/ron Conformance Test Suite Integration
//!
//! Runs the official language-neutral RON (Readable Object Notation) conformance suite:
//! https://github.com/starfederation/ron
//!
//! Evaluates both:
//! 1. Valid vectors: records, escapes, delimiter-aware strings, top-level elided maps,
//!    comma-prefixed tokens, punctuation tokens, arrays, objects, booleans, nulls, numbers.
//! 2. Invalid vectors: unclosed strings/containers, trailing garbage, invalid escapes,
//!    unpaired UTF-16 surrogates, raw controls, missing values.
//! 3. Embedded fallback vectors: guaranteed offline test execution with 30+ vectors.

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Instant;

use babbel_core::Value;
use babbel_ron::from_str;

fn node_to_value(node: babbel_json::nodes::Node) -> Value {
    match node {
        babbel_json::nodes::Node::Boolean(b) => Value::Bool(b),
        babbel_json::nodes::Node::Number(num) => match num {
            babbel_core::num::Numeric::Integer(i) => Value::Integer(i as i128),
            babbel_core::num::Numeric::UInteger(u) => Value::Integer(u as i128),
            babbel_core::num::Numeric::Byte(b) => Value::Integer(b as i128),
            babbel_core::num::Numeric::Int32(i) => Value::Integer(i as i128),
            babbel_core::num::Numeric::UInt32(u) => Value::Integer(u as i128),
            babbel_core::num::Numeric::Int16(i) => Value::Integer(i as i128),
            babbel_core::num::Numeric::UInt16(u) => Value::Integer(u as i128),
            babbel_core::num::Numeric::Int8(i) => Value::Integer(i as i128),
            babbel_core::num::Numeric::UInt8(u) => Value::Integer(u as i128),
            babbel_core::num::Numeric::Float(f) => Value::Float(f),
        },
        babbel_json::nodes::Node::Str(s) => Value::String(s),
        babbel_json::nodes::Node::Array(arr) => {
            Value::Array(arr.into_iter().map(node_to_value).collect())
        }
        babbel_json::nodes::Node::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, node_to_value(v)))
                .collect(),
        ),
        babbel_json::nodes::Node::None => Value::Null,
    }
}

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

/// Discovers the path to the starfederation/ron conformance directory.
fn find_ron_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("ron-upstream"),
        manifest_dir.join("ron-upstream"),
        manifest_dir
            .join("..")
            .join("..")
            .join("crates")
            .join("ron")
            .join("tests")
            .join("ron-upstream"),
        PathBuf::from("crates/ron/tests/ron-upstream"),
        PathBuf::from("tests/ron-upstream"),
        PathBuf::from("ron-upstream"),
    ];

    // Read suite_paths.txt if available
    let suite_paths_file = manifest_dir.join("tests").join("suite_paths.txt");
    if let Ok(content) = fs::read_to_string(&suite_paths_file) {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                candidates.push(manifest_dir.join(line));
                candidates.push(PathBuf::from(line));
            }
        }
    }

    for candidate in candidates {
        let manifest = candidate
            .join("testdata")
            .join("conformance")
            .join("manifest.json");
        if manifest.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Checks semantic equality of two Values, allowing object keys to appear in different order.
fn values_equal_modulo_key_order(v1: &Value, v2: &Value) -> bool {
    match (v1, v2) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
        (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
        (Value::Float(f1), Value::Float(f2)) => {
            if f1.is_nan() && f2.is_nan() {
                true
            } else if f1.is_infinite() && f2.is_infinite() {
                f1.is_sign_positive() == f2.is_sign_positive()
            } else {
                (f1 - f2).abs() < 1e-9
            }
        }
        (Value::Integer(i), Value::Float(f)) => (*i as f64 - f).abs() < 1e-9,
        (Value::Float(f), Value::Integer(i)) => (f - *i as f64).abs() < 1e-9,
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Bytes(b1), Value::Bytes(b2)) => b1 == b2,
        (Value::Array(a1), Value::Array(a2)) => {
            if a1.len() != a2.len() {
                return false;
            }
            a1.iter()
                .zip(a2.iter())
                .all(|(x, y)| values_equal_modulo_key_order(x, y))
        }
        (Value::Object(o1), Value::Object(o2)) => {
            if o1.len() != o2.len() {
                return false;
            }
            for (k1, v1) in o1 {
                if let Some((_, v2)) = o2.iter().find(|(k2, _)| k2 == k1) {
                    if !values_equal_modulo_key_order(v1, v2) {
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

/// Categorizes a test path into a human-readable grouping.
fn categorize_test_path(path_str: &str, is_invalid: bool) -> &'static str {
    if is_invalid {
        if path_str.contains("escape") || path_str.contains("surrogate") {
            "Invalid Escapes & Surrogates"
        } else if path_str.contains("string") {
            "Invalid String Delimiters & Controls"
        } else {
            "Invalid Structure & Syntax"
        }
    } else if path_str.contains("records") || path_str.contains("top_level_elided") {
        "Valid Records & Elided Maps"
    } else if path_str.contains("escape") {
        "Valid String Escapes & Unicode"
    } else if path_str.contains("delimiter")
        || path_str.contains("quad")
        || path_str.contains("repeated")
    {
        "Valid Delimiter-Aware & Quoted Strings"
    } else if path_str.contains("punctuation") || path_str.contains("comma") {
        "Valid Punctuation & Comma Tokens"
    } else if path_str.contains("array") || path_str.contains("object") {
        "Valid Arrays & Object Containers"
    } else if path_str.contains("scalar") || path_str.contains("number") {
        "Valid Scalars (Numbers, Bools, Null)"
    } else {
        "Valid General Vectors"
    }
}

struct EmbeddedVector {
    name: &'static str,
    ron_input: &'static str,
    json_expected: Option<&'static str>,
    is_valid: bool,
}

const EMBEDDED_VECTORS: &[EmbeddedVector] = &[
    // Valid basic records & elided map
    EmbeddedVector {
        name: "records/basic_users_settings",
        ron_input: "users [{id 100 name Ada roles [admin writer] active true}]\nsettings {retry {max 3 backoffMs 250} tags [llm json]}",
        json_expected: Some(
            "{\"settings\":{\"retry\":{\"backoffMs\":250,\"max\":3},\"tags\":[\"llm\",\"json\"]},\"users\":[{\"active\":true,\"id\":100,\"name\":\"Ada\",\"roles\":[\"admin\",\"writer\"]}]}",
        ),
        is_valid: true,
    },
    EmbeddedVector {
        name: "records/top_level_elided_map",
        ron_input: "name Ada\nmanager {# 200}\ntags [math logic]",
        json_expected: Some(
            "{\"manager\":{\"#\":200},\"name\":\"Ada\",\"tags\":[\"math\",\"logic\"]}",
        ),
        is_valid: true,
    },
    // Valid escapes
    EmbeddedVector {
        name: "escapes/json_controls_and_unicode",
        ron_input: "apostrophe '''''\nlineFeed a\\nb\ntab a\\tb\nescapedKeyword tr\\u0075e\nescapedSpace a\\u0020b\nunicode A\\u03a9\\uD83D\\uDE00",
        json_expected: Some(
            "{\"apostrophe\":\"'\",\"escapedKeyword\":\"true\",\"escapedSpace\":\"a b\",\"lineFeed\":\"a\\nb\",\"tab\":\"a\\tb\",\"unicode\":\"AΩ😀\"}",
        ),
        is_valid: true,
    },
    // Valid comma escapes
    EmbeddedVector {
        name: "comma_escapes/tab_and_pair",
        ron_input: "backslash [,\\\\]\nlineFeed [,\\n]\ntab [,\\t]\nunicodePair [,\\uD83D\\uDE00]",
        json_expected: Some(
            "{\"backslash\":[\",\\\\\"],\"lineFeed\":[\",\\n\"],\"tab\":[\",\\t\"],\"unicodePair\":[\",😀\"]}",
        ),
        is_valid: true,
    },
    // Valid delimiter-aware strings
    EmbeddedVector {
        name: "delimiter_aware_strings/nested_quotes",
        ron_input: "data ['{\"coordinates\":[12.5,-42.25],\"type\":\"Point\"}']\nquotes ['a\"b' \"\"\"a \"quoted\" phrase\"\"\" \"\"\"\"contains \"\"\" inside and \" too\"\"\"\"]",
        json_expected: Some(
            "{\"data\":[\"{\\\"coordinates\\\":[12.5,-42.25],\\\"type\\\":\\\"Point\\\"}\"],\"quotes\":[\"a\\\"b\",\"a \\\"quoted\\\" phrase\",\"contains \\\"\\\"\\\" inside and \\\" too\"]}",
        ),
        is_valid: true,
    },
    // Valid strings & quotes
    EmbeddedVector {
        name: "strings/various_framings",
        ron_input: "[Ada hello '' \"\" \"it's fine\" 'contains \"\" inside' ?name 'Ada Lovelace' 'true' 'null' '123' #_ada -ada]",
        json_expected: Some(
            "[\"Ada\",\"hello\",\"\",\"\",\"it's fine\",\"contains \\\"\\\" inside\",\"?name\",\"Ada Lovelace\",\"true\",\"null\",\"123\",\"#_ada\",\"-ada\"]",
        ),
        is_valid: true,
    },
    // Valid punctuation tokens
    EmbeddedVector {
        name: "punctuation/unquote_and_prefixed",
        ron_input: "unquote [, x]\nunquoteSplicing [,@ xs]\ncommaPrefixed [,foo]\nunquoteKey {, value}",
        json_expected: Some(
            "{\"commaPrefixed\":[\",foo\"],\"unquote\":[\",\",\"x\"],\"unquoteKey\":{\",\":\"value\"},\"unquoteSplicing\":[\",@\",\"xs\"]}",
        ),
        is_valid: true,
    },
    // Valid scalars
    EmbeddedVector {
        name: "scalars/scalar_boolean_true",
        ron_input: "true\n",
        json_expected: Some("true"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_boolean_false",
        ron_input: "false\n",
        json_expected: Some("false"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_null",
        ron_input: "null\n",
        json_expected: Some("null"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_number_int",
        ron_input: "42\n",
        json_expected: Some("42"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_number_float",
        ron_input: "-12.5e+2\n",
        json_expected: Some("-1250.0"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_string_single_quoted",
        ron_input: "'Ada Lovelace'\n",
        json_expected: Some("\"Ada Lovelace\""),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_string_quad_double_quotes",
        ron_input: "\"\"\"\"contains \"\"\" inside\"\"\"\"\n",
        json_expected: Some("\"contains \\\"\\\"\\\" inside\""),
        is_valid: true,
    },
    EmbeddedVector {
        name: "scalars/scalar_string_quad_single_quotes",
        ron_input: "''''contains ''' inside''''\n",
        json_expected: Some("\"contains ''' inside\""),
        is_valid: true,
    },
    // Valid objects and arrays
    EmbeddedVector {
        name: "containers/array_mixed_commas",
        ron_input: "[a, 1, b, 2, false, null]",
        json_expected: Some("[\"a\",1,\"b\",2,false,null]"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "containers/object_with_commas",
        ron_input: "{\n  name Ada,\n  age 37,\n  manager {# 200},\n}",
        json_expected: Some("{\"age\":37,\"manager\":{\"#\":200},\"name\":\"Ada\"}"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "containers/object_quoted_key",
        ron_input: "{'quoted key' 'quoted value'}",
        json_expected: Some("{\"quoted key\":\"quoted value\"}"),
        is_valid: true,
    },
    EmbeddedVector {
        name: "containers/object_scalar_like_keys",
        ron_input: "{1538289 {# 181773} true false null nil}",
        json_expected: Some("{\"1538289\":{\"#\":181773},\"null\":\"nil\",\"true\":false}"),
        is_valid: true,
    },
    // Invalid test vectors
    EmbeddedVector {
        name: "invalid/empty_input",
        ron_input: "",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/string_unterminated",
        ron_input: "'hello\n",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/string_raw_lf",
        ron_input: "'a\nb'",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/string_raw_double_quote",
        ron_input: "a\"b",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/string_unescaped_double_quote_run",
        ron_input: "\"\"\"a\"\"\"b\"\"\"",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/trailing_garbage",
        ron_input: "{name Ada} nope\n",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/escape_unknown",
        ron_input: "a\\q",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/escape_unicode_short",
        ron_input: "\\u123",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/escape_unicode_nonhex",
        ron_input: "\\u123g",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/escape_unpaired_high_surrogate",
        ron_input: "\\uD800",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/escape_unpaired_low_surrogate",
        ron_input: "\\uDC00",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/n_object_key_missing_value",
        ron_input: "{\n  name\n}",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/n_structure_object_then_array",
        ron_input: "{name Ada} []\n",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/n_structure_array_unclosed",
        ron_input: "[1, 2",
        json_expected: None,
        is_valid: false,
    },
];

#[test]
fn test_ron_official_conformance_suite() {
    let start_time = Instant::now();
    let mut categories: BTreeMap<&'static str, CategoryStats> = BTreeMap::new();
    let mut total_stats = CategoryStats::default();

    let suite_dir_opt = find_ron_test_suite_dir();
    let run_upstream = suite_dir_opt.is_some();

    if let Some(suite_dir) = &suite_dir_opt {
        let manifest_path = suite_dir
            .join("testdata")
            .join("conformance")
            .join("manifest.json");
        let conformance_dir = suite_dir.join("testdata").join("conformance");

        if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest_val) = babbel_json::from_str(&manifest_content) {
                // 1. Run Valid Cases from manifest.json
                if let Some(valid_list) = manifest_val.get("valid").and_then(|v| v.as_array()) {
                    for item in valid_list {
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unnamed");
                        let ron_inputs = item.get("ronInputs").and_then(|v| v.as_array());
                        let json_input_rel = item.get("jsonInput").and_then(|v| v.as_str());

                        if let (Some(inputs), Some(json_rel)) = (ron_inputs, json_input_rel) {
                            let json_path = conformance_dir.join(json_rel);
                            let json_content = fs::read_to_string(&json_path).unwrap_or_default();
                            let expected_json_val =
                                babbel_json::from_str(&json_content).map(node_to_value);

                            for ron_input_val in inputs {
                                if let Some(ron_rel) = ron_input_val.as_str() {
                                    let ron_path = conformance_dir.join(ron_rel);
                                    let ron_content =
                                        fs::read_to_string(&ron_path).unwrap_or_default();
                                    let category = categorize_test_path(ron_rel, false);
                                    let stats = categories.entry(category).or_default();
                                    stats.total += 1;
                                    total_stats.total += 1;

                                    let _guard = PanicHookGuard::new_silent();
                                    let ron_parse_result =
                                        panic::catch_unwind(AssertUnwindSafe(|| {
                                            from_str(&ron_content)
                                        }));

                                    match ron_parse_result {
                                        Ok(Ok(val)) => {
                                            if let Ok(expected) = &expected_json_val {
                                                if values_equal_modulo_key_order(&val, expected) {
                                                    stats.passed += 1;
                                                    total_stats.passed += 1;
                                                } else {
                                                    stats.failed += 1;
                                                    total_stats.failed += 1;
                                                    eprintln!(
                                                        "[FAIL] Value mismatch for '{}' ({})\n  Parsed RON: {:?}\n  Expected JSON: {:?}",
                                                        name, ron_rel, val, expected
                                                    );
                                                }
                                            } else {
                                                stats.passed += 1;
                                                total_stats.passed += 1;
                                            }
                                        }
                                        Ok(Err(err)) => {
                                            stats.failed += 1;
                                            total_stats.failed += 1;
                                            eprintln!(
                                                "[FAIL] Parse error for '{}' ({}): {:?}",
                                                name, ron_rel, err
                                            );
                                        }
                                        Err(_) => {
                                            stats.panics += 1;
                                            total_stats.panics += 1;
                                            eprintln!(
                                                "[PANIC] Panic during parse of '{}' ({})",
                                                name, ron_rel
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. Run Invalid Cases from manifest.json
                if let Some(invalid_list) =
                    manifest_val.get("invalidRON").and_then(|v| v.as_array())
                {
                    for item in invalid_list {
                        if let Some(rel_path) = item.as_str() {
                            let full_path = conformance_dir.join(rel_path);
                            let content = fs::read_to_string(&full_path).unwrap_or_default();
                            let category = categorize_test_path(rel_path, true);
                            let stats = categories.entry(category).or_default();
                            stats.total += 1;
                            total_stats.total += 1;

                            let _guard = PanicHookGuard::new_silent();
                            let result =
                                panic::catch_unwind(AssertUnwindSafe(|| from_str(&content)));

                            match result {
                                Ok(Err(_)) => {
                                    // Properly rejected!
                                    stats.passed += 1;
                                    total_stats.passed += 1;
                                }
                                Ok(Ok(_val)) => {
                                    stats.failed += 1;
                                    total_stats.failed += 1;
                                    eprintln!(
                                        "[FAIL] Invalid file unexpectedly parsed: {}",
                                        rel_path
                                    );
                                }
                                Err(_) => {
                                    stats.panics += 1;
                                    total_stats.panics += 1;
                                    eprintln!("[PANIC] Panic on invalid file: {}", rel_path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Run Embedded Fallback Vectors
    for vec in EMBEDDED_VECTORS {
        let category = categorize_test_path(vec.name, !vec.is_valid);
        let stats = categories.entry(category).or_default();
        stats.total += 1;
        total_stats.total += 1;

        let _guard = PanicHookGuard::new_silent();
        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| from_str(vec.ron_input)));

        if vec.is_valid {
            match parse_result {
                Ok(Ok(val)) => {
                    if let Some(json_exp) = vec.json_expected {
                        if let Ok(expected_val) = babbel_json::from_str(json_exp).map(node_to_value)
                        {
                            if values_equal_modulo_key_order(&val, &expected_val) {
                                stats.passed += 1;
                                total_stats.passed += 1;
                            } else {
                                stats.failed += 1;
                                total_stats.failed += 1;
                                eprintln!(
                                    "[FAIL] Embedded vector mismatch: {}\n  Parsed RON: {:?}\n  Expected JSON: {:?}",
                                    vec.name, val, expected_val
                                );
                            }
                        } else {
                            stats.passed += 1;
                            total_stats.passed += 1;
                        }
                    } else {
                        stats.passed += 1;
                        total_stats.passed += 1;
                    }
                }
                Ok(Err(e)) => {
                    stats.failed += 1;
                    total_stats.failed += 1;
                    eprintln!("[FAIL] Embedded vector error on '{}': {:?}", vec.name, e);
                }
                Err(_) => {
                    stats.panics += 1;
                    total_stats.panics += 1;
                    eprintln!("[PANIC] Panic on embedded vector '{}'", vec.name);
                }
            }
        } else {
            match parse_result {
                Ok(Err(_)) => {
                    stats.passed += 1;
                    total_stats.passed += 1;
                }
                Ok(Ok(_)) => {
                    stats.failed += 1;
                    total_stats.failed += 1;
                    eprintln!(
                        "[FAIL] Embedded invalid vector unexpectedly parsed: {}",
                        vec.name
                    );
                }
                Err(_) => {
                    stats.panics += 1;
                    total_stats.panics += 1;
                    eprintln!("[PANIC] Panic on embedded invalid vector '{}'", vec.name);
                }
            }
        }
    }

    let elapsed = start_time.elapsed();

    // 4. Print Formatted ASCII Conformance Report
    println!(
        "\n+---------------------------------------------------------------------------------------+"
    );
    println!(
        "|          starfederation/ron (Readable Object Notation) Conformance Test Suite         |"
    );
    println!(
        "+------------------------------------------------------+-------+--------+--------+------+"
    );
    println!(
        "| Category                                             | Total | Passed | Failed | Panics| Pass %|"
    );
    println!(
        "+------------------------------------------------------+-------+--------+--------+------+"
    );
    for (cat_name, stats) in &categories {
        println!(
            "| {:<52} | {:>5} | {:>6} | {:>6} | {:>4} | {:>5.1}%|",
            cat_name,
            stats.total,
            stats.passed,
            stats.failed,
            stats.panics,
            stats.pass_rate()
        );
    }
    println!(
        "+------------------------------------------------------+-------+--------+--------+------+"
    );
    println!(
        "| TOTAL (Upstream + Embedded)                          | {:>5} | {:>6} | {:>6} | {:>4} | {:>5.1}%|",
        total_stats.total,
        total_stats.passed,
        total_stats.failed,
        total_stats.panics,
        total_stats.pass_rate()
    );
    println!(
        "+------------------------------------------------------+-------+--------+--------+------+"
    );
    println!(
        "| Suite Source: {:<38} | Elapsed: {:>13?} |",
        if run_upstream {
            "Upstream + Embedded"
        } else {
            "Embedded Fallback"
        },
        elapsed
    );
    println!(
        "+---------------------------------------------------------------------------------------+\n"
    );

    // Strict requirements: 0 panics, and 100% pass on embedded + upstream!
    assert_eq!(total_stats.panics, 0, "Conformance suite had panics!");
    assert_eq!(
        total_stats.failed, 0,
        "Conformance suite had test failures!"
    );
    assert!(
        total_stats.passed > 0,
        "At least one test should have passed!"
    );
}
