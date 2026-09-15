use babbel_core::Value;
use babbel_ron::{from_str, to_string, to_string_pretty};

#[test]
#[allow(clippy::approx_constant)]
fn test_ron_primitives() {
    assert_eq!(from_str("()").unwrap(), Value::Null);
    assert_eq!(from_str("None").unwrap(), Value::Null);
    assert_eq!(from_str("Some(42)").unwrap(), Value::Integer(42));
    assert_eq!(from_str("true").unwrap(), Value::Bool(true));
    assert_eq!(from_str("false").unwrap(), Value::Bool(false));
    assert_eq!(from_str("123").unwrap(), Value::Integer(123));
    assert_eq!(from_str("-456").unwrap(), Value::Integer(-456));
    assert_eq!(from_str("0x10").unwrap(), Value::Integer(16));
    assert_eq!(from_str("0b1010").unwrap(), Value::Integer(10));
    assert_eq!(from_str("0o77").unwrap(), Value::Integer(63));
    assert_eq!(from_str("1_000_000").unwrap(), Value::Integer(1_000_000));
    assert_eq!(from_str("3.14").unwrap(), Value::Float(3.14));
    assert_eq!(from_str("\"hello world\"").unwrap(), Value::String("hello world".into()));
    assert_eq!(from_str("'x'").unwrap(), Value::String("x".into()));
}

#[test]
fn test_ron_nested_comments() {
    let input = r#"
    /* Outer block comment
       /* Nested block comment */
       Still in outer comment
    */
    (
        // Line comment
        field: "value", /* inline comment */
    )
    "#;
    let val = from_str(input).expect("Failed to parse nested comments");
    assert_eq!(
        val,
        Value::Object(vec![("field".into(), Value::String("value".into()))])
    );
}

#[test]
fn test_ron_struct_and_map() {
    let struct_input = r#"
    Config(
        window_width: 1920,
        window_height: 1080,
        vsync: true,
        title: "My Game",
    )
    "#;
    let struct_val = from_str(struct_input).expect("Failed to parse struct");
    assert_eq!(struct_val.get("window_width").and_then(|v| v.as_i64()), Some(1920));
    assert_eq!(struct_val.get("vsync").and_then(|v| v.as_bool()), Some(true));

    let map_input = r#"
    {
        "weapons": 5,
        "armors": 12,
    }
    "#;
    let map_val = from_str(map_input).expect("Failed to parse map");
    assert_eq!(map_val.get("weapons").and_then(|v| v.as_i64()), Some(5));
}

#[test]
fn test_ron_lists_and_tuples() {
    let list_input = "[1, 2, 3, 4,]";
    let list_val = from_str(list_input).expect("Failed to parse list with trailing comma");
    assert_eq!(
        list_val,
        Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ])
    );

    let tuple_input = "(10, \"foo\", true)";
    let tuple_val = from_str(tuple_input).expect("Failed to parse tuple");
    assert_eq!(
        tuple_val,
        Value::Array(vec![
            Value::Integer(10),
            Value::String("foo".into()),
            Value::Bool(true),
        ])
    );
}

#[test]
fn test_ron_raw_and_byte_strings() {
    let raw_input = r###"r#"C:\Program Files\Rust"#"###;
    assert_eq!(
        from_str(raw_input).unwrap(),
        Value::String(r"C:\Program Files\Rust".into())
    );

    let byte_input = r#"b"binary data""#;
    assert_eq!(
        from_str(byte_input).unwrap(),
        Value::Bytes(b"binary data".to_vec())
    );
}

#[test]
fn test_ron_roundtrip() {
    let original = Value::Object(vec![
        ("name".into(), Value::String("bevy_app".into())),
        ("version".into(), Value::Integer(1)),
        ("features".into(), Value::Array(vec![
            Value::String("2d".into()),
            Value::String("audio".into()),
        ])),
    ]);

    let serialized = to_string(&original).unwrap();
    let deserialized = from_str(&serialized).unwrap();
    assert_eq!(original, deserialized);

    let pretty = to_string_pretty(&original, 2).unwrap();
    let from_pretty = from_str(&pretty).unwrap();
    assert_eq!(original, from_pretty);
}
