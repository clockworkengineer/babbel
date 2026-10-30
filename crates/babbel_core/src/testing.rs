//! Reusable test utilities and conformance test harness primitives.
//!
//! Provides a standardized, zero-boilerplate foundation for format conformance test suites:
//! - [`PanicHookGuard`]: Silences console noise and tallies panics during malformed/fuzz test cases.
//! - [`find_test_suite_dir`]: Resolves external suite directories with fallback candidate searching and `suite_paths.txt`.
//! - [`CategoryStats`]: Accumulates passing, failing, and panic counts per specification category.
//! - [`parse_dense_hex`]: Parses dense hexadecimal strings (`"01020304"`) into raw bytes.
//! - [`ConformanceReport`]: Formats and renders standardized ASCII test summary tables.

use std::fs;
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Guard that suppresses panic backtraces during malformed/fuzz conformance runs,
/// tallies panics, and restores the original panic hook upon drop.
pub struct PanicHookGuard {
    panic_count: Arc<AtomicUsize>,
    prev_hook: Option<Box<dyn Fn(&PanicHookInfo) + Send + Sync + 'static>>,
}

impl PanicHookGuard {
    /// Creates a new panic guard that records panics while preserving default backtrace printing.
    pub fn new() -> Self {
        let panic_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&panic_count);
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            count_clone.fetch_add(1, Ordering::SeqCst);
            prev_hook(info);
        }));
        Self {
            panic_count,
            prev_hook: None,
        }
    }

    /// Creates a new silent panic guard that suppresses panic output and counts panics.
    pub fn new_silent() -> Self {
        let panic_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&panic_count);
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(move |_info| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        }));
        Self {
            panic_count,
            prev_hook: Some(prev_hook),
        }
    }

    /// Returns the number of panics caught since the guard was created.
    pub fn panics(&self) -> usize {
        self.panic_count.load(Ordering::SeqCst)
    }
}

impl Default for PanicHookGuard {
    fn default() -> Self {
        Self::new_silent()
    }
}

impl Drop for PanicHookGuard {
    fn drop(&mut self) {
        if let Some(hook) = self.prev_hook.take() {
            panic::set_hook(hook);
        }
    }
}

/// Discovers the path to an external conformance test suite directory.
///
/// Searches candidate relative directories and inspects `tests/suite_paths.txt` if available.
pub fn find_test_suite_dir(
    manifest_dir: &str,
    suite_subpath: &str,
    marker_file_or_dir: &str,
) -> Option<PathBuf> {
    let manifest = PathBuf::from(manifest_dir);
    let mut candidates = vec![
        manifest.join("tests").join(suite_subpath),
        manifest.join(suite_subpath),
        manifest.join("..").join("..").join(suite_subpath),
        PathBuf::from(format!("crates/{}/tests/{}", suite_subpath, suite_subpath)),
        PathBuf::from(format!("tests/{}", suite_subpath)),
        PathBuf::from(suite_subpath),
    ];

    // Read suite_paths.txt if available in tests/ or crate root
    let suite_paths_candidates = [
        manifest.join("tests").join("suite_paths.txt"),
        manifest.join("suite_paths.txt"),
        PathBuf::from("tests/suite_paths.txt"),
        PathBuf::from(format!("crates/{}/tests/suite_paths.txt", suite_subpath)),
    ];

    for file in &suite_paths_candidates {
        if let Ok(content) = fs::read_to_string(file) {
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    candidates.push(manifest.join(line));
                    candidates.push(manifest.join("tests").join(line));
                    candidates.push(PathBuf::from(line));
                }
            }
            break;
        }
    }

    candidates
        .into_iter()
        .find(|p| p.join(marker_file_or_dir).exists())
}

/// Statistics accumulator for a specification conformance category.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CategoryStats {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub panics: usize,
}

impl CategoryStats {
    /// Creates a new empty category statistics tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the outcome of a single test case.
    pub fn record(&mut self, ok: bool, panicked: bool) {
        self.total += 1;
        if ok {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        if panicked {
            self.panics += 1;
        }
    }

    /// Calculates the percentage pass rate (0.0% to 100.0%).
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

/// Parses a dense or hyphen-delimited hexadecimal string into raw bytes.
///
/// Handles both `"01020304"` and `"01-02-03-04"`.
pub fn parse_dense_hex(s: &str) -> Result<Vec<u8>, String> {
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

/// Standardized ASCII reporting runner for specification conformance test suites.
pub struct ConformanceReport {
    pub title: &'static str,
    pub categories: Vec<(&'static str, CategoryStats)>,
    pub start_time: Instant,
}

impl ConformanceReport {
    /// Creates a new report with the given suite title.
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            categories: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Adds a named category and its accumulated statistics.
    pub fn add_category(&mut self, name: &'static str, stats: CategoryStats) {
        self.categories.push((name, stats));
    }

    /// Calculates total statistics across all categories.
    pub fn totals(&self) -> CategoryStats {
        let mut total = CategoryStats::default();
        for (_, cat) in &self.categories {
            total.total += cat.total;
            total.passed += cat.passed;
            total.failed += cat.failed;
            total.panics += cat.panics;
        }
        total
    }

    /// Prints a beautifully formatted ASCII table summarizing all categories.
    pub fn print_table(&self) {
        let elapsed = self.start_time.elapsed();
        let totals = self.totals();

        println!(
            "\n================================================================================="
        );
        println!("{:^81}", self.title);
        println!(
            "================================================================================="
        );
        println!(
            " {:<38} | {:>7} | {:>7} | {:>7} | {:>8}",
            "Category", "Vectors", "Passed", "Panics", "Status"
        );
        println!(
            "---------------------------------------------------------------------------------"
        );

        for (name, stats) in &self.categories {
            if stats.total > 0 {
                let status = if stats.passed == stats.total && stats.panics == 0 {
                    "PASSED"
                } else {
                    "FAILED"
                };
                println!(
                    " {:<38} | {:>7} | {:>7} | {:>7} | {:>8}",
                    name, stats.total, stats.passed, stats.panics, status
                );
            }
        }

        println!(
            "---------------------------------------------------------------------------------"
        );
        let overall_status = if totals.passed == totals.total && totals.panics == 0 {
            "100% OK"
        } else {
            "FAIL"
        };
        println!(
            " {:<38} | {:>7} | {:>7} | {:>7} | {:>8}",
            "TOTAL", totals.total, totals.passed, totals.panics, overall_status
        );
        println!(
            "================================================================================="
        );
        println!("Execution Time: {:.2?}", elapsed);
        println!("Zero Panics Guarantee: {} panics recorded", totals.panics);
        println!(
            "=================================================================================\n"
        );
    }

    /// Asserts 0 panics and 0 failures, ensuring strict conformance.
    pub fn assert_all_passed(&self) {
        let totals = self.totals();
        assert_eq!(
            totals.panics, 0,
            "FATAL: {} conformance runner encountered panics!",
            self.title
        );
        assert_eq!(
            totals.failed, 0,
            "FATAL: {} conformance runner encountered test failures!",
            self.title
        );
        assert!(
            totals.passed > 0,
            "FATAL: Expected at least one test to pass in {}!",
            self.title
        );
    }
}

// ==========================================
// Mock I/O Failure Injection Harnesses
// ==========================================

use crate::error::ErrorCode;
use crate::io::traits::{IByteReader, IByteWriter, ICharStream, IDestination, IPeekable, ISource};

/// A mock input source designed to inject errors, early EOFs, and artificial stream interruptions.
pub struct FaultySource<'a> {
    data: &'a [u8],
    pos: usize,
    fail_after_bytes: usize,
    has_failed: bool,
    error_to_return: ErrorCode,
}

impl<'a> FaultySource<'a> {
    /// Creates a source that succeeds up to `fail_after_bytes` and then fails with `error`.
    pub fn fail_after(data: &'a [u8], fail_after_bytes: usize, error: ErrorCode) -> Self {
        Self {
            data,
            pos: 0,
            fail_after_bytes,
            has_failed: false,
            error_to_return: error,
        }
    }

    /// Creates a source that unexpectedly truncates after `truncate_at` bytes.
    pub fn truncate_at(data: &'a [u8], truncate_at: usize) -> Self {
        Self::fail_after(data, truncate_at, ErrorCode::UnexpectedEof)
    }

    /// Returns the number of bytes read so far.
    pub fn bytes_read(&self) -> usize {
        self.pos
    }

    /// Whether the fault has triggered.
    pub fn has_faulted(&self) -> bool {
        self.has_failed
    }

    /// The injected error code.
    pub fn injected_error(&self) -> ErrorCode {
        self.error_to_return
    }
}

impl<'a> IByteReader for FaultySource<'a> {
    fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.fail_after_bytes {
            self.has_failed = true;
            return None;
        }
        if self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            Some(b)
        } else {
            None
        }
    }
}

impl<'a> IPeekable for FaultySource<'a> {
    fn peek_byte(&mut self) -> Option<u8> {
        if self.pos >= self.fail_after_bytes || self.pos >= self.data.len() {
            None
        } else {
            Some(self.data[self.pos])
        }
    }
}

impl<'a> ICharStream for FaultySource<'a> {
    fn next(&mut self) {
        let _ = self.read_byte();
    }

    fn current(&mut self) -> Option<char> {
        if self.pos >= self.fail_after_bytes || self.pos >= self.data.len() {
            None
        } else {
            Some(self.data[self.pos] as char)
        }
    }

    fn more(&mut self) -> bool {
        self.pos < self.fail_after_bytes && self.pos < self.data.len()
    }
}

impl<'a> ISource for FaultySource<'a> {
    fn next(&mut self) {
        let _ = self.read_byte();
    }

    fn current(&mut self) -> Option<char> {
        if self.pos >= self.fail_after_bytes || self.pos >= self.data.len() {
            None
        } else {
            Some(self.data[self.pos] as char)
        }
    }

    fn more(&mut self) -> bool {
        self.pos < self.fail_after_bytes && self.pos < self.data.len()
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.has_failed = false;
    }

    fn can_rewind(&self) -> bool {
        true
    }
}

/// A mock output destination designed to inject write failures after a byte threshold.
pub struct FaultyDestination {
    buffer: Vec<u8>,
    fail_after_bytes: usize,
    has_failed: bool,
    error_code: ErrorCode,
}

impl FaultyDestination {
    /// Creates a destination that fails when written bytes reach or exceed `fail_after_bytes`.
    pub fn fail_after(fail_after_bytes: usize, error_code: ErrorCode) -> Self {
        Self {
            buffer: Vec::new(),
            fail_after_bytes,
            has_failed: false,
            error_code,
        }
    }

    /// Returns the bytes successfully written before failure.
    pub fn written_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Whether the fault has triggered.
    pub fn has_faulted(&self) -> bool {
        self.has_failed
    }

    /// The configured error code.
    pub fn error_code(&self) -> ErrorCode {
        self.error_code
    }
}

impl IDestination for FaultyDestination {
    fn add_byte(&mut self, byte: u8) {
        if self.buffer.len() >= self.fail_after_bytes {
            self.has_failed = true;
            return;
        }
        self.buffer.push(byte);
    }

    fn add_bytes(&mut self, bytes: &str) {
        self.add_raw_bytes(bytes.as_bytes());
    }

    fn add_raw_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            if self.buffer.len() >= self.fail_after_bytes {
                self.has_failed = true;
                break;
            }
            self.buffer.push(b);
        }
    }

    fn last(&self) -> Option<u8> {
        self.buffer.last().copied()
    }
}

impl IByteWriter for FaultyDestination {
    fn write_byte(&mut self, byte: u8) -> Result<(), ErrorCode> {
        if self.buffer.len() >= self.fail_after_bytes {
            self.has_failed = true;
            return Err(self.error_code);
        }
        self.buffer.push(byte);
        Ok(())
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ErrorCode> {
        for &b in bytes {
            if self.buffer.len() >= self.fail_after_bytes {
                self.has_failed = true;
                return Err(self.error_code);
            }
            self.buffer.push(b);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faulty_source_injection() {
        let payload = b"hello world";
        let mut src = FaultySource::truncate_at(payload, 5);

        assert_eq!(src.read_byte(), Some(b'h'));
        assert_eq!(src.read_byte(), Some(b'e'));
        assert_eq!(src.read_byte(), Some(b'l'));
        assert_eq!(src.read_byte(), Some(b'l'));
        assert_eq!(src.read_byte(), Some(b'o'));

        // 6th byte must trigger fault
        assert_eq!(src.read_byte(), None);
        assert!(src.has_faulted());
        assert_eq!(src.bytes_read(), 5);
    }

    #[test]
    fn test_faulty_destination_injection() {
        let mut dest = FaultyDestination::fail_after(4, ErrorCode::IoError);

        assert!(dest.write_byte(b't').is_ok());
        assert!(dest.write_byte(b'e').is_ok());
        assert!(dest.write_byte(b's').is_ok());
        assert!(dest.write_byte(b't').is_ok());

        // 5th byte fails
        assert_eq!(dest.write_byte(b'!'), Err(ErrorCode::IoError));
        assert!(dest.has_faulted());
        assert_eq!(dest.written_bytes(), b"test");
    }
}
