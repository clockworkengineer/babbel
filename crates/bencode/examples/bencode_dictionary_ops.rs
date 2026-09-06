//! Example demonstrating dictionary manipulation operations.
//! This shows how to create, query, modify, and iterate through dictionary nodes
//! in various scenarios commonly needed when working with bencode data.

use bencode_lib::{make_node, Node};
use std::collections::HashMap;

fn main() {
    println!("=== Dictionary Manipulation Examples ===\n");

    // Example 1: Creating dictionaries
    demonstrate_creation();

    // Example 2: Querying dictionary values
    demonstrate_querying();

    // Example 3: Modifying dictionaries
    demonstrate_modification();

    // Example 4: Iterating through dictionaries
    demonstrate_iteration();

    // Example 5: Nested dictionaries
    demonstrate_nested_operations();

    // Example 6: Building a torrent metadata structure
    demonstrate_torrent_metadata();
}

/// Demonstrates various ways to create dictionary nodes
fn demonstrate_creation() {
    println!("--- Creating Dictionaries ---");

    // Method 1: Create empty and populate
    let mut dict1 = HashMap::new();
    dict1.insert("key1".to_string(), make_node("value1"));
    dict1.insert("key2".to_string(), make_node(42));
    let node1 = Node::Dictionary(dict1);
    println!("Method 1: {}", node1);

    // Method 2: Build inline
    let node2 = Node::Dictionary({
        let mut d = HashMap::new();
        d.insert("name".to_string(), make_node("Alice"));
        d.insert("age".to_string(), make_node(30));
        d
    });
    println!("Method 2: {}", node2);

    // Method 3: Start empty, modify with get_mut pattern
    let mut node3 = Node::Dictionary(HashMap::new());
    if let Node::Dictionary(dict) = &mut node3 {
        dict.insert("status".to_string(), make_node("active"));
        dict.insert("count".to_string(), make_node(100));
    }
    println!("Method 3: {}\n", node3);
}

/// Demonstrates querying dictionary values safely
fn demonstrate_querying() {
    println!("--- Querying Dictionary Values ---");

    let mut dict = HashMap::new();
    dict.insert("name".to_string(), make_node("Bob"));
    dict.insert("score".to_string(), make_node(95));
    dict.insert("active".to_string(), make_node(1));
    dict.insert(
        "tags".to_string(),
        make_node(vec![make_node("rust"), make_node("beginner")]),
    );

    let node = Node::Dictionary(dict);

    // Using get() method
    if let Some(name) = node.get("name") {
        println!("Name found: {}", name);
    }

    // Checking if key exists
    println!("Has 'email' key: {}", node.get("email").is_some());

    // Getting and unwrapping value
    if let Some(score_node) = node.get("score") {
        if let Some(score) = score_node.as_integer() {
            println!("Score value: {}", score);
            if *score >= 90 {
                println!("  Grade: A");
            }
        }
    }

    // Working with nested values
    if let Some(tags) = node.get("tags") {
        if let Some(tag_list) = tags.as_list() {
            println!("Tags: {} items", tag_list.len());
            for tag in tag_list {
                if let Some(s) = tag.as_string() {
                    println!("  - {}", s);
                }
            }
        }
    }

    println!();
}

/// Demonstrates modifying dictionary contents
fn demonstrate_modification() {
    println!("--- Modifying Dictionaries ---");

    let mut dict = HashMap::new();
    dict.insert("counter".to_string(), make_node(0));
    dict.insert("status".to_string(), make_node("pending"));

    let mut node = Node::Dictionary(dict);

    println!("Initial: {}", node);

    // Update an existing value using get_mut
    if let Some(counter) = node.get_mut("counter") {
        *counter = make_node(5);
    }
    println!("After incrementing counter: {}", node);

    // Add a new key
    if let Node::Dictionary(dict) = &mut node {
        dict.insert("timestamp".to_string(), make_node(1699564800));
    }
    println!("After adding timestamp: {}", node);

    // Update status
    if let Some(status) = node.get_mut("status") {
        *status = make_node("completed");
    }
    println!("After updating status: {}", node);

    // Remove a key
    if let Node::Dictionary(dict) = &mut node {
        dict.remove("counter");
    }
    println!("After removing counter: {}\n", node);
}

/// Demonstrates iterating through dictionary entries
fn demonstrate_iteration() {
    println!("--- Iterating Through Dictionaries ---");

    let mut dict = HashMap::new();
    dict.insert("alpha".to_string(), make_node(1));
    dict.insert("beta".to_string(), make_node(2));
    dict.insert("gamma".to_string(), make_node(3));
    dict.insert("delta".to_string(), make_node(4));

    let node = Node::Dictionary(dict);

    // Iterate and print all entries
    if let Some(dict) = node.as_dictionary() {
        println!("All entries:");
        for (key, value) in dict {
            println!("  {} => {}", key, value);
        }

        // Count entries by type
        let mut int_count = 0;
        let mut str_count = 0;
        for (_, value) in dict {
            if value.is_integer() {
                int_count += 1;
            } else if value.is_string() {
                str_count += 1;
            }
        }
        println!("Integers: {}, Strings: {}", int_count, str_count);

        // Collect all keys sorted
        let mut keys: Vec<_> = dict.keys().collect();
        keys.sort();
        println!("Sorted keys: {:?}", keys);
    }

    println!();
}

/// Demonstrates working with nested dictionaries
fn demonstrate_nested_operations() {
    println!("--- Nested Dictionary Operations ---");

    // Create nested structure
    let mut inner_dict = HashMap::new();
    inner_dict.insert("city".to_string(), make_node("New York"));
    inner_dict.insert("zip".to_string(), make_node("10001"));

    let mut outer_dict = HashMap::new();
    outer_dict.insert("name".to_string(), make_node("Charlie"));
    outer_dict.insert("address".to_string(), Node::Dictionary(inner_dict));

    let mut node = Node::Dictionary(outer_dict);

    println!("Nested structure: {}", node);

    // Access nested value
    if let Some(address) = node.get("address") {
        if let Some(city) = address.get("city") {
            println!("City: {}", city);
        }
    }

    // Modify nested value
    if let Some(address) = node.get_mut("address") {
        if let Some(zip) = address.get_mut("zip") {
            *zip = make_node("10002");
        }
    }
    println!("After updating zip: {}", node);

    // Add to nested dictionary
    if let Some(address) = node.get_mut("address") {
        if let Node::Dictionary(dict) = address {
            dict.insert("country".to_string(), make_node("USA"));
        }
    }
    println!("After adding country: {}\n", node);
}

/// Demonstrates building a realistic torrent metadata structure
fn demonstrate_torrent_metadata() {
    println!("--- Building Torrent Metadata ---");

    // Build file info
    let mut file_info = HashMap::new();
    file_info.insert("length".to_string(), make_node(1048576));
    file_info.insert(
        "path".to_string(),
        make_node(vec![make_node("documents"), make_node("readme.txt")]),
    );

    // Build info dictionary
    let mut info_dict = HashMap::new();
    info_dict.insert("name".to_string(), make_node("example-file"));
    info_dict.insert("piece length".to_string(), make_node(262144));
    info_dict.insert("pieces".to_string(), make_node("dummy_hash_value"));
    info_dict.insert(
        "files".to_string(),
        make_node(vec![Node::Dictionary(file_info)]),
    );

    // Build root dictionary
    let mut root = HashMap::new();
    root.insert(
        "announce".to_string(),
        make_node("udp://tracker.example.com:6969/announce"),
    );
    root.insert("created by".to_string(), make_node("bencode_lib example"));
    root.insert("creation date".to_string(), make_node(1699564800));
    root.insert("info".to_string(), Node::Dictionary(info_dict));

    let torrent = Node::Dictionary(root);

    println!("Torrent structure:");
    println!("{}", torrent);

    // Query specific metadata
    if let Some(announce) = torrent.get("announce") {
        println!("\nTracker URL: {}", announce);
    }

    if let Some(info) = torrent.get("info") {
        if let Some(name) = info.get("name") {
            println!("Torrent name: {}", name);
        }
        if let Some(piece_length) = info.get("piece length") {
            if let Some(len) = piece_length.as_integer() {
                println!("Piece length: {} bytes ({} KB)", len, len / 1024);
            }
        }
    }

    println!();
}
