//! Demonstrates programmatic JSON construction using Node types
//!
//! Shows various ways to build JSON structures in memory using the Node API,
//! including type conversions, indexing, and builder patterns.

use json_lib::{BufferDestination, Node, Numeric};
use std::collections::HashMap;

fn main() {
    println!("=== JSON Node Builder Demo ===\n");

    // Example 1: Creating nodes from primitive values
    println!("1. Creating nodes from primitives:");
    let str_node = Node::from("Hello, World!");
    let int_node = Node::from(42i32);
    let float_node = Node::from(3.14f64);
    let bool_node = Node::from(true);
    let null_node = Node::None;

    println!("  String: {:?}", str_node.as_str());
    println!("  Integer: {:?}", int_node.as_number());
    println!("  Float: {:?}", float_node.as_number());
    println!("  Boolean: {:?}", bool_node.as_bool());
    println!("  Null: {}", null_node.is_null());

    // Example 2: Building arrays
    println!("\n2. Building arrays:");
    let simple_array = Node::Array(vec![
        Node::from(1),
        Node::from(2),
        Node::from(3),
    ]);

    let mixed_array = Node::Array(vec![
        Node::from("text"),
        Node::from(42),
        Node::from(true),
        Node::None,
    ]);

    // Using From trait for Vec
    let converted_array: Node = vec![10, 20, 30].into();

    println!("  Simple array length: {}", simple_array.as_array().unwrap().len());
    println!("  Mixed array has string: {}", mixed_array[0].is_string());
    println!("  Converted array: {:?}", converted_array.as_array());

    // Example 3: Building objects
    println!("\n3. Building objects:");
    let mut person_map = HashMap::new();
    person_map.insert("name".to_string(), Node::from("Alice"));
    person_map.insert("age".to_string(), Node::from(30));
    person_map.insert("active".to_string(), Node::from(true));
    let person = Node::Object(person_map);

    println!("  Person name: {:?}", person["name"].as_str());
    println!("  Person age: {:?}", person["age"].as_number());

    // Example 4: Nested structures
    println!("\n4. Building nested structures:");
    
    let mut address = HashMap::new();
    address.insert("street".to_string(), Node::from("123 Main St"));
    address.insert("city".to_string(), Node::from("Springfield"));
    address.insert("zip".to_string(), Node::from("12345"));
    
    let skills = vec![
        Node::from("Rust"),
        Node::from("Python"),
        Node::from("JavaScript"),
    ];
    
    let mut employee = HashMap::new();
    employee.insert("id".to_string(), Node::from(1001));
    employee.insert("name".to_string(), Node::from("Bob"));
    employee.insert("address".to_string(), Node::Object(address));
    employee.insert("skills".to_string(), Node::Array(skills));
    employee.insert("salary".to_string(), Node::Number(Numeric::Float(75000.50)));
    
    let employee_node = Node::Object(employee);
    
    println!("  Employee structure created with {} top-level fields",
             employee_node.as_object().unwrap().len());

    // Example 5: Indexing and accessing nested data
    println!("\n5. Indexing and navigation:");
    
    // Array indexing
    let arr = Node::Array(vec![
        Node::from("first"),
        Node::from("second"),
        Node::from("third"),
    ]);
    println!("  arr[1]: {:?}", arr[1].as_str());
    
    // Object indexing
    if let Some(name) = employee_node.get("name") {
        println!("  employee.name: {:?}", name.as_str());
    }
    
    // Safe navigation with at() and get()
    if let Some(city) = employee_node.get("address")
        .and_then(|addr| addr.get("city")) {
        println!("  employee.address.city: {:?}", city.as_str());
    }
    
    if let Some(first_skill) = employee_node.get("skills")
        .and_then(|s| s.at(0)) {
        println!("  employee.skills[0]: {:?}", first_skill.as_str());
    }

    // Example 6: Mutable operations
    println!("\n6. Mutable modifications:");
    let mut config = Node::Object(HashMap::new());
    
    // Add fields to object
    if let Some(obj) = config.as_object_mut() {
        obj.insert("version".to_string(), Node::from("1.0"));
        obj.insert("debug".to_string(), Node::from(true));
        obj.insert("timeout".to_string(), Node::from(30));
    }
    
    // Modify array elements
    let mut numbers = Node::Array(vec![
        Node::from(1),
        Node::from(2),
        Node::from(3),
    ]);
    
    if let Some(arr) = numbers.as_array_mut() {
        arr.push(Node::from(4));
        arr[0] = Node::from(100);
    }
    
    println!("  Modified first element: {:?}", numbers[0].as_number());
    println!("  Array length after push: {}", numbers.as_array().unwrap().len());

    // Example 7: Different numeric types
    println!("\n7. Numeric type conversions:");
    let nums = Node::Object({
        let mut map = HashMap::new();
        map.insert("i8".to_string(), Node::Number(Numeric::Int8(127)));
        map.insert("i16".to_string(), Node::Number(Numeric::Int16(32767)));
        map.insert("i32".to_string(), Node::Number(Numeric::Int32(2147483647)));
        map.insert("i64".to_string(), Node::Number(Numeric::Integer(9223372036854775807)));
        map.insert("u8".to_string(), Node::Number(Numeric::Byte(255)));
        map.insert("u16".to_string(), Node::Number(Numeric::UInt16(65535)));
        map.insert("u32".to_string(), Node::Number(Numeric::UInt32(4294967295)));
        map.insert("u64".to_string(), Node::Number(Numeric::UInteger(18446744073709551615)));
        map.insert("float".to_string(), Node::Number(Numeric::Float(3.14159)));
        map
    });
    
    println!("  Created object with {} numeric types", nums.as_object().unwrap().len());

    // Example 8: Type checking
    println!("\n8. Type checking:");
    let values = Node::Array(vec![
        Node::from("string"),
        Node::from(42),
        Node::from(true),
        Node::Array(vec![]),
        Node::Object(HashMap::new()),
        Node::None,
    ]);
    
    if let Some(arr) = values.as_array() {
        for (i, val) in arr.iter().enumerate() {
            let type_str = if val.is_string() {
                "string"
            } else if val.is_number() {
                "number"
            } else if val.is_boolean() {
                "boolean"
            } else if val.is_array() {
                "array"
            } else if val.is_object() {
                "object"
            } else if val.is_null() {
                "null"
            } else {
                "unknown"
            };
            println!("  values[{}] is {}", i, type_str);
        }
    }

    // Example 9: Complex example - API response structure
    println!("\n9. Building a complex API response:");
    
    let user1 = {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Node::from(1));
        map.insert("username".to_string(), Node::from("alice"));
        map.insert("email".to_string(), Node::from("alice@example.com"));
        Node::Object(map)
    };
    
    let user2 = {
        let mut map = HashMap::new();
        map.insert("id".to_string(), Node::from(2));
        map.insert("username".to_string(), Node::from("bob"));
        map.insert("email".to_string(), Node::from("bob@example.com"));
        Node::Object(map)
    };
    
    let api_response = {
        let mut map = HashMap::new();
        map.insert("success".to_string(), Node::from(true));
        map.insert("count".to_string(), Node::from(2));
        map.insert("data".to_string(), Node::Array(vec![user1, user2]));
        map.insert("timestamp".to_string(), Node::from(1699564800));
        Node::Object(map)
    };
    
    let mut output = BufferDestination::new();
    json_lib::print(&api_response, &mut output, 2);
    println!("{}", output.to_string());

    println!("\n=== Demo Complete ===");
}
