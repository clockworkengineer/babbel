//! # Parquet FormatEngine Example
//!
//! Demonstrates polymorphic usage of [`ParquetEngine`] via the [`FormatEngine`]
//! abstraction (Open-Closed Principle / Dependency Inversion Principle).

use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_parquet::ParquetEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel Parquet FormatEngine Example ===\n");

    let engine = ParquetEngine;
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    let dataset = Value::Array(vec![
        Value::Object(vec![
            ("id".into(), Value::Integer(1)),
            ("label".into(), Value::String("Sensor A".into())),
            ("active".into(), Value::Bool(true)),
        ]),
        Value::Object(vec![
            ("id".into(), Value::Integer(2)),
            ("label".into(), Value::String("Sensor B".into())),
            ("active".into(), Value::Bool(false)),
        ]),
    ]);

    println!("\nSerializing dataset through FormatEngine::serialize_to_bytes()...");
    let mut dest = babbel_core::BufferDestination::new();
    engine.serialize(&dataset, &mut dest, &FormatOptions::compact())?;
    let bytes = dest.into_vec();
    println!("Serialized binary size: {} bytes", bytes.len());

    println!("Deserializing bytes through FormatEngine::parse_bytes()...");
    let decoded = engine.parse_bytes(&bytes)?;
    println!("Decoded Value:\n{:?}", decoded);

    assert_eq!(decoded, dataset);
    println!("\nFormatEngine roundtrip successful!");

    Ok(())
}
