//! # MessagePack FormatEngine Example
//!
//! Demonstrates using `MsgPackEngine` via Babbel's unified `FormatEngine` trait.

use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_msgpack::MsgPackEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel MessagePack FormatEngine Example ===\n");

    let engine = MsgPackEngine::default();

    // Inspect engine metadata
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    // Build sample AST
    let original = Value::Object(vec![
        ("engine".into(), Value::String("babbel_msgpack".into())),
        ("version".into(), Value::String("0.2.1".into())),
        ("speed_rank".into(), Value::Integer(1)),
    ]);

    // Serialize using the FormatEngine trait
    println!("\nSerializing Value through FormatEngine::serialize_to_vec()...");
    let options = FormatOptions::default();
    let bytes = engine.serialize_to_vec(&original, &options)?;
    println!("Serialized length: {} bytes", bytes.len());

    // Deserialize using FormatEngine trait
    println!("Deserializing Value through FormatEngine::parse_bytes()...");
    let decoded = engine.parse_bytes(&bytes)?;
    println!("Decoded Value: {:?}", decoded);

    assert_eq!(decoded, original);
    println!("\nFormatEngine roundtrip successful!");

    Ok(())
}
