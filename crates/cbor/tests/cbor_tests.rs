use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_cbor::{from_bytes, to_vec, CborEngine, CborError, Decoder, DecoderConfig};

#[test]
fn test_roundtrip_primitives() {
    // Null
    let val = Value::Null;
    let bytes = to_vec(&val).unwrap();
    assert_eq!(bytes, vec![0xf6]);
    assert_eq!(from_bytes(&bytes).unwrap(), val);

    // Bool
    let t = Value::Bool(true);
    let f = Value::Bool(false);
    assert_eq!(to_vec(&t).unwrap(), vec![0xf5]);
    assert_eq!(to_vec(&f).unwrap(), vec![0xf4]);
    assert_eq!(from_bytes(&[0xf5]).unwrap(), t);
    assert_eq!(from_bytes(&[0xf4]).unwrap(), f);
}

#[test]
fn test_roundtrip_integers_rfc8949() {
    // 0 -> 0x00
    assert_eq!(to_vec(&Value::Integer(0)).unwrap(), vec![0x00]);
    assert_eq!(from_bytes(&[0x00]).unwrap(), Value::Integer(0));

    // 1 -> 0x01
    assert_eq!(to_vec(&Value::Integer(1)).unwrap(), vec![0x01]);
    assert_eq!(from_bytes(&[0x01]).unwrap(), Value::Integer(1));

    // 10 -> 0x0a
    assert_eq!(to_vec(&Value::Integer(10)).unwrap(), vec![0x0a]);
    assert_eq!(from_bytes(&[0x0a]).unwrap(), Value::Integer(10));

    // 23 -> 0x17 (max direct unsigned)
    assert_eq!(to_vec(&Value::Integer(23)).unwrap(), vec![0x17]);
    assert_eq!(from_bytes(&[0x17]).unwrap(), Value::Integer(23));

    // 24 -> 0x18, 0x18
    assert_eq!(to_vec(&Value::Integer(24)).unwrap(), vec![0x18, 0x18]);
    assert_eq!(from_bytes(&[0x18, 0x18]).unwrap(), Value::Integer(24));

    // 25 -> 0x18, 0x19
    assert_eq!(to_vec(&Value::Integer(25)).unwrap(), vec![0x18, 0x19]);
    assert_eq!(from_bytes(&[0x18, 0x19]).unwrap(), Value::Integer(25));

    // 100 -> 0x18, 0x64
    assert_eq!(to_vec(&Value::Integer(100)).unwrap(), vec![0x18, 0x64]);
    assert_eq!(from_bytes(&[0x18, 0x64]).unwrap(), Value::Integer(100));

    // 1000 -> 0x19, 0x03, 0xe8
    assert_eq!(to_vec(&Value::Integer(1000)).unwrap(), vec![0x19, 0x03, 0xe8]);
    assert_eq!(from_bytes(&[0x19, 0x03, 0xe8]).unwrap(), Value::Integer(1000));

    // 1000000 -> 0x1a, 0x00, 0x0f, 0x42, 0x40
    assert_eq!(to_vec(&Value::Integer(1000000)).unwrap(), vec![0x1a, 0x00, 0x0f, 0x42, 0x40]);
    assert_eq!(from_bytes(&[0x1a, 0x00, 0x0f, 0x42, 0x40]).unwrap(), Value::Integer(1000000));

    // 1000000000000 -> 0x1b, ...
    let big_val = Value::Integer(1000000000000);
    let bytes = to_vec(&big_val).unwrap();
    assert_eq!(bytes[0], 0x1b);
    assert_eq!(from_bytes(&bytes).unwrap(), big_val);
}

#[test]
fn test_negative_integers_rfc8949() {
    // -1 -> 0x20
    assert_eq!(to_vec(&Value::Integer(-1)).unwrap(), vec![0x20]);
    assert_eq!(from_bytes(&[0x20]).unwrap(), Value::Integer(-1));

    // -10 -> 0x29
    assert_eq!(to_vec(&Value::Integer(-10)).unwrap(), vec![0x29]);
    assert_eq!(from_bytes(&[0x29]).unwrap(), Value::Integer(-10));

    // -24 -> 0x37
    assert_eq!(to_vec(&Value::Integer(-24)).unwrap(), vec![0x37]);
    assert_eq!(from_bytes(&[0x37]).unwrap(), Value::Integer(-24));

    // -25 -> 0x38, 0x18
    assert_eq!(to_vec(&Value::Integer(-25)).unwrap(), vec![0x38, 0x18]);
    assert_eq!(from_bytes(&[0x38, 0x18]).unwrap(), Value::Integer(-25));

    // -100 -> 0x38, 0x63
    assert_eq!(to_vec(&Value::Integer(-100)).unwrap(), vec![0x38, 0x63]);
    assert_eq!(from_bytes(&[0x38, 0x63]).unwrap(), Value::Integer(-100));

    // -1000 -> 0x39, 0x03, 0xe7
    assert_eq!(to_vec(&Value::Integer(-1000)).unwrap(), vec![0x39, 0x03, 0xe7]);
    assert_eq!(from_bytes(&[0x39, 0x03, 0xe7]).unwrap(), Value::Integer(-1000));
}

#[test]
fn test_roundtrip_floats() {
    let pi = Value::Float(3.141592653589793);
    let bytes = to_vec(&pi).unwrap();
    assert_eq!(bytes[0], 0xfb);
    let decoded = from_bytes(&bytes).unwrap();
    if let Value::Float(f) = decoded {
        assert!((f - 3.141592653589793).abs() < 1e-12);
    } else {
        panic!("expected float");
    }

    // Decode RFC 8949 half float 1.5 (0xf9, 0x3e, 0x00)
    let half_val = from_bytes(&[0xf9, 0x3e, 0x00]).unwrap();
    assert_eq!(half_val, Value::Float(1.5));

    // Decode RFC 8949 single float 100000.0 (0xfa, 0x47, 0xc3, 0x50, 0x00)
    let single_val = from_bytes(&[0xfa, 0x47, 0xc3, 0x50, 0x00]).unwrap();
    assert_eq!(single_val, Value::Float(100000.0));
}

#[test]
fn test_roundtrip_strings_rfc8949() {
    // Empty string -> 0x60
    assert_eq!(to_vec(&Value::String("".into())).unwrap(), vec![0x60]);
    assert_eq!(from_bytes(&[0x60]).unwrap(), Value::String("".into()));

    // "a" -> 0x61, 0x61
    assert_eq!(to_vec(&Value::String("a".into())).unwrap(), vec![0x61, 0x61]);
    assert_eq!(from_bytes(&[0x61, 0x61]).unwrap(), Value::String("a".into()));

    // "IETF" -> 0x64, 'I', 'E', 'T', 'F'
    let s = Value::String("IETF".into());
    let bytes = to_vec(&s).unwrap();
    assert_eq!(bytes, vec![0x64, b'I', b'E', b'T', b'F']);
    assert_eq!(from_bytes(&bytes).unwrap(), s);

    // Medium string (length 100) -> 0x78, 100, ...
    let med_s = Value::String("x".repeat(100));
    let bytes = to_vec(&med_s).unwrap();
    assert_eq!(bytes[0], 0x78);
    assert_eq!(bytes[1], 100);
    assert_eq!(from_bytes(&bytes).unwrap(), med_s);
}

#[test]
fn test_roundtrip_bytes_rfc8949() {
    // Empty byte string -> 0x40
    assert_eq!(to_vec(&Value::Bytes(vec![])).unwrap(), vec![0x40]);
    assert_eq!(from_bytes(&[0x40]).unwrap(), Value::Bytes(vec![]));

    // 4 bytes -> 0x44, ...
    let bin = Value::Bytes(vec![1, 2, 3, 4]);
    let bytes = to_vec(&bin).unwrap();
    assert_eq!(bytes, vec![0x44, 1, 2, 3, 4]);
    assert_eq!(from_bytes(&bytes).unwrap(), bin);
}

#[test]
fn test_roundtrip_arrays_rfc8949() {
    // Empty array -> 0x80
    assert_eq!(to_vec(&Value::Array(vec![])).unwrap(), vec![0x80]);
    assert_eq!(from_bytes(&[0x80]).unwrap(), Value::Array(vec![]));

    // [1, 2, 3] -> 0x83, 0x01, 0x02, 0x03
    let arr = Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]);
    assert_eq!(to_vec(&arr).unwrap(), vec![0x83, 0x01, 0x02, 0x03]);
    assert_eq!(from_bytes(&[0x83, 0x01, 0x02, 0x03]).unwrap(), arr);

    // [1, [2, 3], [4, 5]] -> 0x83, 0x01, 0x82, 0x02, 0x03, 0x82, 0x04, 0x05
    let nested = Value::Array(vec![
        Value::Integer(1),
        Value::Array(vec![Value::Integer(2), Value::Integer(3)]),
        Value::Array(vec![Value::Integer(4), Value::Integer(5)]),
    ]);
    let bytes = to_vec(&nested).unwrap();
    assert_eq!(bytes, vec![0x83, 0x01, 0x82, 0x02, 0x03, 0x82, 0x04, 0x05]);
    assert_eq!(from_bytes(&bytes).unwrap(), nested);
}

#[test]
fn test_roundtrip_maps_rfc8949() {
    // Empty map -> 0xa0
    assert_eq!(to_vec(&Value::Object(vec![])).unwrap(), vec![0xa0]);
    assert_eq!(from_bytes(&[0xa0]).unwrap(), Value::Object(vec![]));

    // {"a": 1, "b": [2, 3]} -> 0xa2, 0x61, 0x61, 0x01, 0x61, 0x62, 0x82, 0x02, 0x03
    let map = Value::Object(vec![
        ("a".into(), Value::Integer(1)),
        ("b".into(), Value::Array(vec![Value::Integer(2), Value::Integer(3)])),
    ]);
    let bytes = to_vec(&map).unwrap();
    assert_eq!(bytes, vec![0xa2, 0x61, 0x61, 0x01, 0x61, 0x62, 0x82, 0x02, 0x03]);
    assert_eq!(from_bytes(&bytes).unwrap(), map);
}

#[test]
fn test_indefinite_containers() {
    // Indefinite array: 0x9f, 0x01, 0x02, 0xff -> [1, 2]
    let arr = from_bytes(&[0x9f, 0x01, 0x02, 0xff]).unwrap();
    assert_eq!(arr, Value::Array(vec![Value::Integer(1), Value::Integer(2)]));

    // Indefinite map: 0xbf, 0x61, 0x61, 0x01, 0xff -> {"a": 1}
    let map = from_bytes(&[0xbf, 0x61, 0x61, 0x01, 0xff]).unwrap();
    assert_eq!(map, Value::Object(vec![("a".into(), Value::Integer(1))]));

    // Indefinite text string: 0x7f, 0x62, b's', b't', 0x62, b'r', b'!', 0xff -> "str!"
    let s = from_bytes(&[0x7f, 0x62, b's', b't', 0x62, b'r', b'!', 0xff]).unwrap();
    assert_eq!(s, Value::String("str!".into()));

    // Indefinite byte string: 0x5f, 0x42, 1, 2, 0x41, 3, 0xff -> [1, 2, 3]
    let b = from_bytes(&[0x5f, 0x42, 1, 2, 0x41, 3, 0xff]).unwrap();
    assert_eq!(b, Value::Bytes(vec![1, 2, 3]));
}

#[test]
fn test_tagged_values() {
    // Tag 0 (standard datetime string): 0xc0, 0x64, 't', 'e', 's', 't'
    let tagged = from_bytes(&[0xc0, 0x64, b't', b'e', b's', b't']).unwrap();
    assert_eq!(tagged, Value::String("test".into()));
}

#[test]
fn test_recursion_limit() {
    let mut curr = Value::Integer(1);
    for _ in 0..10 {
        curr = Value::Array(vec![curr]);
    }
    let bytes = to_vec(&curr).unwrap();

    let config = DecoderConfig {
        max_depth: 5,
        max_size: 1024 * 1024,
    };
    let mut decoder = Decoder::with_config(&bytes, config);
    let err = decoder.decode_value().unwrap_err();
    assert!(matches!(err, CborError::RecursionLimitExceeded(_)));
}

#[test]
fn test_size_limit() {
    let payload = vec![0x58, 0x20]; // byte string with length 32, but limit is 10
    let config = DecoderConfig {
        max_depth: 10,
        max_size: 10,
    };
    let mut decoder = Decoder::with_config(&payload, config);
    let err = decoder.decode_value().unwrap_err();
    assert!(matches!(err, CborError::SizeLimitExceeded { .. }));
}

#[test]
fn test_unexpected_eof() {
    // 2-byte integer marker without 2 bytes
    let err = from_bytes(&[0x19, 0x01]).unwrap_err();
    assert!(matches!(err, CborError::UnexpectedEof { .. }));
}

#[test]
fn test_format_engine_integration() {
    let engine = CborEngine;
    assert_eq!(engine.format_id(), "cbor");
    assert_eq!(engine.mime_type(), "application/cbor");
    assert_eq!(engine.file_extensions(), &["cbor"]);

    let val = Value::Object(vec![
        ("item".into(), Value::String("rust".into())),
        ("price".into(), Value::Integer(99)),
    ]);

    let bytes = engine.serialize_to_vec(&val, &FormatOptions::compact()).unwrap();
    let parsed = engine.parse_bytes(&bytes).unwrap();
    assert_eq!(val, parsed);
}

#[test]
fn test_cbor_pull_parser() {
    use babbel_cbor::{CborPullEvent, CborPullParser};

    // [1, "hello", true]
    let bytes = to_vec(&Value::Array(vec![
        Value::Integer(1),
        Value::String("hello".into()),
        Value::Bool(true),
    ])).unwrap();

    let mut parser = CborPullParser::new(&bytes);
    assert_eq!(parser.next_event().unwrap(), CborPullEvent::ArrayStart(Some(3)));
    assert_eq!(parser.next_event().unwrap(), CborPullEvent::Unsigned(1));
    assert_eq!(parser.next_event().unwrap(), CborPullEvent::TextString { len: Some(5), text: "hello" });
    assert_eq!(parser.next_event().unwrap(), CborPullEvent::Simple(21));
    assert_eq!(parser.next_event().unwrap(), CborPullEvent::End);
}

#[test]
fn test_cbor_edn() {
    use babbel_cbor::{from_edn, to_edn};

    let val = Value::Object(vec![
        ("a".into(), Value::Integer(42)),
        ("b".into(), Value::Bytes(vec![0x01, 0x02, 0x03])),
        ("c".into(), Value::Bool(true)),
    ]);

    let edn = to_edn(&val);
    assert!(edn.contains("42"));
    assert!(edn.contains("h'010203'"));
    assert!(edn.contains("true"));

    let parsed_bytes = from_edn("h'010203'").unwrap();
    assert_eq!(parsed_bytes, Value::Bytes(vec![0x01, 0x02, 0x03]));

    let parsed_int = from_edn("100").unwrap();
    assert_eq!(parsed_int, Value::Integer(100));
}

