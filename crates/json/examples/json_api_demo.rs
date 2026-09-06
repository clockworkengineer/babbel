//! Demonstrates new API improvements in Phase 7
//!
//! Shows usage of:
//! - .into_*() consuming methods
//! - Numeric conversion helpers (as_i64, as_f64, as_u64)
//! - Iterator methods (keys, object_values, array_iter)
//! - TryFrom implementations
//! - Display trait

use json_lib::Node;
use std::collections::HashMap;
use std::convert::TryFrom;

fn main() {
    println!("=== Phase 7 API Improvements Demo ===\n");

    // 1. into_*() consuming methods - zero-copy extraction
    println!("1. Consuming .into_*() methods:");
    let node = Node::from("hello world");
    let s = node.into_string().unwrap();
    println!("  Extracted string: {}", s);

    let node = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
    let vec = node.into_array().unwrap();
    println!("  Extracted array with {} elements", vec.len());

    let mut map = HashMap::new();
    map.insert("key".to_string(), Node::from(42));
    let node = Node::Object(map);
    let extracted_map = node.into_object().unwrap();
    println!("  Extracted object with {} keys", extracted_map.len());

    // 2. Numeric conversion helpers
    println!("\n2. Numeric conversion helpers:");
    let node = Node::from(42);
    println!("  as_i64(): {:?}", node.as_i64());
    println!("  as_f64(): {:?}", node.as_f64());
    println!("  as_u64(): {:?}", node.as_u64());

    let node = Node::from(42.7);
    println!("  Float as_i64(): {:?}", node.as_i64());
    println!("  Float as_f64(): {:?}", node.as_f64());

    let node = Node::from(-42);
    println!("  Negative as_u64(): {:?}", node.as_u64()); // None

    // 3. Iterator methods
    println!("\n3. Iterator methods:");

    let mut map = HashMap::new();
    map.insert("name".to_string(), Node::from("Alice"));
    map.insert("age".to_string(), Node::from(30));
    map.insert("active".to_string(), Node::from(true));
    let node = Node::Object(map);

    println!("  Object keys:");
    for key in node.keys().unwrap() {
        println!("    - {}", key);
    }

    println!("  Object values:");
    for value in node.object_values().unwrap() {
        println!("    - {}", value);
    }

    let arr = Node::Array(vec![Node::from(10), Node::from(20), Node::from(30)]);

    println!("  Array elements:");
    for elem in arr.array_iter().unwrap() {
        println!("    - {}", elem);
    }

    // 4. Mutable iteration
    println!("\n4. Mutable iteration:");
    let mut arr = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);

    println!("  Before doubling: {}", arr);
    for elem in arr.array_iter_mut().unwrap() {
        if let Some(n) = elem.as_i64() {
            *elem = Node::from(n * 2);
        }
    }
    println!("  After doubling: {}", arr);

    // 5. TryFrom conversions
    println!("\n5. TryFrom conversions:");

    let node = Node::from("hello");
    let result: Result<String, _> = TryFrom::try_from(node);
    println!("  String: {:?}", result);

    let node = Node::from(42);
    let result: Result<i64, _> = TryFrom::try_from(node);
    println!("  i64: {:?}", result);

    let node = Node::from(42);
    let result: Result<String, _> = TryFrom::try_from(node);
    println!("  Wrong type (i64 -> String): {:?}", result);

    let node = Node::from(true);
    let result: Result<bool, _> = TryFrom::try_from(node);
    println!("  bool: {:?}", result);

    let arr = Node::Array(vec![Node::from(1), Node::from(2)]);
    let result: Result<Vec<Node>, _> = TryFrom::try_from(arr);
    println!("  Vec<Node>: {} elements", result.unwrap().len());

    // 6. Display trait
    println!("\n6. Display trait:");
    println!("  null: {}", Node::None);
    println!("  boolean: {}", Node::from(true));
    println!("  number: {}", Node::from(42));
    println!("  string: {}", Node::from("test"));
    println!(
        "  array: {}",
        Node::Array(vec![Node::from(1), Node::from(2)])
    );

    let mut obj = HashMap::new();
    obj.insert("x".to_string(), Node::from(10));
    obj.insert("y".to_string(), Node::from(20));
    println!("  object: {}", Node::Object(obj));

    // 7. Chaining with new APIs
    println!("\n7. Chaining example:");
    let mut data = HashMap::new();
    data.insert(
        "scores".to_string(),
        Node::Array(vec![Node::from(85), Node::from(92), Node::from(78)]),
    );
    let node = Node::Object(data);

    if let Some(scores_node) = node.get("scores") {
        if let Some(iter) = scores_node.array_iter() {
            let sum: i64 = iter.filter_map(|n| n.as_i64()).sum();
            let count = scores_node.len().unwrap();
            let avg = sum as f64 / count as f64;
            println!("  Average score: {:.1}", avg);
        }
    }

    // 8. Length and emptiness
    println!("\n8. Length and emptiness:");
    let arr = Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]);
    println!("  Array len: {:?}", arr.len());
    println!("  Array is_empty: {}", arr.is_empty());

    let obj = Node::Object(HashMap::new());
    println!("  Empty object len: {:?}", obj.len());
    println!("  Empty object is_empty: {}", obj.is_empty());

    let num = Node::from(42);
    println!("  Number len: {:?}", num.len());
    println!("  Number is_empty: {}", num.is_empty());

    println!("\n=== Demo Complete ===");
}
