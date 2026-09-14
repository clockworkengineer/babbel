//! Cross-format conversion integration tests for MessagePack.

use babbel::convert::*;
use babbel::default_registry;

#[test]
fn test_json_msgpack_roundtrip() {
    let original_json = r#"{"name":"babbel","version":2,"active":true}"#;

    // JSON -> MsgPack
    let msgpack_bytes = json_to_msgpack(original_json).expect("failed json to msgpack");

    // MsgPack -> JSON
    let roundtrip_json = msgpack_to_json(&msgpack_bytes).expect("failed msgpack to json");

    assert!(roundtrip_json.contains("babbel"));
    assert!(roundtrip_json.contains("version"));
    assert!(roundtrip_json.contains("true"));
}

#[test]
fn test_yaml_msgpack_roundtrip() {
    let original_yaml = "project: babbel\ntags:\n  - serialization\n  - binary\n";

    // YAML -> MsgPack
    let msgpack_bytes = yaml_to_msgpack(original_yaml).expect("failed yaml to msgpack");

    // MsgPack -> YAML
    let roundtrip_yaml = msgpack_to_yaml(&msgpack_bytes).expect("failed msgpack to yaml");

    assert!(roundtrip_yaml.contains("project"));
    assert!(roundtrip_yaml.contains("babbel"));
    assert!(roundtrip_yaml.contains("serialization"));
}

#[test]
fn test_toml_msgpack_roundtrip() {
    let toml_str = "[server]\nhost = \"127.0.0.1\"\nport = 8080\n";

    // TOML -> MsgPack
    let msgpack_bytes = toml_to_msgpack(toml_str).expect("failed toml to msgpack");

    // MsgPack -> TOML
    let roundtrip_toml = msgpack_to_toml(&msgpack_bytes).expect("failed msgpack to toml");

    assert!(roundtrip_toml.contains("host"));
    assert!(roundtrip_toml.contains("127.0.0.1"));
    assert!(roundtrip_toml.contains("8080"));
}

#[test]
fn test_csv_msgpack_conversion() {
    let csv_str = "id,name\n1,Alice\n2,Bob\n";

    // CSV -> MsgPack
    let msgpack_bytes = csv_to_msgpack(csv_str).expect("failed csv to msgpack");

    // MsgPack -> CSV
    let roundtrip_csv = msgpack_to_csv(&msgpack_bytes).expect("failed msgpack to csv");

    assert!(roundtrip_csv.contains("Alice"));
    assert!(roundtrip_csv.contains("Bob"));
}

#[test]
fn test_default_registry_includes_msgpack() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"msgpack"), "registry should contain msgpack");

    let engine = registry.get_by_id("msgpack").expect("should get msgpack by id");
    assert_eq!(engine.mime_type(), "application/msgpack");

    let engine_ext = registry.get_by_extension("msgpack").expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "msgpack");

    let engine_mp = registry.get_by_extension("mp").expect("should get by .mp");
    assert_eq!(engine_mp.format_id(), "msgpack");
}
