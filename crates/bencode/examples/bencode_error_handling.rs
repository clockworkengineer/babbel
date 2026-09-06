//! Example demonstrating proper error handling when working with bencode data.
//! This shows how to handle parsing errors, invalid data, I/O errors, and
//! how to write robust code that gracefully handles failures.

use bencode_lib::{make_node, parse, parse_bytes, parse_str, FileSource, Node};

fn main() {
    println!("=== Error Handling Examples ===\n");

    // Example 1: Parsing invalid bencode data
    demonstrate_parse_errors();

    // Example 2: Handling file I/O errors
    demonstrate_file_errors();

    // Example 3: Type mismatch errors
    demonstrate_type_errors();

    // Example 4: Robust parsing with validation
    demonstrate_validation();

    // Example 5: Error recovery strategies
    demonstrate_recovery();
}

/// Demonstrates various parsing error scenarios
fn demonstrate_parse_errors() {
    println!("--- Parse Error Handling ---");

    // Test cases with invalid bencode data
    let invalid_cases = vec![
        ("Incomplete integer", b"i42" as &[u8]),
        ("Invalid integer format", b"i12a34e"),
        ("Negative zero", b"i-0e"),
        ("String length mismatch", b"10:short"),
        ("Missing string", b"5:"),
        ("Incomplete list", b"li1ei2e"),
        ("Incomplete dictionary", b"d3:key5:value"),
        ("Invalid dictionary key", b"di1e5:valuee"),
        ("Empty input", b""),
        ("Garbage data", b"xyz123"),
        ("Leading zero in integer", b"i01e"),
        ("Leading zero in string length", b"05:hello"),
    ];

    for (description, data) in invalid_cases {
        print!("{}: ", description);
        match parse_bytes(data) {
            Ok(node) => println!("Unexpectedly succeeded: {}", node),
            Err(e) => println!("Error (as expected): {}", e),
        }
    }

    // Demonstrate error handling with match
    let result = parse_str("i42");
    match result {
        Ok(node) => println!("\nParsed successfully: {}", node),
        Err(e) => println!("\nParse failed: {}", e),
    }

    // Demonstrate error handling with if-let
    if let Err(e) = parse_bytes(b"invalid") {
        println!("Caught error with if-let: {}", e);
    }

    println!();
}

/// Demonstrates handling file I/O errors
fn demonstrate_file_errors() {
    println!("--- File I/O Error Handling ---");

    // Try to open non-existent file
    let nonexistent_path = "nonexistent_file.bencode";
    match FileSource::new(nonexistent_path) {
        Ok(mut source) => {
            println!("File opened successfully");
            match parse(&mut source) {
                Ok(node) => println!("Parsed: {}", node),
                Err(e) => println!("Parse error: {}", e),
            }
        }
        Err(e) => println!("Failed to open '{}': {}", nonexistent_path, e),
    }

    // Try to open invalid path
    let invalid_path = "";
    match FileSource::new(invalid_path) {
        Ok(_) => println!("Unexpected success with empty path"),
        Err(e) => println!("Failed to open empty path: {}", e),
    }

    // Demonstrate safe file reading with proper error propagation
    match safe_read_file("example.bencode") {
        Ok(node) => println!("Successfully read file: {}", node),
        Err(e) => println!("Failed to read file: {}", e),
    }

    println!();
}

/// Helper function demonstrating proper error propagation
fn safe_read_file(path: &str) -> Result<Node, String> {
    let mut source =
        FileSource::new(path).map_err(|e| format!("Cannot open file '{}': {}", path, e))?;

    let node = parse(&mut source).map_err(|e| format!("Cannot parse file '{}': {}", path, e))?;

    Ok(node)
}

/// Demonstrates handling type mismatch errors
fn demonstrate_type_errors() {
    println!("--- Type Mismatch Handling ---");

    let data = make_node(vec![
        make_node(42),
        make_node("text"),
        make_node(vec![make_node(1), make_node(2)]),
    ]);

    println!("Working with: {}", data);

    // Safe type checking before access
    if data.is_list() {
        println!("✓ Data is a list");

        if let Some(items) = data.as_list() {
            for (i, item) in items.iter().enumerate() {
                print!("Item {}: ", i);

                // Check type before unwrapping
                if item.is_integer() {
                    if let Some(val) = item.as_integer() {
                        println!("integer value = {}", val);
                    }
                } else if item.is_string() {
                    if let Some(val) = item.as_string() {
                        println!("string value = '{}'", val);
                    }
                } else if item.is_list() {
                    println!("nested list with {} items", item.len());
                } else {
                    println!("unknown type: {}", item.type_name());
                }
            }
        }
    } else {
        println!("✗ Data is not a list, it's a {}", data.type_name());
    }

    // Demonstrate unsafe access (returns None)
    let integer_node = Node::Integer(42);
    match integer_node.as_string() {
        Some(s) => println!("Got string: {}", s),
        None => println!("Cannot access integer as string (type mismatch)"),
    }

    println!();
}

/// Demonstrates validation of parsed data
fn demonstrate_validation() {
    println!("--- Data Validation ---");

    // Simulate parsing torrent metadata
    let bencode_data = b"d8:announce35:udp://tracker.example.com:6969/announce4:infod4:name12:example-file12:piece lengthi262144e6:pieces16:dummy_hash_valueee";

    match parse_bytes(bencode_data) {
        Ok(node) => match validate_torrent_metadata(&node) {
            Ok(_) => println!("✓ Torrent metadata is valid"),
            Err(e) => println!("✗ Validation failed: {}", e),
        },
        Err(e) => println!("✗ Parse failed: {}", e),
    }

    // Test with invalid structure
    let invalid_data = make_node(vec![make_node(1), make_node(2)]);
    match validate_torrent_metadata(&invalid_data) {
        Ok(_) => println!("✓ Validation passed"),
        Err(e) => println!("✗ Validation failed (expected): {}", e),
    }

    println!();
}

/// Helper function to validate torrent metadata structure
fn validate_torrent_metadata(node: &Node) -> Result<(), String> {
    // Check if it's a dictionary
    let dict = node.as_dictionary().ok_or("Root must be a dictionary")?;

    // Check required fields
    dict.get("announce")
        .ok_or("Missing 'announce' field")?
        .as_string()
        .ok_or("'announce' must be a string")?;

    let info = dict
        .get("info")
        .ok_or("Missing 'info' field")?
        .as_dictionary()
        .ok_or("'info' must be a dictionary")?;

    info.get("name")
        .ok_or("Missing 'info.name' field")?
        .as_string()
        .ok_or("'info.name' must be a string")?;

    info.get("piece length")
        .ok_or("Missing 'info.piece length' field")?
        .as_integer()
        .ok_or("'info.piece length' must be an integer")?;

    info.get("pieces")
        .ok_or("Missing 'info.pieces' field")?
        .as_string()
        .ok_or("'info.pieces' must be a string")?;

    Ok(())
}

/// Demonstrates error recovery strategies
fn demonstrate_recovery() {
    println!("--- Error Recovery Strategies ---");

    // Strategy 1: Provide default values
    let data = parse_str("i42e").unwrap_or_else(|_| {
        println!("Parse failed, using default");
        make_node(0)
    });
    println!("With fallback: {}", data);

    // Strategy 2: Skip invalid items in a collection
    let mixed_data = vec![
        ("valid1", "i100e"),
        ("invalid", "i42"),
        ("valid2", "5:hello"),
        ("invalid2", "xyz"),
    ];

    println!("\nProcessing mixed data with error recovery:");
    let mut valid_items = Vec::new();
    for (name, bencode) in mixed_data {
        match parse_str(bencode) {
            Ok(node) => {
                println!("  ✓ {}: {}", name, node);
                valid_items.push(node);
            }
            Err(e) => {
                println!("  ✗ {}: {} (skipping)", name, e);
            }
        }
    }
    println!("Successfully parsed {} out of 4 items", valid_items.len());

    // Strategy 3: Partial parsing with detailed error context
    println!("\nPartial parsing with context:");
    let complex_data = "d3:key5:value3:badi42ee";
    match parse_str(complex_data) {
        Ok(node) => {
            println!("Parsed: {}", node);
            // Even if parsing succeeded, validate the content
            if let Some(dict) = node.as_dictionary() {
                for (key, value) in dict {
                    if value.is_none() {
                        println!("  Warning: key '{}' has None value", key);
                    }
                }
            }
        }
        Err(e) => {
            println!("Parse error: {}", e);
            println!("Input was: {}", complex_data);
            println!("Attempting partial recovery not possible for this input");
        }
    }

    println!();
}
