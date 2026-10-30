//! Integration tests verifying Babbel's top-level API facade, cross-format converters,
//! fluent AST builders, and formatting/parsing option builders (Phase 2 refactor).

use babbel::prelude::*;

#[test]
fn test_universal_top_level_facade() {
    let json_text = r#"{"name":"babbel","version":"0.2.2","ports":[8080,9090]}"#;

    // from_str
    let val = from_str(Format::Json, json_text).expect("parse json via top-level from_str");
    assert_eq!(val.get_path("name").and_then(|v| v.as_str()), Some("babbel"));
    assert_eq!(val.get_path("ports.0").and_then(|v| v.as_i64()), Some(8080));

    // to_string (compact)
    let compact_json = to_string(Format::Json, &val).expect("to_string json");
    assert!(compact_json.contains(r#""name":"babbel""#));

    // to_string_pretty
    let pretty_json = to_string_pretty(Format::Json, &val).expect("to_string_pretty json");
    assert!(pretty_json.contains('\n'));
    assert!(pretty_json.contains("  \"name\": \"babbel\""));

    // to_vec and from_slice
    let cbor_bytes = to_vec(Format::Cbor, &val).expect("to_vec cbor");
    let cbor_val = from_slice(Format::Cbor, &cbor_bytes).expect("from_slice cbor");
    assert_eq!(cbor_val, val);

    // from_file_str & from_file_bytes (extension detection)
    let file_val = from_file_str("settings.json", json_text).expect("from_file_str json");
    assert_eq!(file_val, val);

    let file_cbor = from_file_bytes("archive.cbor", &cbor_bytes).expect("from_file_bytes cbor");
    assert_eq!(file_cbor, val);
}

#[test]
fn test_universal_converter() {
    let json_src = r#"{"service":"auth","active":true}"#;

    // convert string
    let yaml_dest = convert(Format::Json, Format::Yaml, json_src).expect("convert json to yaml");
    assert!(yaml_dest.contains("service: auth"));
    assert!(yaml_dest.contains("active: true"));

    let toml_dest = convert(Format::Yaml, Format::Toml, &yaml_dest).expect("convert yaml to toml");
    assert!(toml_dest.contains("service = \"auth\""));

    // convert bytes
    let msgpack_bytes = to_vec(Format::MsgPack, &from_str(Format::Json, json_src).unwrap()).unwrap();
    let cbor_bytes = convert_bytes(Format::MsgPack, Format::Cbor, &msgpack_bytes).expect("convert_bytes msgpack to cbor");
    let decoded = from_slice(Format::Cbor, &cbor_bytes).expect("decode converted cbor");
    assert_eq!(decoded.get_path("service").and_then(|v| v.as_str()), Some("auth"));
}

#[test]
fn test_fluent_value_builders_and_mutation() {
    // Fluent builder constructing nested AST
    let config: Value = Value::object()
        .insert("server", "gateway")
        .insert("port", 443)
        .insert("tls", true)
        .insert(
            "endpoints",
            Value::array().push("/api/v1").push("/healthz").build(),
        )
        .build();

    assert_eq!(config.get_path("server").and_then(|v| v.as_str()), Some("gateway"));
    assert_eq!(config.get_path("port").and_then(|v| v.as_i64()), Some(443));
    assert_eq!(config.get_path("endpoints.1").and_then(|v| v.as_str()), Some("/healthz"));

    // Mutable path navigation and in-place updates
    let mut updated = config;
    if let Some(port) = updated.get_path_mut("port") {
        *port = Value::Integer(8443);
    }
    assert_eq!(updated.get_path("port").and_then(|v| v.as_i64()), Some(8443));

    // set & push convenience helpers
    updated.set("env", "production");
    assert_eq!(updated.get_path("env").and_then(|v| v.as_str()), Some("production"));

    if let Some(endpoints) = updated.get_path_mut("endpoints") {
        endpoints.push("/metrics");
    }
    assert_eq!(updated.get_path("endpoints.2").and_then(|v| v.as_str()), Some("/metrics"));
}

#[test]
fn test_format_and_parse_options_builders() {
    let fmt_opts = FormatOptions::builder()
        .pretty(true)
        .indent(4)
        .sort_keys(true)
        .crlf(true)
        .build();

    assert!(fmt_opts.pretty);
    assert_eq!(fmt_opts.indent, 4);
    assert!(fmt_opts.sort_keys);
    assert!(fmt_opts.crlf);

    let parse_opts = ParseOptions::new()
        .with_max_depth(64)
        .with_max_size(1024 * 1024)
        .with_duplicate_keys(DuplicateKeyMode::Error);

    assert_eq!(parse_opts.max_depth, 64);
    assert_eq!(parse_opts.max_size, 1024 * 1024);
    assert_eq!(parse_opts.duplicate_keys, DuplicateKeyMode::Error);
}

