//! Example demonstrating in-memory parsing and stringification of bencode data.
//! This shows how to work with bencode data without file I/O using convenience functions
//! and BufferSource/BufferDestination for more control.

use bencode_lib::{
    make_node, parse_borrowed, validate_bencode, stringify, stringify_to_bytes, stringify_to_string,
    BufferDestination, Node,
};
use std::collections::HashMap;

fn main() {
    println!("=== In-Memory Bencode Operations ===\n");

    // Example 1: Using convenience functions
    demonstrate_convenience_functions();

    // Example 2: Using Buffer types for more control
    demonstrate_buffer_types();

    // Example 3: Round-trip conversions
    demonstrate_round_trips();

    // Example 4: Working with byte arrays
    demonstrate_byte_operations();
}

/// Demonstrates the convenient parse_bytes, parse_str, stringify_to_bytes, and stringify_to_string functions
fn demonstrate_convenience_functions() {
    println!("--- Convenience Functions (Zero-Copy Preferred) ---");

    // Parse from byte slice (zero-copy)
    let bencode_bytes = b"i42e";
    match parse_borrowed(bencode_bytes) {
        Ok(node) => println!("Parsed from bytes (zero-copy): {}", node),
        Err(e) => eprintln!("Parse error: {}", e),
    }

    // Validate bencode without parsing
    match validate_bencode(bencode_bytes) {
        Ok(_) => println!("Validated bencode (no allocation)"),
        Err(e) => eprintln!("Validation error: {}", e),
    }

    // Create a node and convert to string
    let node = make_node(vec![make_node(1), make_node(2), make_node(3)]);
    match stringify_to_string(&node) {
        Ok(output) => println!("Stringified to string: {}", output),
        Err(e) => eprintln!("Stringify error: {}", e),
    }

    // Convert to bytes
    match stringify_to_bytes(&node) {
        Ok(output) => println!("Stringified to bytes: {:?}", output),
        Err(e) => eprintln!("Stringify error: {}", e),
    }

    println!();
}

/// Demonstrates using BufferSource and BufferDestination for more control
fn demonstrate_buffer_types() {
    println!("--- Buffer Types (Zero-Copy Preferred) ---");

    // Create bencode data in memory
    let bencode_data = b"d4:name4:John3:agei30e7:hobbieslll7:reading6:codingeee";

    // Parse using zero-copy
    match parse_borrowed(bencode_data) {
        Ok(node) => {
            println!("Parsed dictionary (zero-copy):");
            if let Some(dict) = node.as_dictionary() {
                for (key, value) in dict {
                    println!("  {:?}: {:?}", key, value);
                }
            }

            // Stringify back using BufferDestination
            let mut destination = BufferDestination::new();
            // Convert BorrowedNode to Node for stringification
            let node_owned: Node = node.to_node();
            match stringify(&node_owned, &mut destination) {
                Ok(_) => {
                    let output = &destination.buffer;
                    println!("Stringified back: {:?}", output);
                    println!("As UTF-8: {}", String::from_utf8_lossy(output));
                }
                Err(e) => eprintln!("Stringify error: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }

    println!();
}

/// Demonstrates round-trip conversions: create -> stringify -> parse -> compare
fn demonstrate_round_trips() {
    println!("--- Round-Trip Conversions ---");

    // Create a complex structure
    let mut dict = HashMap::new();
    dict.insert("title".to_string(), make_node("Bencode Example"));
    dict.insert("year".to_string(), make_node(2024));
    dict.insert(
        "tags".to_string(),
        make_node(vec![
            make_node("rust"),
            make_node("bencode"),
            make_node("serialization"),
        ]),
    );

    let original = Node::Dictionary(dict);
    println!("Original: {}", original);

    // Convert to bencode bytes
    match stringify_to_bytes(&original) {
        Ok(bytes) => {
            println!("Bencode bytes: {} bytes", bytes.len());
            println!("Bencode string: {}", String::from_utf8_lossy(&bytes));

            // Parse it back
            match parse_borrowed(&bytes) {
                Ok(parsed) => {
                    // Convert BorrowedNode to Node for comparison
                    let parsed_owned: Node = parsed.to_node();
                    println!("Parsed back: {}", parsed_owned);
                    println!("Matches original: {}", original == parsed_owned);
                }
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
        Err(e) => eprintln!("Stringify error: {}", e),
    }

    println!();
}

/// Demonstrates working with raw byte arrays for binary data
fn demonstrate_byte_operations() {
    println!("--- Byte Operations ---");

    // Various bencode-encoded values
    let examples = vec![
        ("Integer", b"i-123e" as &[u8]),
        ("String", b"11:hello world"),
        ("Empty list", b"le"),
        ("List with items", b"li1ei2ei3ee"),
        ("Empty dict", b"de"),
        ("Simple dict", b"d3:key5:valuee"),
    ];

    for (description, bytes) in examples {
        print!("{}: ", description);
        match parse_borrowed(bytes) {
            Ok(node) => {
                // Convert BorrowedNode to Node for display and stringification
                let node_owned: Node = node.to_node();
                println!("{:?}", node_owned);

                // Verify we can convert back
                if let Ok(output) = stringify_to_bytes(&node_owned) {
                    if output == bytes {
                        println!("  ✓ Round-trip successful");
                    } else {
                        println!("  ✗ Round-trip mismatch: {:?} != {:?}", output, bytes);
                    }
                }
            }
            Err(e) => println!("Parse error: {}", e),
        }
    }

    println!();
}
