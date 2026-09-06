//! Minimal bencode example demonstrating smallest possible binary size
//!
//! This example shows how to use the bencode library with no default features,
//! resulting in the smallest possible binary. This is ideal for embedded systems
//! with strict size constraints.
//!
//! Features disabled:
//! - json, toml, xml, yaml format conversions
//!
//! Only core bencode parsing and stringification is available.

use bencode_lib::{parse_bytes, stringify_to_bytes};

fn main() {
    // Parse a simple bencode integer
    let input = b"i42e";
    match parse_bytes(input) {
        Ok(node) => {
            // Convert back to bencode
            match stringify_to_bytes(&node) {
                Ok(output) => {
                    assert_eq!(output, input);
                    println!("Parsed and stringified integer successfully");
                }
                Err(e) => println!("Stringify error: {}", e),
            }

            // No format conversions available in minimal build
            // to_json, to_toml, to_xml, to_yaml are not compiled in
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }

    // Parse a bencode dictionary
    let dict = b"d3:agei25e4:name4:Johne";
    if let Ok(node) = parse_bytes(dict) {
        if let Ok(output) = stringify_to_bytes(&node) {
            assert_eq!(output, dict);
            println!("Parsed and stringified dictionary successfully");
        }
    }

    // This minimal build is perfect for:
    // - Embedded systems with limited flash
    // - Environments where only bencode I/O is needed
    // - Applications that implement their own serialization
    // - Reducing binary size by excluding unused format converters
}
