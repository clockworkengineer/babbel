//! # Parquet Read and Write Example
//!
//! Demonstrates constructing a tabular dataset with `Value::Array`,
//! serializing to standard Apache Parquet (`.parquet`) binary bytes,
//! inspecting the file structure and magic bytes, and deserializing back.

use babbel_core::Value;
use babbel_parquet::{read_parquet, write_parquet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel Parquet Read & Write Example ===\n");

    // 1. Construct tabular dataset
    let dataset = Value::Array(vec![
        Value::Object(vec![
            ("station_id".into(), Value::Integer(1001)),
            ("city".into(), Value::String("Stockholm".into())),
            ("temperature_c".into(), Value::Float(12.4)),
            ("humidity_pct".into(), Value::Float(68.0)),
            ("online".into(), Value::Bool(true)),
        ]),
        Value::Object(vec![
            ("station_id".into(), Value::Integer(1002)),
            ("city".into(), Value::String("Oslo".into())),
            ("temperature_c".into(), Value::Float(11.8)),
            ("humidity_pct".into(), Value::Float(72.5)),
            ("online".into(), Value::Bool(true)),
        ]),
        Value::Object(vec![
            ("station_id".into(), Value::Integer(1003)),
            ("city".into(), Value::String("Helsinki".into())),
            ("temperature_c".into(), Value::Float(9.5)),
            ("humidity_pct".into(), Value::Float(81.0)),
            ("online".into(), Value::Bool(false)),
        ]),
    ]);

    println!(
        "--- 1. Input Tabular Dataset ({} rows) ---",
        dataset.as_array().unwrap().len()
    );
    for (i, row) in dataset.as_array().unwrap().iter().enumerate() {
        println!("Row {}: {:?}", i, row);
    }

    // 2. Serialize to Apache Parquet binary bytes
    println!("\n--- 2. Serializing to Apache Parquet ---");
    let parquet_bytes = write_parquet(&dataset)?;
    println!("Generated Parquet binary: {} bytes", parquet_bytes.len());
    println!("Header Magic: {:?}", &parquet_bytes[0..4]);
    println!(
        "Footer Magic: {:?}",
        &parquet_bytes[parquet_bytes.len() - 4..]
    );

    assert_eq!(&parquet_bytes[0..4], b"PAR1");
    assert_eq!(&parquet_bytes[parquet_bytes.len() - 4..], b"PAR1");

    // 3. Deserialize back into Value AST
    println!("\n--- 3. Deserializing from Parquet bytes ---");
    let decoded = read_parquet(&parquet_bytes)?;
    println!("Decoded rows: {}", decoded.as_array().unwrap().len());

    // 4. Verify roundtrip equality
    assert_eq!(decoded, dataset);
    println!("\nRoundtrip fidelity verified: Decoded Value AST exactly matches original dataset!");

    Ok(())
}
