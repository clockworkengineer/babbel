//! Cross-format conversion integration tests for Rusty Object Notation (RON).

use babbel::convert::*;
use babbel::{FormatEmitter, FormatParser, default_registry};

#[test]
fn test_json_ron_roundtrip() {
    let original_json = r#"{"name":"babbel","version":2,"active":true}"#;

    // JSON -> RON
    let ron_str = json_to_ron(original_json).expect("failed json to ron");
    assert!(ron_str.contains("name: \"babbel\"") || ron_str.contains("name:"));

    // RON -> JSON
    let roundtrip_json = ron_to_json(&ron_str).expect("failed ron to json");
    assert!(roundtrip_json.contains("babbel"));
    assert!(roundtrip_json.contains("version"));
    assert!(roundtrip_json.contains("true"));
}

#[test]
fn test_yaml_ron_roundtrip() {
    let original_yaml = "graphics:\n  resolution:\n    width: 1920\n    height: 1080\n";

    // YAML -> RON
    let ron_str = yaml_to_ron(original_yaml).expect("failed yaml to ron");
    assert!(ron_str.contains("resolution:"));

    // RON -> YAML
    let roundtrip_yaml = ron_to_yaml(&ron_str).expect("failed ron to yaml");
    assert!(roundtrip_yaml.contains("resolution"));
    assert!(roundtrip_yaml.contains("1920"));
    assert!(roundtrip_yaml.contains("1080"));
}

#[test]
fn test_toml_ron_roundtrip() {
    let toml_str = "[package]\nname = \"game\"\nversion = \"1.0.0\"\n";

    // TOML -> RON
    let ron_str = toml_to_ron(toml_str).expect("failed toml to ron");
    assert!(ron_str.contains("package:"));

    // RON -> TOML
    let roundtrip_toml = ron_to_toml(&ron_str).expect("failed ron to toml");
    assert!(roundtrip_toml.contains("package"));
    assert!(roundtrip_toml.contains("game"));
}

#[test]
fn test_xml_ron_roundtrip() {
    let xml_str = "<config><title>SpaceGame</title><fps>60</fps></config>";

    // XML -> RON
    let ron_str = xml_to_ron(xml_str).expect("failed xml to ron");
    assert!(ron_str.contains("config:"));

    // RON -> XML
    let roundtrip_xml = ron_to_xml(&ron_str).expect("failed ron to xml");
    assert!(roundtrip_xml.contains("SpaceGame"));
    assert!(roundtrip_xml.contains("60"));
}

#[test]
fn test_msgpack_ron_roundtrip() {
    let json_str = r#"{"engine":"bevy","entities":42}"#;
    let msgpack_bytes = json_to_msgpack(json_str).expect("failed json to msgpack");

    // MsgPack -> RON
    let ron_str = msgpack_to_ron(&msgpack_bytes).expect("failed msgpack to ron");
    assert!(ron_str.contains("engine: \"bevy\""));

    // RON -> MsgPack
    let roundtrip_msgpack = ron_to_msgpack(&ron_str).expect("failed ron to msgpack");
    let final_json = msgpack_to_json(&roundtrip_msgpack).expect("failed msgpack to json");
    assert!(final_json.contains("bevy"));
    assert!(final_json.contains("42"));
}

#[test]
fn test_cbor_ron_roundtrip() {
    let json_str = r#"{"renderer":"Vulkan","msaa":4}"#;
    let cbor_bytes = json_to_cbor(json_str).expect("failed json to cbor");

    // CBOR -> RON
    let ron_str = cbor_to_ron(&cbor_bytes).expect("failed cbor to ron");
    assert!(ron_str.contains("renderer: \"Vulkan\""));

    // RON -> CBOR
    let roundtrip_cbor = ron_to_cbor(&ron_str).expect("failed ron to cbor");
    let final_json = cbor_to_json(&roundtrip_cbor).expect("failed cbor to json");
    assert!(final_json.contains("Vulkan"));
    assert!(final_json.contains("4"));
}

#[test]
fn test_bson_ron_roundtrip() {
    let original_json = r#"{"level":"dungeon","depth":10}"#;
    let bson_bytes = json_to_bson(original_json).expect("failed json to bson");

    // BSON -> RON
    let ron_str = bson_to_ron(&bson_bytes).expect("failed bson to ron");
    assert!(ron_str.contains("level: \"dungeon\""));

    // RON -> BSON
    let roundtrip_bson = ron_to_bson(&ron_str).expect("failed ron to bson");
    let final_json = bson_to_json(&roundtrip_bson).expect("failed bson to json");
    assert!(final_json.contains("dungeon"));
    assert!(final_json.contains("10"));
}

#[test]
fn test_json5_ron_roundtrip() {
    let json5_str = "{ // trailing comma and unquoted key\nspeed: 120.5,\n}";

    // JSON5 -> RON
    let ron_str = json5_to_ron(json5_str).expect("failed json5 to ron");
    assert!(ron_str.contains("speed: 120.5"));

    // RON -> JSON5
    let roundtrip_json5 = ron_to_json5(&ron_str).expect("failed ron to json5");
    assert!(roundtrip_json5.contains("speed"));
    assert!(roundtrip_json5.contains("120.5"));
}

#[test]
fn test_default_registry_includes_ron() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"ron"), "registry should contain ron");

    let engine = registry.get_by_id("ron").expect("should get ron by id");
    assert_eq!(engine.mime_type(), "application/ron");

    let engine_ext = registry.get_by_extension("ron").expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "ron");
}

#[test]
fn test_ron_parser_and_emitter_traits() {
    let ron_input = "(title: \"Rustacean\", stars: 5)";
    let parsed = RonParser
        .parse_str(ron_input)
        .expect("RonParser should parse");
    assert_eq!(
        parsed.get("title").and_then(|v| v.as_str()),
        Some("Rustacean")
    );
    assert_eq!(parsed.get("stars").and_then(|v| v.as_i64()), Some(5));

    let mut buf = babbel::core::Buffer::new();
    RonEmitter
        .emit(&parsed, &mut buf)
        .expect("RonEmitter should emit");
    let output = buf.to_string();
    assert!(output.contains("title: \"Rustacean\""));
    assert!(output.contains("stars: 5"));
}
