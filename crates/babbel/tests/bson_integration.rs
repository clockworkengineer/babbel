//! Cross-format conversion integration tests for BSON.

use babbel::convert::*;
use babbel::default_registry;

#[test]
fn test_json_bson_roundtrip() {
    let original_json = r#"{"name":"babbel","version":2,"active":true}"#;

    // JSON -> BSON
    let bson_bytes = json_to_bson(original_json).expect("failed json to bson");

    // BSON -> JSON
    let roundtrip_json = bson_to_json(&bson_bytes).expect("failed bson to json");

    assert!(roundtrip_json.contains("babbel"));
    assert!(roundtrip_json.contains("version"));
    assert!(roundtrip_json.contains("true"));
}

#[test]
fn test_yaml_bson_roundtrip() {
    let original_yaml = "database: mongodb\ntags:\n  - document\n  - nosql\n";

    // YAML -> BSON
    let bson_bytes = yaml_to_bson(original_yaml).expect("failed yaml to bson");

    // BSON -> YAML
    let roundtrip_yaml = bson_to_yaml(&bson_bytes).expect("failed bson to yaml");

    assert!(roundtrip_yaml.contains("database"));
    assert!(roundtrip_yaml.contains("mongodb"));
    assert!(roundtrip_yaml.contains("document"));
}

#[test]
fn test_toml_bson_roundtrip() {
    let toml_str = "[database]\nhost = \"127.0.0.1\"\nport = 27017\n";

    // TOML -> BSON
    let bson_bytes = toml_to_bson(toml_str).expect("failed toml to bson");

    // BSON -> TOML
    let roundtrip_toml = bson_to_toml(&bson_bytes).expect("failed bson to toml");

    assert!(roundtrip_toml.contains("database"));
    assert!(roundtrip_toml.contains("127.0.0.1"));
    assert!(roundtrip_toml.contains("27017"));
}

#[test]
fn test_msgpack_bson_roundtrip() {
    let json_str = r#"{"collection":"metrics","count":500}"#;
    let msgpack_bytes = json_to_msgpack(json_str).expect("failed json to msgpack");

    // MsgPack -> BSON
    let bson_bytes = msgpack_to_bson(&msgpack_bytes).expect("failed msgpack to bson");

    // BSON -> MsgPack
    let roundtrip_msgpack = bson_to_msgpack(&bson_bytes).expect("failed bson to msgpack");

    let final_json = msgpack_to_json(&roundtrip_msgpack).expect("failed msgpack to json");
    assert!(final_json.contains("metrics"));
    assert!(final_json.contains("500"));
}

#[test]
fn test_cbor_bson_roundtrip() {
    let json_str = r#"{"cluster":"shard-01","healthy":true}"#;
    let cbor_bytes = json_to_cbor(json_str).expect("failed json to cbor");

    // CBOR -> BSON
    let bson_bytes = cbor_to_bson(&cbor_bytes).expect("failed cbor to bson");

    // BSON -> CBOR
    let roundtrip_cbor = bson_to_cbor(&bson_bytes).expect("failed bson to cbor");

    let final_json = cbor_to_json(&roundtrip_cbor).expect("failed cbor to json");
    assert!(final_json.contains("shard-01"));
    assert!(final_json.contains("true"));
}

#[test]
fn test_csv_bson_conversion() {
    let csv_str = "id,name\n1,Alice\n2,Bob\n";

    // CSV -> BSON
    let bson_bytes = csv_to_bson(csv_str).expect("failed csv to bson");

    // BSON -> CSV
    let roundtrip_csv = bson_to_csv(&bson_bytes).expect("failed bson to csv");

    assert!(roundtrip_csv.contains("Alice"));
    assert!(roundtrip_csv.contains("Bob"));
}

#[test]
fn test_default_registry_includes_bson() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"bson"), "registry should contain bson");

    let engine = registry.get_by_id("bson").expect("should get bson by id");
    assert_eq!(engine.mime_type(), "application/bson");

    let engine_ext = registry.get_by_extension("bson").expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "bson");
}
