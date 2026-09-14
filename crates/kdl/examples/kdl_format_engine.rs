//! # KDL FormatEngine Example
//!
//! Demonstrates using [`KdlEngine`] via the polymorphic [`FormatEngine`]
//! abstraction (Open-Closed Principle / Dependency Inversion Principle).

use babbel_core::{FormatEngine, FormatOptions, Value};
use babbel_kdl::KdlEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel KDL FormatEngine Example ===\n");

    let engine = KdlEngine;
    println!("Engine format ID:   {}", engine.format_id());
    println!("File extensions:    {:?}", engine.file_extensions());
    println!("MIME type:          {}", engine.mime_type());

    // Construct a polymorphic document with Value AST
    let config = Value::Object(vec![
        ("format".into(), Value::String("KDL".into())),
        ("engine".into(), Value::String("babbel_kdl".into())),
        ("version".into(), Value::String("0.2.1".into())),
        (
            "features".into(),
            Value::Array(vec![
                Value::String("nodes".into()),
                Value::String("slashdash_comments".into()),
                Value::String("nested_children".into()),
            ]),
        ),
    ]);

    println!("\nSerializing Value through FormatEngine::serialize_to_string()...");
    let serialized = engine.serialize_to_string(&config, &FormatOptions::pretty())?;
    println!("Serialized output:\n{}", serialized);

    println!("Deserializing Value through FormatEngine::parse_str()...");
    let decoded = engine.parse_str(&serialized)?;
    println!("Decoded Value:\n{:?}", decoded);

    assert_eq!(decoded, config);
    println!("\nFormatEngine roundtrip successful!");

    Ok(())
}
