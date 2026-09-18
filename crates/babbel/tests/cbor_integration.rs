//! Cross-format conversion integration tests for CBOR (RFC 8949).

use babbel::convert::*;
use babbel::default_registry;

#[test]
fn test_json_cbor_roundtrip() {
    let original_json = r#"{"name":"babbel","version":2,"active":true}"#;

    // JSON -> CBOR
    let cbor_bytes = json_to_cbor(original_json).expect("failed json to cbor");

    // CBOR -> JSON
    let roundtrip_json = cbor_to_json(&cbor_bytes).expect("failed cbor to json");

    assert!(roundtrip_json.contains("babbel"));
    assert!(roundtrip_json.contains("version"));
    assert!(roundtrip_json.contains("true"));
}

#[test]
fn test_yaml_cbor_roundtrip() {
    let original_yaml = "protocol: coap\ntags:\n  - iot\n  - binary\n";

    // YAML -> CBOR
    let cbor_bytes = yaml_to_cbor(original_yaml).expect("failed yaml to cbor");

    // CBOR -> YAML
    let roundtrip_yaml = cbor_to_yaml(&cbor_bytes).expect("failed cbor to yaml");

    assert!(roundtrip_yaml.contains("protocol"));
    assert!(roundtrip_yaml.contains("coap"));
    assert!(roundtrip_yaml.contains("iot"));
}

#[test]
fn test_toml_cbor_roundtrip() {
    let toml_str = "[device]\nip = \"192.168.1.1\"\nport = 5683\n";

    // TOML -> CBOR
    let cbor_bytes = toml_to_cbor(toml_str).expect("failed toml to cbor");

    // CBOR -> TOML
    let roundtrip_toml = cbor_to_toml(&cbor_bytes).expect("failed cbor to toml");

    assert!(roundtrip_toml.contains("device"));
    assert!(roundtrip_toml.contains("192.168.1.1"));
    assert!(roundtrip_toml.contains("5683"));
}

#[test]
fn test_msgpack_cbor_roundtrip() {
    let json_str = r#"{"sensor":"temperature","reading":23.5}"#;
    let msgpack_bytes = json_to_msgpack(json_str).expect("failed json to msgpack");

    // MsgPack -> CBOR
    let cbor_bytes = msgpack_to_cbor(&msgpack_bytes).expect("failed msgpack to cbor");

    // CBOR -> MsgPack
    let roundtrip_msgpack = cbor_to_msgpack(&cbor_bytes).expect("failed cbor to msgpack");

    let final_json = msgpack_to_json(&roundtrip_msgpack).expect("failed msgpack to json");
    assert!(final_json.contains("temperature"));
    assert!(final_json.contains("reading"));
}

#[test]
fn test_csv_cbor_conversion() {
    let csv_str = "device,status\nnode-1,online\nnode-2,offline\n";

    // CSV -> CBOR
    let cbor_bytes = csv_to_cbor(csv_str).expect("failed csv to cbor");

    // CBOR -> CSV
    let roundtrip_csv = cbor_to_csv(&cbor_bytes).expect("failed cbor to csv");

    assert!(roundtrip_csv.contains("node-1"));
    assert!(roundtrip_csv.contains("node-2"));
}

#[test]
fn test_default_registry_includes_cbor() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"cbor"), "registry should contain cbor");

    let engine = registry.get_by_id("cbor").expect("should get cbor by id");
    assert_eq!(engine.mime_type(), "application/cbor");

    let engine_ext = registry
        .get_by_extension("cbor")
        .expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "cbor");
}
