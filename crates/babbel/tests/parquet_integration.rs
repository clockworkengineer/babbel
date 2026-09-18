//! Cross-format conversion integration tests for Apache Parquet.

use babbel::convert::*;
use babbel::{FormatEmitter, FormatParser, Value, default_registry};

#[test]
fn test_json_parquet_roundtrip() {
    let json_table =
        r#"[{"id":1,"city":"Prague","score":95.5},{"id":2,"city":"Vienna","score":91.0}]"#;

    // JSON -> Parquet
    let pq_bytes = json_to_parquet(json_table).expect("failed json to parquet");
    assert_eq!(&pq_bytes[0..4], b"PAR1");

    // Parquet -> JSON
    let roundtrip_json = parquet_to_json(&pq_bytes).expect("failed parquet to json");
    assert!(roundtrip_json.contains("Prague"));
    assert!(roundtrip_json.contains("Vienna"));
    assert!(roundtrip_json.contains("95.5"));
}

#[test]
fn test_csv_parquet_roundtrip() {
    let csv_data = "id,name,score\n10,Alice,100\n20,Bob,85\n";

    // CSV -> Parquet
    let pq_bytes = csv_to_parquet(csv_data).expect("failed csv to parquet");
    assert_eq!(&pq_bytes[0..4], b"PAR1");

    // Parquet -> CSV
    let roundtrip_csv = parquet_to_csv(&pq_bytes).expect("failed parquet to csv");
    assert!(roundtrip_csv.contains("Alice"));
    assert!(roundtrip_csv.contains("Bob"));
}

#[test]
fn test_yaml_parquet_roundtrip() {
    let yaml_data = "- id: 1\n  name: Alpha\n- id: 2\n  name: Beta\n";

    // YAML -> Parquet
    let pq_bytes = yaml_to_parquet(yaml_data).expect("failed yaml to parquet");
    assert_eq!(&pq_bytes[0..4], b"PAR1");

    // Parquet -> YAML
    let roundtrip_yaml = parquet_to_yaml(&pq_bytes).expect("failed parquet to yaml");
    assert!(roundtrip_yaml.contains("Alpha"));
    assert!(roundtrip_yaml.contains("Beta"));
}

#[test]
fn test_toml_parquet_roundtrip() {
    let json_data = r#"[{"pkg":"babbel","stars":500}]"#;
    let pq_bytes = json_to_parquet(json_data).expect("failed to parquet");

    // Parquet -> TOML
    let toml_str = parquet_to_toml(&pq_bytes).expect("failed parquet to toml");
    assert!(toml_str.contains("babbel"));
    assert!(toml_str.contains("500"));
}

#[test]
fn test_msgpack_parquet_roundtrip() {
    let json_data = r#"[{"metric":"cpu","pct":45.2}]"#;
    let msgpack_bytes = json_to_msgpack(json_data).expect("failed json to msgpack");

    // MsgPack -> Parquet
    let pq_bytes = msgpack_to_parquet(&msgpack_bytes).expect("failed msgpack to parquet");
    assert_eq!(&pq_bytes[0..4], b"PAR1");

    // Parquet -> MsgPack
    let roundtrip_msgpack = parquet_to_msgpack(&pq_bytes).expect("failed parquet to msgpack");
    let final_json = msgpack_to_json(&roundtrip_msgpack).expect("failed msgpack to json");
    assert!(final_json.contains("cpu"));
    assert!(final_json.contains("45.2"));
}

#[test]
fn test_cbor_parquet_roundtrip() {
    let json_data = r#"[{"sensor":"temp","value":21.5}]"#;
    let cbor_bytes = json_to_cbor(json_data).expect("failed json to cbor");

    // CBOR -> Parquet
    let pq_bytes = cbor_to_parquet(&cbor_bytes).expect("failed cbor to parquet");
    assert_eq!(&pq_bytes[0..4], b"PAR1");

    // Parquet -> CBOR
    let roundtrip_cbor = parquet_to_cbor(&pq_bytes).expect("failed parquet to cbor");
    let final_json = cbor_to_json(&roundtrip_cbor).expect("failed cbor to json");
    assert!(final_json.contains("temp"));
    assert!(final_json.contains("21.5"));
}

#[test]
fn test_bson_parquet_roundtrip() {
    let json_data = r#"[{"id":1,"status":"ok"}]"#;
    let pq_bytes = json_to_parquet(json_data).expect("failed json to parquet");

    // Parquet -> BSON
    let bson_bytes = parquet_to_bson(&pq_bytes).expect("failed parquet to bson");
    assert!(!bson_bytes.is_empty());

    // BSON -> Parquet
    let roundtrip_pq = bson_to_parquet(&bson_bytes).expect("failed bson to parquet");
    assert_eq!(&roundtrip_pq[0..4], b"PAR1");
}

#[test]
fn test_default_registry_includes_parquet() {
    let registry = default_registry();
    let formats = registry.available_formats();
    assert!(
        formats.contains(&"parquet"),
        "registry should contain parquet"
    );

    let engine = registry
        .get_by_id("parquet")
        .expect("should get parquet by id");
    assert_eq!(engine.mime_type(), "application/vnd.apache.parquet");

    let engine_ext = registry
        .get_by_extension("parquet")
        .expect("should get by ext");
    assert_eq!(engine_ext.format_id(), "parquet");
}

#[test]
fn test_parquet_parser_and_emitter_traits() {
    let dataset = Value::Array(vec![Value::Object(vec![
        ("id".into(), Value::Integer(42)),
        ("label".into(), Value::String("Answer".into())),
    ])]);

    let mut buf = babbel::core::Buffer::new();
    ParquetEmitter
        .emit(&dataset, &mut buf)
        .expect("ParquetEmitter should emit");
    let bytes = buf.into_vec();
    assert_eq!(&bytes[0..4], b"PAR1");

    let decoded = ParquetParser
        .parse_bytes(&bytes)
        .expect("ParquetParser should parse");
    assert_eq!(decoded, dataset);
}
