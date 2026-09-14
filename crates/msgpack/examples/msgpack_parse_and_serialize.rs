//! # MessagePack Parse and Serialize Example
//!
//! Demonstrates constructing Babbel `Value` AST, serializing to compact
//! MessagePack binary with `to_vec`, parsing back with `from_bytes`, and inspecting fields.

use babbel_core::Value;
use babbel_msgpack::{from_bytes, to_vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel MessagePack Parse & Serialize Example ===\n");

    // 1. Build a rich data structure using Babbel Value
    let firmware_signature = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];
    let root = Value::Object(vec![
        ("device_id".into(), Value::String("sensor-alpha-042".into())),
        ("reading_count".into(), Value::Integer(1_000_000)),
        ("temperature".into(), Value::Float(21.75)),
        ("is_active".into(), Value::Bool(true)),
        ("calibration".into(), Value::Null),
        ("signature".into(), Value::Bytes(firmware_signature.clone())),
        (
            "history".into(),
            Value::Array(vec![
                Value::Float(21.4),
                Value::Float(21.5),
                Value::Float(21.7),
                Value::Float(21.75),
            ]),
        ),
    ]);

    // 2. Serialize to compact MessagePack binary
    println!("--- 1. Serializing to MessagePack binary ---");
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
    println!("\n--- 2. Parsing MessagePack binary ---");
    let parsed = from_bytes(&encoded)?;
    println!("Successfully parsed MessagePack document!");

    // 4. Access and inspect fields using Value helper methods
    if let Some(dev) = parsed.get("device_id").and_then(|v| v.as_str()) {
        println!("Device:      {}", dev);
    }
    if let Some(cnt) = parsed.get("reading_count").and_then(|v| v.as_i64()) {
        println!("Readings:    {}", cnt);
    }
    if let Some(temp) = parsed.get("temperature").and_then(|v| v.as_f64()) {
        println!("Temperature: {:.2} °C", temp);
    }
    if let Some(active) = parsed.get("is_active").and_then(|v| v.as_bool()) {
        println!("Active:      {}", active);
    }
    if let Some(sig) = parsed.get("signature").and_then(|v| v.as_bytes()) {
        print!("Signature:   0x");
        for b in sig {
            print!("{:02X}", b);
        }
        println!();
        assert_eq!(sig, firmware_signature.as_slice());
    }
    if let Some(hist) = parsed.get("history").and_then(|v| v.as_array()) {
        println!("History:     {} data points", hist.len());
    }

    // 5. Verify roundtrip equality
    assert_eq!(parsed, root);
    println!("\nRoundtrip fidelity verified: Parsed document exactly matches original.");

    Ok(())
}
