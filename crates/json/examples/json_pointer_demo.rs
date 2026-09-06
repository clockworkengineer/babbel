//! Demonstrates JSON Pointer (RFC 6901) operations
//!
//! Shows how to navigate, query, and modify JSON structures using JSON Pointer syntax.
//! JSON Pointers provide a standard way to identify specific values within a JSON document.

use json_lib::{
    parse, pointer_get, pointer_get_mut, pointer_remove, pointer_set, stringify, BufferDestination,
    BufferSource, Node, Numeric,
};
use std::collections::HashMap;

fn main() {
    println!("=== JSON Pointer Demo (RFC 6901) ===\n");

    // Create a sample JSON document
    let json_data = r#"{
  "store": {
    "book": [
      {
        "category": "reference",
        "author": "Nigel Rees",
        "title": "Sayings of the Century",
        "price": 8.95
      },
      {
        "category": "fiction",
        "author": "Herman Melville",
        "title": "Moby Dick",
        "isbn": "0-553-21311-3",
        "price": 8.99
      }
    ],
    "bicycle": {
      "color": "red",
      "price": 19.95
    }
  },
  "expensive": 10
}"#;

    let mut source = BufferSource::new(json_data.as_bytes());
    let mut root = parse(&mut source).expect("Failed to parse JSON");

    // Example 1: Basic pointer get operations
    println!("1. Getting values with JSON Pointer:");
    if let Some(title) = pointer_get(&root, "/store/book/0/title") {
        println!("  First book title: {:?}", title.as_str());
    }

    if let Some(color) = pointer_get(&root, "/store/bicycle/color") {
        println!("  Bicycle color: {:?}", color.as_str());
    }

    if let Some(expensive) = pointer_get(&root, "/expensive") {
        println!("  Expensive threshold: {:?}", expensive.as_number());
    }

    // Example 2: Accessing array elements by index
    println!("\n2. Array access by index:");
    if let Some(book) = pointer_get(&root, "/store/book/1") {
        println!("  Second book: {}", {
            let mut buf = BufferDestination::new();
            stringify(book, &mut buf).unwrap();
            buf.to_string()
        });
    }

    // Example 3: Setting values with JSON Pointer
    println!("\n3. Setting values:");
    pointer_set(
        &mut root,
        "/store/bicycle/price",
        Node::Number(Numeric::Float(24.95)),
    )
    .expect("Failed to set price");
    println!("  Updated bicycle price to 24.95");

    // Add a new field
    pointer_set(
        &mut root,
        "/store/bicycle/brand",
        Node::Str("Trek".to_string()),
    )
    .expect("Failed to add brand");
    println!("  Added bicycle brand: Trek");

    // Example 4: Creating nested structures with set
    println!("\n4. Creating nested structures:");
    // First create the magazine array
    pointer_set(&mut root, "/store/magazine", Node::Array(vec![]))
        .expect("Failed to create magazine array");
    // Then add items
    if let Some(magazine) = pointer_get_mut(&mut root, "/store/magazine") {
        if let Some(arr) = magazine.as_array_mut() {
            let mut item = HashMap::new();
            item.insert("title".to_string(), Node::Str("Tech Monthly".to_string()));
            item.insert("price".to_string(), Node::Number(Numeric::Float(5.99)));
            arr.push(Node::Object(item));
        }
    }
    println!("  Created /store/magazine array with first item");

    // Example 5: Using get_mut for in-place modification
    println!("\n5. In-place modification with get_mut:");
    if let Some(price) = pointer_get_mut(&mut root, "/store/book/0/price") {
        *price = Node::Number(Numeric::Float(9.95));
        println!("  Updated first book price to 9.95");
    }

    // Example 6: Removing values
    println!("\n6. Removing values:");
    match pointer_remove(&mut root, "/store/book/0/price") {
        Ok(Some(removed)) => {
            println!("  Removed price from first book: {:?}", removed.as_number());
        }
        Ok(None) => println!("  Nothing to remove"),
        Err(e) => println!("  Error: {}", e),
    }

    // Example 7: Special characters in keys
    println!("\n7. Keys with special characters:");
    pointer_set(
        &mut root,
        "/special~0key",
        Node::Str("tilde in key".to_string()),
    )
    .expect("Failed to set special key");
    pointer_set(
        &mut root,
        "/another~1key",
        Node::Str("slash in key".to_string()),
    )
    .expect("Failed to set another special key");
    println!("  JSON Pointer uses ~0 for '~' and ~1 for '/'");

    if let Some(val) = pointer_get(&root, "/special~0key") {
        println!("  Retrieved /special~0key: {:?}", val.as_str());
    }

    // Example 8: Handling errors
    println!("\n8. Error handling:");
    match pointer_get(&root, "/nonexistent/path") {
        Some(_) => println!("  Found value"),
        None => println!("  Path not found (expected)"),
    }

    match pointer_set(&mut root, "invalid", Node::None) {
        Ok(_) => println!("  Successfully set value"),
        Err(e) => println!("  Error: {} (expected - no leading '/')", e),
    }

    // Example 9: Display final modified JSON
    println!("\n9. Final modified JSON:");
    let mut output = BufferDestination::new();
    json_lib::print(&root, &mut output, 2);
    println!("{}", output.to_string());

    println!("\n=== Demo Complete ===");
}
