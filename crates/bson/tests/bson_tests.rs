use babbel_bson::{BsonEngine, BsonError, Decoder, DecoderConfig, from_bytes, to_vec};
use babbel_core::{FormatEngine, FormatOptions, Value};

#[test]
fn test_roundtrip_document_primitives() {
    let doc = Value::Object(vec![
        ("null_val".into(), Value::Null),
        ("bool_true".into(), Value::Bool(true)),
        ("bool_false".into(), Value::Bool(false)),
        ("int32_val".into(), Value::Integer(12345)),
        ("int64_val".into(), Value::Integer(9_000_000_000)),
        ("double_val".into(), Value::Float(3.14159)),
        ("str_val".into(), Value::String("hello bson".into())),
    ]);

    let bytes = to_vec(&doc).unwrap();
    let decoded = from_bytes(&bytes).unwrap();
    assert_eq!(doc, decoded);
}

#[test]
fn test_roundtrip_binary_data() {
    let payload = vec![0xca, 0xfe, 0xba, 0xbe, 0x00, 0x01, 0x02];
    let doc = Value::Object(vec![("data".into(), Value::Bytes(payload.clone()))]);

    let bytes = to_vec(&doc).unwrap();
    let decoded = from_bytes(&bytes).unwrap();
    assert_eq!(doc, decoded);
}

#[test]
fn test_roundtrip_array() {
    let arr = Value::Array(vec![
        Value::Integer(1),
        Value::String("two".into()),
        Value::Bool(true),
        Value::Null,
    ]);

    let bytes = to_vec(&arr).unwrap();
    let decoded = from_bytes(&bytes).unwrap();
    assert_eq!(arr, decoded);
}

#[test]
fn test_roundtrip_nested_document() {
    let doc = Value::Object(vec![
        ("service".into(), Value::String("database".into())),
        (
            "config".into(),
            Value::Object(vec![
                ("host".into(), Value::String("127.0.0.1".into())),
                ("port".into(), Value::Integer(27017)),
                (
                    "shards".into(),
                    Value::Array(vec![Value::String("s1".into()), Value::String("s2".into())]),
                ),
            ]),
        ),
    ]);

    let bytes = to_vec(&doc).unwrap();
    let decoded = from_bytes(&bytes).unwrap();
    assert_eq!(doc, decoded);
}

#[test]
fn test_primitive_scalar_wrapping() {
    // When a single scalar is serialized to BSON, it is wrapped into {"value": ...}
    let scalar = Value::String("standalone".into());
    let bytes = to_vec(&scalar).unwrap();
    let decoded = from_bytes(&bytes).unwrap();

    let expected = Value::Object(vec![("value".into(), Value::String("standalone".into()))]);
    assert_eq!(decoded, expected);
}

#[test]
fn test_invalid_doc_length() {
    // Length is 3 bytes (must be >= 5)
    let bytes = vec![0x03, 0x00, 0x00, 0x00, 0x00];
    let err = from_bytes(&bytes).unwrap_err();
    assert!(matches!(err, BsonError::InvalidDocumentLength { .. }));
}

#[test]
fn test_unexpected_eof() {
    // Header claims 20 bytes, but slice only has 6
    let bytes = vec![0x14, 0x00, 0x00, 0x00, 0x01, 0x00];
    let err = from_bytes(&bytes).unwrap_err();
    assert!(matches!(err, BsonError::InvalidDocumentLength { .. }));
}

#[test]
fn test_recursion_limit() {
    let mut curr = Value::Object(vec![("key".into(), Value::Integer(1))]);
    for _ in 0..10 {
        curr = Value::Object(vec![("nested".into(), curr)]);
    }
    let bytes = to_vec(&curr).unwrap();

    let config = DecoderConfig {
        max_depth: 5,
        max_size: 1024 * 1024,
    };
    let mut decoder = Decoder::with_config(&bytes, config);
    let err = decoder.decode_document().unwrap_err();
    assert!(matches!(err, BsonError::RecursionLimitExceeded(_)));
}

#[test]
fn test_size_limit() {
    let doc = Value::Object(vec![("large".into(), Value::String("A".repeat(50)))]);
    let bytes = to_vec(&doc).unwrap();

    let config = DecoderConfig {
        max_depth: 10,
        max_size: 20, // less than document size
    };
    let mut decoder = Decoder::with_config(&bytes, config);
    let err = decoder.decode_document().unwrap_err();
    assert!(matches!(err, BsonError::SizeLimitExceeded { .. }));
}

#[test]
fn test_format_engine_integration() {
    let engine = BsonEngine;
    assert_eq!(engine.format_id(), "bson");
    assert_eq!(engine.mime_type(), "application/bson");
    assert_eq!(engine.file_extensions(), &["bson"]);

    let val = Value::Object(vec![
        ("app".into(), Value::String("babbel".into())),
        ("version".into(), Value::Integer(2)),
    ]);

    let bytes = engine
        .serialize_to_vec(&val, &FormatOptions::compact())
        .unwrap();
    let parsed = engine.parse_bytes(&bytes).unwrap();
    assert_eq!(val, parsed);
}
