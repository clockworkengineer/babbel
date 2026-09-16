//! Official Apache Avro Specification & drnice/AvroTest Conformance Suite
//!
//! Verifies Babbel's Apache Avro implementation against:
//! - The official `drnice/AvroTest` repository (https://github.com/drnice/AvroTest.git)
//! - Apache Avro 1.x Specification binary encoding rules
//! - Object Container File (OCF) framing, headers, metadata, blocks, and sync markers
//! - Schema-driven decoding and serialization
//! - Zigzag variable-length integer encoding (RFC/spec compliance)
//! - IEEE 754 float/double Little-Endian encoding
//! - Error recovery, corrupted stream rejection, and FormatEngine contract

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use babbel_avro::{
    from_bytes_ocf, from_bytes_with_schema,
    to_vec_ocf, to_vec_with_schema,
    AvroDecoder, AvroEncoder, AvroEngine, AvroError, AvroSchema,
};
use babbel_core::{FormatEngine, Value};

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

fn locate_suite_dir() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("crates/avro/tests/avro-test-suite"),
        PathBuf::from("tests/avro-test-suite"),
        PathBuf::from("avro-test-suite"),
        PathBuf::from("../../crates/avro/tests/avro-test-suite"),
    ];
    for c in &candidates {
        if c.exists() && c.join("users2.avro").exists() {
            return Some(c.clone());
        }
    }
    None
}

#[test]
fn test_avro_conformance_full_suite() {
    let start_time = Instant::now();
    let mut categories: BTreeMap<&'static str, CategoryStats> = BTreeMap::new();

    println!("\n===============================================================================");
    println!(" Babbel Apache Avro Specification & Conformance Test Suite");
    println!(" Corpus: https://github.com/drnice/AvroTest.git + Avro 1.x Spec");
    println!("===============================================================================\n");

    let suite_dir = locate_suite_dir();
    if let Some(ref dir) = suite_dir {
        println!("  Found official Avro test suite at: {}\n", dir.display());
    } else {
        println!("  NOTICE: Official Avro test suite not found. Run scripts/fetch_avro_test_suite.ps1\n");
    }

    // -------------------------------------------------------------------------
    // 1. Official drnice/AvroTest OCF Suite (users2.avro)
    // -------------------------------------------------------------------------
    {
        let cat = categories.entry("1. Official drnice/AvroTest OCF Suite").or_default();

        if let Some(ref dir) = suite_dir {
            let users2_path = dir.join("users2.avro");
            cat.total += 1;
            let file_bytes = match fs::read(&users2_path) {
                Ok(b) => {
                    cat.passed += 1;
                    b
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Failed to read users2.avro: {}", e);
                    Vec::new()
                }
            };

            if !file_bytes.is_empty() {
                // Test 1: Verify OCF magic header
                cat.total += 1;
                if file_bytes.len() >= 4 && &file_bytes[0..4] == b"Obj\x01" {
                    cat.passed += 1;
                } else {
                    cat.failed += 1;
                    eprintln!("Invalid magic bytes in users2.avro");
                }

                // Test 2: Parse OCF container via from_bytes_ocf
                cat.total += 1;
                let decoded = match from_bytes_ocf(&file_bytes) {
                    Ok(val) => {
                        cat.passed += 1;
                        val
                    }
                    Err(e) => {
                        cat.failed += 1;
                        eprintln!("from_bytes_ocf failed on users2.avro: {}", e);
                        Value::Null
                    }
                };

                // Test 3: Validate exactly 3 records present
                cat.total += 1;
                let records = decoded.as_array();
                if let Some(arr) = records {
                    if arr.len() == 3 {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Expected 3 records in users2.avro, found {}", arr.len());
                    }

                    // Test 4: Validate Record 1 (Alyssa)
                    cat.total += 1;
                    let u1 = &arr[0];
                    let u1_name = u1.get("name").and_then(|v| v.as_str());
                    let u1_num = u1.get("favorite_number").and_then(|v| v.as_i64());
                    let u1_color = u1.get("favorite_color");
                    if u1_name == Some("Alyssa") && u1_num == Some(256) && matches!(u1_color, Some(Value::Null)) {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Record 1 mismatch: {:?}", u1);
                    }

                    // Test 5: Validate Record 2 (Ben)
                    cat.total += 1;
                    let u2 = &arr[1];
                    let u2_name = u2.get("name").and_then(|v| v.as_str());
                    let u2_num = u2.get("favorite_number").and_then(|v| v.as_i64());
                    let u2_color = u2.get("favorite_color").and_then(|v| v.as_str());
                    if u2_name == Some("Ben") && u2_num == Some(7) && u2_color == Some("red") {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Record 2 mismatch: {:?}", u2);
                    }

                    // Test 6: Validate Record 3 (Charlie)
                    cat.total += 1;
                    let u3 = &arr[2];
                    let u3_name = u3.get("name").and_then(|v| v.as_str());
                    let u3_num = u3.get("favorite_number");
                    let u3_color = u3.get("favorite_color").and_then(|v| v.as_str());
                    if u3_name == Some("Charlie") && matches!(u3_num, Some(Value::Null)) && u3_color == Some("blue") {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Record 3 mismatch: {:?}", u3);
                    }

                    // Test 7: OCF round-trip re-encoding and decoding
                    cat.total += 1;
                    let schema_json = r#"{"type":"record","name":"User","namespace":"","fields":[{"name":"name","type":"string"},{"name":"favorite_number","type":["int","null"]},{"name":"favorite_color","type":["string","null"]}]}"#;
                    match to_vec_ocf(&decoded, schema_json) {
                        Ok(re_encoded) => {
                            match from_bytes_ocf(&re_encoded) {
                                Ok(re_decoded) => {
                                    if re_decoded == decoded {
                                        cat.passed += 1;
                                    } else {
                                        cat.failed += 1;
                                        eprintln!("Re-decoded value differs from original: {:?} vs {:?}", re_decoded, decoded);
                                    }
                                }
                                Err(e) => {
                                    cat.failed += 1;
                                    eprintln!("Failed to decode re-encoded OCF: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            cat.failed += 1;
                            eprintln!("to_vec_ocf failed on users2: {}", e);
                        }
                    }

                    // Test 8: AvroEngine parse_bytes on OCF payload
                    cat.total += 1;
                    let engine = AvroEngine;
                    match engine.parse_bytes(&file_bytes) {
                        Ok(engine_val) => {
                            if engine_val == decoded {
                                cat.passed += 1;
                            } else {
                                cat.failed += 1;
                                eprintln!("Engine parsed value differs: {:?}", engine_val);
                            }
                        }
                        Err(e) => {
                            cat.failed += 1;
                            eprintln!("AvroEngine parse_bytes failed on users2.avro: {}", e);
                        }
                    }
                } else {
                    cat.failed += 1;
                    eprintln!("decoded value is not an array");
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 2. Avro Schema Parsing & Types
    // -------------------------------------------------------------------------
    {
        let cat = categories.entry("2. Avro Schema Specification Conformance").or_default();

        let test_cases: Vec<(&str, Box<dyn Fn(&AvroSchema) -> bool>)> = vec![
            (r#""null""#, Box::new(|s| matches!(s, AvroSchema::Null))),
            (r#""boolean""#, Box::new(|s| matches!(s, AvroSchema::Boolean))),
            (r#""int""#, Box::new(|s| matches!(s, AvroSchema::Int))),
            (r#""long""#, Box::new(|s| matches!(s, AvroSchema::Long))),
            (r#""float""#, Box::new(|s| matches!(s, AvroSchema::Float))),
            (r#""double""#, Box::new(|s| matches!(s, AvroSchema::Double))),
            (r#""bytes""#, Box::new(|s| matches!(s, AvroSchema::Bytes))),
            (r#""string""#, Box::new(|s| matches!(s, AvroSchema::String))),
            (
                r#"["int", "null"]"#,
                Box::new(|s| matches!(s, AvroSchema::Union(v) if v.len() == 2 && v[0] == AvroSchema::Int && v[1] == AvroSchema::Null)),
            ),
            (
                r#"{"type": "enum", "name": "Suit", "symbols": ["SPADES", "HEARTS", "DIAMONDS", "CLUBS"]}"#,
                Box::new(|s| matches!(s, AvroSchema::Enum { name, symbols } if name == "Suit" && symbols.len() == 4)),
            ),
            (
                r#"{"type": "array", "items": "string"}"#,
                Box::new(|s| matches!(s, AvroSchema::Array { items } if **items == AvroSchema::String)),
            ),
            (
                r#"{"type": "map", "values": "long"}"#,
                Box::new(|s| matches!(s, AvroSchema::Map { values } if **values == AvroSchema::Long)),
            ),
            (
                r#"{"type": "fixed", "name": "md5", "size": 16}"#,
                Box::new(|s| matches!(s, AvroSchema::Fixed { name, size } if name == "md5" && *size == 16)),
            ),
            (
                r#"{"type": "record", "name": "Point", "fields": [{"name": "x", "type": "int"}, {"name": "y", "type": "int"}]}"#,
                Box::new(|s| matches!(s, AvroSchema::Record { name, fields, .. } if name == "Point" && fields.len() == 2)),
            ),
            (
                r#"{"type": "record", "name": "LinkedList", "fields": [{"name": "value", "type": "int"}, {"name": "next", "type": ["null", "LinkedList"]}]}"#,
                Box::new(|s| matches!(s, AvroSchema::Record { name, fields, .. } if name == "LinkedList" && fields.len() == 2)),
            ),
        ];

        for (schema_json, validator) in test_cases {
            cat.total += 1;
            match AvroSchema::parse_str(schema_json) {
                Ok(parsed) => {
                    if validator(&parsed) {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Schema validator failed for: {}", schema_json);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Schema parsing failed for {}: {}", schema_json, e);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 3. Avro Binary Primitive Encoding (Zigzag & IEEE 754)
    // -------------------------------------------------------------------------
    {
        let cat = categories.entry("3. Avro Binary Primitive Encoding").or_default();

        let zigzag_cases: Vec<i64> = vec![
            0, -1, 1, -2, 2, 63, -64, 64, 127, -128, 128, 256, -256,
            1000, -1000, 65535, -65536, i32::MAX as i64, i32::MIN as i64,
            i64::MAX, i64::MIN,
        ];

        for num in zigzag_cases {
            cat.total += 1;
            let mut enc = AvroEncoder::new();
            enc.write_long(num);
            let bytes = enc.into_vec();
            let mut dec = AvroDecoder::new(&bytes);
            match dec.read_long() {
                Ok(read) if read == num => cat.passed += 1,
                Ok(read) => {
                    cat.failed += 1;
                    eprintln!("Zigzag mismatch: wrote {}, got {}", num, read);
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Zigzag decode failed for {}: {}", num, e);
                }
            }
        }

        // Float cases (IEEE 754 little-endian)
        let float_cases: Vec<f32> = vec![0.0, -0.0, 1.0, -1.0, 3.14159, -42.5, f32::MAX, f32::MIN_POSITIVE];
        for f in float_cases {
            cat.total += 1;
            let mut enc = AvroEncoder::new();
            enc.write_float(f);
            let bytes = enc.into_vec();
            let mut dec = AvroDecoder::new(&bytes);
            match dec.read_float() {
                Ok(read) if read.to_bits() == f.to_bits() => cat.passed += 1,
                Ok(read) => {
                    cat.failed += 1;
                    eprintln!("Float mismatch: wrote {}, got {}", f, read);
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Float decode failed for {}: {}", f, e);
                }
            }
        }

        // Double cases (IEEE 754 little-endian)
        let double_cases: Vec<f64> = vec![0.0, -0.0, 1.0, -1.0, 2.718281828459, 1e100, f64::MAX, f64::MIN_POSITIVE];
        for d in double_cases {
            cat.total += 1;
            let mut enc = AvroEncoder::new();
            enc.write_double(d);
            let bytes = enc.into_vec();
            let mut dec = AvroDecoder::new(&bytes);
            match dec.read_double() {
                Ok(read) if read.to_bits() == d.to_bits() => cat.passed += 1,
                Ok(read) => {
                    cat.failed += 1;
                    eprintln!("Double mismatch: wrote {}, got {}", d, read);
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Double decode failed for {}: {}", d, e);
                }
            }
        }

        // String cases
        let string_cases = vec!["", "hello", "Babbel Avro Engine", "🌟 UTF-8 Emoji 🚀", "Русский текст", "日本語テスト"];
        for s in string_cases {
            cat.total += 1;
            let mut enc = AvroEncoder::new();
            enc.write_string(s);
            let bytes = enc.into_vec();
            let mut dec = AvroDecoder::new(&bytes);
            match dec.read_string() {
                Ok(read) if read == s => cat.passed += 1,
                Ok(read) => {
                    cat.failed += 1;
                    eprintln!("String mismatch: wrote {:?}, got {:?}", s, read);
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("String decode failed for {:?}: {}", s, e);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. Avro Complex Types & Schema-Driven Codec
    // -------------------------------------------------------------------------
    {
        let cat = categories.entry("4. Avro Complex Types & Schema-Driven Codec").or_default();

        // Test 1: Record schema round-trip
        cat.total += 1;
        let schema_json = r#"{
            "type": "record",
            "name": "Employee",
            "fields": [
                {"name": "id", "type": "long"},
                {"name": "name", "type": "string"},
                {"name": "active", "type": "boolean"},
                {"name": "score", "type": "double"}
            ]
        }"#;
        let schema = AvroSchema::parse_str(schema_json).unwrap();
        let record = Value::Object(vec![
            ("id".into(), Value::Integer(101)),
            ("name".into(), Value::String("Alice Walker".into())),
            ("active".into(), Value::Bool(true)),
            ("score".into(), Value::Float(98.5)),
        ]);
        match to_vec_with_schema(&record, &schema) {
            Ok(bytes) => match from_bytes_with_schema(&bytes, &schema) {
                Ok(decoded) => {
                    if decoded == record {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Record decoded mismatch: {:?} vs {:?}", decoded, record);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Record decode failed: {}", e);
                }
            },
            Err(e) => {
                cat.failed += 1;
                eprintln!("Record encode failed: {}", e);
            }
        }

        // Test 2: Array of records round-trip
        cat.total += 1;
        let array_schema_json = r#"{"type": "array", "items": "int"}"#;
        let array_schema = AvroSchema::parse_str(array_schema_json).unwrap();
        let array_val = Value::Array(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30),
            Value::Integer(-40),
        ]);
        match to_vec_with_schema(&array_val, &array_schema) {
            Ok(bytes) => match from_bytes_with_schema(&bytes, &array_schema) {
                Ok(decoded) => {
                    if decoded == array_val {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Array decoded mismatch: {:?} vs {:?}", decoded, array_val);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Array decode failed: {}", e);
                }
            },
            Err(e) => {
                cat.failed += 1;
                eprintln!("Array encode failed: {}", e);
            }
        }

        // Test 3: Map schema round-trip
        cat.total += 1;
        let map_schema_json = r#"{"type": "map", "values": "string"}"#;
        let map_schema = AvroSchema::parse_str(map_schema_json).unwrap();
        let map_val = Value::Object(vec![
            ("key1".into(), Value::String("val1".into())),
            ("key2".into(), Value::String("val2".into())),
        ]);
        match to_vec_with_schema(&map_val, &map_schema) {
            Ok(bytes) => match from_bytes_with_schema(&bytes, &map_schema) {
                Ok(decoded) => {
                    if decoded == map_val {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Map decoded mismatch: {:?} vs {:?}", decoded, map_val);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Map decode failed: {}", e);
                }
            },
            Err(e) => {
                cat.failed += 1;
                eprintln!("Map encode failed: {}", e);
            }
        }

        // Test 4: Enum schema round-trip
        cat.total += 1;
        let enum_schema_json = r#"{"type": "enum", "name": "Status", "symbols": ["PENDING", "ACTIVE", "CLOSED"]}"#;
        let enum_schema = AvroSchema::parse_str(enum_schema_json).unwrap();
        let enum_val = Value::String("ACTIVE".into());
        match to_vec_with_schema(&enum_val, &enum_schema) {
            Ok(bytes) => match from_bytes_with_schema(&bytes, &enum_schema) {
                Ok(decoded) => {
                    if decoded == enum_val {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Enum decoded mismatch: {:?} vs {:?}", decoded, enum_val);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Enum decode failed: {}", e);
                }
            },
            Err(e) => {
                cat.failed += 1;
                eprintln!("Enum encode failed: {}", e);
            }
        }

        // Test 5: Fixed bytes schema round-trip
        cat.total += 1;
        let fixed_schema_json = r#"{"type": "fixed", "name": "Hash", "size": 4}"#;
        let fixed_schema = AvroSchema::parse_str(fixed_schema_json).unwrap();
        let fixed_val = Value::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        match to_vec_with_schema(&fixed_val, &fixed_schema) {
            Ok(bytes) => match from_bytes_with_schema(&bytes, &fixed_schema) {
                Ok(decoded) => {
                    if decoded == fixed_val {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Fixed decoded mismatch: {:?} vs {:?}", decoded, fixed_val);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Fixed decode failed: {}", e);
                }
            },
            Err(e) => {
                cat.failed += 1;
                eprintln!("Fixed encode failed: {}", e);
            }
        }
    }

    // -------------------------------------------------------------------------
    // 5. Avro OCF Framing, Sync Markers & Robustness
    // -------------------------------------------------------------------------
    {
        let cat = categories.entry("5. Avro OCF Framing, Sync Markers & Robustness").or_default();

        // Test 1: Invalid magic header detection
        cat.total += 1;
        let mut corrupted_magic = vec![b'B', b'a', b'd', 0x01];
        corrupted_magic.extend_from_slice(&[0u8; 30]);
        match from_bytes_ocf(&corrupted_magic) {
            Err(AvroError::InvalidMagic) => cat.passed += 1,
            other => {
                cat.failed += 1;
                eprintln!("Expected InvalidMagic error, got: {:?}", other);
            }
        }

        // Test 2: Truncated OCF file detection (< 24 bytes)
        cat.total += 1;
        let short_bytes = [b'O', b'b', b'j', 1, 0, 0];
        match from_bytes_ocf(&short_bytes) {
            Err(AvroError::UnexpectedEof { .. }) => cat.passed += 1,
            other => {
                cat.failed += 1;
                eprintln!("Expected UnexpectedEof for short file, got: {:?}", other);
            }
        }

        // Test 3: Sync marker mismatch detection
        cat.total += 1;
        let schema_json = r#"{"type": "record", "name": "Item", "fields": [{"name": "id", "type": "int"}]}"#;
        let rec = Value::Object(vec![("id".into(), Value::Integer(42))]);
        let valid_ocf = to_vec_ocf(&rec, schema_json).unwrap();
        let mut corrupted_sync = valid_ocf.clone();
        // Corrupt trailing sync marker (last 16 bytes)
        let last_idx = corrupted_sync.len() - 1;
        corrupted_sync[last_idx] ^= 0xFF;
        match from_bytes_ocf(&corrupted_sync) {
            Err(AvroError::SyncMarkerMismatch) => cat.passed += 1,
            other => {
                cat.failed += 1;
                eprintln!("Expected SyncMarkerMismatch, got: {:?}", other);
            }
        }
    }

    // -------------------------------------------------------------------------
    // 6. FormatEngine Trait Conformance
    // -------------------------------------------------------------------------
    {
        let cat = categories.entry("6. FormatEngine Trait Conformance").or_default();

        let engine = AvroEngine;

        // Test 1: format_id
        cat.total += 1;
        if engine.format_id() == "avro" {
            cat.passed += 1;
        } else {
            cat.failed += 1;
        }

        // Test 2: mime_type
        cat.total += 1;
        if engine.mime_type() == "application/avro" {
            cat.passed += 1;
        } else {
            cat.failed += 1;
        }

        // Test 3: file_extensions
        cat.total += 1;
        if engine.file_extensions() == &["avro"] {
            cat.passed += 1;
        } else {
            cat.failed += 1;
        }

        // Test 4: is_binary
        cat.total += 1;
        if engine.is_binary() {
            cat.passed += 1;
        } else {
            cat.failed += 1;
        }

        // Test 5: engine binary round-trip
        cat.total += 1;
        let test_val = Value::Object(vec![
            ("count".into(), Value::Integer(100)),
            ("title".into(), Value::String("Babbel FormatEngine".into())),
        ]);
        let options = babbel_core::FormatOptions::default();
        match engine.serialize_to_vec(&test_val, &options) {
            Ok(dest) => match engine.parse_bytes(&dest) {
                Ok(roundtrip) => {
                    if roundtrip == test_val {
                        cat.passed += 1;
                    } else {
                        cat.failed += 1;
                        eprintln!("Engine roundtrip mismatch: {:?} vs {:?}", roundtrip, test_val);
                    }
                }
                Err(e) => {
                    cat.failed += 1;
                    eprintln!("Engine parse_bytes failed: {}", e);
                }
            },
            Err(e) => {
                cat.failed += 1;
                eprintln!("Engine serialize failed: {}", e);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Print Conformance Report Table
    // -------------------------------------------------------------------------
    let elapsed = start_time.elapsed();
    let mut grand_total = 0;
    let mut grand_passed = 0;
    let mut grand_failed = 0;
    let mut grand_panics = 0;

    println!("{:<55} {:>7} {:>7} {:>7} {:>7} {:>9}", "Category", "Total", "Passed", "Failed", "Panics", "Pass Rate");
    println!("{}", "-".repeat(95));

    for (cat_name, stats) in &categories {
        grand_total += stats.total;
        grand_passed += stats.passed;
        grand_failed += stats.failed;
        grand_panics += stats.panics;

        println!(
            "{:<55} {:>7} {:>7} {:>7} {:>7} {:>8.1}%",
            cat_name,
            stats.total,
            stats.passed,
            stats.failed,
            stats.panics,
            stats.pass_rate()
        );
    }

    println!("{}", "-".repeat(95));
    let grand_pass_rate = if grand_total == 0 {
        0.0
    } else {
        (grand_passed as f64 / grand_total as f64) * 100.0
    };
    println!(
        "{:<55} {:>7} {:>7} {:>7} {:>7} {:>8.1}%",
        "GRAND TOTAL", grand_total, grand_passed, grand_failed, grand_panics, grand_pass_rate
    );
    println!("\nTotal Execution Time: {:.2?}", elapsed);
    println!("===============================================================================\n");

    assert_eq!(grand_failed, 0, "All Avro conformance test vectors must pass with 0 failures");
    assert_eq!(grand_panics, 0, "All Avro conformance test vectors must pass with 0 panics");
    assert_eq!(grand_pass_rate, 100.0, "Pass rate must be exactly 100.0%");
}
