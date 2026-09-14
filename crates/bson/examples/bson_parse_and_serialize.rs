//! # BSON Parse and Serialize Example
//!
//! Demonstrates constructing Babbel `Value` AST representing a BSON document,
//! serializing to binary BSON format with `to_vec`, parsing back with `from_bytes`,
//! and inspecting typed fields (int32/int64, doubles, strings, subdocuments, arrays, binary).

use babbel_core::Value;
use babbel_bson::{from_bytes, to_vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel BSON Parse & Serialize Example ===\n");

    // 1. Build a BSON document.
    // In the BSON specification (bsonspec.org), a top-level BSON datum is always a Document (Object).
    let binary_uuid = vec![
        0xA1, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6, 0x07, 0x18,
        0x29, 0x3A, 0x4B, 0x5C, 0x6D, 0x7E, 0x8F, 0x90,
    ];

    let root = Value::Object(vec![
        ("_id".into(), Value::Bytes(binary_uuid.clone())),
        ("title".into(), Value::String("Advanced Systems Engineering".into())),
        ("views".into(), Value::Integer(1_250_000)), // int32 / int64
        ("rating".into(), Value::Float(4.92)),       // 64-bit IEEE 754 float
        ("published".into(), Value::Bool(true)),
        ("deprecated_at".into(), Value::Null),
        (
            "author".into(),
            Value::Object(vec![
                ("name".into(), Value::String("Ada Lovelace".into())),
                ("affiliation".into(), Value::String("Analytical Engines".into())),
            ]),
        ),
        (
            "tags".into(),
            Value::Array(vec![
                Value::String("computing".into()),
                Value::String("architecture".into()),
                Value::String("rust".into()),
            ]),
        ),
    ]);

    // 2. Serialize to binary BSON
    println!("--- 1. Serializing to BSON binary ---");
    let encoded = to_vec(&root)?;
    println!("Serialized byte length: {} bytes", encoded.len());

    // Display first 4 bytes which encode total document length in little-endian
    let doc_len = u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
    println!("Document length header (first 4 bytes little-endian): {} bytes", doc_len);
    assert_eq!(doc_len as usize, encoded.len());

    println!("\nHex representation:");
    for (i, chunk) in encoded.chunks(16).enumerate() {
        print!("{:04x}:  ", i * 16);
        for b in chunk {
            print!("{:02x} ", b);
        }
        println!();
    }

    // 3. Parse back from binary
    println!("\n--- 2. Parsing BSON binary ---");
    let parsed = from_bytes(&encoded)?;
    println!("Successfully parsed BSON document!");

    // 4. Access fields
    if let Some(title) = parsed.get("title").and_then(|v| v.as_str()) {
        println!("Title:      {}", title);
    }
    if let Some(views) = parsed.get("views").and_then(|v| v.as_i64()) {
        println!("Views:      {}", views);
    }
    if let Some(rating) = parsed.get("rating").and_then(|v| v.as_f64()) {
        println!("Rating:     {:.2}", rating);
    }
    if let Some(publ) = parsed.get("published").and_then(|v| v.as_bool()) {
        println!("Published:  {}", publ);
    }
    if let Some(author) = parsed.get("author") {
        if let Some(name) = author.get("name").and_then(|v| v.as_str()) {
            println!("Author:     {}", name);
        }
    }
    if let Some(tags) = parsed.get("tags").and_then(|v| v.as_array()) {
        println!("Tags ({}):", tags.len());
        for t in tags {
            if let Some(s) = t.as_str() {
                println!("  - {}", s);
            }
        }
    }

    // 5. Verify roundtrip equality
    assert_eq!(parsed, root);
    println!("\nRoundtrip fidelity verified: Parsed BSON exactly matches original.");

    Ok(())
}
