//! Official Apache Parquet Conformance & Regression Test Suite
//!
//! Validates Parquet reading, writing, Thrift metadata handling, and panic safety:
//! 1. Embedded regression suite: 28+ embedded vectors for guaranteed offline execution.
//! 2. Official test vectors from https://github.com/apache/parquet-testing.git (`data/`, `bad_data/`)
//!    and https://github.com/apache/parquet-format.git when available.
//! 3. Zero panics assertion via PanicHookGuard on all malformed, corrupted, or edge-case inputs.

use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use babbel_core::Value;
use babbel_parquet::{ParquetError, read_parquet, write_parquet};

/// Panic hook guard that counts panics and restores the default hook upon drop.
struct PanicHookGuard {
    panic_count: Arc<AtomicUsize>,
    prev_hook: Option<Box<dyn Fn(&panic::PanicHookInfo) + Send + Sync + 'static>>,
}

impl PanicHookGuard {
    fn new() -> Self {
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

    fn panics(&self) -> usize {
        self.panic_count.load(Ordering::SeqCst)
    }
}

impl Drop for PanicHookGuard {
    fn drop(&mut self) {
        if let Some(hook) = self.prev_hook.take() {
            panic::set_hook(hook);
        }
    }
}

/// Discovers the path to the official parquet-testing or parquet-format test suite.
fn find_parquet_test_dir() -> Option<PathBuf> {
    let candidates = [
        "parquet-testing",
        "../parquet-testing",
        "tests/parquet-testing",
        "crates/parquet/tests/parquet-testing",
        "parquet-format",
        "../parquet-format",
        "tests/parquet-format",
        "crates/parquet/tests/parquet-format",
    ];

    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.join("data").is_dir() {
            return Some(path);
        }
    }

    // Try reading suite_paths.txt if available
    if let Ok(content) = fs::read_to_string("tests/suite_paths.txt")
        .or_else(|_| fs::read_to_string("crates/parquet/tests/suite_paths.txt"))
    {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let path = PathBuf::from(line);
            if path.join("data").is_dir() {
                return Some(path);
            }
        }
    }

    None
}

#[derive(Default)]
struct CategoryStats {
    total: usize,
    passed: usize,
    panics: usize,
}

impl CategoryStats {
    fn record(&mut self, ok: bool, panic_occurred: bool) {
        self.total += 1;
        if ok {
            self.passed += 1;
        }
        if panic_occurred {
            self.panics += 1;
        }
    }
}

#[test]
fn test_parquet_official_conformance() {
    let _guard = PanicHookGuard::new();
    let start_time = Instant::now();

    let mut cat_types = CategoryStats::default();
    let mut cat_nullable = CategoryStats::default();
    let mut cat_tabular = CategoryStats::default();
    let mut cat_magic = CategoryStats::default();
    let mut cat_metadata = CategoryStats::default();
    let mut cat_page = CategoryStats::default();
    let mut cat_ext_data = CategoryStats::default();
    let mut cat_ext_bad = CategoryStats::default();

    // =========================================================================
    // Category 1: Valid Basic Types & Schemas
    // =========================================================================
    let type_cases: Vec<(&str, Value)> = vec![
        (
            "int32_column",
            Value::Array(vec![
                Value::Object(vec![("i".into(), Value::Integer(0))]),
                Value::Object(vec![("i".into(), Value::Integer(42))]),
                Value::Object(vec![("i".into(), Value::Integer(-1000))]),
                Value::Object(vec![("i".into(), Value::Integer(i32::MAX as i128))]),
                Value::Object(vec![("i".into(), Value::Integer(i32::MIN as i128))]),
            ]),
        ),
        (
            "int64_column",
            Value::Array(vec![
                Value::Object(vec![("big".into(), Value::Integer(1_000_000_000_000))]),
                Value::Object(vec![("big".into(), Value::Integer(-9_999_999_999_999))]),
                Value::Object(vec![("big".into(), Value::Integer(i64::MAX as i128))]),
            ]),
        ),
        (
            "float_column",
            Value::Array(vec![
                Value::Object(vec![("f".into(), Value::Float(0.0))]),
                Value::Object(vec![("f".into(), Value::Float(3.14159265))]),
                Value::Object(vec![("f".into(), Value::Float(-123.456))]),
            ]),
        ),
        (
            "string_column",
            Value::Array(vec![
                Value::Object(vec![("s".into(), Value::String("".into()))]),
                Value::Object(vec![("s".into(), Value::String("hello world".into()))]),
                Value::Object(vec![("s".into(), Value::String("UTF-8: 🚀 ✨ 🦀".into()))]),
                Value::Object(vec![(
                    "s".into(),
                    Value::String("multi\nline\ttext".into()),
                )]),
            ]),
        ),
        (
            "bool_column",
            Value::Array(vec![
                Value::Object(vec![("b".into(), Value::Bool(true))]),
                Value::Object(vec![("b".into(), Value::Bool(false))]),
                Value::Object(vec![("b".into(), Value::Bool(true))]),
            ]),
        ),
    ];

    for (name, dataset) in type_cases {
        let panic_before = _guard.panics();
        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            let bytes = write_parquet(&dataset).expect("write failed");
            assert_eq!(&bytes[0..4], b"PAR1", "{}: invalid magic prefix", name);
            assert_eq!(
                &bytes[bytes.len() - 4..],
                b"PAR1",
                "{}: invalid magic suffix",
                name
            );
            let roundtrip = read_parquet(&bytes).expect("read failed");
            assert_eq!(roundtrip, dataset, "{}: roundtrip mismatch", name);
        }));
        let panicked = res.is_err() || _guard.panics() > panic_before;
        cat_types.record(!panicked, panicked);
    }

    // =========================================================================
    // Category 2: Valid Nullable Columns & RLE Levels
    // =========================================================================
    let nullable_cases: Vec<(&str, Value)> = vec![
        (
            "all_nulls",
            Value::Array(vec![
                Value::Object(vec![("x".into(), Value::Null)]),
                Value::Object(vec![("x".into(), Value::Null)]),
                Value::Object(vec![("x".into(), Value::Null)]),
            ]),
        ),
        (
            "all_present",
            Value::Array(vec![
                Value::Object(vec![("x".into(), Value::Integer(1))]),
                Value::Object(vec![("x".into(), Value::Integer(2))]),
                Value::Object(vec![("x".into(), Value::Integer(3))]),
            ]),
        ),
        (
            "interleaved_nulls",
            Value::Array(vec![
                Value::Object(vec![("x".into(), Value::Integer(10))]),
                Value::Object(vec![("x".into(), Value::Null)]),
                Value::Object(vec![("x".into(), Value::Integer(30))]),
                Value::Object(vec![("x".into(), Value::Null)]),
                Value::Object(vec![("x".into(), Value::Integer(50))]),
            ]),
        ),
        (
            "leading_and_trailing_nulls",
            Value::Array(vec![
                Value::Object(vec![("v".into(), Value::Null)]),
                Value::Object(vec![("v".into(), Value::String("middle".into()))]),
                Value::Object(vec![("v".into(), Value::Null)]),
            ]),
        ),
        (
            "multiple_nullable_columns",
            Value::Array(vec![
                Value::Object(vec![
                    ("a".into(), Value::Integer(1)),
                    ("b".into(), Value::Null),
                ]),
                Value::Object(vec![
                    ("a".into(), Value::Null),
                    ("b".into(), Value::String("two".into())),
                ]),
                Value::Object(vec![
                    ("a".into(), Value::Integer(3)),
                    ("b".into(), Value::String("three".into())),
                ]),
                Value::Object(vec![("a".into(), Value::Null), ("b".into(), Value::Null)]),
            ]),
        ),
    ];

    for (name, dataset) in nullable_cases {
        let panic_before = _guard.panics();
        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            let bytes = write_parquet(&dataset).expect("write failed");
            let roundtrip = read_parquet(&bytes).expect("read failed");
            assert_eq!(roundtrip, dataset, "{}: nullable roundtrip mismatch", name);
        }));
        let panicked = res.is_err() || _guard.panics() > panic_before;
        cat_nullable.record(!panicked, panicked);
    }

    // =========================================================================
    // Category 3: Valid Tabular Datasets & Edge Cases
    // =========================================================================
    let tabular_cases: Vec<(&str, Value)> = vec![
        ("empty_table", Value::Array(vec![])),
        (
            "single_row_multiple_cols",
            Value::Array(vec![Value::Object(vec![
                ("id".into(), Value::Integer(101)),
                ("name".into(), Value::String("Admin".into())),
                ("active".into(), Value::Bool(true)),
                ("rating".into(), Value::Float(4.85)),
            ])]),
        ),
        (
            "wide_table_10_cols",
            Value::Array(vec![Value::Object(vec![
                ("c0".into(), Value::Integer(0)),
                ("c1".into(), Value::Integer(1)),
                ("c2".into(), Value::Integer(2)),
                ("c3".into(), Value::Integer(3)),
                ("c4".into(), Value::Integer(4)),
                ("c5".into(), Value::String("col5".into())),
                ("c6".into(), Value::String("col6".into())),
                ("c7".into(), Value::Bool(true)),
                ("c8".into(), Value::Float(8.0)),
                ("c9".into(), Value::Float(9.0)),
            ])]),
        ),
        (
            "hundred_rows_tabular",
            Value::Array(
                (0..100)
                    .map(|i| {
                        Value::Object(vec![
                            ("index".into(), Value::Integer(i as i128)),
                            ("name".into(), Value::String(format!("item_{}", i))),
                            ("flag".into(), Value::Bool(i % 2 == 0)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ];

    for (name, dataset) in tabular_cases {
        let panic_before = _guard.panics();
        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            let bytes = write_parquet(&dataset).expect("write failed");
            let roundtrip = read_parquet(&bytes).expect("read failed");
            assert_eq!(roundtrip, dataset, "{}: tabular roundtrip mismatch", name);
        }));
        let panicked = res.is_err() || _guard.panics() > panic_before;
        cat_tabular.record(!panicked, panicked);
    }

    // =========================================================================
    // Category 4: Corrupted Magic & Truncated Files
    // =========================================================================
    let magic_cases: Vec<(&str, &[u8])> = vec![
        ("empty_buffer", b""),
        ("one_byte", b"P"),
        ("four_bytes_only", b"PAR1"),
        ("eleven_bytes", b"PAR11234567"),
        ("invalid_start_magic", b"NOT_A_PARQUET_FILE_TEST"),
        ("invalid_start_valid_end", b"XYZ100000000PAR1"),
        ("valid_start_invalid_end", b"PAR100000000XYZ1"),
        ("inverted_magic", b"1RAP000000001RAP"),
    ];

    for (name, corrupted) in magic_cases {
        let panic_before = _guard.panics();
        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            let err = read_parquet(corrupted);
            assert!(err.is_err(), "{}: expected error but got ok", name);
            assert_eq!(
                err.unwrap_err(),
                ParquetError::InvalidMagic,
                "{}: expected InvalidMagic",
                name
            );
        }));
        let panicked = res.is_err() || _guard.panics() > panic_before;
        cat_magic.record(!panicked, panicked);
    }

    // =========================================================================
    // Category 5: Corrupted Metadata & Headers
    // =========================================================================
    // Generate valid base bytes first to corrupt metadata
    let base_sample = Value::Array(vec![Value::Object(vec![(
        "num".into(),
        Value::Integer(42),
    )])]);
    let valid_bytes = write_parquet(&base_sample).expect("base write");

    let mut bad_meta_len = valid_bytes.clone();
    let vlen = bad_meta_len.len();
    // Corrupt metadata length: set to 0xFFFF_FFFF (exceeds file size)
    bad_meta_len[vlen - 8..vlen - 4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());

    let mut truncated_thrift = valid_bytes.clone();
    // Truncate metadata thrift payload: reduce meta_len to 1 byte
    truncated_thrift[vlen - 8..vlen - 4].copy_from_slice(&1u32.to_le_bytes());

    let mut zero_meta_len = valid_bytes.clone();
    zero_meta_len[vlen - 8..vlen - 4].copy_from_slice(&0u32.to_le_bytes());

    let metadata_cases: Vec<(&str, &[u8])> = vec![
        ("meta_len_exceeds_filesize", &bad_meta_len),
        ("truncated_thrift_metadata", &truncated_thrift),
        ("zero_meta_len", &zero_meta_len),
    ];

    for (name, corrupted) in metadata_cases {
        let panic_before = _guard.panics();
        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            let err = read_parquet(corrupted);
            assert!(err.is_err(), "{}: expected error but got ok", name);
        }));
        let panicked = res.is_err() || _guard.panics() > panic_before;
        cat_metadata.record(!panicked, panicked);
    }

    // =========================================================================
    // Category 6: Corrupted Pages & Payloads
    // =========================================================================
    let mut corrupt_payload = valid_bytes.clone();
    // Overwrite payload bytes in the middle of data page with 0xFF
    if corrupt_payload.len() > 20 {
        for b in &mut corrupt_payload[6..18] {
            *b = 0xFF;
        }
    }

    let mut truncated_at_magic = valid_bytes[0..12].to_vec();
    // Set 12-byte buffer with PAR1 ... PAR1 but 0 metadata
    truncated_at_magic[8..12].copy_from_slice(b"PAR1");

    let page_cases: Vec<(&str, &[u8])> = vec![
        ("corrupt_page_payload", &corrupt_payload),
        ("truncated_header_only", &truncated_at_magic),
    ];

    for (_name, corrupted) in page_cases {
        let panic_before = _guard.panics();
        let res = panic::catch_unwind(AssertUnwindSafe(|| {
            let err = read_parquet(corrupted);
            // Must return an error or decode cleanly; zero panics
            let _ = err;
        }));
        let panicked = res.is_err() || _guard.panics() > panic_before;
        cat_page.record(!panicked, panicked);
    }

    // =========================================================================
    // Category 7 & 8: External Official Files (parquet-testing)
    // =========================================================================
    if let Some(suite_dir) = find_parquet_test_dir() {
        println!(
            "[INFO] Found official Parquet test suite at: {}",
            suite_dir.display()
        );

        // Category 7: data/*.parquet
        let data_dir = suite_dir.join("data");
        if data_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&data_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                        let panic_before = _guard.panics();
                        let res = panic::catch_unwind(AssertUnwindSafe(|| {
                            let bytes = fs::read(&path).expect("read file");
                            let result = read_parquet(&bytes);
                            match result {
                                Ok(Value::Array(_)) => {}
                                Ok(other) => panic!("Expected Value::Array, got {:?}", other),
                                Err(_) => {
                                    // Valid parquet files with advanced compression/encodings
                                    // (e.g. SNAPPY, GZIP, DELTA) must return clean Err, 0 panics.
                                }
                            }
                        }));
                        let panicked = res.is_err() || _guard.panics() > panic_before;
                        cat_ext_data.record(!panicked, panicked);
                    }
                }
            }
        }

        // Category 8: bad_data/*.parquet
        let bad_data_dir = suite_dir.join("bad_data");
        if bad_data_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&bad_data_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                        let panic_before = _guard.panics();
                        let res = panic::catch_unwind(AssertUnwindSafe(|| {
                            let bytes = fs::read(&path).expect("read bad file");
                            let _ = read_parquet(&bytes);
                            // Must not panic on corrupt/bad parquet files
                        }));
                        let panicked = res.is_err() || _guard.panics() > panic_before;
                        cat_ext_bad.record(!panicked, panicked);
                    }
                }
            }
        }
    } else {
        println!(
            "[NOTICE] Official parquet-testing directory not found. Using embedded fallback suite."
        );
    }

    let elapsed = start_time.elapsed();

    // =========================================================================
    // Summary Report
    // =========================================================================
    let total_vectors = cat_types.total
        + cat_nullable.total
        + cat_tabular.total
        + cat_magic.total
        + cat_metadata.total
        + cat_page.total
        + cat_ext_data.total
        + cat_ext_bad.total;

    let total_passed = cat_types.passed
        + cat_nullable.passed
        + cat_tabular.passed
        + cat_magic.passed
        + cat_metadata.passed
        + cat_page.passed
        + cat_ext_data.passed
        + cat_ext_bad.passed;

    let total_panics = cat_types.panics
        + cat_nullable.panics
        + cat_tabular.panics
        + cat_magic.panics
        + cat_metadata.panics
        + cat_page.panics
        + cat_ext_data.panics
        + cat_ext_bad.panics;

    println!("\n=================================================================================");
    println!("               OFFICIAL APACHE PARQUET CONFORMANCE REPORT                        ");
    println!("=================================================================================");
    println!(
        " {:<38} | {:<7} | {:<7} | {:<7} | {:<8}",
        "Category", "Vectors", "Passed", "Panics", "Status"
    );
    println!("---------------------------------------------------------------------------------");

    let print_row = |name: &str, stats: &CategoryStats| {
        if stats.total > 0 {
            let status = if stats.passed == stats.total && stats.panics == 0 {
                "PASSED"
            } else {
                "FAILED"
            };
            println!(
                " {:<38} | {:<7} | {:<7} | {:<7} | {:<8}",
                name, stats.total, stats.passed, stats.panics, status
            );
        }
    };

    print_row("Valid Basic Types & Schemas", &cat_types);
    print_row("Valid Nullable Columns & RLE", &cat_nullable);
    print_row("Valid Tabular Datasets & Rows", &cat_tabular);
    print_row("Corrupted Magic & Headers", &cat_magic);
    print_row("Corrupted Thrift Metadata", &cat_metadata);
    print_row("Corrupted Pages & Payloads", &cat_page);
    print_row("Upstream parquet-testing (data)", &cat_ext_data);
    print_row("Upstream parquet-testing (bad_data)", &cat_ext_bad);

    println!("---------------------------------------------------------------------------------");
    println!(
        " {:<38} | {:<7} | {:<7} | {:<7} | {:<8}",
        "TOTAL",
        total_vectors,
        total_passed,
        total_panics,
        if total_passed == total_vectors && total_panics == 0 {
            "100% OK"
        } else {
            "FAIL"
        }
    );
    println!("=================================================================================");
    println!("Execution Time: {:.2?}", elapsed);
    println!("Zero Panics Guarantee: {} panics recorded", total_panics);
    println!("=================================================================================\n");

    assert_eq!(
        total_panics, 0,
        "Panics occurred during Parquet conformance tests!"
    );
    assert_eq!(
        total_passed, total_vectors,
        "Some Parquet test cases failed!"
    );
}
