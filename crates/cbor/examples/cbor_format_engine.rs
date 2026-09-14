//! # CBOR FormatEngine Example
//!
//! Demonstrates using `CborEngine` via Babbel's unified `FormatEngine` trait.

use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_cbor::CborEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel CBOR FormatEngine Example ===\n");

    let engine = CborEngine::default();

    // Inspect engine metadata
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    // Build sample AST
    let original = Value::Object(vec![
        ("standard".into(), Value::String("RFC 8949".into())),
        ("engine".into(), Value::String("babbel_cbor".into())),
        ("fast".into(), Value::Bool(true)),
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
