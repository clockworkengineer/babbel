//! End-to-end smoke test verifying polyglot format usage via the babbel facade.

#[test]
fn test_polyglot_smoke() {
    // 1. Babbel Core
    let mut dest = babbel::core::StringDestination::new();
    babbel::core::format_integer(42, &mut dest);
    assert_eq!(dest.into_string(), "42");

    // 2. JSON via babbel::json
    let json_text = r#"{"name": "babbel", "valid": true}"#;
    let json_doc = babbel::json::from_str(json_text);
    assert!(json_doc.is_ok(), "JSON parsing failed");

    // 3. YAML via babbel::yaml
    let yaml_text = "project: babbel\nstatus: ready\n";
    let yaml_doc = babbel::yaml::parse_string(yaml_text);
    assert!(yaml_doc.is_ok(), "YAML parsing failed");

    // 4. XML via babbel::xml
    let xml_text = "<babbel status=\"active\"><item id=\"1\"/></babbel>";
    let xml_doc = babbel::xml::parse(xml_text);
    assert!(xml_doc.is_ok(), "XML parsing failed");

    // 5. Bencode via babbel::bencode (keys sorted lexicographically: 'name' < 'spam')
    let bencode_data = b"d4:name6:babbel4:spaml4:eggsee";
    let bencode_doc = babbel::bencode::parse_bytes(bencode_data);
    assert!(bencode_doc.is_ok(), "Bencode parsing failed");

    // 6. TOML via babbel::toml
    let toml_text = "service = \"babbel\"\nport = 8080\nenabled = true\n";
    let toml_doc = babbel::toml::from_str(toml_text);
    assert!(toml_doc.is_ok(), "TOML parsing failed");
}

#[test]
fn test_cross_format_conversions() {
    // 1. JSON -> YAML
    let json = r#"{"name":"babbel","stars":100}"#;
    let yaml = babbel::convert::json_to_yaml(json).expect("JSON -> YAML failed");
    assert!(yaml.contains("name: babbel") || yaml.contains("name: \"babbel\""));

    // 2. YAML -> JSON
    let yaml_src = "service: api\nport: 8080\n";
    let json_out = babbel::convert::yaml_to_json(yaml_src).expect("YAML -> JSON failed");
    assert!(json_out.contains("\"service\"") && json_out.contains("\"api\""));

    // 3. Bencode -> JSON
    let bencode = b"d3:agei30e4:user5:alicee";
    let json_from_bencode = babbel::convert::bencode_to_json(bencode).expect("Bencode -> JSON failed");
    assert!(json_from_bencode.contains("\"user\"") && json_from_bencode.contains("\"alice\""));

    // 4. Bencode -> YAML
    let yaml_from_bencode = babbel::convert::bencode_to_yaml(bencode).expect("Bencode -> YAML failed");
    assert!(yaml_from_bencode.contains("alice"));

    // 5. JSON -> Bencode
    let bencode_out = babbel::convert::json_to_bencode(json).expect("JSON -> Bencode failed");
    let reparsed = babbel::bencode::parse_bytes(&bencode_out).expect("Reparsing bencode failed");
    assert!(reparsed.is_dictionary());

    // 6. TOML -> JSON
    let toml_sample = "app = \"babbel\"\nthreads = 4\n";
    let json_from_toml = babbel::convert::toml_to_json(toml_sample).expect("TOML -> JSON failed");
    assert!(json_from_toml.contains("babbel") && json_from_toml.contains("4"));

    // 7. JSON -> TOML
    let toml_from_json = babbel::convert::json_to_toml(r#"{"app":"babbel","threads":4}"#).expect("JSON -> TOML failed");
    assert!(toml_from_json.contains("app = \"babbel\""));

    // 8. TOML -> YAML
    let yaml_from_toml = babbel::convert::toml_to_yaml(toml_sample).expect("TOML -> YAML failed");
    assert!(yaml_from_toml.contains("babbel"));
}
