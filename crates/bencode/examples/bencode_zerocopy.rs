//! Example demonstrating zero-copy parsing for embedded systems.
//!
//! This example shows how to use BorrowedNode and parse_borrowed() to parse
//! bencode data without allocating or copying the input data. This is ideal
//! for memory-constrained embedded systems.

use bencode_lib::{parse_borrowed, validate_bencode, BorrowedNode};

fn main() {
    println!("=== Zero-Copy Bencode Parsing Example ===\n");

    // Example 1: Parse an integer
    let int_data = b"i42e";
    println!("Input: {:?}", core::str::from_utf8(int_data).unwrap());

    match parse_borrowed(int_data) {
        Ok(node) => {
            println!("Parsed: {}", node);
            if let Some(val) = node.as_integer() {
                println!("Integer value: {}\n", val);
            }
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 2: Parse a byte string (no allocation!)
    let str_data = b"13:Hello, World!";
    println!("Input: {:?}", core::str::from_utf8(str_data).unwrap());

    match parse_borrowed(str_data) {
        Ok(node) => {
            println!("Parsed: {}", node);
            if let Some(bytes) = node.as_bytes() {
                println!("Bytes (borrowed from input): {:?}", bytes);
                println!("As UTF-8: {}\n", core::str::from_utf8(bytes).unwrap());
            }
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 3: Parse a list
    let list_data = b"li1ei2ei3e5:helloe";
    println!("Input: {:?}", core::str::from_utf8(list_data).unwrap());

    match parse_borrowed(list_data) {
        Ok(node) => {
            println!("Parsed: {}", node);
            if let Some(list) = node.as_list() {
                println!("List has {} elements:", list.len());
                for (i, item) in list.iter().enumerate() {
                    println!("  [{}]: {}", i, item);
                }
                println!();
            }
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 4: Parse a dictionary (keys must be in lexicographic order)
    let dict_data = b"d3:agei25e4:name3:Bob5:scorei100e6:statusd6:active4:trueee";
    println!("Input: {:?}", core::str::from_utf8(dict_data).unwrap());

    match parse_borrowed(dict_data) {
        Ok(node) => {
            println!("Parsed: {}", node);
            if let Some(dict) = node.as_dictionary() {
                println!("Dictionary has {} entries:", dict.len());
                for (key, value) in dict.iter() {
                    let key_str = core::str::from_utf8(key).unwrap_or("<non-UTF8>");
                    println!("  {}: {}", key_str, value);
                }
                println!();
            }
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 5: Validation without parsing (minimal memory usage)
    println!("=== Validation-Only Mode ===\n");

    let valid_data = b"d4:name8:test.txt4:sizei1024ee";
    println!(
        "Validating: {:?}",
        core::str::from_utf8(valid_data).unwrap()
    );
    match validate_bencode(valid_data) {
        Ok(_) => println!("✓ Valid bencode\n"),
        Err(e) => println!("✗ Invalid: {}\n", e),
    }

    let invalid_data = b"i42"; // Missing end marker
    println!(
        "Validating: {:?}",
        core::str::from_utf8(invalid_data).unwrap()
    );
    match validate_bencode(invalid_data) {
        Ok(_) => println!("✓ Valid bencode\n"),
        Err(e) => println!("✗ Invalid: {}\n", e),
    }

    // Example 6: Memory efficiency demonstration
    println!("=== Memory Efficiency ===\n");

    let large_data = b"l5:item15:item25:item35:item45:item5e";
    println!("Input size: {} bytes", large_data.len());

    match parse_borrowed(large_data) {
        Ok(node) => {
            println!("Parsed without copying any string data!");
            println!("All strings are borrowed from the original buffer.");
            if let Some(list) = node.as_list() {
                println!("List contains {} items", list.len());

                // Demonstrate that the bytes are actually borrowed
                for (i, item) in list.iter().enumerate() {
                    if let Some(bytes) = item.as_bytes() {
                        // bytes is a reference to the original large_data buffer
                        println!("  Item {}: borrowed {} bytes", i, bytes.len());
                    }
                }
            }
            println!();
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 7: Torrent-like structure
    println!("=== Torrent-Like Structure (Zero-Copy) ===\n");

    let torrent_data = b"d8:announce32:http://example.com:6969/announce4:infod6:lengthi104857600e4:name10:ubuntu.isoee";

    match parse_borrowed(torrent_data) {
        Ok(node) => {
            if let Some(dict) = node.as_dictionary() {
                // Access announce URL (borrowed)
                if let Some(BorrowedNode::Bytes(announce)) = dict.get(&b"announce"[..]) {
                    println!("Announce: {}", core::str::from_utf8(announce).unwrap());
                }

                // Access info dictionary
                if let Some(BorrowedNode::Dictionary(info)) = dict.get(&b"info"[..]) {
                    if let Some(BorrowedNode::Integer(length)) = info.get(&b"length"[..]) {
                        println!("File size: {} bytes", length);
                    }
                    if let Some(BorrowedNode::Bytes(name)) = info.get(&b"name"[..]) {
                        println!("File name: {}", core::str::from_utf8(name).unwrap());
                    }
                }
            }
            println!("\n✓ Parsed entire torrent metadata without copying string data!");
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Benefits for Embedded Systems ===");
    println!("• No heap allocation for string data");
    println!("• Reduced memory fragmentation");
    println!("• Faster parsing (no memcpy)");
    println!("• Validation-only mode for checking data before committing resources");
    println!("• Works in no_std environments with alloc");
}
