//! TOML stringification module provides functionality for converting Node structures into TOML format.
//!
//! This module implements conversion of various Node types into their TOML string representations:
//! - Objects are converted to TOML tables
//! - Arrays are converted to TOML arrays (must contain elements of the same type)
//! - Primitive values (strings, numbers, booleans) are converted to their TOML equivalents
//! - Nested structures are handled with proper table syntax
//! - Array tables are supported for collections of objects
//!
//! The module ensures compliance with TOML specification including
//! - Proper quoting of strings
//! - Correct table and array table syntax
//! - Type consistency in arrays
//! - Proper nesting of tables and sub-tables
//!

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::collections::{BTreeMap, HashMap};

use crate::Node;
use crate::io::traits::{BencodeWrite, IDestination};
use crate::stringify::common::escape_string;

/// Converts a Node structure to a TOML formatted string
///
/// # Anode` - The root Node to convert
/// * `destination` - The destination to write the TOML string to
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if the root node is not an Object
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Dictionary(dict) => stringify_object(dict, "", destination),
        _ => Err("TOML format requires a dictionary at the root level".to_string()),
    }
}

/// Converts a Node value to its TOML string representation
///
/// # Arguments
/// * `value` - The Node to convert
/// * `add_cr` - Whether to add a carriage return after the value
/// * `destination` - The destination to write to
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if the array contains mixed types
fn stringify_value(
    value: &Node,
    add_cr: bool,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    match value {
        Node::Str(s) => stringify_str(s, destination),
        Node::Integer(value) => stringify_number(value, destination),
        Node::List(items) => stringify_array(items, destination)?,
        Node::None => destination.write_bytes(b"null"),
        Node::Dictionary(_) => return Ok(()), // Handled separately for table syntax
    }
    if add_cr {
        destination.write_bytes(b"\n");
    }
    Ok(())
}
/// Converts a string value to its TOML string representation with quotes
///
/// # Arguments
/// * `s` - The string to convert
/// * `destination` - The destination to write to
fn stringify_str(s: &str, destination: &mut (impl BencodeWrite + ?Sized)) {
    destination.write_bytes(b"\"");
    escape_string(s, destination);
    destination.write_bytes(b"\"");
}

/// Converts a numeric value to its TOML string representation
/// Handles different numeric types including integers, floats, and bytes
///
/// # Arguments
/// * `value` - The numeric value to convert
/// * `destination` - The destination to write to
fn stringify_number(value: &i64, destination: &mut (impl BencodeWrite + ?Sized)) {
    destination.write_bytes(value.to_string().as_bytes())
}

/// Converts an array of Nodes to its TOML string representation
/// Ensures all array elements are of the same type as required by TOML spec
///
/// # Arguments
/// * `items` - The vector of Nodes to convert
/// * `destination` - The destination to write to
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if the array contains mixed types
fn stringify_array(items: &Vec<Node>, destination: &mut (impl BencodeWrite + ?Sized)) -> Result<(), String> {
    let first_type = get_node_type(&items[0]);

    for item in items {
        if get_node_type(item) != first_type {
            return Err("TOML lists must contain elements of the same type".to_string());
        }
    }

    destination.write_bytes(b"[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            destination.write_bytes(b", ");
        }
        stringify_value(item, false, destination)?;
    }
    destination.write_bytes(b"]");
    Ok(())
}

/// Returns the type of Node as a static string
/// Used for type checking in arrays
///
/// # Arguments
/// * `node` - The Node to get the type of
///
/// # Returns
/// A string representing the Node type
fn get_node_type(node: &Node) -> &'static str {
    match node {
        Node::Str(_) => "string",
        Node::Integer(_) => "integer",
        Node::List(_) => "list",
        Node::Dictionary(_) => "object",
        Node::None => "null",
    }
}
/// Converts a key-value pair to its TOML string representation
/// Handles table headers and nested structures
///
/// # Arguments
/// * `prefix` - The current table path prefix
/// * `destination` - The destination to write to
/// * `is_first` - Whether this is the first entry in a table
/// * `key` - The key of the pair
/// * `value` - The value Node
///
/// # Returns
/// * `Ok(())` if successful
fn stringify_key_value_pair(
    prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
    is_first: &mut bool,
    key: &String,
    value: &Node,
) -> Result<(), String> {
    if !prefix.is_empty() && *is_first {
        destination.write_bytes(b"[");
        destination.write_bytes(prefix.as_bytes());
        destination.write_bytes(b"]\n");
        *is_first = false;
    }

    destination.write_bytes(key.as_bytes());
    destination.write_bytes(b" = ");
    stringify_value(value, true, destination)?;

    Ok(())
}

/// Converts a HashMap representing a TOML table to its string representation
/// Handles nested tables, array tables, and maintains proper TOML formatting.
/// This function processes the input dictionary in multiple steps:
/// 1. Sorts key-value pairs for consistent output
/// 2. Processes simple key-value pairs first
/// 3. Handles nested tables
/// 4. Handles array tables
///
/// # Arguments
/// * `dict` - The HashMap to convert containing key-value pairs
/// * `prefix` - The current table path prefix for nested structures
/// * `destination` - The destination to write the formatted TOML output
///
/// # Returns
/// * `Ok(())` if conversion was successful
/// * `Err(String)` if an error occurred during conversion
fn stringify_object(
    dict: &HashMap<String, Node>,
    prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    if dict.is_empty() {
        return Ok(());
    }

    let dict_sorted: BTreeMap<_, _> = dict.iter().collect();
    let (tables_dict, array_tables_dict) = get_tables_in_dict(dict);
    let tables: BTreeMap<_, _> = tables_dict.iter().map(|(k, v)| (k, *v)).collect();
    let array_tables: BTreeMap<_, _> = array_tables_dict.iter().map(|(k, v)| (k, *v)).collect();

    process_key_value_pairs(&dict_sorted, prefix, destination)?;
    process_nested_tables(&tables, prefix, destination)?;
    process_array_tables(&array_tables, prefix, destination)?;

    Ok(())
}
/// Gets tables and array tables from a dictionary by categorizing its entries
/// This helper function processes a dictionary and identifies table entries,
/// separating them into regular tables and array tables.
///
/// # Arguments
/// * `dict` - The dictionary to process
///
/// # Returns
/// A tuple containing:
/// * HashMap of regular tables
/// * HashMap of array tables
fn get_tables_in_dict(
    dict: &HashMap<String, Node>,
) -> (
    HashMap<String, &HashMap<String, Node>>,
    HashMap<String, &Vec<Node>>,
) {
    let mut tables = HashMap::new();
    let mut array_tables = HashMap::new();

    for (key, value) in dict {
        match value {
            Node::Dictionary(nested) => {
                tables.insert(key.clone(), nested);
            }
            Node::List(items) if items.iter().all(|item| matches!(item, Node::Dictionary(_))) => {
                array_tables.insert(key.clone(), items);
            }
            _ => {}
        }
    }

    (tables, array_tables)
}

/// Processes key-value pairs in a TOML structure by iterating through sorted dictionary entries
/// This function handles simple key-value pairs while skipping tables and array tables
/// that need special processing
///
/// # Arguments
/// * `dict_sorted` - BTreeMap containing sorted key-value pairs to process
/// * `prefix` - Current table path prefix for nested structures
/// * `destination` - Destination to write the formatted TOML output
/// * `is_first` - Mutable flag indicating if this is the first entry in current table
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_key_value_pairs<'a>(
    dict_sorted: &BTreeMap<&'a String, &'a Node>,
    prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    for (key, value) in dict_sorted {
        match value {
            Node::Dictionary(_) => {
                continue;
            }
            Node::List(items) => {
                if items.iter().all(|item| matches!(item, Node::Dictionary(_))) {
                    continue;
                }
            }
            _ => {}
        }
        let mut is_first = true;
        stringify_key_value_pair(prefix, destination, &mut is_first, key, value)?;
    }
    Ok(())
}

/// Processes nested tables in a TOML structure, handling proper formatting and recursion
/// This function iterates through the sorted tables and processes each nested table
/// while maintaining proper TOML table hierarchy and formatting
///
/// # Arguments
/// * `tables` - BTreeMap containing the nested table structures to process
/// * `prefix` - Current table path prefix for nested structures
/// * `destination` - Destination to write the formatted TOML output
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_nested_tables(
    tables: &BTreeMap<&String, &HashMap<String, Node>>,
    prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    for (key, nested) in tables {
        let new_prefix = calculate_prefix(prefix, key);
        stringify_object(nested, &new_prefix, destination)?;
    }
    Ok(())
}

/// Processes array tables in a TOML structure, handling proper formatting and recursion
/// This function iterates through sorted array tables and processes each table entry
/// while maintaining proper TOML array table syntax and hierarchy
///
/// # Arguments
/// * `array_tables` - BTreeMap containing the array table structures to process
/// * `prefix` - Current table path prefix for nested structures
/// * `destination` - Destination to write the formatted TOML output
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_array_tables(
    array_tables: &BTreeMap<&String, &Vec<Node>>,
    prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    for (key, items) in array_tables {
        for item in &**items {
            if let Node::Dictionary(nested) = item {
                let new_prefix = calculate_prefix(prefix, key);
                destination.write_bytes(b"[[");
                destination.write_bytes(new_prefix.as_bytes());
                destination.write_bytes(b"]]\n");
                process_nested_array_table(nested, &new_prefix, destination)?;
            }
        }
    }
    Ok(())
}

/// Processes a nested array table by handling both simple values and nested objects
/// This function sorts the input HashMap and processes its contents in two phases:
/// 1. Processes simple key-value pairs (non-object types)
/// 2. Processes nested objects and arrays of objects
///
/// # Arguments
/// * `nested` - HashMap containing the nested array table key-value pairs
/// * `new_prefix` - Current table path prefix for the nested structure
/// * `destination` - Destination to write the formatted TOML output
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_nested_array_table(
    nested: &HashMap<String, Node>,
    new_prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    let nested_sorted: BTreeMap<_, _> = nested.iter().collect();
    let _ = process_simple_values(&nested_sorted, destination)?;
    process_nested_objects(&nested_sorted, new_prefix, destination)?;
    Ok(())
}

/// Processes simple (non-object, non-array) key-value pairs in a TOML structure
/// This function handles basic value types like strings, numbers, and booleans
///
/// # Arguments
/// * `nested_sorted` - BTreeMap containing sorted key-value pairs to process
/// * `destination` - Destination to write the formatted TOML output
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_simple_values(
    nested_sorted: &BTreeMap<&String, &Node>,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    for (inner_key, inner_value) in nested_sorted {
        match inner_value {
            Node::Dictionary(_) => {}

            Node::Integer(_) | Node::Str(_) => {
                let mut is_first = true;
                stringify_key_value_pair("", destination, &mut is_first, inner_key, inner_value)?;
            }
            Node::List(items) => {
                if items
                    .iter()
                    .all(|item| matches!(item, Node::Integer(_) | Node::Str(_)))
                {
                    let mut is_first = true;
                    stringify_key_value_pair(
                        "",
                        destination,
                        &mut is_first,
                        inner_key,
                        inner_value,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Processes nested objects and array tables within a TOML structure
/// This function handles complex nested structures by recursively processing them
///
/// # Arguments
/// * `nested_sorted` - BTreeMap containing sorted key-value pairs with nested structures
/// * `new_prefix` - Current table path prefix for the nested structure
/// * `destination` - Destination to write the formatted TOML output
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_nested_objects(
    nested_sorted: &BTreeMap<&String, &Node>,
    new_prefix: &str,
    destination: &mut (impl BencodeWrite + ?Sized),
) -> Result<(), String> {
    for (inner_key, inner_value) in nested_sorted {
        match inner_value {
            Node::Dictionary(inner_nested) => {
                let inner_prefix = format!("{}.{}", new_prefix, inner_key);
                stringify_object(inner_nested, &inner_prefix, destination)?;
            }
            Node::List(inner_items)
                if inner_items
                    .iter()
                    .all(|item| matches!(item, Node::Dictionary(_))) =>
            {
                for inner_item in inner_items {
                    if let Node::Dictionary(deepest) = inner_item {
                        let inner_prefix = format!("{}.{}", new_prefix, inner_key);
                        stringify_object(&deepest, &inner_prefix, destination)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Calculates a new prefix for nested TOML tables by combining the current prefix with a key
///
/// # Arguments
/// * `prefix` - The current table path prefix. Empty string for root level
/// * `key` - The key to append to the prefix
///
/// # Returns
/// A new string containing the combined prefix path:
/// - If the prefix is empty, returns the key as-is
/// - If the prefix exists, returns "prefix.key"
fn calculate_prefix(prefix: &str, key: &String) -> String {
    let new_prefix = if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", prefix, key)
    };
    new_prefix
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::BufferDestination;
    use crate::nodes::node::make_node;
    use std::collections::HashMap;

    #[test]
    fn test_stringify_string() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), make_node("test"));
        let node = make_node(dict);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "key = \"test\"\n");
    }

    #[test]
    fn test_stringify_integer() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), make_node(42));
        let node = make_node(dict);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "key = 42\n");
    }

    #[test]
    fn test_stringify_list() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            "key".to_string(),
            make_node(vec![make_node(1), make_node(2), make_node(3)]),
        );
        let node = make_node(dict);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "key = [1, 2, 3]\n");
    }

    #[test]
    fn test_stringify_dictionary() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), make_node("value"));
        let node = make_node(dict);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "key = \"value\"\n");
    }

    #[test]
    fn test_stringify_none() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Node::None);
        let node = make_node(dict);
        stringify(&node, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "key = null\n");
    }

    #[test]
    fn test_non_dictionary_root_fails() {
        let mut destination = BufferDestination::new();
        let node = make_node(42); // Integer instead of Dictionary
        let result = stringify(&node, &mut destination);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML format requires a dictionary at the root level"
        );
    }

    #[test]
    fn test_non_dictionary_string_root_fails() {
        let mut destination = BufferDestination::new();
        let node = make_node("test");
        let result = stringify(&node, &mut destination);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_dictionary_list_root_fails() {
        let mut destination = BufferDestination::new();
        let node = make_node(vec![make_node(1), make_node(2)]);
        let result = stringify(&node, &mut destination);
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_type_array_fails() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        // Array with mixed types: integer and string
        dict.insert(
            "mixed".to_string(),
            make_node(vec![make_node(1), make_node("text")]),
        );
        let node = make_node(dict);
        let result = stringify(&node, &mut destination);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("must contain elements of the same type")
        );
    }

    #[test]
    fn test_array_with_mixed_types_int_list() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(
            "array".to_string(),
            make_node(vec![make_node(1), make_node(vec![make_node(2)])]),
        );
        let node = make_node(dict);
        let result = stringify(&node, &mut destination);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_dictionary() {
        let mut destination = BufferDestination::new();
        let dict: HashMap<String, Node> = HashMap::new();
        let node = make_node(dict);
        let result = stringify(&node, &mut destination);
        assert!(result.is_ok());
        assert_eq!(destination.to_string(), "");
    }

    // --- Integer edge cases ---

    #[test]
    fn test_stringify_integer_zero() {
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), Node::Integer(0));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "n = 0\n");
    }

    #[test]
    fn test_stringify_integer_negative() {
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), Node::Integer(-42));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "n = -42\n");
    }

    #[test]
    fn test_stringify_integer_max_i64() {
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), Node::Integer(i64::MAX));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("n = {}\n", i64::MAX));
    }

    #[test]
    fn test_stringify_integer_min_i64() {
        let mut dict = HashMap::new();
        dict.insert("n".to_string(), Node::Integer(i64::MIN));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("n = {}\n", i64::MIN));
    }

    // --- String edge cases ---

    #[test]
    fn test_stringify_empty_string_value() {
        let mut dict = HashMap::new();
        dict.insert("s".to_string(), make_node(""));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "s = \"\"\n");
    }

    #[test]
    fn test_stringify_string_with_special_chars() {
        let mut dict = HashMap::new();
        dict.insert("s".to_string(), make_node("say \"hi\""));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "s = \"say \\\"hi\\\"\"\n");
    }

    #[test]
    fn test_stringify_string_with_newline() {
        let mut dict = HashMap::new();
        dict.insert("s".to_string(), make_node("a\nb"));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "s = \"a\\u000ab\"\n");
    }

    // --- List edge cases ---

    #[test]
    fn test_stringify_empty_list_of_integers() {
        // An empty list - stringify_array panics if items is empty because it accesses items[0]
        // Just ensure a non-empty same-type list works with one element
        let mut dict = HashMap::new();
        dict.insert("nums".to_string(), make_node(vec![make_node(5i64)]));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "nums = [5]\n");
    }

    #[test]
    fn test_stringify_string_list() {
        let mut dict = HashMap::new();
        dict.insert(
            "words".to_string(),
            make_node(vec![make_node("foo"), make_node("bar")]),
        );
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "words = [\"foo\", \"bar\"]\n");
    }

    // --- None value ---

    #[test]
    fn test_none_value_writes_null() {
        let mut dict = HashMap::new();
        dict.insert("x".to_string(), Node::None);
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "x = null\n");
    }

    // --- Non-dict root failures ---

    #[test]
    fn test_none_root_fails() {
        let mut dest = BufferDestination::new();
        assert!(stringify(&Node::None, &mut dest).is_err());
    }

    // --- Nested table ---

    #[test]
    fn test_nested_table() {
        let mut inner = HashMap::new();
        inner.insert("x".to_string(), make_node(1i64));
        let mut outer = HashMap::new();
        outer.insert("section".to_string(), make_node(inner));
        let mut dest = BufferDestination::new();
        stringify(&make_node(outer), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("[section]"));
        assert!(s.contains("x = 1"));
    }

    #[test]
    fn test_deeply_nested_table() {
        let mut innermost = HashMap::new();
        innermost.insert("val".to_string(), make_node(99i64));
        let mut middle = HashMap::new();
        middle.insert("deep".to_string(), make_node(innermost));
        let mut outer = HashMap::new();
        outer.insert("mid".to_string(), make_node(middle));
        let mut dest = BufferDestination::new();
        stringify(&make_node(outer), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("val = 99"));
    }

    // --- Array table (list of dicts) ---

    #[test]
    fn test_array_table_single_entry() {
        let mut item = HashMap::new();
        item.insert("name".to_string(), make_node("Alice"));
        let list = vec![make_node(item)];
        let mut outer = HashMap::new();
        outer.insert("people".to_string(), Node::List(list));
        let mut dest = BufferDestination::new();
        stringify(&make_node(outer), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.contains("[[people]]"));
        assert!(s.contains("name = \"Alice\""));
    }

    #[test]
    fn test_array_table_multiple_entries() {
        let mut item1 = HashMap::new();
        item1.insert("id".to_string(), make_node(1i64));
        let mut item2 = HashMap::new();
        item2.insert("id".to_string(), make_node(2i64));
        let list = vec![make_node(item1), make_node(item2)];
        let mut outer = HashMap::new();
        outer.insert("rows".to_string(), Node::List(list));
        let mut dest = BufferDestination::new();
        stringify(&make_node(outer), &mut dest).unwrap();
        let s = dest.to_string();
        assert_eq!(s.matches("[[rows]]").count(), 2);
    }

    // --- Multiple root-level keys sorted ---

    #[test]
    fn test_multiple_keys_output_sorted() {
        let mut dict = HashMap::new();
        dict.insert("z".to_string(), make_node(1i64));
        dict.insert("a".to_string(), make_node(2i64));
        let mut dest = BufferDestination::new();
        stringify(&make_node(dict), &mut dest).unwrap();
        let s = dest.to_string();
        let pos_a = s.find("a = ").unwrap();
        let pos_z = s.find("z = ").unwrap();
        assert!(pos_a < pos_z, "keys should be sorted alphabetically");
    }

    // --- calculate_prefix helper ---

    #[test]
    fn test_calculate_prefix_empty_prefix() {
        let key = "section".to_string();
        assert_eq!(calculate_prefix("", &key), "section");
    }

    #[test]
    fn test_calculate_prefix_with_existing_prefix() {
        let key = "sub".to_string();
        assert_eq!(calculate_prefix("top", &key), "top.sub");
    }

    #[test]
    fn test_calculate_prefix_deep_nesting() {
        let key = "leaf".to_string();
        assert_eq!(calculate_prefix("a.b.c", &key), "a.b.c.leaf");
    }

    // --- get_node_type helper ---

    #[test]
    fn test_get_node_type_string() {
        assert_eq!(get_node_type(&Node::Str("x".to_string())), "string");
    }

    #[test]
    fn test_get_node_type_integer() {
        assert_eq!(get_node_type(&Node::Integer(1)), "integer");
    }

    #[test]
    fn test_get_node_type_list() {
        assert_eq!(get_node_type(&Node::List(vec![])), "list");
    }

    #[test]
    fn test_get_node_type_dict() {
        assert_eq!(get_node_type(&Node::Dictionary(HashMap::new())), "object");
    }

    #[test]
    fn test_get_node_type_none() {
        assert_eq!(get_node_type(&Node::None), "null");
    }
}
