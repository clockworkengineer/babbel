//! # MessagePack File I/O Example
//!
//! Demonstrates saving and loading MessagePack binary files to and from disk
//! using Babbel's `to_vec` and `from_bytes`.

use babbel_core::Value;
use babbel_msgpack::{from_bytes, to_vec};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel MessagePack File I/O Example ===\n");

    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("babbel_sensor_config.msgpack");
    println!("Target file path: {:?}", file_path);

    // 1. Prepare configuration document
    let document = Value::Object(vec![
        ("service".into(), Value::String("payment-gateway".into())),
        ("port".into(), Value::Integer(8443)),
        ("rate_limit_per_sec".into(), Value::Integer(5000)),
        ("tls_enabled".into(), Value::Bool(true)),
        (
            "allowed_ips".into(),
            Value::Array(vec![
                Value::String("10.0.0.1".into()),
                Value::String("10.0.0.2".into()),
                Value::String("192.168.1.100".into()),
            ]),
        ),
    ]);

    // 2. Serialize and write to disk
    println!("Writing configuration to binary MessagePack file...");
    let binary_data = to_vec(&document)?;
    fs::write(&file_path, &binary_data)?;
    let metadata = fs::metadata(&file_path)?;
    println!("Successfully wrote file ({} bytes)", metadata.len());

    // 3. Read back from disk and parse
    println!("Reading configuration back from file...");
    let loaded_bytes = fs::read(&file_path)?;
    let loaded = from_bytes(&loaded_bytes)?;
    println!("Successfully loaded and parsed file!");

    println!(
        "Service:    {:?}",
        loaded.get("service").and_then(|v| v.as_str())
    );
    println!(
        "Port:       {:?}",
        loaded.get("port").and_then(|v| v.as_i64())
    );
    println!(
        "Rate limit: {:?}",
        loaded.get("rate_limit_per_sec").and_then(|v| v.as_i64())
    );
    println!(
        "TLS:        {:?}",
        loaded.get("tls_enabled").and_then(|v| v.as_bool())
    );

    assert_eq!(loaded, document);
    println!("File roundtrip verification successful!");

    // Clean up temporary file
    let _ = fs::remove_file(&file_path);
    println!("Cleaned up temporary test file.");

    Ok(())
}
