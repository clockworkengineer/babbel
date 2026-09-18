use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_msgpack::{Decoder, DecoderConfig, MsgPackEngine, MsgPackError, from_bytes, to_vec};

#[test]
fn test_roundtrip_primitives() {
    // Null
    let val = Value::Null;
    let bytes = to_vec(&val).unwrap();
    assert_eq!(bytes, vec![0xc0]);
    assert_eq!(from_bytes(&bytes).unwrap(), val);

    // Bool
    let t = Value::Bool(true);
    let f = Value::Bool(false);
    assert_eq!(to_vec(&t).unwrap(), vec![0xc3]);
    assert_eq!(to_vec(&f).unwrap(), vec![0xc2]);
    assert_eq!(from_bytes(&[0xc3]).unwrap(), t);
    assert_eq!(from_bytes(&[0xc2]).unwrap(), f);
}

#[test]
fn test_roundtrip_integers() {
    // Positive fixint (0..=127)
    let small = Value::Integer(42);
    let bytes = to_vec(&small).unwrap();
    assert_eq!(bytes, vec![42]);
    assert_eq!(from_bytes(&bytes).unwrap(), small);

    // Negative fixint (-32..=-1)
    let neg_small = Value::Integer(-15);
    let bytes = to_vec(&neg_small).unwrap();
    assert_eq!(bytes, vec![(-15i8) as u8]);
    assert_eq!(from_bytes(&bytes).unwrap(), neg_small);

    // uint 8
    let u8_val = Value::Integer(200);
    let bytes = to_vec(&u8_val).unwrap();
    assert_eq!(bytes, vec![0xcc, 200]);
    assert_eq!(from_bytes(&bytes).unwrap(), u8_val);

    // uint 16
    let u16_val = Value::Integer(5000);
    let bytes = to_vec(&u16_val).unwrap();
    assert_eq!(bytes, vec![0xcd, 0x13, 0x88]);
    assert_eq!(from_bytes(&bytes).unwrap(), u16_val);

    // uint 32
    let u32_val = Value::Integer(100_000);
    let bytes = to_vec(&u32_val).unwrap();
    assert_eq!(bytes, vec![0xce, 0x00, 0x01, 0x86, 0xa0]);
    assert_eq!(from_bytes(&bytes).unwrap(), u32_val);

    // uint 64
    let u64_val = Value::Integer(10_000_000_000);
    let bytes = to_vec(&u64_val).unwrap();
    assert_eq!(bytes[0], 0xcf);
    assert_eq!(from_bytes(&bytes).unwrap(), u64_val);

    // int 8
    let i8_val = Value::Integer(-100);
    let bytes = to_vec(&i8_val).unwrap();
    assert_eq!(bytes, vec![0xd0, (-100i8) as u8]);
    assert_eq!(from_bytes(&bytes).unwrap(), i8_val);

    // int 16
    let i16_val = Value::Integer(-5000);
    let bytes = to_vec(&i16_val).unwrap();
    assert_eq!(bytes[0], 0xd1);
    assert_eq!(from_bytes(&bytes).unwrap(), i16_val);

    // int 32
    let i32_val = Value::Integer(-100_000);
    let bytes = to_vec(&i32_val).unwrap();
    assert_eq!(bytes[0], 0xd2);
    assert_eq!(from_bytes(&bytes).unwrap(), i32_val);

    // int 64
    let i64_val = Value::Integer(-10_000_000_000);
    let bytes = to_vec(&i64_val).unwrap();
    assert_eq!(bytes[0], 0xd3);
    assert_eq!(from_bytes(&bytes).unwrap(), i64_val);
}

#[test]
fn test_roundtrip_floats() {
    let pi = Value::Float(3.141592653589793);
    let bytes = to_vec(&pi).unwrap();
    assert_eq!(bytes[0], 0xcb);
    let decoded = from_bytes(&bytes).unwrap();
    if let Value::Float(f) = decoded {
        assert!((f - 3.141592653589793).abs() < 1e-12);
    } else {
        panic!("expected float");
    }
}

#[test]
fn test_roundtrip_strings() {
    // FixStr
    let short = Value::String("hello".into());
    let bytes = to_vec(&short).unwrap();
    assert_eq!(bytes[0], 0xa0 | 5);
    assert_eq!(&bytes[1..], b"hello");
    assert_eq!(from_bytes(&bytes).unwrap(), short);

    // Str 8
    let medium_str = "A".repeat(100);
    let med = Value::String(medium_str);
    let bytes = to_vec(&med).unwrap();
    assert_eq!(bytes[0], 0xd9);
    assert_eq!(bytes[1], 100);
    assert_eq!(from_bytes(&bytes).unwrap(), med);

    // Str 16
    let large_str = "B".repeat(1000);
    let lg = Value::String(large_str);
    let bytes = to_vec(&lg).unwrap();
    assert_eq!(bytes[0], 0xda);
    assert_eq!(from_bytes(&bytes).unwrap(), lg);
}

#[test]
fn test_roundtrip_binary() {
    let payload = vec![0xde, 0xad, 0xbe, 0xef, 0x00, 0xff];
    let bin = Value::Bytes(payload.clone());
    let bytes = to_vec(&bin).unwrap();
    assert_eq!(bytes[0], 0xc4);
    assert_eq!(bytes[1], payload.len() as u8);
    assert_eq!(&bytes[2..], payload.as_slice());
    assert_eq!(from_bytes(&bytes).unwrap(), bin);
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
    assert_eq!(bytes[0], 0x90 | 4); // fixarray of 4
    assert_eq!(from_bytes(&bytes).unwrap(), arr);

    // Larger array (> 15 items -> array16)
    let big_arr = Value::Array((0..20).map(Value::Integer).collect());
    let bytes = to_vec(&big_arr).unwrap();
    assert_eq!(bytes[0], 0xdc);
    assert_eq!(from_bytes(&bytes).unwrap(), big_arr);
}

#[test]
fn test_roundtrip_map() {
    let map = Value::Object(vec![
        ("name".into(), Value::String("Babbel".into())),
        ("version".into(), Value::String("0.2.1".into())),
        ("speed".into(), Value::Integer(100)),
        ("active".into(), Value::Bool(true)),
    ]);
    let bytes = to_vec(&map).unwrap();
    assert_eq!(bytes[0], 0x80 | 4); // fixmap of 4
    assert_eq!(from_bytes(&bytes).unwrap(), map);

    // Larger map (> 15 entries -> map16)
    let big_entries: Vec<(String, Value)> = (0..20)
        .map(|i| (format!("key_{}", i), Value::Integer(i as i128)))
        .collect();
    let big_map = Value::Object(big_entries);
    let bytes = to_vec(&big_map).unwrap();
    assert_eq!(bytes[0], 0xde);
    assert_eq!(from_bytes(&bytes).unwrap(), big_map);
}

#[test]
fn test_nested_structures() {
    let complex = Value::Object(vec![
        (
            "users".into(),
            Value::Array(vec![
                Value::Object(vec![
                    ("id".into(), Value::Integer(1)),
                    ("name".into(), Value::String("Alice".into())),
                    (
                        "roles".into(),
                        Value::Array(vec![
                            Value::String("admin".into()),
                            Value::String("dev".into()),
                        ]),
                    ),
                ]),
                Value::Object(vec![
                    ("id".into(), Value::Integer(2)),
                    ("name".into(), Value::String("Bob".into())),
                    ("roles".into(), Value::Array(vec![])),
                ]),
            ]),
        ),
        (
            "meta".into(),
            Value::Object(vec![
                ("count".into(), Value::Integer(2)),
                ("raw".into(), Value::Bytes(vec![1, 2, 3, 4])),
            ]),
        ),
    ]);

    let bytes = to_vec(&complex).unwrap();
    let decoded = from_bytes(&bytes).unwrap();
    assert_eq!(complex, decoded);
}

#[test]
fn test_recursion_limit() {
    // Build deeply nested array: [[[[...]]]]
    let mut curr = Value::Integer(1);
    for _ in 0..10 {
        curr = Value::Array(vec![curr]);
    }
    let bytes = to_vec(&curr).unwrap();

    // With depth limit of 5, should fail
    let config = DecoderConfig {
        max_depth: 5,
        max_size: 1024 * 1024,
    };
    let mut decoder = Decoder::with_config(&bytes, config);
    let err = decoder.decode_value().unwrap_err();
    assert!(matches!(err, MsgPackError::RecursionLimitExceeded(_)));
}

#[test]
fn test_size_limit() {
    let payload = vec![0xc4, 0x10]; // bin8 with length 16, but limit is 8
    let config = DecoderConfig {
        max_depth: 10,
        max_size: 8,
    };
    let mut decoder = Decoder::with_config(&payload, config);
    let err = decoder.decode_value().unwrap_err();
    assert!(matches!(err, MsgPackError::SizeLimitExceeded { .. }));
}

#[test]
fn test_unexpected_eof() {
    // uint 16 marker without 2 bytes
    let err = from_bytes(&[0xcd, 0x01]).unwrap_err();
    assert!(matches!(err, MsgPackError::UnexpectedEof { .. }));

    // fixstr with declared length 5 but only 3 bytes
    let err = from_bytes(&[0xa5, b'f', b'o', b'o']).unwrap_err();
    assert!(matches!(err, MsgPackError::UnexpectedEof { .. }));
}

#[test]
fn test_invalid_marker() {
    let err = from_bytes(&[0xc1]).unwrap_err();
    assert!(matches!(err, MsgPackError::InvalidMarker(0xc1)));
}

#[test]
fn test_format_engine_integration() {
    let engine = MsgPackEngine;
    assert_eq!(engine.format_id(), "msgpack");
    assert_eq!(engine.mime_type(), "application/msgpack");
    assert_eq!(engine.file_extensions(), &["msgpack", "mp"]);

    let val = Value::Object(vec![
        ("item".into(), Value::String("rust".into())),
        ("price".into(), Value::Integer(99)),
    ]);

    let bytes = engine
        .serialize_to_vec(&val, &FormatOptions::compact())
        .unwrap();
    let parsed = engine.parse_bytes(&bytes).unwrap();
    assert_eq!(val, parsed);
}
