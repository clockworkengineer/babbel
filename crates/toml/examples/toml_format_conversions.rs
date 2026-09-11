//! # TOML Cross-Format Conversions Example
//!
//! Demonstrates converting TOML documents to and from JSON, YAML, XML, and Bencode
//! through Babbel's universal `babbel_core::model::Value` AST.

use babbel_core::io::BufferDestination;
use babbel_core::model::Value;
use babbel_toml::{from_str, to_string_pretty, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Polyglot Cross-Format Conversions ===\n");

    let toml_input = r#"
[package]
name = "babbel"
version = "0.2.0"
edition = "2024"
authors = [ "ClockworkEngineer", "Babbel Contributors" ]

[dependencies]
babbel_core = "0.2.0"
arrayvec = "0.7.6"
smallvec = "1.15.1"

[features]
default = [ "std", "json", "toml" ]
std = true
"#;

    println!("--- 1. Original TOML Document ---");
    let toml_node = from_str(toml_input)?;
    println!("{}", to_string_pretty(&toml_node)?);

    // Convert TOML Node to Universal Babbel Value AST
    let universal_value: Value = Value::from(&toml_node);

    // 1. Convert to JSON
    println!("--- 2. Converted to JSON ---");
    let mut json_dest = BufferDestination::new();
    universal_value.serialize_json(&mut json_dest);
    let json_output = json_dest.to_string();
    println!("{}", json_output);

    // 2. Convert to YAML
    println!("--- 3. Converted to YAML ---");
    let mut yaml_dest = BufferDestination::new();
    universal_value.serialize_yaml(&mut yaml_dest, 0);
    let yaml_output = yaml_dest.to_string();
    println!("{}", yaml_output);

    // 3. Convert to XML
    println!("--- 4. Converted to XML ---");
    let mut xml_dest = BufferDestination::new();
    universal_value.serialize_xml(&mut xml_dest, Some("manifest"));
    let xml_output = xml_dest.to_string();
    println!("{}", xml_output);

    // 4. Convert to Bencode
    println!("--- 5. Converted to Bencode ---");
    let mut bencode_dest = BufferDestination::new();
    universal_value.serialize_bencode(&mut bencode_dest);
    let bencode_output = bencode_dest.to_string();
    println!("Bencode binary representation ({} bytes):", bencode_output.len());
    println!("{:?}", &bencode_output[..bencode_output.len().min(80)]);

    // 5. Convert Universal Value back to TOML Node
    println!("\n--- 6. Roundtrip Back to TOML Node ---");
    let roundtripped_node = Node::from(&universal_value);
    let roundtripped_toml = to_string_pretty(&roundtripped_node)?;
    println!("{}", roundtripped_toml);

    // Assert that key fields survived roundtrip through the universal AST
    let pkg = roundtripped_node.get("package").expect("package table exists");
    assert_eq!(pkg.get("name").and_then(|n| n.as_str()), Some("babbel"));
    assert_eq!(pkg.get("version").and_then(|n| n.as_str()), Some("0.2.0"));

    println!("Universal cross-format pipeline verified successfully!");
    Ok(())
}
