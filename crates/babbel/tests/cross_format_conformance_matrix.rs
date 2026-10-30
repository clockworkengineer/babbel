//! Cross-format semantic conformance matrix across all supported data formats.
//!
//! Validates that a canonical corpus of structured documents can be translated
//! across all text and binary formats without loss of semantic meaning or structure.

use babbel::prelude::*;

#[test]
fn test_semantic_conformance_matrix_microservice_config() {
    // Canonical configuration document
    let initial_yaml = r#"
server:
  host: 127.0.0.1
  port: 8080
  tls_enabled: true
database:
  name: production_db
  connections: 25
  replica_count: 3
tags:
  - microservice
  - primary
  - v2
"#;

    // Supported formats for microservice configurations
    let config_formats = [
        Format::Yaml,
        Format::Json,
        Format::Toml,
        Format::Ron,
        Format::Hcl,
        Format::Kdl,
        Format::Cbor,
        Format::MsgPack,
        Format::Bson,
    ];

    // 1. Initial parse
    let root_val = babbel::from_str(Format::Yaml, initial_yaml)
        .expect("Failed to parse initial YAML configuration");

    assert_eq!(root_val.get_path("server.port").and_then(|v| v.as_i64()), Some(8080));
    assert_eq!(root_val.get_path("server.tls_enabled").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(root_val.get_path("database.name").and_then(|v| v.as_str()), Some("production_db"));

    // 2. Perform cross-format translation cycle:
    // YAML -> JSON -> TOML -> CBOR -> MsgPack -> BSON -> RON -> KDL -> HCL -> JSON
    let mut current_val = root_val.clone();

    for &fmt in &config_formats {
        let serialized_bytes = babbel::to_vec(fmt, &current_val)
            .unwrap_or_else(|e| panic!("Failed to serialize to format {:?}: {}", fmt, e));

        let deserialized = babbel::from_slice(fmt, &serialized_bytes)
            .unwrap_or_else(|e| panic!("Failed to deserialize from format {:?}: {}", fmt, e));

        // Validate semantic invariant across every single format in the chain
        assert_eq!(
            deserialized.get_path("server.port").and_then(|v| v.as_i64()),
            Some(8080),
            "Port lost or corrupted in {:?}",
            fmt
        );
        assert_eq!(
            deserialized.get_path("server.tls_enabled").and_then(|v| v.as_bool()),
            Some(true),
            "tls_enabled corrupted in {:?}",
            fmt
        );
        assert_eq!(
            deserialized.get_path("database.name").and_then(|v| v.as_str()),
            Some("production_db"),
            "database name corrupted in {:?}",
            fmt
        );

        current_val = deserialized;
    }

    // Verify final state matches original semantic structure
    assert_eq!(current_val.get_path("database.connections").and_then(|v| v.as_i64()), Some(25));
}

#[test]
fn test_cross_format_tabular_metrics_matrix() {
    let initial_csv = "service,latency_ms,requests,error_rate\nauth,12,15000,0.01\npayment,45,3200,0.05\norders,28,8900,0.02\n";

    // Parse CSV
    let val = babbel::from_str(Format::Csv, initial_csv)
        .expect("Failed to parse initial CSV table");

    // CSV parses into an Array of row Objects
    if let Value::Array(rows) = &val {
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].get("service").and_then(|v| v.as_str()), Some("auth"));
    } else {
        panic!("Expected Array of rows from CSV");
    }

    // Convert CSV table into JSON, CBOR, MsgPack, YAML
    let target_formats = [Format::Json, Format::Yaml, Format::Cbor, Format::MsgPack];

    for &fmt in &target_formats {
        let converted_bytes = babbel::convert_bytes(Format::Csv, fmt, initial_csv.as_bytes())
            .unwrap_or_else(|e| panic!("Failed converting CSV to {:?}: {}", fmt, e));

        let roundtrip_val = babbel::from_slice(fmt, &converted_bytes)
            .unwrap_or_else(|e| panic!("Failed re-parsing {:?}: {}", fmt, e));

        let rows = roundtrip_val.as_array().expect("Expected array of rows");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1].get("service").and_then(|v| v.as_str()), Some("payment"));
    }
}

#[test]
fn test_direct_universal_convert_pairwise_matrix() {
    let sample_json = r#"{"title":"Babbel Multi-Format","version":2,"features":["fast","modular","safe"]}"#;

    let formats = [
        Format::Json,
        Format::Yaml,
        Format::Toml,
        Format::Ron,
        Format::Kdl,
        Format::Hcl,
    ];

    for &f1 in &formats {
        // Convert JSON -> F1
        let text_f1 = babbel::convert(Format::Json, f1, sample_json)
            .unwrap_or_else(|e| panic!("Conversion JSON -> {:?} failed: {}", f1, e));

        for &f2 in &formats {
            // Convert F1 -> F2
            let text_f2 = babbel::convert(f1, f2, &text_f1)
                .unwrap_or_else(|e| panic!("Conversion {:?} -> {:?} failed: {}", f1, f2, e));

            // Parse F2 back to Value
            let val_f2 = babbel::from_str(f2, &text_f2)
                .unwrap_or_else(|e| panic!("Parsing {:?} failed: {}", f2, e));

            assert_eq!(
                val_f2.get("title").and_then(|v| v.as_str()),
                Some("Babbel Multi-Format"),
                "Pairwise conversion {:?} -> {:?} title mismatch",
                f1,
                f2
            );
            assert_eq!(
                val_f2.get("version").and_then(|v| v.as_i64()),
                Some(2),
                "Pairwise conversion {:?} -> {:?} version mismatch",
                f1,
                f2
            );
        }
    }
}

