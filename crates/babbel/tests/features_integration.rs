//! Integration tests for newly added universal features:
//! - RFC 9535 JSONPath query engine
//! - RFC 6902 JSON Patch & RFC 7396 JSON Merge Patch
//! - Structural AST diffing
//! - Universal JSON Schema validation

use babbel::prelude::*;
use babbel::{FormatEngine, JsonEngine, TomlEngine, YamlEngine};

#[test]
fn test_universal_jsonpath_across_formats() {
    // 1. JSONPath on TOML parsed into Value
    let toml_source = r#"
        [server]
        host = "localhost"
        port = 8080
        [[server.endpoints]]
        path = "/api/v1"
        rate_limit = 100
        [[server.endpoints]]
        path = "/api/v2"
        rate_limit = 200
    "#;

    let toml_val = FormatEngine::parse_str(&TomlEngine, toml_source).unwrap();
    let paths = toml_val.jsonpath("$.server.endpoints[*].path").unwrap();
    assert_eq!(paths.len(), 2);
    assert_eq!(paths[0].as_str(), Some("/api/v1"));
    assert_eq!(paths[1].as_str(), Some("/api/v2"));

    // Filter query: endpoint with rate_limit > 150
    let high_rate = toml_val
        .jsonpath("$.server.endpoints[?(@.rate_limit > 150)].path")
        .unwrap();
    assert_eq!(high_rate.len(), 1);
    assert_eq!(high_rate[0].as_str(), Some("/api/v2"));
}

#[test]
fn test_cross_format_patch_and_diff() {
    let source_json = r#"{"name": "babbel", "version": "0.2.0", "formats": ["json", "yaml"]}"#;
    let target_json = r#"{"name": "babbel", "version": "0.3.0", "formats": ["json", "yaml", "cbor"], "active": true}"#;

    let src_val = FormatEngine::parse_str(&JsonEngine, source_json).unwrap();
    let tgt_val = FormatEngine::parse_str(&JsonEngine, target_json).unwrap();

    // 1. Compute structural diff
    let patch = babbel::diff::diff(&src_val, &tgt_val);
    assert!(!patch.ops.is_empty());

    // 2. Apply patch to source
    let patched_val = src_val.patch(&patch).unwrap();
    assert_eq!(patched_val, tgt_val);

    // 3. Compute and test merge patch
    let merge_patch = babbel::diff::diff_merge_patch(&src_val, &tgt_val);
    let mut merged = src_val.clone();
    merged.merge_patch(&merge_patch);
    assert_eq!(merged, tgt_val);
}

#[test]
fn test_universal_schema_validation_across_formats() {
    let schema_json = r#"{
        "type": "object",
        "required": ["service", "port", "routes"],
        "properties": {
            "service": {"type": "string", "minLength": 3},
            "port": {"type": "integer", "minimum": 1024, "maximum": 65535},
            "routes": {
                "type": "array",
                "minItems": 1,
                "items": {"type": "string"}
            }
        }
    }"#;

    let schema_val = FormatEngine::parse_str(&JsonEngine, schema_json).unwrap();
    let compiled = CompiledSchema::compile(&schema_val).unwrap();

    // Valid YAML input validated against JSON Schema
    let yaml_valid = r#"
        service: auth-service
        port: 8080
        routes:
          - /login
          - /logout
    "#;
    let yaml_val = FormatEngine::parse_str(&YamlEngine, yaml_valid).unwrap();
    assert!(compiled.is_valid(&yaml_val));

    // Invalid YAML input (port < 1024)
    let yaml_invalid_port = r#"
        service: auth-service
        port: 80
        routes:
          - /login
    "#;
    let yaml_val_bad = FormatEngine::parse_str(&YamlEngine, yaml_invalid_port).unwrap();
    let errs = compiled.validate(&yaml_val_bad).unwrap_err();
    assert!(errs.iter().any(|e| e.keyword == "minimum"));

    // Invalid TOML input (missing required 'routes')
    let toml_missing_routes = r#"
        service = "auth-service"
        port = 8080
    "#;
    let toml_val_bad = FormatEngine::parse_str(&TomlEngine, toml_missing_routes).unwrap();
    let errs_toml = compiled.validate(&toml_val_bad).unwrap_err();
    assert!(errs_toml.iter().any(|e| e.keyword == "required"));
}
