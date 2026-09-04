//! Integration tests for JSON validation, convenience parse functions,
//! and whitespace stripping.
//!
//! Covers `validate_json`, `from_str`, `from_bytes`, `parse_with_config`,
//! and `strip_whitespace` using both real files from the `files/` directory
//! and inline JSON strings/bytes.

#[cfg(test)]
mod tests {

    use crate::{
        BufferDestination, BufferSource, FileSource, ParserConfig, from_bytes, from_str,
        parse_with_config, strip_whitespace, validate_json,
    };
    use std::fs;

    /// Collect all .json file paths in a directory (non-recursive).
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

    // ─── validate_json: real files ────────────────────────────────────────────

    #[test]
    fn test_validate_all_test_files_pass() {
        let files = json_files_in("../files");
        assert!(
            !files.is_empty(),
            "No JSON files found in ../files – check working directory"
        );
        let config = ParserConfig::new();
        for file_path in &files {
            let mut source = FileSource::new(file_path)
                .unwrap_or_else(|e| panic!("Failed to open {}: {}", file_path, e));
            assert!(
                validate_json(&mut source, &config).is_ok(),
                "validate_json failed for {}",
                file_path
            );
        }
    }

    // ─── validate_json: valid inline JSON ─────────────────────────────────────

    #[test]
    fn test_validate_valid_object() {
        let json = br#"{"name": "Alice", "age": 30, "active": true}"#;
        let mut source = BufferSource::new(json);
        assert!(validate_json(&mut source, &ParserConfig::new()).is_ok());
    }

    #[test]
    fn test_validate_valid_array() {
        let json = br#"[1, 2, 3, "four", true, null]"#;
        let mut source = BufferSource::new(json);
        assert!(validate_json(&mut source, &ParserConfig::new()).is_ok());
    }

    #[test]
    fn test_validate_valid_nested_structure() {
        let json = br#"{"users": [{"id": 1, "name": "Bob"}, {"id": 2, "name": "Carol"}]}"#;
        let mut source = BufferSource::new(json);
        assert!(validate_json(&mut source, &ParserConfig::new()).is_ok());
    }

    #[test]
    fn test_validate_valid_all_primitive_types() {
        let json = br#"{"string": "hello", "int": 42, "float": 3.14, "bool": true, "nil": null}"#;
        let mut source = BufferSource::new(json);
        assert!(validate_json(&mut source, &ParserConfig::new()).is_ok());
    }

    #[test]
    fn test_validate_valid_empty_object() {
        let json = br#"{}"#;
        let mut source = BufferSource::new(json);
        assert!(validate_json(&mut source, &ParserConfig::new()).is_ok());
    }

    #[test]
    fn test_validate_valid_empty_array() {
        let json = br#"[]"#;
        let mut source = BufferSource::new(json);
        assert!(validate_json(&mut source, &ParserConfig::new()).is_ok());
    }

    // ─── validate_json: invalid inline JSON ───────────────────────────────────

    #[test]
    fn test_validate_invalid_unclosed_object() {
        let json = br#"{"key": "value""#;
        let mut source = BufferSource::new(json);
        assert!(
            validate_json(&mut source, &ParserConfig::new()).is_err(),
            "unclosed object should fail validation"
        );
    }

    #[test]
    fn test_validate_invalid_unclosed_array() {
        let json = br#"[1, 2, 3"#;
        let mut source = BufferSource::new(json);
        assert!(
            validate_json(&mut source, &ParserConfig::new()).is_err(),
            "unclosed array should fail validation"
        );
    }

    #[test]
    fn test_validate_invalid_unquoted_key() {
        let json = br#"{key: "value"}"#;
        let mut source = BufferSource::new(json);
        assert!(
            validate_json(&mut source, &ParserConfig::new()).is_err(),
            "unquoted object key should fail validation"
        );
    }

    #[test]
    fn test_validate_invalid_bare_word() {
        let json = br#"undefined"#;
        let mut source = BufferSource::new(json);
        assert!(
            validate_json(&mut source, &ParserConfig::new()).is_err(),
            "'undefined' is not valid JSON"
        );
    }

    // ─── from_str ─────────────────────────────────────────────────────────────

    #[test]
    fn test_from_str_parses_object() {
        let node = from_str(r#"{"city": "Berlin", "population": 3769000}"#).unwrap();
        assert!(node.is_object());
        assert_eq!(node.get("city").and_then(|n| n.as_str()), Some("Berlin"));
        assert_eq!(
            node.get("population").and_then(|n| n.as_i64()),
            Some(3_769_000)
        );
    }

    #[test]
    fn test_from_str_parses_array() {
        let node = from_str(r#"["apple", "banana", "cherry"]"#).unwrap();
        assert!(node.is_array());
        let arr = node.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str(), Some("apple"));
        assert_eq!(arr[2].as_str(), Some("cherry"));
    }

    #[test]
    fn test_from_str_parses_nested_object() {
        let json = r#"{"server": {"host": "localhost", "port": 8080}, "debug": false}"#;
        let node = from_str(json).unwrap();
        assert!(node.is_object());
        let server = node.get("server").unwrap();
        assert!(server.is_object());
        assert_eq!(
            server.get("host").and_then(|n| n.as_str()),
            Some("localhost")
        );
        assert_eq!(server.get("port").and_then(|n| n.as_i64()), Some(8080));
        assert_eq!(node.get("debug").and_then(|n| n.as_bool()), Some(false));
    }

    #[test]
    fn test_from_str_parses_null_and_boolean() {
        let node = from_str(r#"{"a": true, "b": false, "c": null}"#).unwrap();
        assert_eq!(node.get("a").and_then(|n| n.as_bool()), Some(true));
        assert_eq!(node.get("b").and_then(|n| n.as_bool()), Some(false));
        assert!(node.get("c").unwrap().is_null());
    }

    #[test]
    fn test_from_str_parses_float() {
        let node = from_str(r#"{"pi": 3.14159}"#).unwrap();
        let pi = node.get("pi").and_then(|n| n.as_f64()).unwrap();
        assert!(
            (pi - 3.14159_f64).abs() < 1e-5,
            "unexpected pi value: {}",
            pi
        );
    }

    #[test]
    fn test_from_str_fails_on_invalid_json() {
        assert!(from_str("{invalid}").is_err(), "{{invalid}} should fail");
        assert!(from_str("").is_err(), "empty string should fail");
        assert!(from_str("{\"a\":}").is_err(), "missing value should fail");
    }

    // ─── from_bytes ───────────────────────────────────────────────────────────

    #[test]
    fn test_from_bytes_parses_object() {
        let bytes = br#"{"version": "2.0", "stable": true}"#;
        let node = from_bytes(bytes).unwrap();
        assert!(node.is_object());
        assert_eq!(node.get("version").and_then(|n| n.as_str()), Some("2.0"));
        assert_eq!(node.get("stable").and_then(|n| n.as_bool()), Some(true));
    }

    #[test]
    fn test_from_bytes_parses_numeric_array() {
        let bytes = br#"[10, 20, 30, 40, 50]"#;
        let node = from_bytes(bytes).unwrap();
        assert!(node.is_array());
        let arr = node.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_i64(), Some(10));
        assert_eq!(arr[4].as_i64(), Some(50));
    }

    #[test]
    fn test_from_bytes_parses_nested_array_of_objects() {
        let bytes = br#"[{"id": 1, "active": true}, {"id": 2, "active": false}]"#;
        let node = from_bytes(bytes).unwrap();
        assert!(node.is_array());
        let arr = node.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].get("id").and_then(|n| n.as_i64()), Some(1));
        assert_eq!(arr[1].get("active").and_then(|n| n.as_bool()), Some(false));
    }

    #[test]
    fn test_from_bytes_fails_on_invalid_input() {
        assert!(from_bytes(b"not json").is_err());
        assert!(from_bytes(b"").is_err());
    }

    // ─── parse_with_config ────────────────────────────────────────────────────

    #[test]
    fn test_parse_with_config_default_succeeds_on_simple_json() {
        let json = br#"{"key": "value"}"#;
        let mut source = BufferSource::new(json);
        let config = ParserConfig::new();
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_parse_with_config_strict_succeeds_on_shallow_json() {
        let json = br#"{"name": "test", "value": 1}"#;
        let mut source = BufferSource::new(json);
        let config = ParserConfig::strict();
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_parse_with_config_unlimited_handles_deep_nesting() {
        // 5 levels deep – unlimited config should handle this without error
        let json = br#"{"a": {"b": {"c": {"d": {"e": "leaf"}}}}}"#;
        let mut source = BufferSource::new(json);
        let config = ParserConfig::unlimited();
        assert!(parse_with_config(&mut source, &config).is_ok());
    }

    #[test]
    fn test_parse_with_config_depth_limit_exceeded() {
        // 3 levels deep: { → { → { → "leaf"
        let json = br#"{"a": {"b": {"c": "leaf"}}}"#;
        let mut source = BufferSource::new(json);
        let config = ParserConfig::new().with_max_depth(Some(2));
        assert!(
            parse_with_config(&mut source, &config).is_err(),
            "JSON exceeding depth limit should fail"
        );
    }

    #[test]
    fn test_parse_with_config_array_size_limit_exceeded() {
        // Array with 5 elements vs limit of 3
        let json = br#"[1, 2, 3, 4, 5]"#;
        let mut source = BufferSource::new(json);
        let config = ParserConfig::new().with_max_array_size(Some(3));
        assert!(
            parse_with_config(&mut source, &config).is_err(),
            "array exceeding size limit should fail"
        );
    }

    // ─── strip_whitespace ─────────────────────────────────────────────────────

    #[test]
    fn test_strip_whitespace_output_is_valid_json() {
        let json = br#"  {  "key"  :  "value"  ,  "num"  :  42  }  "#;
        let mut source = BufferSource::new(json);
        let mut dest = BufferDestination::new();
        strip_whitespace(&mut source, &mut dest);
        let stripped = dest.to_string();
        assert!(!stripped.is_empty(), "stripped output should not be empty");
        assert!(
            from_str(&stripped).is_ok(),
            "stripped output is not valid JSON: {}",
            stripped
        );
    }

    #[test]
    fn test_strip_whitespace_on_file_produces_valid_json() {
        let mut source =
            FileSource::new("../files/testfile004.json").expect("open testfile004.json");
        let mut dest = BufferDestination::new();
        strip_whitespace(&mut source, &mut dest);
        let stripped = dest.to_string();
        assert!(
            from_str(&stripped).is_ok(),
            "stripped testfile004.json is not valid JSON: {}",
            stripped
        );
    }

    #[test]
    fn test_strip_whitespace_result_parses_to_same_values() {
        let json = br#"{ "latitude" : 48.8581 , "longitude" : 2.29469 }"#;
        let mut source = BufferSource::new(json);
        let mut dest = BufferDestination::new();
        strip_whitespace(&mut source, &mut dest);
        let stripped = dest.to_string();

        let node = from_str(&stripped).expect("stripped output should parse");
        let lat = node.get("latitude").and_then(|n| n.as_f64()).unwrap();
        let lon = node.get("longitude").and_then(|n| n.as_f64()).unwrap();
        assert!((lat - 48.8581_f64).abs() < 1e-4);
        assert!((lon - 2.29469_f64).abs() < 1e-4);
    }
}
