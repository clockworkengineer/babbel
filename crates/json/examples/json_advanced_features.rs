use json_lib::{
    parse, pointer_get, pointer_remove, pointer_set, stringify, BufferDestination, BufferSource,
    Node, Numeric,
};
use std::collections::HashMap;

fn main() {
    println!("=== JSON Pointer and Node Navigation Examples ===\n");

    // Example 1: Using JSON Pointer to navigate JSON
    println!("1. JSON Pointer navigation:");
    let json_str = r#"{
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ],
        "settings": {
            "theme": "dark",
            "notifications": true
        }
    }"#;

    let mut source = BufferSource::new(json_str.as_bytes());
    let mut node = parse(&mut source).expect("Failed to parse");

    // Get values using JSON Pointer
    if let Some(alice) = pointer_get(&node, "/users/0/name") {
        println!("  First user name: {}", alice.as_str().unwrap_or("N/A"));
    }

    if let Some(theme) = pointer_get(&node, "/settings/theme") {
        println!("  Theme setting: {}", theme.as_str().unwrap_or("N/A"));
    }

    // Example 2: Modifying values with JSON Pointer
    println!("\n2. Modifying with JSON Pointer:");
    pointer_set(
        &mut node,
        "/users/0/age",
        Node::Number(Numeric::Integer(31)),
    )
    .expect("Failed to set");
    println!("  Updated Alice's age to 31");

    pointer_set(&mut node, "/settings/language", Node::Str("en".to_string()))
        .expect("Failed to set");
    println!("  Added new setting: language = en");

    // Example 3: Safe navigation without JSON Pointer
    println!("\n3. Safe navigation methods:");
    if let Node::Object(obj) = &node {
        if let Some(users_node) = obj.get("users") {
            if let Some(first_user) = users_node.at(0) {
                if let Some(name) = first_user.get("name") {
                    println!(
                        "  Safe navigation to first user: {}",
                        name.as_str().unwrap()
                    );
                }
            }
        }
    }

    // Example 4: Type checking and conversion
    println!("\n4. Type checking:");
    if let Some(settings) = pointer_get(&node, "/settings") {
        println!("  /settings is an object: {}", settings.is_object());
        println!("  /settings is an array: {}", settings.is_array());
    }

    if let Some(users) = pointer_get(&node, "/users") {
        if let Some(arr) = users.as_array() {
            println!("  Number of users: {}", arr.len());
        }
    }

    // Example 5: Removing values
    println!("\n5. Removing values:");
    if let Ok(Some(removed)) = pointer_remove(&mut node, "/users/1") {
        println!("  Removed user: {:?}", removed.get("name"));
    }

    // Example 6: Merging nodes
    println!("\n6. Merging nodes:");
    let mut base = Node::Object({
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::Str("base_a".to_string()));
        map.insert("b".to_string(), Node::Str("base_b".to_string()));
        map
    });

    let overlay = Node::Object({
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::Str("overlay_a".to_string()));
        map.insert("c".to_string(), Node::Str("new_c".to_string()));
        map
    });

    base.merge(overlay);
    println!("  Merged result has key 'a': {}", base.get("a").is_some());
    println!("  Merged result has key 'b': {}", base.get("b").is_some());
    println!("  Merged result has key 'c': {}", base.get("c").is_some());

    // Example 7: Pretty print the modified JSON
    println!("\n7. Final JSON structure:");
    let mut output = BufferDestination::new();
    stringify(&node, &mut output).expect("Failed to stringify");
    println!("{}", output.to_string());

    println!("\n=== Examples Complete ===");
}
