//! # CBOR Indefinite-Length Streaming Example
//!
//! Demonstrates parsing CBOR payloads using indefinite-length containers and chunked strings
//! terminated by a `BREAK` stop code (`0xFF`), as defined in RFC 8949 Section 3.2.

use babbel_cbor::from_bytes;
use babbel_core::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel CBOR Indefinite-Length & Streaming Example ===\n");

    // 1. Indefinite-Length Array: `0x9F` (array start) followed by items, ending with `0xFF` (break)
    // Items: 1 (0x01), 2 (0x02), 3 (0x03)
    let indefinite_array_bytes = vec![0x9F, 0x01, 0x02, 0x03, 0xFF];
    println!("--- 1. Decoding Indefinite-Length Array ---");
    println!("Raw bytes: {:02X?}", indefinite_array_bytes);
    let array_val = from_bytes(&indefinite_array_bytes)?;
    println!("Parsed AST: {:?}", array_val);
    assert_eq!(
        array_val,
        Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3)
        ])
    );
    println!("Array decoding verified successfully!\n");

    // 2. Indefinite-Length Text String: `0x7F` followed by chunked definite text strings, ending with `0xFF`
    // Chunks: "Hello " (0x66, ...), "World!" (0x66, ...)
    let indefinite_text_bytes = vec![
        0x7F, // Indefinite text string start
        0x66, b'H', b'e', b'l', b'l', b'o', b' ', // definite text chunk 1 (6 bytes)
        0x66, b'W', b'o', b'r', b'l', b'd', b'!', // definite text chunk 2 (6 bytes)
        0xFF, // Break code
    ];
    println!("--- 2. Decoding Indefinite-Length (Chunked) Text String ---");
    println!("Raw bytes: {:02X?}", indefinite_text_bytes);
    let text_val = from_bytes(&indefinite_text_bytes)?;
    println!("Parsed text: {:?}", text_val);
    assert_eq!(text_val, Value::String("Hello World!".to_string()));
    println!("Chunked text reassembly verified successfully!\n");

    // 3. Indefinite-Length Map: `0xBF` followed by key-value pairs, ending with `0xFF`
    // Key: "sensor" -> "temp", Key: "active" -> true
    let indefinite_map_bytes = vec![
        0xBF, // Indefinite map start
        0x66, b's', b'e', b'n', b's', b'o', b'r', // key: "sensor"
        0x64, b't', b'e', b'm', b'p', // value: "temp"
        0x66, b'a', b'c', b't', b'i', b'v', b'e', // key: "active"
        0xF5, // value: true
        0xFF, // Break code
    ];
    println!("--- 3. Decoding Indefinite-Length Map ---");
    println!("Raw bytes: {:02X?}", indefinite_map_bytes);
    let map_val = from_bytes(&indefinite_map_bytes)?;
    println!("Parsed map: {:?}", map_val);
    assert_eq!(
        map_val,
        Value::Object(vec![
            ("sensor".into(), Value::String("temp".into())),
            ("active".into(), Value::Bool(true)),
        ])
    );
    println!("Indefinite map decoding verified successfully!\n");

    println!("All indefinite-length streaming examples passed!");
    Ok(())
}
