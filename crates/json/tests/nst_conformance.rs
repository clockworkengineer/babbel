//! Official JSONTestSuite (RFC 8259) Conformance Test Suite Integration
//!
//! This test suite runs the official test cases from JSONTestSuite
//! (created by Nicolas Seriot): https://github.com/nst/JSONTestSuite
//!
//! The suite contains comprehensive test cases covering:
//! - RFC 8259 JSON syntax (y_ must accept, n_ must reject, i_ implementation-defined)
//! - Number parsing, exponents, leading zero checks, large decimals
//! - String escaping, Unicode surrogate pairs, control character enforcement
//! - Structural nesting limits and whitespace handling
//! - Transform edge cases (test_transform/)

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Guard to silence panic output during malformed JSON test runs.
/// This keeps the test output focused on per-case pass/fail statistics
/// while allowing `catch_unwind` to detect panics.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    MustAccept,             // y_
    MustReject,             // n_
    ImplementationDefined,  // i_
    Transform,              // test_transform/
}

#[derive(Debug, Clone)]
struct TestCase {
    id: String,
    category: String,
    outcome: ExpectedOutcome,
    file_path: PathBuf,
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

/// Discovers the path to the JSONTestSuite directory.
fn find_json_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("JSONTestSuite"),
        manifest_dir.join("JSONTestSuite"),
        manifest_dir.join("..").join("..").join("crates").join("json").join("tests").join("JSONTestSuite"),
        PathBuf::from("crates/json/tests/JSONTestSuite"),
        PathBuf::from("tests/JSONTestSuite"),
        PathBuf::from("JSONTestSuite"),
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

    candidates.into_iter().find(|p| p.join("test_parsing").exists())
}

/// Discovers all test cases from the test suite directory.
fn discover_test_cases(suite_dir: &Path) -> Vec<TestCase> {
    let mut cases = Vec::new();

    // 1. Parsing test suite
    let parsing_dir = suite_dir.join("test_parsing");
    if let Ok(entries) = fs::read_dir(parsing_dir) {
        let mut file_entries: Vec<_> = entries.flatten().collect();
        file_entries.sort_by_key(|e| e.file_name());

        for entry in file_entries {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.ends_with(".json") {
                continue;
            }

            let (outcome, category) = if file_name.starts_with("y_") {
                (ExpectedOutcome::MustAccept, "Parsing (Must Accept)")
            } else if file_name.starts_with("n_") {
                (ExpectedOutcome::MustReject, "Parsing (Must Reject)")
            } else if file_name.starts_with("i_") {
                (ExpectedOutcome::ImplementationDefined, "Parsing (Implementation Defined)")
            } else {
                continue;
            };

            cases.push(TestCase {
                id: file_name,
                category: category.to_string(),
                outcome,
                file_path: path,
            });
        }
    }

    // 2. Transform test suite
    let transform_dir = suite_dir.join("test_transform");
    if let Ok(entries) = fs::read_dir(transform_dir) {
        let mut file_entries: Vec<_> = entries.flatten().collect();
        file_entries.sort_by_key(|e| e.file_name());

        for entry in file_entries {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.ends_with(".json") {
                continue;
            }

            cases.push(TestCase {
                id: file_name,
                category: "Transform (Edge Cases)".to_string(),
                outcome: ExpectedOutcome::Transform,
                file_path: path,
            });
        }
    }

    cases
}

#[test]
fn test_nst_json_conformance_suite() {
    let suite_dir = match find_json_test_suite_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  nst/JSONTestSuite (RFC 8259 Conformance Suite) not found.");
            println!("  To download and install the official test suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_json_test_suite.ps1");
            println!("    (or ./scripts/fetch_json_test_suite.sh on Unix)");
            println!("============================================================\n");
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running Official nst/JSONTestSuite (RFC 8259 Conformance)");
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

        let bytes = match fs::read(&test.file_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Parse inside catch_unwind to detect any panics
        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| {
            babbel_json::from_bytes(&bytes)
        }));

        match parse_result {
            Ok(Ok(_node)) => {
                match test.outcome {
                    ExpectedOutcome::MustAccept => {
                        stats.passed += 1;
                        overall.passed += 1;
                    }
                    ExpectedOutcome::MustReject => {
                        stats.failed += 1;
                        overall.failed += 1;
                        failures.push(format!(
                            "[{}] {}: parser accepted invalid JSON (expected rejection)",
                            test.category, test.id
                        ));
                    }
                    ExpectedOutcome::ImplementationDefined | ExpectedOutcome::Transform => {
                        // Acceptance is valid per RFC 8259
                        stats.passed += 1;
                        overall.passed += 1;
                    }
                }
            }
            Ok(Err(err)) => {
                match test.outcome {
                    ExpectedOutcome::MustAccept => {
                        stats.failed += 1;
                        overall.failed += 1;
                        failures.push(format!(
                            "[{}] {}: parser rejected valid JSON with error: {}",
                            test.category, test.id, err
                        ));
                    }
                    ExpectedOutcome::MustReject => {
                        // Correctly rejected
                        stats.passed += 1;
                        overall.passed += 1;
                    }
                    ExpectedOutcome::ImplementationDefined | ExpectedOutcome::Transform => {
                        // Rejection without panic is valid per RFC 8259
                        stats.passed += 1;
                        overall.passed += 1;
                    }
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

    // Drop silent panic guard before assertions
    drop(_panic_guard);

    assert!(
        failures.is_empty(),
        "JSONTestSuite conformance suite had {} failure(s):\n  {}",
        failures.len(),
        failures.iter().take(25).cloned().collect::<Vec<_>>().join("\n  ")
    );

    assert_eq!(
        overall.panics, 0,
        "Zero panics requirement in JSONTestSuite conformance suite (got {})",
        overall.panics
    );

    assert_eq!(
        overall.passed, overall.total,
        "JSONTestSuite conformance suite must achieve 100% pass rate (got {}/{})",
        overall.passed, overall.total
    );

    assert!(
        overall.total >= 300,
        "Expected at least 300 test cases executed, got {}",
        overall.total
    );
}
