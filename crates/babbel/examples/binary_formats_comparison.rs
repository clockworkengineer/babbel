//! # Binary Formats Size & Structure Comparison
//!
//! Compares payload sizes and byte representations of MessagePack, CBOR, BSON, Bencode, and JSON
//! for an identical domain payload.

use babbel::core::{FormatEngine, FormatOptions, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel Binary Formats Comparison ===\n");

    // Standard payload representative of microservice RPC and document storage
    let payload = Value::Object(vec![
        ("sensor_id".into(), Value::String("temp-probe-42".into())),
        ("timestamp_utc".into(), Value::Integer(1_700_000_000)),
        ("reading_celsius".into(), Value::Float(23.85)),
        ("status_ok".into(), Value::Bool(true)),
        ("flags".into(), Value::Array(vec![
            Value::Integer(1),
            Value::Integer(4),
            Value::Integer(16),
        ])),
    ]);

    let options = FormatOptions::default();

    // 1. JSON (minified text)
    let json_text = babbel::JsonEngine.serialize_to_string(&payload, &options)?;

    // 2. Bencode (compact binary-safe text)
    let bencode_bytes = babbel::BencodeEngine.serialize_to_vec(&payload, &options)?;

    // 3. MessagePack (compact binary)
    let msgpack_bytes = babbel::msgpack::to_vec(&payload)?;

    // 4. CBOR (RFC 8949 binary)
    let cbor_bytes = babbel::cbor::to_vec(&payload)?;

    // 5. BSON (MongoDB Binary JSON document)
    let bson_bytes = babbel::bson::to_vec(&payload)?;

    println!("{:<15} {:>10}  {}", "Format", "Size (bytes)", "Notes");
    println!("{:-<15} {:-<10}  {:-<35}", "", "", "");
    println!("{:<15} {:>10}  Minified text", "JSON", json_text.len());
    println!("{:<15} {:>10}  BitTorrent dictionary encoding", "Bencode", bencode_bytes.len());
    println!("{:<15} {:>10}  Prefix-typed BSON document", "BSON", bson_bytes.len());
    println!("{:<15} {:>10}  RFC 8949 major types", "CBOR", cbor_bytes.len());
    println!("{:<15} {:>10}  Fixnum and fixmap compact packing", "MessagePack", msgpack_bytes.len());

    println!("\nHex inspection of binary formats:");
    println!("--- MessagePack ({} bytes) ---", msgpack_bytes.len());
    print_hex(&msgpack_bytes);

    println!("\n--- CBOR ({} bytes) ---", cbor_bytes.len());
    print_hex(&cbor_bytes);

    println!("\n--- BSON ({} bytes) ---", bson_bytes.len());
    print_hex(&bson_bytes);

    // Verify all can be roundtripped back to identical AST
    assert_eq!(babbel::msgpack::from_bytes(&msgpack_bytes)?, payload);
    assert_eq!(babbel::cbor::from_bytes(&cbor_bytes)?, payload);
    assert_eq!(babbel::bson::from_bytes(&bson_bytes)?, payload);

    println!("\nAll binary formats successfully decoded with 100% AST parity!");
    Ok(())
}

fn print_hex(bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        print!("{:04x}:  ", i * 16);
        for b in chunk {
            print!("{:02x} ", b);
        }
        println!();
    }
}
