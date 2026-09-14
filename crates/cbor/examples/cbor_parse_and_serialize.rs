//! # CBOR Parse and Serialize Example
//!
//! Demonstrates constructing Babbel `Value` AST, serializing to standard
//! RFC 8949 CBOR binary with `to_vec`, parsing back with `from_bytes`, and inspecting fields.

use babbel_core::Value;
use babbel_cbor::{from_bytes, to_vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel CBOR Parse & Serialize Example ===\n");

    // 1. Build a rich data structure exercising RFC 8949 CBOR major types
    // - Major 0: Unsigned integer
    // - Major 1: Negative integer
    // - Major 2: Byte string
    // - Major 3: UTF-8 text string
    // - Major 4: Array
    // - Major 5: Map
    // - Major 7: Simple types (bool, null) and IEEE 754 floating point
    let crypto_token = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x01, 0x02, 0x03, 0x04];
    let root = Value::Object(vec![
        ("protocol".into(), Value::String("RFC 8949".into())),
        ("version".into(), Value::Integer(1)),
        ("temperature_offset".into(), Value::Integer(-12)), // Negative integer (major 1)
        ("accuracy".into(), Value::Float(99.985)),
        ("active".into(), Value::Bool(true)),
        ("legacy_mode".into(), Value::Null),
        ("auth_token".into(), Value::Bytes(crypto_token.clone())),
        (
            "supported_ciphers".into(),
            Value::Array(vec![
                Value::String("ChaCha20-Poly1305".into()),
                Value::String("AES-256-GCM".into()),
            ]),
        ),
    ]);

    // 2. Serialize to CBOR binary
    println!("--- 1. Serializing to CBOR binary ---");
    let encoded = to_vec(&root)?;
    println!("Serialized byte length: {} bytes", encoded.len());
    println!("Hex representation:");
    for (i, chunk) in encoded.chunks(16).enumerate() {
        print!("{:04x}:  ", i * 16);
        for b in chunk {
            print!("{:02x} ", b);
        }
        println!();
    }

    // 3. Parse back from binary
    println!("\n--- 2. Parsing CBOR binary ---");
    let parsed = from_bytes(&encoded)?;
    println!("Successfully parsed CBOR document!");

    // 4. Access and inspect fields using Value helper methods
    if let Some(proto) = parsed.get("protocol").and_then(|v| v.as_str()) {
        println!("Protocol:           {}", proto);
    }
    if let Some(offset) = parsed.get("temperature_offset").and_then(|v| v.as_i64()) {
        println!("Temperature Offset: {}", offset);
    }
    if let Some(acc) = parsed.get("accuracy").and_then(|v| v.as_f64()) {
        println!("Accuracy:           {:.3}%", acc);
    }
    if let Some(active) = parsed.get("active").and_then(|v| v.as_bool()) {
        println!("Active:             {}", active);
    }
    if let Some(token) = parsed.get("auth_token").and_then(|v| v.as_bytes()) {
        print!("Auth Token:         0x");
        for b in token {
            print!("{:02X}", b);
        }
        println!();
        assert_eq!(token, crypto_token.as_slice());
    }
    if let Some(ciphers) = parsed.get("supported_ciphers").and_then(|v| v.as_array()) {
        println!("Supported Ciphers:  {} ciphers", ciphers.len());
        for (i, c) in ciphers.iter().enumerate() {
            if let Some(s) = c.as_str() {
                println!("  [{}] {}", i, s);
            }
        }
    }

    // 5. Verify roundtrip equality
    assert_eq!(parsed, root);
    println!("\nRoundtrip fidelity verified: Parsed document exactly matches original.");

    Ok(())
}
