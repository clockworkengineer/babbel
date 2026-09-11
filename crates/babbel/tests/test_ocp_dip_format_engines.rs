//! Integration tests for Phase 4: Open-Closed Principle (OCP) and
//! Dependency Inversion Principle (DIP) FormatEngine Architecture.

use babbel::{
    default_registry, find_engine, find_engine_by_extension, find_engine_by_mime,
    BencodeEngine, FormatEngine, FormatOptions, JsonEngine, TomlEngine, Value, XmlEngine,
    YamlEngine,
};

#[test]
fn test_format_engine_metadata() {
    let json = JsonEngine;
    assert_eq!(json.format_id(), "json");
    assert_eq!(json.mime_type(), "application/json");
    assert!(json.file_extensions().contains(&"json"));

    let yaml = YamlEngine;
    assert_eq!(yaml.format_id(), "yaml");
    assert_eq!(yaml.mime_type(), "application/yaml");
    assert!(yaml.file_extensions().contains(&"yaml"));
    assert!(yaml.file_extensions().contains(&"yml"));

    let xml = XmlEngine;
    assert_eq!(xml.format_id(), "xml");
    assert_eq!(xml.mime_type(), "application/xml");
    assert!(xml.file_extensions().contains(&"xml"));

    let bencode = BencodeEngine;
    assert_eq!(bencode.format_id(), "bencode");
    assert_eq!(bencode.mime_type(), "application/x-bencode");
    assert!(bencode.file_extensions().contains(&"torrent"));

    let toml = TomlEngine;
    assert_eq!(toml.format_id(), "toml");
    assert_eq!(toml.mime_type(), "application/toml");
    assert!(toml.file_extensions().contains(&"toml"));
}

#[test]
fn test_format_engine_bidirectional_roundtrip() {
    let json = JsonEngine;
    let yaml = YamlEngine;
    let toml = TomlEngine;

    // 1. Parse JSON -> Value
    let json_text = r#"{"name":"babbel","version":1}"#;
    let val = json.parse_str(json_text).expect("JSON should parse into Value");
    assert!(matches!(&val, Value::Object(_)));

    // 2. Value -> YAML string via YamlEngine
    let yaml_str = yaml.serialize_to_string(&val, &FormatOptions::default()).expect("YAML serialize");
    assert!(yaml_str.contains("name: babbel"));
    assert!(yaml_str.contains("version: 1"));

    // 3. YAML string -> Value via YamlEngine
    let val_from_yaml = yaml.parse_str(&yaml_str).expect("YAML parse into Value");
    assert!(matches!(&val_from_yaml, Value::Object(_)));

    // 4. Value -> TOML string via TomlEngine
    let toml_str = toml.serialize_to_string(&val_from_yaml, &FormatOptions::default()).expect("TOML serialize");
    assert!(toml_str.contains("name = \"babbel\""));
    assert!(toml_str.contains("version = 1"));

    // 5. TOML string -> Value via TomlEngine
    let val_from_toml = toml.parse_str(&toml_str).expect("TOML parse into Value");
    assert!(matches!(&val_from_toml, Value::Object(_)));
}

#[test]
fn test_bencode_engine_bytes_roundtrip() {
    let bencode = BencodeEngine;

    // Raw bencode dictionary: d4:name6:babbel7:versioni1ee
    let bytes = b"d4:name6:babbel7:versioni1ee";
    let val = bencode.parse_bytes(bytes).expect("Bencode should parse bytes");
    assert!(matches!(&val, Value::Object(_)));

    let serialized = bencode.serialize_to_vec(&val, &FormatOptions::compact()).expect("Bencode serialize");
    assert_eq!(serialized, bytes);
}

#[test]
fn test_xml_engine_parse_and_emit() {
    let xml = XmlEngine;

    let xml_text = r#"<config version="1"><name>babbel</name></config>"#;
    let val = xml.parse_str(xml_text).expect("XML should parse into Value");
    assert!(matches!(&val, Value::Object(_)));

    let emitted = xml.serialize_to_string(&val, &FormatOptions::default()).expect("XML serialize");
    assert!(emitted.contains("<config"));
    assert!(emitted.contains("</config>"));
}

#[test]
fn test_dynamic_format_registry() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"json"));
    assert!(formats.contains(&"yaml"));
    assert!(formats.contains(&"xml"));
    assert!(formats.contains(&"bencode"));
    assert!(formats.contains(&"toml"));

    // Lookup by ID (case-insensitive)
    assert!(registry.get_by_id("JSON").is_some());
    assert!(registry.get_by_id("yaml").is_some());
    assert!(registry.get_by_id("nonexistent").is_none());

    // Lookup by MIME
    assert_eq!(registry.get_by_mime("application/json").unwrap().format_id(), "json");
    assert_eq!(registry.get_by_mime("application/yaml").unwrap().format_id(), "yaml");

    // Lookup by extension (with or without dot)
    assert_eq!(registry.get_by_extension("json").unwrap().format_id(), "json");
    assert_eq!(registry.get_by_extension(".yml").unwrap().format_id(), "yaml");
    assert_eq!(registry.get_by_extension(".torrent").unwrap().format_id(), "bencode");
    assert_eq!(registry.get_by_extension("toml").unwrap().format_id(), "toml");
}

#[test]
fn test_static_slice_engine_lookups() {
    let engines: &[&dyn FormatEngine] = &[
        &JsonEngine,
        &YamlEngine,
        &XmlEngine,
        &BencodeEngine,
        &TomlEngine,
    ];

    assert_eq!(find_engine(engines, "JSON").unwrap().format_id(), "json");
    assert_eq!(find_engine_by_extension(engines, ".yml").unwrap().format_id(), "yaml");
    assert_eq!(find_engine_by_mime(engines, "application/xml").unwrap().format_id(), "xml");
}
