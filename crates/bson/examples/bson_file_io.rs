//! # BSON File I/O Example
//!
//! Demonstrates saving and loading BSON database records to and from disk
//! using Babbel's `to_vec` and `from_bytes`.

use babbel_bson::{from_bytes, to_vec};
use babbel_core::Value;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel BSON File I/O Example ===\n");

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("babbel_mongodb_record.bson");
    println!("Target file path: {:?}", file_path);

    // 1. Prepare database record document
    let record = Value::Object(vec![
        ("collection".into(), Value::String("orders".into())),
        ("order_id".into(), Value::Integer(987654321)),
        ("customer".into(), Value::String("Jane Doe".into())),
        ("amount_usd".into(), Value::Float(149.99)),
        ("shipped".into(), Value::Bool(true)),
        (
            "line_items".into(),
            Value::Array(vec![
                Value::Object(vec![
                    ("sku".into(), Value::String("RUST-BOOK".into())),
                    ("qty".into(), Value::Integer(1)),
                ]),
                Value::Object(vec![
                    ("sku".into(), Value::String("DEV-STICKER".into())),
                    ("qty".into(), Value::Integer(5)),
                ]),
            ]),
        ),
    ]);

    // 2. Serialize and write to disk
    println!("Writing BSON record to disk...");
    let binary_bson = to_vec(&record)?;
    fs::write(&file_path, &binary_bson)?;
    let file_size = fs::metadata(&file_path)?.len();
    println!("Successfully wrote file ({} bytes)", file_size);

    // 3. Read back from disk and parse
    println!("Reading BSON record back from disk...");
    let read_bytes = fs::read(&file_path)?;
    let loaded = from_bytes(&read_bytes)?;
    println!("Successfully loaded and parsed BSON file!");

    if let Some(coll) = loaded.get("collection").and_then(|v| v.as_str()) {
        println!("Collection:  {}", coll);
    }
    if let Some(id) = loaded.get("order_id").and_then(|v| v.as_i64()) {
        println!("Order ID:    {}", id);
    }
    if let Some(amount) = loaded.get("amount_usd").and_then(|v| v.as_f64()) {
        println!("Amount USD:  ${:.2}", amount);
    }
    if let Some(items) = loaded.get("line_items").and_then(|v| v.as_array()) {
        println!("Line Items:  {} entries", items.len());
    }

    assert_eq!(loaded, record);
    println!("File roundtrip verification successful!");

    // Clean up temporary file
    let _ = fs::remove_file(&file_path);
    println!("Cleaned up temporary test file.");

    Ok(())
}
