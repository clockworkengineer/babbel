use babbel_core::Value;
use babbel_parquet::{read_parquet, write_parquet, ParquetError};
use babbel_parquet::thrift::{ThriftReader, ThriftWriter, TYPE_I32, TYPE_STOP};

#[test]
fn test_thrift_codec_roundtrip() {
    let mut w = ThriftWriter::new();
    w.write_struct_begin();
    w.write_field_header(1, TYPE_I32);
    w.write_i32(42);
    w.write_field_header(2, TYPE_I32);
    w.write_i64(1000000);
    w.write_field_header(3, babbel_parquet::thrift::TYPE_BINARY);
    w.write_string("hello thrift");
    w.write_bool_field(4, true);
    w.write_struct_end();

    let bytes = w.into_bytes();
    let mut r = ThriftReader::new(&bytes);
    r.read_struct_begin();

    let (f1, _t1) = r.read_field_header().unwrap();
    assert_eq!(f1, 1);
    assert_eq!(r.read_i32().unwrap(), 42);

    let (f2, _) = r.read_field_header().unwrap();
    assert_eq!(f2, 2);
    assert_eq!(r.read_i64().unwrap(), 1000000);

    let (f3, _) = r.read_field_header().unwrap();
    assert_eq!(f3, 3);
    assert_eq!(r.read_string().unwrap(), "hello thrift");

    let (f4, t4) = r.read_field_header().unwrap();
    assert_eq!(f4, 4);
    assert_eq!(t4, babbel_parquet::thrift::TYPE_BOOLEAN_TRUE);

    let (_, stop) = r.read_field_header().unwrap();
    assert_eq!(stop, TYPE_STOP);
}

#[test]
fn test_typed_columns_roundtrip() {
    let dataset = Value::Array(vec![
        Value::Object(vec![
            ("id".into(), Value::Integer(1)),
            ("title".into(), Value::String("Alpha".into())),
            ("score".into(), Value::Float(88.5)),
            ("verified".into(), Value::Bool(true)),
        ]),
        Value::Object(vec![
            ("id".into(), Value::Integer(2)),
            ("title".into(), Value::String("Beta".into())),
            ("score".into(), Value::Float(92.0)),
            ("verified".into(), Value::Bool(false)),
        ]),
        Value::Object(vec![
            ("id".into(), Value::Integer(3)),
            ("title".into(), Value::String("Gamma".into())),
            ("score".into(), Value::Float(76.25)),
            ("verified".into(), Value::Bool(true)),
        ]),
    ]);

    let bytes = write_parquet(&dataset).expect("failed to write parquet");
    assert_eq!(&bytes[0..4], b"PAR1");
    assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");

    let decoded = read_parquet(&bytes).expect("failed to read parquet");
    assert_eq!(decoded, dataset);
}

#[test]
fn test_nullable_columns_roundtrip() {
    let dataset = Value::Array(vec![
        Value::Object(vec![
            ("id".into(), Value::Integer(1)),
            ("comment".into(), Value::String("First".into())),
        ]),
        Value::Object(vec![
            ("id".into(), Value::Integer(2)),
            ("comment".into(), Value::Null),
        ]),
        Value::Object(vec![
            ("id".into(), Value::Integer(3)),
            ("comment".into(), Value::String("Third".into())),
        ]),
    ]);

    let bytes = write_parquet(&dataset).expect("failed to write nullable parquet");
    let decoded = read_parquet(&bytes).expect("failed to read nullable parquet");
    assert_eq!(decoded, dataset);
}

#[test]
fn test_invalid_magic_error() {
    let corrupted = b"NOT_A_PARQUET_FILE";
    let res = read_parquet(corrupted);
    assert_eq!(res.unwrap_err(), ParquetError::InvalidMagic);
}

#[test]
fn test_large_dataset_roundtrip() {
    let mut rows = Vec::new();
    for i in 0..100 {
        rows.push(Value::Object(vec![
            ("index".into(), Value::Integer(i as i128)),
            ("label".into(), Value::String(format!("row-{}", i))),
            ("value".into(), Value::Float(i as f64 * 1.5)),
            ("even".into(), Value::Bool(i % 2 == 0)),
        ]));
    }
    let dataset = Value::Array(rows);

    let bytes = write_parquet(&dataset).expect("failed to write 100-row parquet");
    let decoded = read_parquet(&bytes).expect("failed to read 100-row parquet");
    assert_eq!(decoded, dataset);
}
