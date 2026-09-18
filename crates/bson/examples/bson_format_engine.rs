//! # BSON FormatEngine Example
//!
//! Demonstrates using `BsonEngine` via Babbel's unified `FormatEngine` trait.

use babbel_bson::BsonEngine;
use babbel_core::{FormatEngine, FormatOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel BSON FormatEngine Example ===\n");

    let engine = BsonEngine::default();

    // Inspect engine metadata
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    // Build sample AST (top-level document)
    let original = Value::Object(vec![
        ("format".into(), Value::String("BSON".into())),
        ("engine".into(), Value::String("babbel_bson".into())),
        ("version".into(), Value::String("0.2.1".into())),
        ("compatible_mongo".into(), Value::Bool(true)),
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
