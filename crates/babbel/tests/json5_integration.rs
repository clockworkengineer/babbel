use babbel::convert::*;
use babbel::default_registry;

#[test]
fn test_default_registry_includes_json5() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(formats.contains(&"json5"), "registry should contain json5");

    let engine = registry.get_by_id("json5").expect("json5 should be registered in default_registry");
    assert_eq!(engine.format_id(), "json5");
    assert_eq!(engine.mime_type(), "application/json5");
    assert!(engine.file_extensions().contains(&"json5"));
    assert!(engine.file_extensions().contains(&"jsonc"));

    let engine_ext = registry.get_by_extension("json5").expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "json5");
}


#[test]
fn test_json5_to_json_conversion() {
    let json5_input = r#"{
        // Server configuration
        host: 'localhost',
        port: 8080,
        /* Development flags */
        debug: true,
        allowed_origins: [
            'https://example.com',
            'http://localhost:3000',
        ],
    }"#;

    let json_output = json5_to_json(json5_input).expect("Conversion json5 -> json failed");
    let parsed_json = babbel::json::from_str(&json_output).expect("Invalid JSON produced");
    assert_eq!(parsed_json["host"].as_str(), Some("localhost"));
    assert_eq!(parsed_json["port"].as_i64(), Some(8080));
    assert_eq!(parsed_json["debug"].as_bool(), Some(true));
    assert_eq!(parsed_json["allowed_origins"].len(), Some(2));
}

#[test]
fn test_json5_to_yaml_and_toml() {
    let json5_input = r#"{
        service: 'metrics-agent',
        interval_ms: 1000,
        tags: ['prod', 'aws',],
    }"#;

    let yaml_out = json5_to_yaml(json5_input).expect("Conversion json5 -> yaml failed");
    assert!(yaml_out.contains("service: metrics-agent"));

    let toml_out = json5_to_toml(json5_input).expect("Conversion json5 -> toml failed");
    assert!(toml_out.contains("service = \"metrics-agent\""));
}

#[test]
fn test_json5_hex_and_specials() {
    let json5_input = r#"{
        hex_val: 0xff,
        neg_hex: -0x20,
        lead_dot: .75,
        plus_sign: +100,
    }"#;

    let json_out = json5_to_json(json5_input).expect("Conversion failed");
    let parsed = babbel::json::from_str(&json_out).expect("Parse failed");
    assert_eq!(parsed["hex_val"].as_i64(), Some(255));
    assert_eq!(parsed["neg_hex"].as_i64(), Some(-32));
    assert_eq!(parsed["lead_dot"].as_f64(), Some(0.75));
    assert_eq!(parsed["plus_sign"].as_i64(), Some(100));
}
