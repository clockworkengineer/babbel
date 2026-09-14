//! Official kdl-org/kdl-test Conformance Test Suite Integration
//!
//! Runs the official language-neutral KDL test suite:
//! https://github.com/kdl-org/kdl-test
//!
//! Validates KDL parsing compliance via the official JSON protocol:
//! 1. Valid vectors: test_cases/valid/*.kdl matching test_cases/valid/*.json
//! 2. Invalid vectors: test_cases/invalid/*.kdl must fail to parse
//! 3. Embedded fallback vectors: 30+ vectors for guaranteed offline execution
//! 4. Zero panics on any malformed or fuzz input.

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Instant;

use babbel_core::Value;
use babbel_kdl::ast::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use babbel_kdl::parse_document;

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
        babbel_json::nodes::Node::Object(map) => {
            Value::Object(map.into_iter().map(|(k, v)| (k, node_to_value(v))).collect())
        }
        babbel_json::nodes::Node::None => Value::Null,
    }
}

/// Convert KdlDocument AST into JSON protocol format for kdl-test.
fn kdl_document_to_test_json(doc: &KdlDocument) -> Value {
    let nodes: Vec<Value> = doc.nodes.iter().map(kdl_node_to_test_json).collect();
    Value::Array(nodes)
}

fn kdl_node_to_test_json(node: &KdlNode) -> Value {
    let mut obj = Vec::new();

    let ty_val = match &node.type_annotation {
        Some(t) => Value::String(t.clone()),
        None => Value::Null,
    };
    obj.push(("type".to_string(), ty_val));
    obj.push(("name".to_string(), Value::String(node.name.clone())));

    let mut args = Vec::new();
    let mut props = Vec::new();

    for entry in &node.entries {
        match entry {
            KdlEntry::Arg(ty, val) => {
                let mut arg_map = Vec::new();
                arg_map.push((
                    "type".to_string(),
                    match ty {
                        Some(t) => Value::String(t.clone()),
                        None => Value::Null,
                    },
                ));
                arg_map.push(("value".to_string(), kdl_val_to_test_json(val)));
                args.push(Value::Object(arg_map));
            }
            KdlEntry::Prop(name, ty, val) => {
                let mut prop_map = Vec::new();
                prop_map.push((
                    "type".to_string(),
                    match ty {
                        Some(t) => Value::String(t.clone()),
                        None => Value::Null,
                    },
                ));
                prop_map.push(("value".to_string(), kdl_val_to_test_json(val)));
                if let Some(pos) = props.iter().position(|(k, _): &(String, Value)| k == name) {
                    props[pos] = (name.clone(), Value::Object(prop_map));
                } else {
                    props.push((name.clone(), Value::Object(prop_map)));
                }
            }
        }
    }

    obj.push(("args".to_string(), Value::Array(args)));
    obj.push(("props".to_string(), Value::Object(props)));

    let children: Vec<Value> = node.children.iter().map(kdl_node_to_test_json).collect();
    obj.push(("children".to_string(), Value::Array(children)));

    Value::Object(obj)
}

fn kdl_val_to_test_json(val: &KdlValue) -> Value {
    let mut obj = Vec::new();
    match val {
        KdlValue::String(s) => {
            obj.push(("type".to_string(), Value::String("string".to_string())));
            obj.push(("value".to_string(), Value::String(s.clone())));
        }
        KdlValue::Integer(i) => {
            obj.push(("type".to_string(), Value::String("number".to_string())));
            obj.push(("value".to_string(), Value::String(format!("{}.0", i))));
        }
        KdlValue::Float(f) => {
            obj.push(("type".to_string(), Value::String("number".to_string())));
            let s = if f.is_nan() {
                "nan".to_string()
            } else if f.is_infinite() {
                if *f < 0.0 {
                    "-inf".to_string()
                } else {
                    "inf".to_string()
                }
            } else if f.fract() == 0.0 {
                format!("{}.0", f)
            } else {
                format!("{}", f)
            };
            obj.push(("value".to_string(), Value::String(s)));
        }
        KdlValue::Bool(b) => {
            obj.push(("type".to_string(), Value::String("boolean".to_string())));
            obj.push((
                "value".to_string(),
                Value::String(if *b { "true" } else { "false" }.to_string()),
            ));
        }
        KdlValue::Null => {
            obj.push(("type".to_string(), Value::String("null".to_string())));
        }
    }
    Value::Object(obj)
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

/// Discovers the path to the kdl-org/kdl-test test suite directory.
fn find_kdl_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("kdl-test"),
        manifest_dir.join("kdl-test"),
        manifest_dir
            .join("..")
            .join("..")
            .join("crates")
            .join("kdl")
            .join("tests")
            .join("kdl-test"),
        PathBuf::from("crates/kdl/tests/kdl-test"),
        PathBuf::from("tests/kdl-test"),
        PathBuf::from("kdl-test"),
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
        let test_cases = candidate.join("test_cases");
        if test_cases.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Checks semantic equality of two Values, allowing object keys to appear in different order
/// and tolerating equivalent numeric formats.
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
        (Value::String(s1), Value::String(s2)) => {
            if s1 == s2 {
                true
            } else {
                // Try comparing as numbers if strings represent numbers
                match (s1.parse::<f64>(), s2.parse::<f64>()) {
                    (Ok(n1), Ok(n2)) => {
                        if n1.is_nan() && n2.is_nan() {
                            true
                        } else if n1.is_infinite() && n2.is_infinite() {
                            n1.is_sign_positive() == n2.is_sign_positive()
                        } else {
                            (n1 - n2).abs() < 1e-9
                                || ((n1 - n2).abs() / n1.abs().max(n2.abs())) < 1e-6
                        }
                    }
                    _ => false,
                }
            }
        }
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

/// Categorizes a test case into a readable group.
fn categorize_test_path(path_str: &str, is_invalid: bool) -> &'static str {
    if is_invalid {
        if path_str.contains("escape") || path_str.contains("string") {
            "Invalid Strings & Escapes"
        } else if path_str.contains("type") || path_str.contains("ident") || path_str.contains("key") {
            "Invalid Types & Identifiers"
        } else {
            "Invalid Structure & Syntax"
        }
    } else if path_str.contains("multiline") || path_str.contains("raw_string") || path_str.contains("string") {
        "Valid Multiline & Raw Strings"
    } else if path_str.contains("prop") || path_str.contains("arg") {
        "Valid Arguments & Properties"
    } else if path_str.contains("num") || path_str.contains("hex") || path_str.contains("float") || path_str.contains("exp") || path_str.contains("bool") {
        "Valid Numbers & Keywords"
    } else if path_str.contains("child") || path_str.contains("block") || path_str.contains("slashdash") {
        "Valid Children & Blocks"
    } else {
        "Valid Basic & Nodes"
    }
}

struct EmbeddedVector {
    name: &'static str,
    kdl_input: &'static str,
    json_expected: Option<&'static str>,
    is_valid: bool,
}

const EMBEDDED_VECTORS: &[EmbeddedVector] = &[
    // Basic node
    EmbeddedVector {
        name: "basic_node",
        kdl_input: "node\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Node with bare argument
    EmbeddedVector {
        name: "arg_bare",
        kdl_input: "node arg\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"string","value":"arg"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Node with string argument
    EmbeddedVector {
        name: "string_arg",
        kdl_input: "node \"hello world\"\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"string","value":"hello world"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Node with integer argument
    EmbeddedVector {
        name: "int_arg",
        kdl_input: "node 123\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"123.0"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Node with negative integer
    EmbeddedVector {
        name: "neg_int_arg",
        kdl_input: "node -42\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"-42.0"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Node with property
    EmbeddedVector {
        name: "prop_basic",
        kdl_input: "node key=\"value\"\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[],"props":{"key":{"type":null,"value":{"type":"string","value":"value"}}},"children":[]}]"#),
        is_valid: true,
    },
    // Node with multiple properties & args
    EmbeddedVector {
        name: "args_and_props",
        kdl_input: "server \"production\" port=8080 active=#true\n",
        json_expected: Some(r#"[{"type":null,"name":"server","args":[{"type":null,"value":{"type":"string","value":"production"}}],"props":{"port":{"type":null,"value":{"type":"number","value":"8080.0"}},"active":{"type":null,"value":{"type":"boolean","value":"true"}}},"children":[]}]"#),
        is_valid: true,
    },
    // Node with children block
    EmbeddedVector {
        name: "node_children",
        kdl_input: "parent {\n    child_one\n    child_two\n}\n",
        json_expected: Some(r#"[{"type":null,"name":"parent","args":[],"props":{},"children":[{"type":null,"name":"child_one","args":[],"props":{},"children":[]},{"type":null,"name":"child_two","args":[],"props":{},"children":[]}]}]"#),
        is_valid: true,
    },
    // Slashdash node
    EmbeddedVector {
        name: "slashdash_node",
        kdl_input: "/- hidden_node\nvisible_node\n",
        json_expected: Some(r#"[{"type":null,"name":"visible_node","args":[],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Slashdash arg
    EmbeddedVector {
        name: "slashdash_arg",
        kdl_input: "node /- \"skip\" \"keep\"\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"string","value":"keep"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Slashdash prop
    EmbeddedVector {
        name: "slashdash_prop",
        kdl_input: "node /- drop=1 keep=2\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[],"props":{"keep":{"type":null,"value":{"type":"number","value":"2.0"}}},"children":[]}]"#),
        is_valid: true,
    },
    // Hex number
    EmbeddedVector {
        name: "hex_number",
        kdl_input: "node 0x1A2F\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"6703.0"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Octal number
    EmbeddedVector {
        name: "octal_number",
        kdl_input: "node 0o755\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"493.0"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Binary number
    EmbeddedVector {
        name: "binary_number",
        kdl_input: "node 0b1010\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"10.0"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Float number
    EmbeddedVector {
        name: "float_number",
        kdl_input: "node 3.14159\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"3.14159"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Node type annotation
    EmbeddedVector {
        name: "type_node",
        kdl_input: "(custom)node\n",
        json_expected: Some(r#"[{"type":"custom","name":"node","args":[],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Arg type annotation
    EmbeddedVector {
        name: "type_arg",
        kdl_input: "node (uuid)\"1234-5678\"\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":"uuid","value":{"type":"string","value":"1234-5678"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Prop type annotation
    EmbeddedVector {
        name: "type_prop",
        kdl_input: "node id=(u64)100\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[],"props":{"id":{"type":"u64","value":{"type":"number","value":"100.0"}}},"children":[]}]"#),
        is_valid: true,
    },
    // Line comments and block comments
    EmbeddedVector {
        name: "comments",
        kdl_input: "// comment\n/* block */\nnode /* inline */ 1\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"number","value":"1.0"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Semicolon separator
    EmbeddedVector {
        name: "semicolons",
        kdl_input: "node1; node2; node3\n",
        json_expected: Some(r#"[{"type":null,"name":"node1","args":[],"props":{},"children":[]},{"type":null,"name":"node2","args":[],"props":{},"children":[]},{"type":null,"name":"node3","args":[],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Escaped string
    EmbeddedVector {
        name: "escapes",
        kdl_input: "node \"hello\\nworld\\t\\s\"\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"string","value":"hello\nworld\t "}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Raw string
    EmbeddedVector {
        name: "raw_string",
        kdl_input: "node #\"C:\\Users\\User\"#\n",
        json_expected: Some(r#"[{"type":null,"name":"node","args":[{"type":null,"value":{"type":"string","value":"C:\\Users\\User"}}],"props":{},"children":[]}]"#),
        is_valid: true,
    },
    // Empty document
    EmbeddedVector {
        name: "empty_doc",
        kdl_input: "   \n// only comments\n   \n",
        json_expected: Some(r#"[]"#),
        is_valid: true,
    },

    // Invalid vectors
    EmbeddedVector {
        name: "invalid/unclosed_quote",
        kdl_input: "node \"unclosed",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/unclosed_block_comment",
        kdl_input: "node /* unclosed comment",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/unclosed_children",
        kdl_input: "node {\n  child\n",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/unclosed_type",
        kdl_input: "(type node",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/empty_type",
        kdl_input: "()node",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/bad_hex",
        kdl_input: "node 0xG12",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/bad_octal",
        kdl_input: "node 0o89",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/bad_binary",
        kdl_input: "node 0b102",
        json_expected: None,
        is_valid: false,
    },
    EmbeddedVector {
        name: "invalid/unexpected_closing_brace",
        kdl_input: "node }\n",
        json_expected: None,
        is_valid: false,
    },
];

#[test]
fn test_kdl_conformance_suite() {
    let start_time = Instant::now();
    let _panic_guard = PanicHookGuard::new_silent();

    let mut category_stats: BTreeMap<&'static str, CategoryStats> = BTreeMap::new();
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_panics = 0;

    let suite_dir = find_kdl_test_suite_dir();
    let is_upstream = suite_dir.is_some();

    if let Some(ref dir) = suite_dir {
        println!("Loaded official kdl-test suite from: {}", dir.display());
        let valid_dir = dir.join("test_cases").join("valid");
        let invalid_dir = dir.join("test_cases").join("invalid");

        // 1. Valid test cases
        if let Ok(entries) = fs::read_dir(&valid_dir) {
            let mut kdl_files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("kdl") {
                    kdl_files.push(path);
                }
            }
            kdl_files.sort();

            for kdl_path in kdl_files {
                let stem = kdl_path.file_stem().unwrap().to_str().unwrap();
                let json_path = valid_dir.join(format!("{}.json", stem));

                if !json_path.is_file() {
                    continue;
                }

                let cat = categorize_test_path(stem, false);
                let stat = category_stats.entry(cat).or_default();
                stat.total += 1;

                let kdl_str = match fs::read_to_string(&kdl_path) {
                    Ok(s) => s,
                    Err(e) => {
                        stat.failed += 1;
                        total_failed += 1;
                        eprintln!("[FAIL] {}: failed to read KDL file: {}", stem, e);
                        continue;
                    }
                };

                let json_str = match fs::read_to_string(&json_path) {
                    Ok(s) => s,
                    Err(e) => {
                        stat.failed += 1;
                        total_failed += 1;
                        eprintln!("[FAIL] {}: failed to read expected JSON: {}", stem, e);
                        continue;
                    }
                };

                let expected_val = match babbel_json::from_str(&json_str) {
                    Ok(node) => node_to_value(node),
                    Err(e) => {
                        stat.failed += 1;
                        total_failed += 1;
                        eprintln!("[FAIL] {}: failed to parse expected JSON: {}", stem, e);
                        continue;
                    }
                };

                let result = panic::catch_unwind(AssertUnwindSafe(|| parse_document(&kdl_str)));
                match result {
                    Ok(Ok(doc)) => {
                        let actual_val = kdl_document_to_test_json(&doc);
                        if values_equal_modulo_key_order(&actual_val, &expected_val) {
                            stat.passed += 1;
                            total_passed += 1;
                        } else {
                            stat.failed += 1;
                            total_failed += 1;
                            eprintln!(
                                "[FAIL] {}: parsed AST does not match expected JSON.\nActual:\n{:?}\nExpected:\n{:?}",
                                stem, actual_val, expected_val
                            );
                        }
                    }
                    Ok(Err(err)) => {
                        stat.failed += 1;
                        total_failed += 1;
                        eprintln!("[FAIL] {}: parse error: {:?}", stem, err);
                    }
                    Err(_) => {
                        stat.panics += 1;
                        stat.failed += 1;
                        total_panics += 1;
                        total_failed += 1;
                        eprintln!("[PANIC] {}: parser panicked", stem);
                    }
                }
            }
        }

        // 2. Invalid test cases
        if let Ok(entries) = fs::read_dir(&invalid_dir) {
            let mut kdl_files = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("kdl") {
                    kdl_files.push(path);
                }
            }
            kdl_files.sort();

            for kdl_path in kdl_files {
                let stem = kdl_path.file_stem().unwrap().to_str().unwrap();
                let cat = categorize_test_path(stem, true);
                let stat = category_stats.entry(cat).or_default();
                stat.total += 1;

                let kdl_str = match fs::read_to_string(&kdl_path) {
                    Ok(s) => s,
                    Err(_) => {
                        stat.failed += 1;
                        total_failed += 1;
                        continue;
                    }
                };

                let result = panic::catch_unwind(AssertUnwindSafe(|| parse_document(&kdl_str)));
                match result {
                    Ok(Ok(_doc)) => {
                        stat.failed += 1;
                        total_failed += 1;
                        eprintln!("[FAIL] {}: expected parse error, but parsed successfully", stem);
                    }
                    Ok(Err(_)) => {
                        // Expected rejection
                        stat.passed += 1;
                        total_passed += 1;
                    }
                    Err(_) => {
                        stat.panics += 1;
                        stat.failed += 1;
                        total_panics += 1;
                        total_failed += 1;
                        eprintln!("[PANIC] {}: invalid case caused panic", stem);
                    }
                }
            }
        }
    }

    // Always run embedded test vectors
    for vec in EMBEDDED_VECTORS {
        let cat = if vec.is_valid {
            "Embedded Valid Vectors"
        } else {
            "Embedded Invalid Vectors"
        };
        let stat = category_stats.entry(cat).or_default();
        stat.total += 1;

        let result = panic::catch_unwind(AssertUnwindSafe(|| parse_document(vec.kdl_input)));
        match result {
            Ok(Ok(doc)) => {
                if vec.is_valid {
                    if let Some(exp_json) = vec.json_expected {
                        let expected = node_to_value(babbel_json::from_str(exp_json).unwrap());
                        let actual = kdl_document_to_test_json(&doc);
                        if values_equal_modulo_key_order(&actual, &expected) {
                            stat.passed += 1;
                            total_passed += 1;
                        } else {
                            stat.failed += 1;
                            total_failed += 1;
                            eprintln!("[FAIL] {}: AST mismatch", vec.name);
                        }
                    } else {
                        stat.passed += 1;
                        total_passed += 1;
                    }
                } else {
                    stat.failed += 1;
                    total_failed += 1;
                    eprintln!("[FAIL] {}: expected parse error", vec.name);
                }
            }
            Ok(Err(_)) => {
                if !vec.is_valid {
                    stat.passed += 1;
                    total_passed += 1;
                } else {
                    stat.failed += 1;
                    total_failed += 1;
                    eprintln!("[FAIL] {}: unexpected error", vec.name);
                }
            }
            Err(_) => {
                stat.panics += 1;
                stat.failed += 1;
                total_panics += 1;
                total_failed += 1;
                eprintln!("[PANIC] {}: panicked", vec.name);
            }
        }
    }

    let duration = start_time.elapsed();
    let grand_total = total_passed + total_failed;
    let overall_pass_rate = if grand_total == 0 {
        0.0
    } else {
        (total_passed as f64 / grand_total as f64) * 100.0
    };

    println!("\n===================================================================================================");
    println!("                                   KDL CONFORMANCE REPORT                                          ");
    println!("===================================================================================================");
    println!(" Source: {}", if is_upstream { "kdl-org/kdl-test upstream + embedded fallback" } else { "embedded fallback" });
    println!(" Total Vectors Tested: {}", grand_total);
    println!(" Passed:               {}", total_passed);
    println!(" Failed:               {}", total_failed);
    println!(" Panics:               {}", total_panics);
    println!(" Conformance Score:    {:.2}%", overall_pass_rate);
    println!(" Duration:             {:.2?}", duration);
    println!("---------------------------------------------------------------------------------------------------");
    println!(
        " {:<40} | {:>7} | {:>7} | {:>7} | {:>7} | {:>9}",
        "Category", "Total", "Passed", "Failed", "Panics", "Pass Rate"
    );
    println!("---------------------------------------------------------------------------------------------------");

    for (cat_name, stats) in &category_stats {
        println!(
            " {:<40} | {:>7} | {:>7} | {:>7} | {:>7} | {:>8.2}%",
            cat_name, stats.total, stats.passed, stats.failed, stats.panics, stats.pass_rate()
        );
    }
    println!("===================================================================================================\n");

    // Zero-panic assertion is mandatory across all tests
    assert_eq!(total_panics, 0, "FATAL: Conformance runner encountered parser panics!");
    assert_eq!(total_failed, 0, "FATAL: Conformance runner encountered test failures!");
    assert!(total_passed > 0, "FATAL: Expected at least one test to pass!");
}
