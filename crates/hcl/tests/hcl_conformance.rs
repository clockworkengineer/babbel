//! Official HashiCorp HCL Test Suite Integration
//!
//! This test suite runs the official HCL test cases from:
//! https://github.com/kmoneil/hcl-test-suite
//!
//! The suite contains 3,000+ test cases covering:
//! - Lexical elements (identifiers, keywords, comments, whitespace, unicode)
//! - Numbers (integers, decimals, exponents, scientific notation)
//! - Structure (bodies, attributes, one-line blocks, multi-line blocks, labels)
//! - Collections (tuples, objects, nesting, trailing commas)
//! - Heredocs (standard `<<`, indented `<<-`, line stripping)
//! - Strings & Templates (interpolation, escapes, directives)
//! - Operators & Traversals

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Guard to silence panic output during malformed HCL test runs.
struct PanicHookGuard(Option<Box<dyn Fn(&panic::PanicHookInfo) + Send + Sync + 'static>>);

impl PanicHookGuard {
    fn new_silent() -> Self {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {
            // Suppress panic backtraces during fuzz/malformed test runs
        }));
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

#[derive(Debug, Clone)]
struct TestCase {
    id: String,
    category: String,
    expected_to_parse: bool,
    #[allow(dead_code)]
    disputed: bool,
    input_path: PathBuf,
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

/// Discovers the path to the hcl-test-suite directory.
fn find_hcl_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("hcl-test-suite"),
        manifest_dir.join("hcl-test-suite"),
        manifest_dir
            .join("..")
            .join("..")
            .join("crates")
            .join("hcl")
            .join("tests")
            .join("hcl-test-suite"),
        PathBuf::from("crates/hcl/tests/hcl-test-suite"),
        PathBuf::from("tests/hcl-test-suite"),
        PathBuf::from("hcl-test-suite"),
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

    candidates
        .into_iter()
        .find(|p| p.join("tests").join("native").exists())
}

/// Discovers all test cases from the test suite directory.
fn discover_test_cases(suite_dir: &Path) -> Vec<TestCase> {
    let mut cases = Vec::new();
    let native_dir = suite_dir.join("tests").join("native");

    fn collect_cases(dir: &Path, base_dir: &Path, out: &mut Vec<TestCase>) {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut subdirs = Vec::new();
            let mut test_json_path = None;

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    subdirs.push(path);
                } else if path.file_name().and_then(|s| s.to_str()) == Some("test.json") {
                    test_json_path = Some(path);
                }
            }

            if let Some(tj) = test_json_path {
                if let Ok(bytes) = fs::read(&tj) {
                    if let Ok(val) = babbel_json::from_bytes(&bytes) {
                        let op = val.get("op").and_then(|v| v.as_str()).unwrap_or("");
                        let valid = val
                            .get("expect")
                            .and_then(|e| e.get("valid"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let phase = val
                            .get("expect")
                            .and_then(|e| e.get("phase"))
                            .and_then(|v| v.as_str());
                        let disputed = val.get("status").and_then(|v| v.as_str()) == Some("disputed");

                        let input_filename = val
                            .get("input")
                            .and_then(|v| v.as_str())
                            .unwrap_or("input.hcl");
                        let input_path = dir.join(input_filename);

                        let rel = dir.strip_prefix(base_dir).unwrap_or(dir);
                        let mut components = rel.iter();
                        let category = components
                            .next()
                            .and_then(|s| s.to_str())
                            .unwrap_or("general")
                            .to_string();
                        let id = rel.to_string_lossy().replace('\\', "/");

                        // Conformance rule:
                        // parses = valid || (op != "parse" && phase != Some("parse"))
                        let expected_to_parse = valid || (op != "parse" && phase != Some("parse"));

                        let strict = std::env::var("HCL_STRICT").map(|v| v == "1" || v == "true").unwrap_or(false);
                        if input_path.exists() && (strict || !disputed) {
                            out.push(TestCase {
                                id,
                                category,
                                expected_to_parse,
                                disputed,
                                input_path,
                            });
                        }
                    }
                }
            }

            for sub in subdirs {
                collect_cases(&sub, base_dir, out);
            }
        }
    }

    collect_cases(&native_dir, &native_dir, &mut cases);
    cases.sort_by(|a, b| a.id.cmp(&b.id));
    cases
}

#[test]
fn test_official_hcl_test_suite() {
    let suite_dir = match find_hcl_test_suite_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  kmoneil/hcl-test-suite (HCL Conformance Suite) not found.");
            println!("  To download and install the official test suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_hcl_test_suite.ps1");
            println!("    (or ./scripts/fetch_hcl_test_suite.sh on Unix)");
            println!("============================================================\n");
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running Official kmoneil/hcl-test-suite Conformance");
    println!("  Root: {}", suite_dir.display());
    println!("============================================================");

    let test_cases = discover_test_cases(&suite_dir);
    if test_cases.is_empty() {
        println!("No test cases found in {}", suite_dir.display());
        return;
    }

    println!("Discovered {} test cases across categories.\n", test_cases.len());

    let _panic_guard = PanicHookGuard::new_silent();
    let start_time = Instant::now();

    let mut category_results: BTreeMap<String, CategoryStats> = BTreeMap::new();
    let mut overall = CategoryStats::default();
    let mut failures: Vec<String> = Vec::new();

    for test in &test_cases {
        let stats = category_results.entry(test.category.clone()).or_default();
        stats.total += 1;
        overall.total += 1;

        let bytes = match fs::read(&test.input_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| {
            babbel_hcl::from_bytes(&bytes)
        }));

        match parse_result {
            Ok(Ok(_value)) => {
                if test.expected_to_parse {
                    stats.passed += 1;
                    overall.passed += 1;
                } else {
                    stats.failed += 1;
                    overall.failed += 1;
                    failures.push(format!(
                        "[{}] {}: parser accepted invalid HCL (expected parse error)",
                        test.category, test.id
                    ));
                }
            }
            Ok(Err(err)) => {
                if !test.expected_to_parse {
                    stats.passed += 1;
                    overall.passed += 1;
                } else {
                    stats.failed += 1;
                    overall.failed += 1;
                    failures.push(format!(
                        "[{}] {}: parser rejected valid HCL with error: {}",
                        test.category, test.id, err
                    ));
                }
            }
            Err(_panic_err) => {
                stats.panics += 1;
                overall.panics += 1;
                stats.failed += 1;
                overall.failed += 1;
                failures.push(format!(
                    "[{}] {}: parser panicked during execution",
                    test.category, test.id
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

    let mut accepted_invalid = Vec::new();
    let mut rejected_valid = Vec::new();

    for f in &failures {
        if f.contains("accepted invalid HCL") {
            accepted_invalid.push(f.clone());
        } else if f.contains("rejected valid HCL") {
            rejected_valid.push(f.clone());
        }
    }

    println!("Total Failures: {} (Accepted Invalid: {}, Rejected Valid: {})", 
        failures.len(), accepted_invalid.len(), rejected_valid.len());

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = fs::write(manifest_dir.join("tests").join("failures.txt"), failures.join("\n"));

    if overall.panics > 0 {
        for f in &failures {
            if f.contains("panicked") {
                eprintln!("{}", f);
            }
        }
    }

    drop(_panic_guard);

    assert_eq!(
        overall.panics, 0,
        "Zero panics requirement in HCL conformance suite (got {})",
        overall.panics
    );

    assert!(
        overall.total >= 2000,
        "Expected at least 2,000 HCL test cases, got {}",
        overall.total
    );
}
