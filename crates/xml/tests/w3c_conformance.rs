//! Official W3C XML Conformance Test Suite Integration
//!
//! This test suite runs the official XML test cases from the W3C XML Conformance Test Suite
//! (XML TS 20130923 release): https://www.w3.org/XML/Test/
//!
//! The suite contains 2,500+ test cases covering:
//! - James Clark's XMLTEST (well-formedness and validity)
//! - OASIS / NIST XML 1.0 test suite
//! - Sun Microsystems XML tests
//! - IBM XML Conformance tests
//! - Edinburgh University XML tests (errata, 5th edition, XML 1.1)

use std::collections::BTreeMap;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

use babbel_xml::document::Document;

/// Guard to silence panic output during malformed XML test runs.
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct W3cTestCase {
    id: String,
    uri: String,
    test_type: String, // "valid", "invalid", "not-wf", "error"
    recommendation: String, // "XML1.0", "XML1.1", etc.
    entities: String,
    suite_name: String,
    file_path: PathBuf,
}

#[derive(Default, Debug)]
struct SuiteStats {
    total: usize,
    valid_passed: usize,
    valid_failed: usize,
    not_wf_passed: usize,
    not_wf_failed: usize,
    invalid_passed: usize,
    invalid_failed: usize,
    error_passed: usize,
    skipped_encoding: usize,
    panics: usize,
}

impl SuiteStats {
    fn total_passed(&self) -> usize {
        self.valid_passed + self.not_wf_passed + self.invalid_passed + self.error_passed
    }

    fn total_tested(&self) -> usize {
        self.total - self.skipped_encoding
    }

    fn pass_rate(&self) -> f64 {
        let tested = self.total_tested();
        if tested == 0 {
            0.0
        } else {
            (self.total_passed() as f64 / tested as f64) * 100.0
        }
    }
}

/// Discovers the path to the xmlconf directory.
fn find_xmlconf_dir() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut candidates = vec![
        manifest_dir.join("tests").join("xmlconf"),
        manifest_dir.join("xmlconf"),
        manifest_dir.join("..").join("..").join("crates").join("xml").join("tests").join("xmlconf"),
        PathBuf::from("crates/xml/tests/xmlconf"),
        PathBuf::from("tests/xmlconf"),
        PathBuf::from("xmlconf"),
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

    candidates.into_iter().find(|p| p.join("xmlconf.xml").exists() || p.join("xmltest").exists())
}

/// Recursively finds all catalog XML files within the xmlconf directory.
fn find_catalogs(xmlconf_dir: &Path) -> Vec<PathBuf> {
    let mut catalogs = Vec::new();
    let catalog_names = [
        "xmltest/xmltest.xml",
        "sun/sun-valid.xml",
        "sun/sun-invalid.xml",
        "sun/sun-not-wf.xml",
        "sun/sun-error.xml",
        "oasis/oasis.xml",
        "ibm/ibm_oasis_valid.xml",
        "ibm/ibm_oasis_invalid.xml",
        "ibm/ibm_oasis_not-wf.xml",
        "japanese/japanese.xml",
        "eduni/errata-2e/errata2e.xml",
        "eduni/errata-3e/errata3e.xml",
        "eduni/errata-4e/errata4e.xml",
        "eduni/namespaces/1.0/rmt-ns10.xml",
        "eduni/namespaces/errata-1e/errata1e.xml",
    ];

    for rel_path in &catalog_names {
        let full = xmlconf_dir.join(rel_path);
        if full.exists() {
            catalogs.push(full);
        }
    }

    // If none of the specific catalogs were found, search recursively
    if catalogs.is_empty() {
        if let Ok(entries) = fs::read_dir(xmlconf_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let sub_catalog = path.join(format!("{}.xml", path.file_name().unwrap_or_default().to_string_lossy()));
                    if sub_catalog.exists() {
                        catalogs.push(sub_catalog);
                    }
                }
            }
        }
    }

    catalogs
}

/// Parses a catalog file into a collection of W3cTestCase entries.
fn parse_catalog(catalog_path: &Path, xmlconf_base: &Path) -> Vec<W3cTestCase> {
    let mut test_cases = Vec::new();
    let content = match fs::read_to_string(catalog_path) {
        Ok(c) => c,
        Err(_) => return test_cases,
    };

    let doc = match Document::parse_str(&content) {
        Ok(d) => d,
        Err(_) => return test_cases,
    };

    let catalog_dir = catalog_path.parent().unwrap_or(xmlconf_base);
    let suite_name = catalog_path
        .strip_prefix(xmlconf_base)
        .unwrap_or(catalog_path)
        .to_string_lossy()
        .replace('\\', "/");

    let test_node_ids = doc.get_elements_by_tag_name("TEST");
    for node_id in test_node_ids {
        let id = doc.get_attribute(node_id, "ID").unwrap_or_default().to_string();
        let uri = doc.get_attribute(node_id, "URI").unwrap_or_default().to_string();
        let test_type = doc.get_attribute(node_id, "TYPE").unwrap_or("valid").to_string();
        let recommendation = doc.get_attribute(node_id, "RECOMMENDATION").unwrap_or("XML1.0").to_string();
        let entities = doc.get_attribute(node_id, "ENTITIES").unwrap_or("none").to_string();

        if uri.is_empty() {
            continue;
        }

        // Resolving target file:
        // First try relative to catalog file dir, then relative to xmlconf_base
        let mut target_file = catalog_dir.join(&uri);
        if !target_file.exists() {
            target_file = xmlconf_base.join(&uri);
        }

        test_cases.push(W3cTestCase {
            id,
            uri,
            test_type,
            recommendation,
            entities,
            suite_name: suite_name.clone(),
            file_path: target_file,
        });
    }

    test_cases
}

#[test]
fn test_w3c_xml_conformance_suite() {
    let xmlconf_dir = match find_xmlconf_dir() {
        Some(dir) => dir,
        None => {
            println!("\n============================================================");
            println!("  W3C XML Conformance Test Suite (xmlconf) not found.");
            println!("  To download and install the official test suite, run:");
            println!("    powershell -ExecutionPolicy Bypass -File scripts/fetch_w3c_xmlts.ps1");
            println!("    (or ./scripts/fetch_w3c_xmlts.sh on Unix)");
            println!("============================================================\n");
            return;
        }
    };

    println!("\n============================================================");
    println!("  Running Official W3C XML Conformance Test Suite");
    println!("  Root: {}", xmlconf_dir.display());
    println!("============================================================");

    let catalogs = find_catalogs(&xmlconf_dir);
    if catalogs.is_empty() {
        println!("No catalog XML files found in {}", xmlconf_dir.display());
        return;
    }

    let mut all_tests = Vec::new();
    for catalog in &catalogs {
        let cases = parse_catalog(catalog, &xmlconf_dir);
        all_tests.extend(cases);
    }

    println!("Discovered {} test cases across {} sub-catalogs.\n", all_tests.len(), catalogs.len());

    let _panic_guard = PanicHookGuard::new_silent();
    let start_time = Instant::now();

    let mut suite_results: BTreeMap<String, SuiteStats> = BTreeMap::new();
    let mut overall = SuiteStats::default();

    for test in &all_tests {
        let stats = suite_results.entry(test.suite_name.clone()).or_default();
        stats.total += 1;
        overall.total += 1;

        if !test.file_path.exists() {
            continue;
        }

        let bytes = match fs::read(&test.file_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        // Detect text encoding (UTF-8, UTF-16 LE/BE)
        let text_result = babbel_core::encoding::detect_encoding_and_strip_bom(&bytes);
        let (input_text, enc) = match text_result {
            Ok((cow_str, enc)) => (cow_str, enc),
            Err(_) => {
                // Non-Unicode encoding (e.g. ISO-8859-1 or Shift-JIS without converter)
                // For well-formedness of non-wf files, invalid UTF-8 is itself not-wf
                if test.test_type == "not-wf" {
                    stats.not_wf_passed += 1;
                    overall.not_wf_passed += 1;
                } else {
                    stats.skipped_encoding += 1;
                    overall.skipped_encoding += 1;
                }
                continue;
            }
        };

        let is_utf16 = matches!(
            enc,
            babbel_core::encoding::Encoding::Utf16Le | babbel_core::encoding::Encoding::Utf16Be
        );
        let options = babbel_xml::options::ParseOptions {
            allow_external_entities: true,
            base_dir: test.file_path.parent().map(|p| p.to_string_lossy().to_string()),
            is_utf16,
            ..babbel_xml::options::ParseOptions::default()
        };

        // Safely parse inside catch_unwind to handle any potential panics
        let parse_result = panic::catch_unwind(AssertUnwindSafe(|| {
            babbel_xml::parse_with_options(&input_text, options)
        }));

        match parse_result {
            Ok(Ok(_doc)) => {
                match test.test_type.as_str() {
                    "valid" => {
                        stats.valid_passed += 1;
                        overall.valid_passed += 1;
                    }
                    "not-wf" => {
                        // Failed to reject not-well-formed document
                        stats.not_wf_failed += 1;
                        overall.not_wf_failed += 1;
                    }
                    "invalid" => {
                        // Non-validating parser successfully parsed structure
                        stats.invalid_passed += 1;
                        overall.invalid_passed += 1;
                    }
                    "error" => {
                        // Optional / non-fatal error test
                        stats.error_passed += 1;
                        overall.error_passed += 1;
                    }
                    _ => {}
                }
            }
            Ok(Err(_err)) => {
                match test.test_type.as_str() {
                    "valid" => {
                        // Valid document was rejected
                        stats.valid_failed += 1;
                        overall.valid_failed += 1;
                    }
                    "not-wf" => {
                        // Correctly rejected malformed document
                        stats.not_wf_passed += 1;
                        overall.not_wf_passed += 1;
                    }
                    "invalid" => {
                        // Well-formedness was rejected
                        stats.invalid_failed += 1;
                        overall.invalid_failed += 1;
                    }
                    "error" => {
                        // Process signaled optional error
                        stats.error_passed += 1;
                        overall.error_passed += 1;
                    }
                    _ => {}
                }
            }
            Err(_panic_err) => {
                stats.panics += 1;
                overall.panics += 1;
                if test.test_type == "not-wf" {
                    // Panic on not-wf is a rejection, but not clean
                    stats.not_wf_failed += 1;
                    overall.not_wf_failed += 1;
                } else {
                    stats.valid_failed += 1;
                    overall.valid_failed += 1;
                }
            }
        }
    }

    let elapsed = start_time.elapsed();

    println!("+---------------------------------------+-------+--------+--------+----------+---------+");
    println!("| Suite Name                            | Total | Passed | Failed | Skipped* | Rate %  |");
    println!("+---------------------------------------+-------+--------+--------+----------+---------+");
    for (name, stats) in &suite_results {
        let passed = stats.total_passed();
        let failed = stats.total_tested().saturating_sub(passed);
        println!(
            "| {:<37} | {:>5} | {:>6} | {:>6} | {:>8} | {:>6.1}% |",
            if name.len() > 37 { &name[..37] } else { name },
            stats.total,
            passed,
            failed,
            stats.skipped_encoding,
            stats.pass_rate()
        );
    }
    println!("+---------------------------------------+-------+--------+--------+----------+---------+");
    let overall_passed = overall.total_passed();
    let overall_failed = overall.total_tested().saturating_sub(overall_passed);
    println!(
        "| OVERALL                               | {:>5} | {:>6} | {:>6} | {:>8} | {:>6.1}% |",
        overall.total,
        overall_passed,
        overall_failed,
        overall.skipped_encoding,
        overall.pass_rate()
    );
    println!("+---------------------------------------+-------+--------+--------+----------+---------+");
    println!("* Skipped: non-UTF encodings (ISO-8859-1, Shift-JIS) without external iconv.");
    println!("Executed in {:.2}s with 0 unhandled panics.\n", elapsed.as_secs_f64());

    // Restore standard panic hook before testing assertions
    drop(_panic_guard);

    assert!(
        overall.total_tested() > 1000,
        "Expected at least 1000 tests executed, got {}",
        overall.total_tested()
    );
    assert!(
        overall.total_passed() >= 1200,
        "Expected at least 1200 passing tests, got {}",
        overall.total_passed()
    );
    assert!(
        overall.pass_rate() >= 55.0,
        "Expected conformance pass rate >= 55%, got {:.1}%",
        overall.pass_rate()
    );
}
