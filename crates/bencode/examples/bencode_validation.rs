//! Validation helpers example
//!
//! Demonstrates the new validation helper methods that make it easier
//! to extract and validate required/optional fields from bencode dictionaries.

use bencode_lib::{parse_bytes, Node};

fn main() {
    println!("=== Validation Helpers Demo ===\n");

    // Example 1: Parse torrent metadata with validation helpers
    println!("1. Validating torrent metadata:");
    let torrent_data = b"d8:announce35:http://tracker.example.com/announce4:infod6:lengthi1234567e4:name10:example.txt12:piece lengthi262144e6:pieces20:fake_hash_data_hereee";

    match parse_bytes(torrent_data) {
        Ok(node) => match validate_torrent(&node) {
            Ok(info) => {
                println!("   ✓ Torrent validation passed");
                println!("   - Announce: {}", info.announce);
                println!("   - Name: {}", info.name);
                println!("   - Size: {} bytes", info.length);
                println!("   - Piece length: {}", info.piece_length);
            }
            Err(e) => println!("   ✗ Validation failed: {}", e),
        },
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    // Example 2: Required vs optional fields
    println!("\n2. Handling optional fields:");
    let data_with_optionals = b"d4:name4:John3:agei25e7:comment10:Some notese";

    match parse_bytes(data_with_optionals) {
        Ok(node) => {
            // Required fields - will error if missing or wrong type
            match node.get_string_required("name") {
                Ok(name) => println!("   ✓ Name (required): {}", name),
                Err(e) => println!("   ✗ {}", e),
            }

            match node.get_int_required("age") {
                Ok(age) => println!("   ✓ Age (required): {}", age),
                Err(e) => println!("   ✗ {}", e),
            }

            // Optional fields - returns None if missing
            if let Some(comment) = node.get_string_optional("comment") {
                println!("   ✓ Comment (optional): {}", comment);
            } else {
                println!("   - Comment: not provided");
            }

            if let Some(email) = node.get_string_optional("email") {
                println!("   ✓ Email (optional): {}", email);
            } else {
                println!("   - Email: not provided");
            }
        }
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    // Example 3: Type mismatch errors
    println!("\n3. Type validation:");
    let invalid_types = b"d4:name4:John3:age5:twentye";

    match parse_bytes(invalid_types) {
        Ok(node) => {
            // This should succeed
            match node.get_string_required("name") {
                Ok(name) => println!("   ✓ Name is string: {}", name),
                Err(e) => println!("   ✗ {}", e),
            }

            // This should fail - age is a string, not an integer
            match node.get_int_required("age") {
                Ok(age) => println!("   ✓ Age is integer: {}", age),
                Err(e) => println!("   ✗ Expected error: {}", e),
            }
        }
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    // Example 4: Missing required fields
    println!("\n4. Missing required fields:");
    let incomplete = b"d4:name4:Johne";

    match parse_bytes(incomplete) {
        Ok(node) => {
            match node.get_string_required("name") {
                Ok(name) => println!("   ✓ Name found: {}", name),
                Err(e) => println!("   ✗ {}", e),
            }

            match node.get_int_required("age") {
                Ok(age) => println!("   ✓ Age found: {}", age),
                Err(e) => println!("   ✗ Expected error: {}", e),
            }
        }
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    // Example 5: Nested structure validation
    println!("\n5. Nested structure validation:");
    let nested = b"d4:user d4:name4:John3:agei25ee8:settingsd5:theme4:dark6:notifyi1eee";

    match parse_bytes(nested) {
        Ok(node) => {
            // Get nested dictionary
            match node.get_dict_required("user") {
                Ok(user) => {
                    println!("   ✓ User dictionary found");

                    // Wrap in Node to use validation methods
                    let user_node = Node::Dictionary(user.clone());

                    if let Ok(name) = user_node.get_string_required("name") {
                        println!("     - Name: {}", name);
                    }

                    if let Ok(age) = user_node.get_int_required("age") {
                        println!("     - Age: {}", age);
                    }
                }
                Err(e) => println!("   ✗ {}", e),
            }

            match node.get_dict_required("settings") {
                Ok(settings) => {
                    println!("   ✓ Settings dictionary found");

                    let settings_node = Node::Dictionary(settings.clone());

                    if let Some(theme) = settings_node.get_string_optional("theme") {
                        println!("     - Theme: {}", theme);
                    }

                    if let Some(notify) = settings_node.get_int_optional("notify") {
                        println!(
                            "     - Notifications: {}",
                            if notify == 1 { "enabled" } else { "disabled" }
                        );
                    }
                }
                Err(e) => println!("   ✗ {}", e),
            }
        }
        Err(e) => println!("   ✗ Parse error: {}", e),
    }

    println!("\n=== Advantages of Validation Helpers ===");
    println!("✓ Clear error messages with field names");
    println!("✓ Explicit required vs optional field handling");
    println!("✓ Type-safe extraction with compile-time guarantees");
    println!("✓ Reduces boilerplate validation code");
    println!("✓ Consistent error handling across the codebase");
}

// Example validation function for torrent metadata
struct TorrentInfo {
    announce: String,
    name: String,
    length: i64,
    piece_length: i64,
}

fn validate_torrent(node: &Node) -> Result<TorrentInfo, String> {
    // Get required announce field
    let announce = node.get_string_required("announce")?.to_string();

    // Get required info dictionary
    let info_dict = node.get_dict_required("info")?;
    let info = Node::Dictionary(info_dict.clone());

    // Get required fields from info dictionary
    let name = info.get_string_required("name")?.to_string();
    let length = info.get_int_required("length")?;
    let piece_length = info.get_int_required("piece length")?;

    // Optional: validate pieces field exists
    let _pieces = info.get_string_required("pieces")?;

    // Validate values
    if length <= 0 {
        return Err("File length must be positive".to_string());
    }

    if piece_length <= 0 {
        return Err("Piece length must be positive".to_string());
    }

    Ok(TorrentInfo {
        announce,
        name,
        length,
        piece_length,
    })
}
