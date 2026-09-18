//! # RON FormatEngine Example
//!
//! Demonstrates using `RonEngine` via Babbel's unified `FormatEngine` trait.

use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_ron::RonEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel RON FormatEngine Example ===\n");

    let engine = RonEngine;

    // Inspect engine metadata
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    // Build sample AST
    let original = Value::Object(vec![
        ("format".into(), Value::String("RON".into())),
        ("engine".into(), Value::String("babbel_ron".into())),
        ("version".into(), Value::String("0.2.2".into())),
        (
            "features".into(),
            Value::Array(vec![
                Value::String("structs".into()),
                Value::String("raw_strings".into()),
                Value::String("nested_comments".into()),
            ]),
        ),
    ]);

    // Serialize using the FormatEngine trait
    println!("\nSerializing Value through FormatEngine::serialize_to_string()...");
    let options = FormatOptions {
        pretty: true,
        indent: 2,
    };
    let ron_text = engine.serialize_to_string(&original, &options)?;
    println!("Serialized output:\n{}\n", ron_text);

    // Deserialize using FormatEngine trait
    println!("Deserializing Value through FormatEngine::parse_str()...");
    let decoded = engine.parse_str(&ron_text)?;
    println!("Decoded Value:\n{:?}", decoded);

    assert_eq!(decoded, original);
    println!("\nFormatEngine roundtrip successful!");

    Ok(())
}
