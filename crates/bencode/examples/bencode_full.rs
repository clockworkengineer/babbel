//! Full-featured bencode example with all format conversions
//! 
//! This example demonstrates the library with all default features enabled,
//! including all format conversions (JSON, TOML, XML, YAML).

use bencode_lib::{parse_bytes, stringify_to_bytes, to_json, to_toml, to_xml, to_yaml, BufferDestination};

fn main() {
    // Parse a bencode dictionary
    let dict = b"d3:agei25e4:name4:Johne";
    
    if let Ok(node) = parse_bytes(dict) {
        // Convert back to bencode
        if let Ok(output) = stringify_to_bytes(&node) {
            assert_eq!(output, dict);
            println!("Bencode: {}", String::from_utf8_lossy(&output));
        }
        
        // All format conversions available
        let mut buffer = BufferDestination::new();
        if to_json(&node, &mut buffer).is_ok() {
            println!("JSON: {}", String::from_utf8_lossy(&buffer.buffer));
        }
        
        let mut buffer = BufferDestination::new();
        if to_toml(&node, &mut buffer).is_ok() {
            println!("TOML: {}", String::from_utf8_lossy(&buffer.buffer));
        }
        
        let mut buffer = BufferDestination::new();
        if to_xml(&node, &mut buffer).is_ok() {
            println!("XML: {}", String::from_utf8_lossy(&buffer.buffer));
        }
        
        let mut buffer = BufferDestination::new();
        if to_yaml(&node, &mut buffer).is_ok() {
            println!("YAML: {}", String::from_utf8_lossy(&buffer.buffer));
        }
    }
}