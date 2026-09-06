//! Demonstrates Unicode file handling with BOM detection
//!
//! Shows how to read and write JSON files in different Unicode formats
//! (UTF-8, UTF-16, UTF-32) with proper BOM (Byte Order Mark) handling.

use json_lib::{
    detect_format, parse, read_file_to_string, stringify, write_file_from_string,
    BufferDestination, BufferSource, Format, FileDestination, FileSource, Node,
};
use std::collections::HashMap;
use std::path::Path;

fn main() {
    println!("=== Unicode File Handling Demo ===\n");

    // Example 1: Creating sample JSON data
    println!("1. Creating sample JSON data:");
    
    let mut data = HashMap::new();
    data.insert("greeting".to_string(), Node::from("Hello, 世界! 🌍"));
    data.insert("title".to_string(), Node::from("Καλημέρα κόσμε"));
    data.insert("description".to_string(), Node::from("Привет мир"));
    data.insert("emoji".to_string(), Node::from("🚀 ⭐ 💻"));
    
    let json_node = Node::Object(data);
    
    let mut buffer = BufferDestination::new();
    json_lib::print(&json_node, &mut buffer, 2);
    let json_content = buffer.to_string();
    
    println!("{}", json_content);

    // Example 2: Writing files in different Unicode formats
    println!("\n2. Writing JSON in different Unicode formats:");
    
    let test_files = vec![
        ("test_utf8.json", Format::Utf8),
        ("test_utf8_bom.json", Format::Utf8bom),
        ("test_utf16le.json", Format::Utf16le),
        ("test_utf16be.json", Format::Utf16be),
        ("test_utf32le.json", Format::Utf32le),
        ("test_utf32be.json", Format::Utf32be),
    ];
    
    for (filename, format) in test_files.iter() {
        let format_val = match format {
            Format::Utf8 => Format::Utf8,
            Format::Utf8bom => Format::Utf8bom,
            Format::Utf16le => Format::Utf16le,
            Format::Utf16be => Format::Utf16be,
            Format::Utf32le => Format::Utf32le,
            Format::Utf32be => Format::Utf32be,
        };
        
        match write_file_from_string(filename, &json_content, format_val) {
            Ok(_) => {
                let format_name = match format {
                    Format::Utf8 => "UTF-8 (no BOM)",
                    Format::Utf8bom => "UTF-8 with BOM",
                    Format::Utf16le => "UTF-16 LE",
                    Format::Utf16be => "UTF-16 BE",
                    Format::Utf32le => "UTF-32 LE",
                    Format::Utf32be => "UTF-32 BE",
                };
                println!("  ✓ Wrote {} as {}", filename, format_name);
            }
            Err(e) => println!("  ✗ Failed to write {}: {}", filename, e),
        }
    }

    // Example 3: Detecting file format
    println!("\n3. Detecting Unicode format from files:");
    
    for (filename, _) in &test_files {
        if Path::new(filename).exists() {
            match detect_format(filename) {
                Ok(detected) => {
                    let format_name = match detected {
                        Format::Utf8 => "UTF-8 (no BOM)",
                        Format::Utf8bom => "UTF-8 with BOM",
                        Format::Utf16le => "UTF-16 LE",
                        Format::Utf16be => "UTF-16 BE",
                        Format::Utf32le => "UTF-32 LE",
                        Format::Utf32be => "UTF-32 BE",
                    };
                    println!("  {}: {}", filename, format_name);
                }
                Err(e) => println!("  {}: Error - {}", filename, e),
            }
        }
    }

    // Example 4: Reading files with automatic format detection
    println!("\n4. Reading and parsing files:");
    
    for (filename, _) in test_files.iter().take(3) {
        if Path::new(filename).exists() {
            match read_file_to_string(filename) {
                Ok(content) => {
                    let mut source = BufferSource::new(content.as_bytes());
                    match parse(&mut source) {
                        Ok(node) => {
                            if let Some(greeting) = node.get("greeting").and_then(|g| g.as_str()) {
                                println!("  {} - greeting: {}", filename, greeting);
                            }
                        }
                        Err(e) => println!("  {} - Parse error: {}", filename, e),
                    }
                }
                Err(e) => println!("  {} - Read error: {}", filename, e),
            }
        }
    }

    // Example 5: Round-trip conversion
    println!("\n5. Round-trip format conversion:");
    
    // Read from UTF-8, write as UTF-16
    let input_file = "test_utf8.json";
    let output_file = "converted_utf16.json";
    
    if Path::new(input_file).exists() {
        match read_file_to_string(input_file) {
            Ok(content) => {
                match write_file_from_string(output_file, &content, Format::Utf16le) {
                    Ok(_) => {
                        println!("  ✓ Converted {} (UTF-8) to {} (UTF-16 LE)", input_file, output_file);
                        
                        // Verify by reading back
                        if let Ok(read_back) = read_file_to_string(output_file) {
                            if content == read_back {
                                println!("  ✓ Round-trip verification successful");
                            } else {
                                println!("  ✗ Round-trip verification failed - content mismatch");
                            }
                        }
                    }
                    Err(e) => println!("  ✗ Conversion failed: {}", e),
                }
            }
            Err(e) => println!("  ✗ Failed to read input: {}", e),
        }
    }

    // Example 6: Using FileSource and FileDestination with Unicode
    println!("\n6. Using FileSource/FileDestination:");
    
    let unicode_test_file = "unicode_test.json";
    
    // Create a JSON with various Unicode characters
    let mut unicode_data = HashMap::new();
    unicode_data.insert("ascii".to_string(), Node::from("Hello World"));
    unicode_data.insert("japanese".to_string(), Node::from("こんにちは"));
    unicode_data.insert("arabic".to_string(), Node::from("مرحبا"));
    unicode_data.insert("emoji".to_string(), Node::from("😀 🎉 🚀"));
    unicode_data.insert("symbols".to_string(), Node::from("∑ ∫ ≈ ≠ ∞"));
    
    let unicode_node = Node::Object(unicode_data);
    
    // Write using FileDestination
    match FileDestination::new(unicode_test_file) {
        Ok(mut dest) => {
            if let Err(e) = stringify(&unicode_node, &mut dest) {
                println!("  ✗ Failed to write: {}", e);
            } else {
                println!("  ✓ Wrote {} using FileDestination", unicode_test_file);
            }
        }
        Err(e) => println!("  ✗ Failed to create destination: {}", e),
    }
    
    // Read using FileSource
    if Path::new(unicode_test_file).exists() {
        match FileSource::new(unicode_test_file) {
            Ok(mut source) => {
                match parse(&mut source) {
                    Ok(node) => {
                        println!("  ✓ Read {} using FileSource", unicode_test_file);
                        
                        if let Some(obj) = node.as_object() {
                            println!("    Fields found: {}", obj.len());
                            for key in obj.keys() {
                                println!("      - {}", key);
                            }
                        }
                    }
                    Err(e) => println!("  ✗ Parse error: {}", e),
                }
            }
            Err(e) => println!("  ✗ Failed to open source: {}", e),
        }
    }

    // Example 7: Handling errors gracefully
    println!("\n7. Error handling:");
    
    // Try to detect format of non-existent file
    match detect_format("nonexistent.json") {
        Ok(_) => println!("  Unexpected success"),
        Err(_) => println!("  ✓ Properly handled non-existent file error"),
    }
    
    // Try to read non-existent file
    match read_file_to_string("nonexistent.json") {
        Ok(_) => println!("  Unexpected success"),
        Err(_) => println!("  ✓ Properly handled read error"),
    }

    // Example 8: Format information
    println!("\n8. Unicode format information:");
    println!("  UTF-8 (no BOM): Variable-length encoding (1-4 bytes), most common");
    println!("  UTF-8 with BOM: Starts with EF BB BF");
    println!("  UTF-16 LE: 16-bit little-endian, starts with FF FE");
    println!("  UTF-16 BE: 16-bit big-endian, starts with FE FF");
    println!("  UTF-32 LE: 32-bit little-endian, starts with FF FE 00 00");
    println!("  UTF-32 BE: 32-bit big-endian, starts with 00 00 FE FF");

    // Cleanup - remove test files
    println!("\n9. Cleanup:");
    for (filename, _) in &test_files {
        if Path::new(filename).exists() {
            if let Err(e) = std::fs::remove_file(filename) {
                println!("  Warning: Could not remove {}: {}", filename, e);
            }
        }
    }
    
    let cleanup_files = vec![
        "converted_utf16.json",
        "unicode_test.json",
    ];
    
    for filename in &cleanup_files {
        if Path::new(filename).exists() {
            if let Err(e) = std::fs::remove_file(filename) {
                println!("  Warning: Could not remove {}: {}", filename, e);
            }
        }
    }
    
    println!("  ✓ Cleaned up test files");

    println!("\n=== Demo Complete ===");
}
