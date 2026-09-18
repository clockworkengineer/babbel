//! Integration tests for HCL and Avro engines in Babbel.

use babbel::FormatEmitter;
use babbel::core::Value;
use babbel::default_registry;

#[test]
fn test_default_registry_includes_hcl_and_avro() {
    let registry = default_registry();
    assert!(registry.get("hcl").is_some());
    assert!(registry.get("avro").is_some());
    assert!(registry.get_by_extension("tf").is_some());
    assert!(registry.get_by_extension("avro").is_some());
}

#[test]
fn test_hcl_to_json_and_yaml_conversion() {
    let hcl_input = r#"
        resource "aws_instance" "server" {
            ami           = "ami-123456"
            instance_type = "t3.micro"
            count         = 2
            tags = ["production", "web"]
        }
    "#;

    let registry = default_registry();
    let hcl_engine = registry.get("hcl").unwrap();
    let json_engine = registry.get("json").unwrap();
    let yaml_engine = registry.get("yaml").unwrap();

    let val = hcl_engine.parse_str(hcl_input).unwrap();
    assert!(matches!(val, Value::Object(_)));

    let mut json_dest = babbel::core::io::Buffer::new();
    json_engine.emit(&val, &mut json_dest).unwrap();
    let json_str = json_dest.to_string();
    assert!(json_str.contains("ami-123456"));

    let mut yaml_dest = babbel::core::io::Buffer::new();
    yaml_engine.emit(&val, &mut yaml_dest).unwrap();
    let yaml_str = yaml_dest.to_string();
    assert!(yaml_str.contains("instance_type: t3.micro"));
}

#[test]
fn test_avro_to_cbor_and_msgpack_conversion() {
    let original = Value::Object(vec![
        ("sensor_id".into(), Value::Integer(98765)),
        ("reading".into(), Value::Float(23.75)),
        ("active".into(), Value::Bool(true)),
    ]);

    let registry = default_registry();
    let avro_engine = registry.get("avro").unwrap();
    let cbor_engine = registry.get("cbor").unwrap();
    let msgpack_engine = registry.get("msgpack").unwrap();

    // Avro binary serialization
    let mut avro_dest = babbel::core::io::Buffer::new();
    avro_engine.emit(&original, &mut avro_dest).unwrap();
    let avro_bytes = avro_dest.into_vec();

    // Avro parse
    let parsed_avro = avro_engine.parse_bytes(&avro_bytes).unwrap();
    assert_eq!(
        parsed_avro.get("sensor_id").and_then(|v| v.as_i64()),
        Some(98765)
    );

    // Convert to CBOR
    let mut cbor_dest = babbel::core::io::Buffer::new();
    cbor_engine.emit(&parsed_avro, &mut cbor_dest).unwrap();
    let cbor_bytes = cbor_dest.into_vec();
    let parsed_cbor = cbor_engine.parse_bytes(&cbor_bytes).unwrap();
    assert_eq!(
        parsed_cbor.get("sensor_id").and_then(|v| v.as_i64()),
        Some(98765)
    );

    // Convert to MsgPack
    let mut msgpack_dest = babbel::core::io::Buffer::new();
    msgpack_engine
        .emit(&parsed_cbor, &mut msgpack_dest)
        .unwrap();
    let msgpack_bytes = msgpack_dest.into_vec();
    let parsed_msgpack = msgpack_engine.parse_bytes(&msgpack_bytes).unwrap();
    assert_eq!(
        parsed_msgpack.get("sensor_id").and_then(|v| v.as_i64()),
        Some(98765)
    );
}
