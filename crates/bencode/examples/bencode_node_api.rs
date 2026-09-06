//! Example demonstrating the Node API type checking, accessors, and utility methods.
//! This example showcases the various methods available on the Node enum for
//! inspecting and manipulating bencode data structures.

use bencode_lib::{make_node, Node};
use std::collections::HashMap;

fn main() {
    println!("=== Bencode Node API Examples ===\n");

    // Example 1: Type checking methods
    demonstrate_type_checking();

    // Example 2: Value accessor methods
    demonstrate_value_accessors();

    // Example 3: Dictionary access methods
    demonstrate_dictionary_access();

    // Example 4: Utility methods (len, is_empty, type_name)
    demonstrate_utility_methods();

    // Example 5: Display trait formatting
    demonstrate_display_formatting();
}

/// Demonstrates the type checking methods: is_integer, is_string, is_list, is_dictionary, is_none
fn demonstrate_type_checking() {
    println!("--- Type Checking Methods ---");

    let integer_node = Node::Integer(42);
    let string_node = Node::Str("hello".to_string());
    let list_node = Node::List(vec![Node::Integer(1), Node::Integer(2)]);
    let dict_node = Node::Dictionary(HashMap::new());
    let none_node = Node::None;

    println!(
        "Integer node: is_integer={}, is_string={}",
        integer_node.is_integer(),
        integer_node.is_string()
    );
    println!(
        "String node: is_string={}, is_list={}",
        string_node.is_string(),
        string_node.is_list()
    );
    println!(
        "List node: is_list={}, is_dictionary={}",
        list_node.is_list(),
        list_node.is_dictionary()
    );
    println!(
        "Dictionary node: is_dictionary={}, is_none={}",
        dict_node.is_dictionary(),
        dict_node.is_none()
    );
    println!("None node: is_none={}\n", none_node.is_none());
}

/// Demonstrates the value accessor methods: as_integer, as_string, as_list, as_dictionary
fn demonstrate_value_accessors() {
    println!("--- Value Accessor Methods ---");

    let integer_node = Node::Integer(100);
    let string_node = Node::Str("bencode".to_string());
    let list_node = make_node(vec![make_node(1), make_node(2), make_node(3)]);

    // Safe access to values
    if let Some(value) = integer_node.as_integer() {
        println!("Integer value: {}", value);
        println!("Doubled: {}", value * 2);
    }

    if let Some(value) = string_node.as_string() {
        println!("String value: {}", value);
        println!("Uppercase: {}", value.to_uppercase());
    }

    if let Some(list) = list_node.as_list() {
        println!("List has {} elements", list.len());
        for (i, item) in list.iter().enumerate() {
            if let Some(val) = item.as_integer() {
                println!("  Element {}: {}", i, val);
            }
        }
    }

    // Attempting to access wrong type returns None
    println!(
        "Trying to access integer as string: {:?}",
        integer_node.as_string()
    );
    println!();
}

/// Demonstrates dictionary access methods: get and get_mut
fn demonstrate_dictionary_access() {
    println!("--- Dictionary Access Methods ---");

    let mut dict = HashMap::new();
    dict.insert("name".to_string(), make_node("Alice"));
    dict.insert("age".to_string(), make_node(30));
    dict.insert(
        "hobbies".to_string(),
        make_node(vec![make_node("reading"), make_node("coding")]),
    );

    let mut node = Node::Dictionary(dict);

    // Reading values with get()
    if let Some(name) = node.get("name") {
        println!("Name: {}", name);
    }

    if let Some(age) = node.get("age") {
        if let Some(age_val) = age.as_integer() {
            println!("Age: {}", age_val);
        }
    }

    if let Some(hobbies) = node.get("hobbies") {
        println!("Hobbies: {}", hobbies);
    }

    // Non-existent key returns None
    println!("Email exists: {}", node.get("email").is_some());

    // Modifying values with get_mut()
    if let Some(age_node) = node.get_mut("age") {
        *age_node = make_node(31); // Birthday!
        println!("Updated age to: {}", age_node);
    }

    println!();
}

/// Demonstrates utility methods: len, is_empty, type_name
fn demonstrate_utility_methods() {
    println!("--- Utility Methods ---");

    let empty_list = Node::List(vec![]);
    let filled_list = make_node(vec![make_node(1), make_node(2), make_node(3)]);

    let empty_dict = Node::Dictionary(HashMap::new());
    let mut filled_dict = Node::Dictionary(HashMap::new());
    if let Node::Dictionary(ref mut dict) = filled_dict {
        dict.insert("key1".to_string(), make_node("value1"));
        dict.insert("key2".to_string(), make_node("value2"));
    }

    println!(
        "Empty list: len={}, is_empty={}",
        empty_list.len(),
        empty_list.is_empty()
    );
    println!(
        "Filled list: len={}, is_empty={}",
        filled_list.len(),
        filled_list.is_empty()
    );
    println!(
        "Empty dict: len={}, is_empty={}",
        empty_dict.len(),
        empty_dict.is_empty()
    );
    println!(
        "Filled dict: len={}, is_empty={}",
        filled_dict.len(),
        filled_dict.is_empty()
    );

    // type_name returns the variant name
    println!("\nType names:");
    println!("Integer: {}", Node::Integer(42).type_name());
    println!("String: {}", Node::Str("test".to_string()).type_name());
    println!("List: {}", Node::List(vec![]).type_name());
    println!(
        "Dictionary: {}",
        Node::Dictionary(HashMap::new()).type_name()
    );
    println!("None: {}", Node::None.type_name());

    println!();
}

/// Demonstrates the Display trait for human-readable output
fn demonstrate_display_formatting() {
    println!("--- Display Trait Formatting ---");

    let integer = Node::Integer(42);
    let string = Node::Str("hello world".to_string());
    let list = make_node(vec![make_node(1), make_node(2), make_node(3)]);

    let mut dict = HashMap::new();
    dict.insert("name".to_string(), make_node("Bob"));
    dict.insert("score".to_string(), make_node(95));
    let dictionary = Node::Dictionary(dict);

    println!("Integer: {}", integer);
    println!("String: {}", string);
    println!("List: {}", list);
    println!("Dictionary: {}", dictionary);
    println!("None: {}", Node::None);

    // Display works great for debugging and logging
    println!("\nComplex nested structure:");
    let complex = make_node(vec![
        make_node("item1"),
        make_node(123),
        make_node(vec![make_node("nested"), make_node("list")]),
    ]);
    println!("{}", complex);
}
