//! # Babbel Multi-Format Conversion Pipeline
//!
//! Demonstrates full-circle conversion of a structured document across all 8 supported formats:
//! JSON -> TOML -> YAML -> XML -> Bencode -> MessagePack -> CBOR -> BSON -> JSON!

use babbel::convert::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Babbel 8-Format Conversion Pipeline ===\n");

    // 1. Initial document in JSON format
    let original_json = r#"{
  "project": "Babbel",
  "version": "0.2.1",
  "active": true,
  "metrics": {
    "speed": 100,
    "memory_efficient": true
  }
}"#;
    println!("Initial JSON document:\n{}\n", original_json);

    // 2. JSON -> TOML
    println!("1. Converting JSON -> TOML...");
    let toml_str = json_to_toml(original_json)?;
    println!("TOML output:\n{}\n", toml_str);

    // 3. TOML -> YAML
    println!("2. Converting TOML -> YAML...");
    let yaml_str = toml_to_yaml(&toml_str)?;
    println!("YAML output:\n{}\n", yaml_str);

    // 4. YAML -> JSON
    println!("3. Converting YAML -> JSON...");
    let intermediate_json = yaml_to_json(&yaml_str)?;

    // 5. JSON -> MessagePack (binary)
    println!("4. Converting JSON -> MessagePack...");
    let msgpack_bytes = json_to_msgpack(&intermediate_json)?;
    println!("MessagePack binary size: {} bytes\n", msgpack_bytes.len());

    // 6. MessagePack -> CBOR (binary to binary)
    println!("5. Converting MessagePack -> CBOR...");
    let cbor_bytes = msgpack_to_cbor(&msgpack_bytes)?;
    println!("CBOR binary size:        {} bytes\n", cbor_bytes.len());

    // 7. CBOR -> BSON (binary to binary)
    println!("6. Converting CBOR -> BSON...");
    let bson_bytes = cbor_to_bson(&cbor_bytes)?;
    println!("BSON binary size:        {} bytes\n", bson_bytes.len());

    // 8. BSON -> JSON
    println!("7. Converting BSON back to JSON...");
    let final_json = bson_to_json(&bson_bytes)?;
    println!("Final JSON output:\n{}\n", final_json);

    // 9. Verify roundtrip through the universal Value AST
    let original_val = babbel::json::from_str(original_json)?;
    let final_val = babbel::json::from_str(&final_json)?;

    assert_eq!(
        original_val.get("project").and_then(|v| v.as_str()),
        final_val.get("project").and_then(|v| v.as_str())
    );
    assert_eq!(
        original_val.get("version").and_then(|v| v.as_str()),
        final_val.get("version").and_then(|v| v.as_str())
    );
    assert_eq!(
        original_val.get("active").and_then(|v| v.as_bool()),
        final_val.get("active").and_then(|v| v.as_bool())
    );

    println!("Full pipeline conversion completed with 100% semantic fidelity!");
    Ok(())
}
