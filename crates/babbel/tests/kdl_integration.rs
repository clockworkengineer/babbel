//! Cross-format conversion integration tests for KDL Document Language.

use babbel::convert::*;
use babbel::{default_registry, FormatEmitter, FormatParser};

#[test]
fn test_json_kdl_roundtrip() {
    let original_json = r#"{"name":"babbel","version":2,"active":true}"#;

    // JSON -> KDL
    let kdl_str = json_to_kdl(original_json).expect("failed json to kdl");
    assert!(kdl_str.contains("name \"babbel\"") || kdl_str.contains("name"));

    // KDL -> JSON
    let roundtrip_json = kdl_to_json(&kdl_str).expect("failed kdl to json");
    assert!(roundtrip_json.contains("babbel"));
    assert!(roundtrip_json.contains("version"));
    assert!(roundtrip_json.contains("true"));
}

#[test]
fn test_yaml_kdl_roundtrip() {
    let original_yaml = "server:\n  host: 127.0.0.1\n  port: 8080\n";

    // YAML -> KDL
    let kdl_str = yaml_to_kdl(original_yaml).expect("failed yaml to kdl");
    assert!(kdl_str.contains("host") || kdl_str.contains("server"));

    // KDL -> YAML
    let roundtrip_yaml = kdl_to_yaml(&kdl_str).expect("failed kdl to yaml");
    assert!(roundtrip_yaml.contains("127.0.0.1"));
    assert!(roundtrip_yaml.contains("8080"));
}

#[test]
fn test_toml_kdl_roundtrip() {
    let toml_str = "[package]\nname = \"zellij\"\nversion = \"0.40.0\"\n";

    // TOML -> KDL
    let kdl_str = toml_to_kdl(toml_str).expect("failed toml to kdl");
    assert!(kdl_str.contains("package"));

    // KDL -> TOML
    let roundtrip_toml = kdl_to_toml(&kdl_str).expect("failed kdl to toml");
    assert!(roundtrip_toml.contains("package"));
    assert!(roundtrip_toml.contains("zellij"));
}

#[test]
fn test_xml_kdl_roundtrip() {
    let xml_str = "<config><title>KDLApp</title><timeout>30</timeout></config>";

    // XML -> KDL
    let kdl_str = xml_to_kdl(xml_str).expect("failed xml to kdl");
    assert!(kdl_str.contains("config"));

    // KDL -> XML
    let roundtrip_xml = kdl_to_xml(&kdl_str).expect("failed kdl to xml");
    assert!(roundtrip_xml.contains("KDLApp"));
    assert!(roundtrip_xml.contains("30"));
}

#[test]
fn test_msgpack_kdl_roundtrip() {
    let json_str = r#"{"layout":"compact","panes":4}"#;
    let msgpack_bytes = json_to_msgpack(json_str).expect("failed json to msgpack");

    // MsgPack -> KDL
    let kdl_str = msgpack_to_kdl(&msgpack_bytes).expect("failed msgpack to kdl");
    assert!(kdl_str.contains("compact"));

    // KDL -> MsgPack
    let roundtrip_msgpack = kdl_to_msgpack(&kdl_str).expect("failed kdl to msgpack");
    let final_json = msgpack_to_json(&roundtrip_msgpack).expect("failed msgpack to json");
    assert!(final_json.contains("layout"));
    assert!(final_json.contains("compact"));
    assert!(final_json.contains("4"));
}

#[test]
fn test_cbor_kdl_roundtrip() {
    let json_str = r#"{"terminal":"alacritty","columns":120}"#;
    let cbor_bytes = json_to_cbor(json_str).expect("failed json to cbor");

    // CBOR -> KDL
    let kdl_str = cbor_to_kdl(&cbor_bytes).expect("failed cbor to kdl");
    assert!(kdl_str.contains("alacritty"));

    // KDL -> CBOR
    let roundtrip_cbor = kdl_to_cbor(&kdl_str).expect("failed kdl to cbor");
    let final_json = cbor_to_json(&roundtrip_cbor).expect("failed cbor to json");
    assert!(final_json.contains("alacritty"));
    assert!(final_json.contains("120"));
}

#[test]
fn test_bson_kdl_roundtrip() {
    let original_json = r#"{"cluster":"primary","shards":3}"#;
    let bson_bytes = json_to_bson(original_json).expect("failed json to bson");

    // BSON -> KDL
    let kdl_str = bson_to_kdl(&bson_bytes).expect("failed bson to kdl");
    assert!(kdl_str.contains("primary"));

    // KDL -> BSON
    let roundtrip_bson = kdl_to_bson(&kdl_str).expect("failed kdl to bson");
    let final_json = bson_to_json(&roundtrip_bson).expect("failed bson to json");
    assert!(final_json.contains("primary"));
    assert!(final_json.contains("3"));
}

#[test]
fn test_ron_kdl_roundtrip() {
    let ron_str = "(theme: \"dark\", font_size: 14)";

    // RON -> KDL
    let kdl_str = ron_to_kdl(ron_str).expect("failed ron to kdl");
    assert!(kdl_str.contains("dark"));

    // KDL -> RON
    let roundtrip_ron = kdl_to_ron(&kdl_str).expect("failed kdl to ron");
    assert!(roundtrip_ron.contains("theme"));
    assert!(roundtrip_ron.contains("dark"));
    assert!(roundtrip_ron.contains("14"));
}

#[test]
fn test_json5_kdl_roundtrip() {
    let json5_str = "{ // comment\nservice: 'kdl-gateway',\nport: 9000,\n}";

    // JSON5 -> KDL
    let kdl_str = json5_to_kdl(json5_str).expect("failed json5 to kdl");
    assert!(kdl_str.contains("kdl-gateway"));

    // KDL -> JSON5
    let roundtrip_json5 = kdl_to_json5(&kdl_str).expect("failed kdl to json5");
    assert!(roundtrip_json5.contains("kdl-gateway"));
    assert!(roundtrip_json5.contains("9000"));
}

#[test]
fn test_default_registry_includes_kdl() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"kdl"), "registry should contain kdl");

    let engine = registry.get_by_id("kdl").expect("should get kdl by id");
    assert_eq!(engine.mime_type(), "application/kdl");

    let engine_ext = registry.get_by_extension("kdl").expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "kdl");
}

#[test]
fn test_kdl_parser_and_emitter_traits() {
    let kdl_input = "app \"editor\"\nstatus \"ready\"\n";
    let parsed = KdlParser.parse_str(kdl_input).expect("KdlParser should parse");
    assert_eq!(parsed.get("app").and_then(|v| v.as_str()), Some("editor"));
    assert_eq!(parsed.get("status").and_then(|v| v.as_str()), Some("ready"));

    let mut buf = babbel::core::Buffer::new();
    KdlEmitter.emit(&parsed, &mut buf).expect("KdlEmitter should emit");
    let output = buf.to_string();
    assert!(output.contains("app \"editor\"") || output.contains("app"));
    assert!(output.contains("status \"ready\"") || output.contains("status"));
}
