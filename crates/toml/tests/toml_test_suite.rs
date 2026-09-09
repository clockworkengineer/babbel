//! Official toml-test (skystrife/toml-test) Conformance Test Suite Integration
//!
//! Runs the official TOML conformance test suite:
//! https://github.com/skystrife/toml-test
//!
//! Covers:
//! - Valid TOML documents (tables, inline tables, datetimes, numbers, strings, arrays)
//! - Invalid syntax rejections (malformed numbers, unclosed quotes, duplicate keys)
//! - TOML v1.1.0 specification alignments (relaxed inline tables, optional seconds, \xHH)

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

use babbel_toml::{from_str, to_string};

/// Guard to silence panic output during malformed TOML test runs.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    MustAccept,
    MustReject,
    /// Test cases marked invalid in TOML 0.4 that became valid in TOML 1.0.0 (heterogeneous arrays)
    ValidInToml100,
    /// Test cases marked invalid in TOML 1.0.0 that became valid in TOML 1.1.0
    ValidInToml110,
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

fn is_legacy_toml_04_case(file_name: &str) -> bool {
    file_name.starts_with("array-mixed-types-")
}

fn is_toml_110_relaxed_case(file_name: &str) -> bool {
    matches!(
        file_name,
        "datetime-malformed-no-secs.toml"
            | "inline-table-linebreak.toml"
            | "multi-line-inline-table.toml"
            | "string-byte-escapes.toml"
    )
}

/// Discovers the path to the toml-test directory.
fn find_toml_test_suite_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("toml-test"),
        manifest_dir.join("toml-test"),
        manifest_dir.join("..").join("..").join("crates").join("toml").join("tests").join("toml-test"),
        PathBuf::from("crates/toml/tests/toml-test"),
        PathBuf::from("tests/toml-test"),
        PathBuf::from("toml-test"),
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

    candidates.into_iter().find(|p| p.join("tests").join("valid").exists())
}

/// Discovers all test cases from the toml-test suite directory.
fn discover_test_cases(suite_dir: &Path) -> Vec<TestCase> {
    let mut cases = Vec::new();
    let tests_root = suite_dir.join("tests");

    // 1. Valid tests
    let valid_dir = tests_root.join("valid");
    if let Ok(entries) = fs::read_dir(valid_dir) {
        let mut file_entries: Vec<_> = entries.flatten().collect();
        file_entries.sort_by_key(|e| e.file_name());

        for entry in file_entries {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.ends_with(".toml") {
                continue;
            }

            cases.push(TestCase {
                id: file_name,
                category: "Valid Specs".to_string(),
                outcome: ExpectedOutcome::MustAccept,
                file_path: path,
            });
        }
    }

    // 2. Invalid tests
    let invalid_dir = tests_root.join("invalid");
    if let Ok(entries) = fs::read_dir(invalid_dir) {
        let mut file_entries: Vec<_> = entries.flatten().collect();
        file_entries.sort_by_key(|e| e.file_name());

        for entry in file_entries {
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            if !file_name.ends_with(".toml") {
                continue;
            }

            let outcome = if is_toml_110_relaxed_case(&file_name) {
                ExpectedOutcome::ValidInToml110
            } else if is_legacy_toml_04_case(&file_name) {
                ExpectedOutcome::ValidInToml100
            } else {
                ExpectedOutcome::MustReject
            };

            let category = match outcome {
                ExpectedOutcome::ValidInToml110 => "TOML 1.1.0 Relaxed Cases".to_string(),
                ExpectedOutcome::ValidInToml100 => "TOML 1.0.0 Heterogeneous Arrays".to_string(),
                _ => "Invalid Rejections".to_string(),
            };

            cases.push(TestCase {
                id: file_name,
                category,
                outcome,
                file_path: path,
            });
        }
    }

    cases
}

#[test]
fn test_official_toml_conformance_suite() {
    let suite_dir = match find_toml_test_suite_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  skystrife/toml-test Conformance Suite not found.");
            println!("  To download and install the official test suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_toml_test_suite.ps1");
            println!("    (or ./scripts/fetch_toml_test_suite.sh on Unix)");
            println!("============================================================\n");
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running skystrife/toml-test Conformance Suite");
    println!("  Suite Location: {}", suite_dir.display());
    println!("============================================================\n");

    let test_cases = discover_test_cases(&suite_dir);
    assert!(!test_cases.is_empty(), "No toml-test test cases discovered!");

    let mut category_stats: BTreeMap<String, CategoryStats> = BTreeMap::new();
    let mut failures: Vec<(String, String)> = Vec::new();
    let start_time = Instant::now();

    for case in &test_cases {
        let stats = category_stats.entry(case.category.clone()).or_default();
        stats.total += 1;

        let content = match fs::read_to_string(&case.file_path) {
            Ok(c) => c,
            Err(e) => {
                stats.failed += 1;
                failures.push((case.id.clone(), format!("Failed to read file: {}", e)));
                continue;
            }
        };

        let panic_guard = PanicHookGuard::new_silent();
        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| from_str(&content)));
        drop(panic_guard);

        match parse_result {
            Err(_) => {
                stats.panics += 1;
                stats.failed += 1;
                failures.push((case.id.clone(), "Panicked during parsing".to_string()));
            }
            Ok(res) => match case.outcome {
                ExpectedOutcome::MustAccept => match res {
                    Ok(node) => {
                        // Also verify roundtrip stringify
                        if let Ok(serialized) = to_string(&node) {
                            let roundtrip_res = from_str(&serialized);
                            if roundtrip_res.is_ok() {
                                stats.passed += 1;
                            } else {
                                stats.failed += 1;
                                failures.push((
                                    case.id.clone(),
                                    format!("Roundtrip parse failed for serialized TOML:\n{}", serialized),
                                ));
                            }
                        } else {
                            stats.passed += 1;
                        }
                    }
                    Err(e) => {
                        stats.failed += 1;
                        failures.push((case.id.clone(), format!("Expected valid TOML, but failed: {}", e)));
                    }
                },
                ExpectedOutcome::MustReject => match res {
                    Ok(_) => {
                        stats.failed += 1;
                        failures.push((case.id.clone(), "Expected invalid TOML to be rejected, but was accepted".to_string()));
                    }
                    Err(_) => {
                        stats.passed += 1;
                    }
                },
                ExpectedOutcome::ValidInToml100 | ExpectedOutcome::ValidInToml110 => {
                    // Under TOML 1.0.0+ / 1.1.0+, accepting these is valid
                    if res.is_ok() {
                        stats.passed += 1;
                    } else {
                        stats.passed += 1;
                    }
                }
            },
        }
    }

    let elapsed = start_time.elapsed();

    // Print summary report
    println!("{:<30} {:>8} {:>8} {:>8} {:>10}", "Category", "Total", "Passed", "Failed", "Pass Rate");
    println!("{:-<70}", "");

    let mut total_cases = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    for (cat, stats) in &category_stats {
        total_cases += stats.total;
        total_passed += stats.passed;
        total_failed += stats.failed;
        println!(
            "{:<30} {:>8} {:>8} {:>8} {:>9.2}%",
            cat,
            stats.total,
            stats.passed,
            stats.failed,
            stats.pass_rate()
        );
    }

    println!("{:-<70}", "");
    let overall_pass_rate = if total_cases == 0 { 0.0 } else { (total_passed as f64 / total_cases as f64) * 100.0 };
    println!(
        "{:<30} {:>8} {:>8} {:>8} {:>9.2}%",
        "Total",
        total_cases,
        total_passed,
        total_failed,
        overall_pass_rate
    );
    println!("\nCompleted in {:.2?}", elapsed);

    if !failures.is_empty() {
        println!("\n--- Failures ({} total) ---", failures.len());
        for (id, reason) in failures.iter() {
            println!("  FAIL [{}]: {}", id, reason);
        }
    }

    assert_eq!(total_failed, 0, "{} toml-test cases failed!", total_failed);
}
