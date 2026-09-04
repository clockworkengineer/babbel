//! Integration tests for JSON round-trip parsing and stringification
//!
//! Verifies that JSON read from real files can be parsed into a Node tree,
//! stringified back to JSON text, and then re-parsed to produce the same
//! structure as the original.

#[cfg(test)]
mod tests {

    use crate::{BufferDestination, FileSource, Node, from_str, parse, stringify};
    use std::fs;

    /// Parse a file into a Node, stringify it, then parse the string again.
    fn round_trip_file(file_path: &str) -> Node {
        let mut source = FileSource::new(file_path)
            .unwrap_or_else(|e| panic!("Failed to open {}: {}", file_path, e));
        let node =
            parse(&mut source).unwrap_or_else(|e| panic!("Failed to parse {}: {}", file_path, e));
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest)
            .unwrap_or_else(|e| panic!("Failed to stringify {}: {}", file_path, e));
        let json_str = dest.to_string();
        from_str(&json_str).unwrap_or_else(|e| panic!("Failed to re-parse {}: {}", file_path, e))
    }

    /// Helper: collect all .json file paths in a directory (non-recursive).
    fn json_files_in(directory: &str) -> Vec<String> {
        let mut paths = Vec::new();
        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(p) = path.to_str() {
                        paths.push(p.to_string());
                    }
                }
            }
        }
        paths
    }

    // ─── All test files ───────────────────────────────────────────────────────

    #[test]
    fn test_round_trip_all_test_files() {
        let files = json_files_in("../files");
        assert!(
            !files.is_empty(),
            "No JSON files found in ../files – check working directory"
        );
        for file_path in &files {
            let reparsed = round_trip_file(file_path);
            assert!(
                reparsed.is_object() || reparsed.is_array(),
                "{} did not produce an object or array after round-trip",
                file_path
            );
        }
    }

    // ─── testfile001.json – glossary object ───────────────────────────────────

    #[test]
    fn test_round_trip_testfile001_preserves_root_type() {
        let file = "../files/testfile001.json";
        let mut source = FileSource::new(file).expect("open testfile001.json");
        let original = parse(&mut source).expect("parse testfile001.json");
        assert!(original.is_object(), "testfile001 root should be an object");

        let reparsed = round_trip_file(file);
        assert!(
            reparsed.is_object(),
            "testfile001 root should still be an object after round-trip"
        );
    }

    #[test]
    fn test_round_trip_testfile001_glossary_key_present() {
        let reparsed = round_trip_file("../files/testfile001.json");
        assert!(
            reparsed.get("glossary").is_some(),
            "round-tripped testfile001 should have 'glossary' key"
        );
        assert!(
            reparsed.get("glossary").unwrap().is_object(),
            "'glossary' value should be an object"
        );
    }

    // ─── testfile004.json – latitude/longitude ────────────────────────────────

    #[test]
    fn test_round_trip_testfile004_preserves_float_values() {
        let file = "../files/testfile004.json";
        let reparsed = round_trip_file(file);

        assert!(reparsed.is_object(), "testfile004 root should be an object");

        let lat = reparsed
            .get("latitude")
            .expect("'latitude' key missing after round-trip");
        let lon = reparsed
            .get("longitude")
            .expect("'longitude' key missing after round-trip");

        let lat_val = lat.as_f64().expect("'latitude' should be numeric");
        let lon_val = lon.as_f64().expect("'longitude' should be numeric");

        assert!(
            (lat_val - 48.8581_f64).abs() < 1e-4,
            "latitude value changed after round-trip: got {}",
            lat_val
        );
        assert!(
            (lon_val - 2.29469_f64).abs() < 1e-4,
            "longitude value changed after round-trip: got {}",
            lon_val
        );
    }

    // ─── testfile010.json – employees ────────────────────────────────────────

    #[test]
    fn test_round_trip_testfile010_employees_key_present() {
        let file = "../files/testfile010.json";
        let reparsed = round_trip_file(file);

        assert!(reparsed.is_object(), "testfile010 root should be an object");
        let employees = reparsed
            .get("Employees")
            .expect("'Employees' key missing after round-trip");
        assert!(
            employees.is_array(),
            "'Employees' should be an array after round-trip"
        );
        let arr = employees.as_array().unwrap();
        assert!(!arr.is_empty(), "'Employees' array should not be empty");
    }

    #[test]
    fn test_round_trip_testfile010_employee_count_preserved() {
        // Parse with FileSource to get original count
        let mut source =
            FileSource::new("../files/testfile010.json").expect("open testfile010.json");
        let original = parse(&mut source).expect("parse testfile010.json");
        let original_count = original
            .get("Employees")
            .and_then(|e| e.as_array())
            .map(|a| a.len())
            .expect("original Employees array");

        // Round-trip and compare count
        let reparsed = round_trip_file("../files/testfile010.json");
        let reparsed_count = reparsed
            .get("Employees")
            .and_then(|e| e.as_array())
            .map(|a| a.len())
            .expect("reparsed Employees array");

        assert_eq!(
            original_count, reparsed_count,
            "Employee count changed after round-trip"
        );
    }

    // ─── testfile020.json – users array ──────────────────────────────────────

    #[test]
    fn test_round_trip_testfile020_root_is_array() {
        let reparsed = round_trip_file("../files/testfile020.json");
        assert!(
            reparsed.is_array(),
            "testfile020 root should remain an array after round-trip"
        );
    }

    #[test]
    fn test_round_trip_testfile020_user_count_preserved() {
        let mut source =
            FileSource::new("../files/testfile020.json").expect("open testfile020.json");
        let original = parse(&mut source).expect("parse testfile020.json");
        let original_count = original
            .as_array()
            .map(|a| a.len())
            .expect("testfile020 should be an array");

        let reparsed = round_trip_file("../files/testfile020.json");
        let reparsed_count = reparsed
            .as_array()
            .map(|a| a.len())
            .expect("reparsed testfile020 should be an array");

        assert_eq!(
            original_count, reparsed_count,
            "User count changed after round-trip"
        );
    }

    // ─── Escape sequences (testfile005) ──────────────────────────────────────

    #[test]
    fn test_round_trip_testfile005_escape_sequences() {
        let file = "../files/testfile005.json";
        let mut source = FileSource::new(file).expect("open testfile005.json");
        let original = parse(&mut source).expect("parse testfile005.json");
        assert!(original.is_object());

        // Second parse after stringify should also succeed
        let reparsed = round_trip_file(file);
        assert!(
            reparsed.is_object(),
            "testfile005 should remain an object after round-trip"
        );
    }

    // ─── Primitive type preservation ─────────────────────────────────────────

    #[test]
    fn test_round_trip_preserves_boolean_values() {
        let json = r#"{"enabled": true, "disabled": false}"#;
        let original = from_str(json).expect("parse boolean JSON");
        let mut dest = BufferDestination::new();
        stringify(&original, &mut dest).expect("stringify boolean JSON");
        let reparsed = from_str(&dest.to_string()).expect("reparse boolean JSON");

        assert_eq!(
            reparsed.get("enabled").and_then(|n| n.as_bool()),
            Some(true)
        );
        assert_eq!(
            reparsed.get("disabled").and_then(|n| n.as_bool()),
            Some(false)
        );
    }

    #[test]
    fn test_round_trip_preserves_null_value() {
        let json = r#"{"key": null}"#;
        let original = from_str(json).expect("parse null JSON");
        let mut dest = BufferDestination::new();
        stringify(&original, &mut dest).expect("stringify null JSON");
        let reparsed = from_str(&dest.to_string()).expect("reparse null JSON");

        let null_node = reparsed
            .get("key")
            .expect("'key' should exist after round-trip");
        assert!(null_node.is_null(), "'key' should be null after round-trip");
    }

    #[test]
    fn test_round_trip_preserves_integer_values() {
        let json = r#"{"count": 42, "offset": -7}"#;
        let original = from_str(json).expect("parse integer JSON");
        let mut dest = BufferDestination::new();
        stringify(&original, &mut dest).expect("stringify integer JSON");
        let reparsed = from_str(&dest.to_string()).expect("reparse integer JSON");

        assert_eq!(reparsed.get("count").and_then(|n| n.as_i64()), Some(42));
        assert_eq!(reparsed.get("offset").and_then(|n| n.as_i64()), Some(-7));
    }

    #[test]
    fn test_round_trip_preserves_string_values() {
        let json = r#"{"greeting": "hello world", "empty": ""}"#;
        let original = from_str(json).expect("parse string JSON");
        let mut dest = BufferDestination::new();
        stringify(&original, &mut dest).expect("stringify string JSON");
        let reparsed = from_str(&dest.to_string()).expect("reparse string JSON");

        assert_eq!(
            reparsed.get("greeting").and_then(|n| n.as_str()),
            Some("hello world")
        );
        assert_eq!(reparsed.get("empty").and_then(|n| n.as_str()), Some(""));
    }

    #[test]
    fn test_round_trip_preserves_nested_array_of_objects() {
        let json = r#"{"items": [{"id": 1, "name": "one"}, {"id": 2, "name": "two"}]}"#;
        let original = from_str(json).expect("parse nested JSON");
        let mut dest = BufferDestination::new();
        stringify(&original, &mut dest).expect("stringify nested JSON");
        let reparsed = from_str(&dest.to_string()).expect("reparse nested JSON");

        let items = reparsed
            .get("items")
            .and_then(|n| n.as_array())
            .expect("'items' array missing after round-trip");
        assert_eq!(items.len(), 2, "item count should be 2 after round-trip");
        assert_eq!(items[0].get("id").and_then(|n| n.as_i64()), Some(1));
        assert_eq!(items[1].get("name").and_then(|n| n.as_str()), Some("two"));
    }

    // ─── Stringify output is itself valid parseable JSON ──────────────────────

    #[test]
    fn test_stringify_output_is_valid_for_all_test_files() {
        let files = json_files_in("../files");
        for file_path in &files {
            let mut source =
                FileSource::new(file_path).unwrap_or_else(|e| panic!("open {}: {}", file_path, e));
            let node = parse(&mut source).unwrap_or_else(|e| panic!("parse {}: {}", file_path, e));
            let mut dest = BufferDestination::new();
            stringify(&node, &mut dest)
                .unwrap_or_else(|e| panic!("stringify {}: {}", file_path, e));
            let json_str = dest.to_string();
            assert!(
                !json_str.is_empty(),
                "stringify produced empty output for {}",
                file_path
            );
            assert!(
                from_str(&json_str).is_ok(),
                "stringify output for {} is not valid JSON",
                file_path
            );
        }
    }
}
