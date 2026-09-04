//! TOML stringification module provides functionality for converting Node structures into TOML format.
//!
//! This module implements conversion of various Node types into their TOML string representations:
//! - Objects are converted to TOML tables
//! - Arrays are converted to TOML arrays (must contain elements of the same type)
//! - Primitive values (strings, numbers, booleans) are converted to their TOML equivalents
//! - Nested structures are handled with proper table syntax
//! - Array tables are supported for collections of objects
//!
//! The module ensures compliance with TOML specification including:
//! - Proper quoting of strings
//! - Correct table and array table syntax
//! - Type consistency in arrays
//! - Proper nesting of tables and sub-tables
//!
use crate::io::traits::IDestination;
use crate::{Node, Numeric};

#[cfg(feature = "std")]
use std::collections::{BTreeMap, HashMap};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::{BTreeMap, BTreeMap as HashMap},
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Converts a Node structure to a TOML formatted string
///
/// # Arguments
/// * `node` - The root Node to convert
/// * `destination` - The destination to write the TOML string to
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if the root node is not an Object
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    match node {
        Node::Object(dict) => stringify_object(dict, "", destination),
        _ => Err("TOML format requires a Object at the root level".to_string()),
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
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    match value {
        Node::Str(s) => stringify_str(s, destination),
        Node::Boolean(b) => stringify_bool(b, destination),
        Node::Number(value) => stringify_number(value, destination),
        Node::Array(items) => stringify_array(items, destination)?,
        Node::None => destination.add_bytes("null"),
        Node::Object(_) => return Ok(()), // Handled separately for table syntax
    }
    if add_cr {
        destination.add_bytes("\n");
    }
    Ok(())
}
/// Converts a string value to its TOML string representation with quotes
///
/// # Arguments
/// * `s` - The string to convert
/// * `destination` - The destination to write to
fn stringify_str(s: &str, destination: &mut dyn IDestination) {
    destination.add_bytes("\"");
    destination.add_bytes(s);
    destination.add_bytes("\"");
}

/// Converts a boolean value to its TOML string representation
///
/// # Arguments
/// * `b` - The boolean to convert
/// * `destination` - The destination to write to
fn stringify_bool(b: &bool, destination: &mut dyn IDestination) {
    destination.add_bytes(if *b { "true" } else { "false" })
}

/// Converts a numeric value to its TOML string representation
/// Handles different numeric types including integers, floats, and bytes
///
/// # Arguments
/// * `value` - The numeric value to convert
/// * `destination` - The destination to write to
fn stringify_number(value: &Numeric, destination: &mut dyn IDestination) {
    match value {
        Numeric::Integer(n) => {
            let mut buf = itoa::Buffer::new();
            destination.add_bytes(buf.format(*n));
        }
        Numeric::UInteger(n) => {
            let mut buf = itoa::Buffer::new();
            destination.add_bytes(buf.format(*n));
        }
        Numeric::Float(f) => {
            let mut buf = dtoa::Buffer::new();
            destination.add_bytes(buf.format(*f));
        }
        Numeric::Byte(b) => {
            let mut buf = itoa::Buffer::new();
            destination.add_bytes(buf.format(*b));
        }
        Numeric::Int32(i) => {
            let mut buf = itoa::Buffer::new();
            destination.add_bytes(buf.format(*i));
        }
        Numeric::UInt32(u) => {
            let mut buf = itoa::Buffer::new();
            destination.add_bytes(buf.format(*u));
        }
        #[allow(unreachable_patterns)]
        _ => destination.add_bytes(&format!("{:?}", value)),
    }
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
fn stringify_array(items: &Vec<Node>, destination: &mut dyn IDestination) -> Result<(), String> {
    let first_type = get_node_type(&items[0]);

    for item in items {
        if get_node_type(item) != first_type {
            return Err("TOML arrays must contain elements of the same type".to_string());
        }
    }

    destination.add_bytes("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            destination.add_bytes(", ");
        }
        stringify_value(item, false, destination)?;
    }
    destination.add_bytes("]");
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
        Node::Boolean(_) => "boolean",
        Node::Number(_) => "number",
        Node::Array(_) => "array",
        Node::Object(_) => "object",
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
    destination: &mut dyn IDestination,
    is_first: &mut bool,
    key: &String,
    value: &Node,
) -> Result<(), String> {
    if !prefix.is_empty() && *is_first {
        destination.add_bytes("[");
        destination.add_bytes(prefix);
        destination.add_bytes("]\n");
        *is_first = false;
    }
    destination.add_bytes(key);
    destination.add_bytes(" = ");
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
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    if dict.is_empty() {
        return Ok(());
    }

    let dict_sorted: BTreeMap<_, _> = dict.iter().collect();
    let (tables_dict, array_tables_dict) = get_tables_and_arrays(dict);
    let tables: BTreeMap<_, _> = tables_dict.iter().map(|(k, v)| (k, v)).collect();
    let array_tables: BTreeMap<_, _> = array_tables_dict.iter().map(|(k, v)| (k, v)).collect();

    process_key_value_pairs(&dict_sorted, prefix, destination)?;
    process_nested_tables(&tables, prefix, destination)?;
    process_array_tables(&array_tables, prefix, destination)?;

    Ok(())
}

/// Separates a dictionary into tables and array tables based on their content type
/// Tables are key-value pairs where the value is an Object
/// Array tables are key-value pairs where the value is an Array of Objects
///
/// # Arguments
/// * `dict` - The source HashMap to process containing key-value pairs
///
/// # Returns
/// A tuple containing:
/// * First element: BTreeMap of nested table structures (key -> HashMap)
/// * Second element: BTreeMap of array table structures (key -> Vec<Node>)
fn get_tables_and_arrays(
    dict: &HashMap<String, Node>,
) -> (
    BTreeMap<String, HashMap<String, Node>>,
    BTreeMap<String, Vec<Node>>,
) {
    let dict_sorted: BTreeMap<_, _> = dict.iter().collect();
    let mut tables = BTreeMap::new();
    let mut array_tables = BTreeMap::new();

    for (key, value) in dict_sorted {
        match value {
            Node::Object(nested) => {
                tables.insert(key.clone(), nested.clone());
            }
            Node::Array(items) if items.iter().all(|item| matches!(item, Node::Object(_))) => {
                array_tables.insert(key.clone(), items.clone());
            }
            _ => {}
        }
    }

    (tables, array_tables)
}

/// Processes key-value pairs in a TOML structure, skipping nested tables and array tables
/// This function handles only simple key-value pairs (non-object, non-array table types)
///
/// # Arguments
/// * `dict_sorted` - BTreeMap containing sorted key-value pairs to process
/// * `prefix` - Current table path prefix for nested structures
/// * `destination` - Destination to write the formatted TOML output
/// * `is_first` - Mutable boolean indicating if this is the first entry in a table
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(String)` if an error occurred during processing
fn process_key_value_pairs<'a>(
    dict_sorted: &BTreeMap<&'a String, &'a Node>,
    prefix: &str,
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    let mut is_first = true;
    for (key, value) in dict_sorted {
        match value {
            Node::Object(_) => {
                continue;
            }
            Node::Array(items) => {
                if items.iter().all(|item| matches!(item, Node::Object(_))) {
                    continue;
                }
            }
            _ => {}
        }
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
    destination: &mut dyn IDestination,
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
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    for (key, items) in array_tables {
        for item in &**items {
            if let Node::Object(nested) = item {
                let new_prefix = calculate_prefix(prefix, key);
                destination.add_bytes("[[");
                destination.add_bytes(&new_prefix);
                destination.add_bytes("]]\n");
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
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    let nested_sorted: BTreeMap<_, _> = nested.iter().collect();
    process_simple_values(&nested_sorted, destination)?;
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
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    for (inner_key, inner_value) in nested_sorted {
        match inner_value {
            Node::Object(_) => {}
            Node::Number(_) | &&Node::Str(_) | &&Node::Boolean(_) => {
                let mut is_first = true;
                stringify_key_value_pair("", destination, &mut is_first, inner_key, inner_value)?;
            }
            Node::Array(items) => {
                if items
                    .iter()
                    .all(|item| matches!(item, Node::Number(_) | Node::Str(_) | Node::Boolean(_)))
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
            Node::None => {}
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
    destination: &mut dyn IDestination,
) -> Result<(), String> {
    for (inner_key, inner_value) in nested_sorted {
        match inner_value {
            Node::Object(inner_nested) => {
                let inner_prefix = format!("{}.{}", new_prefix, inner_key);
                stringify_object(inner_nested, &inner_prefix, destination)?;
            }
            Node::Array(inner_items)
                if inner_items
                    .iter()
                    .all(|item| matches!(item, Node::Object(_))) =>
            {
                for inner_item in inner_items {
                    if let Node::Object(deepest) = inner_item {
                        let inner_prefix = format!("{}.{}", new_prefix, inner_key);
                        stringify_object(deepest, &inner_prefix, destination)?;
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
    use crate::nodes::node::make_node;
    use crate::{BufferDestination, BufferSource};
    use std::collections::HashMap;

    #[test]
    fn test_stringify_array() {
        let mut dest = BufferDestination::new();
        let result = stringify(
            &Node::Array(vec![
                Node::Str("a".to_string()),
                Node::Number(Numeric::Float(1.0)),
                Node::Boolean(true),
            ]),
            &mut dest,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML format requires a Object at the root level"
        );
    }

    #[test]
    fn test_stringify_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Node::Str("value".to_string()));
        let mut dest = BufferDestination::new();
        let hashmap = map.into_iter().collect::<HashMap<_, _>>();
        let _ = stringify(&Node::Object(hashmap), &mut dest);
        assert_eq!(dest.to_string(), "key = \"value\"\n");
    }

    #[test]
    fn test_stringify_non_object_root() {
        let mut dest = BufferDestination::new();
        let result = stringify(&Node::Str("test".to_string()), &mut dest);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML format requires a Object at the root level"
        );
    }

    #[test]
    fn test_stringify_nested_object() {
        let mut inner = HashMap::new();
        inner.insert("inner_key".to_string(), Node::Str("value".to_string()));
        let inner_hashmap = inner.into_iter().collect::<HashMap<_, _>>();
        let mut outer = HashMap::new();
        outer.insert("outer".to_string(), Node::Object(inner_hashmap));
        let outer_hashmap = outer.into_iter().collect::<HashMap<_, _>>();
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(outer_hashmap), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[outer]\ninner_key = \"value\"\n");
    }
    // ...existing code...

    #[test]
    fn test_stringify_deeply_nested_object() {
        let mut level3 = HashMap::new();
        level3.insert("deep_key".to_string(), Node::Number(Numeric::Integer(123)));
        let level3 = Node::Object(level3);

        let mut level2 = HashMap::new();
        level2.insert("level3".to_string(), level3);
        let level2 = Node::Object(level2);

        let mut level1 = HashMap::new();
        level1.insert("level2".to_string(), level2);
        let level1 = Node::Object(level1);

        let mut root = HashMap::new();
        root.insert("level1".to_string(), level1);

        let mut dest = BufferDestination::new();
        stringify(&Node::Object(root), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[level1.level2.level3]\ndeep_key = 123\n");
    }

    #[test]
    fn test_stringify_object_with_multiple_nested_tables_and_values() {
        let mut address = HashMap::new();
        address.insert("city".to_string(), Node::Str("Paris".to_string()));
        address.insert("zip".to_string(), Node::Number(Numeric::Integer(75000)));

        let mut profile = HashMap::new();
        profile.insert("name".to_string(), Node::Str("Alice".to_string()));
        profile.insert("age".to_string(), Node::Number(Numeric::Integer(30)));
        profile.insert("address".to_string(), Node::Object(address));

        let mut root = HashMap::new();
        root.insert("profile".to_string(), Node::Object(profile));
        root.insert("active".to_string(), Node::Boolean(true));

        let mut dest = BufferDestination::new();
        stringify(&Node::Object(root), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "active = true\n[profile]\nage = 30\nname = \"Alice\"\n[profile.address]\ncity = \"Paris\"\nzip = 75000\n"
        );
    }

    #[test]
    fn test_stringify_simple_array_of_tables() {
        let mut table1 = HashMap::new();
        table1.insert("name".to_string(), Node::Str("First".to_string()));
        let mut table2 = HashMap::new();
        table2.insert("name".to_string(), Node::Str("Second".to_string()));

        let array = vec![Node::Object(table1), Node::Object(table2)];
        let mut root = HashMap::new();
        root.insert("items".to_string(), Node::Array(array));

        let mut dest = BufferDestination::new();
        stringify(&Node::Object(root), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[items]]\nname = \"First\"\n[[items]]\nname = \"Second\"\n"
        );
    }
    #[test]
    fn test_stringify_table() {
        let mut source = BufferSource::new(
            "{\r\n    \"colors\": [\r\n        {\r\n            \"color\": \"black\",\r\n            \"category\": \"hue\",\r\n            \"type\": \"primary\",\r\n            \"code\": {\r\n                \"rgba\": [\r\n                    255,\r\n                    255,\r\n                    255,\r\n                    1\r\n                ],\r\n                \"hex\": \"#000\"\r\n            }\r\n        }\r\n    ]\r\n}".as_bytes()
        );
        let node = crate::parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[colors]]\ncategory = \"hue\"\ncolor = \"black\"\ntype = \"primary\"\n[colors.code]\nhex = \"#000\"\nrgba = [255, 255, 255, 1]\n"
        );
    }
    #[test]
    fn test_stringify_nested_array_of_tables() {
        let mut batter1 = HashMap::new();
        batter1.insert("id".to_string(), Node::Str("1001".to_string()));
        batter1.insert("type".to_string(), Node::Str("Regular".to_string()));

        let mut batter2 = HashMap::new();
        batter2.insert("id".to_string(), Node::Str("1002".to_string()));
        batter2.insert("type".to_string(), Node::Str("Chocolate".to_string()));

        let batters = vec![Node::Object(batter1), Node::Object(batter2)];
        let mut batters_obj = HashMap::new();
        batters_obj.insert("batter".to_string(), Node::Array(batters));

        let mut item = HashMap::new();
        item.insert("batters".to_string(), Node::Object(batters_obj));
        item.insert("name".to_string(), Node::Str("Cake".to_string()));

        let items = vec![Node::Object(item)];
        let mut root = HashMap::new();
        root.insert("items".to_string(), Node::Array(items));

        let mut dest = BufferDestination::new();
        stringify(&Node::Object(root), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[items]]\nname = \"Cake\"\n[[items.batters.batter]]\nid = \"1001\"\ntype = \"Regular\"\n[[items.batters.batter]]\nid = \"1002\"\ntype = \"Chocolate\"\n"
        );
    }

    #[test]
    fn test_stringify_complex_array_of_tables() {
        let mut code1 = HashMap::new();
        code1.insert("hex".to_string(), Node::Str("#000".to_string()));
        code1.insert(
            "rgba".to_string(),
            Node::Array(vec![
                Node::Number(Numeric::Integer(0)),
                Node::Number(Numeric::Integer(0)),
                Node::Number(Numeric::Integer(0)),
                Node::Number(Numeric::Integer(1)),
            ]),
        );

        let mut color1 = HashMap::new();
        color1.insert("name".to_string(), Node::Str("black".to_string()));
        color1.insert("code".to_string(), Node::Object(code1));

        let colors = vec![Node::Object(color1)];
        let mut root = HashMap::new();
        root.insert("colors".to_string(), Node::Array(colors));

        let mut dest = BufferDestination::new();
        stringify(&Node::Object(root), &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[colors]]\nname = \"black\"\n[colors.code]\nhex = \"#000\"\nrgba = [0, 0, 0, 1]\n"
        );
    }

    #[test]
    fn test_stringify_nested_array() {
        let mut source = BufferSource::new(b"{ \"info\": {\r\n    \"files\": [\r\n      {\r\n        \"length\": 351874,\r\n        \"path\": [\r\n          \"large.jpeg\"\r\n        ]\r\n      },\r\n      {\r\n        \"length\": 100,\r\n        \"path\": [\r\n          \"2\"\r\n        ]\r\n        \r\n      }\r\n      ]\r\n    }\r\n      \r\n   }.");
        let node = crate::parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[[info.files]]\nlength = 351874\npath = [\"large.jpeg\"]\n[[info.files]]\nlength = 100\npath = [\"2\"]\n"
        );
    }
    #[test]
    fn test_stringify_nested_object_with_array() {
        let mut source = BufferSource::new(b"{ \"info\": {\r\n    \"file\": {\"length\": 351874,\r\n        \"path\": [\r\n\"large.jpeg\"\r\n]}}}");
        let node = crate::parse(&mut source).unwrap();
        let mut dest = BufferDestination::new();
        stringify(&node, &mut dest).unwrap();
        assert_eq!(
            dest.to_string(),
            "[info.file]\nlength = 351874\npath = [\"large.jpeg\"]\n"
        );
    }

    #[test]
    fn test_nested_array_tables() {
        let mut dest = BufferDestination::new();
        let mut deepest = HashMap::new();
        deepest.insert("value".to_string(), make_node(42));

        let mut inner = HashMap::new();
        inner.insert(
            "nested".to_string(),
            make_node(vec![make_node(Node::Object(deepest.clone()))]),
        );

        let mut dict = HashMap::new();
        dict.insert(
            "items".to_string(),
            make_node(vec![make_node(Node::Object(inner))]),
        );

        stringify(&Node::Object(dict), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "[[items]]\n[items.nested]\nvalue = 42\n");
    }

    // Root-level type errors
    #[test]
    fn test_stringify_root_null_errors() {
        let mut dest = BufferDestination::new();
        let result = stringify(&Node::None, &mut dest);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML format requires a Object at the root level"
        );
    }

    #[test]
    fn test_stringify_root_boolean_errors() {
        let mut dest = BufferDestination::new();
        let result = stringify(&Node::Boolean(true), &mut dest);
        assert!(result.is_err());
    }

    #[test]
    fn test_stringify_root_number_errors() {
        let mut dest = BufferDestination::new();
        let result = stringify(&Node::Number(Numeric::Integer(1)), &mut dest);
        assert!(result.is_err());
    }

    // Empty root object
    #[test]
    fn test_stringify_empty_object_root() {
        let mut dest = BufferDestination::new();
        let result = stringify(&Node::Object(HashMap::new()), &mut dest);
        assert!(result.is_ok());
        assert_eq!(dest.to_string(), "");
    }

    // Boolean values inside object
    #[test]
    fn test_stringify_bool_true_value() {
        let mut map = HashMap::new();
        map.insert("flag".to_string(), Node::Boolean(true));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "flag = true\n");
    }

    #[test]
    fn test_stringify_bool_false_value() {
        let mut map = HashMap::new();
        map.insert("enabled".to_string(), Node::Boolean(false));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "enabled = false\n");
    }

    // Null value inside object
    #[test]
    fn test_stringify_null_value_in_object() {
        let mut map = HashMap::new();
        map.insert("nothing".to_string(), Node::None);
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "nothing = null\n");
    }

    // Numeric variants
    #[test]
    fn test_stringify_integer_negative() {
        let mut map = HashMap::new();
        map.insert("n".to_string(), Node::Number(Numeric::Integer(-42)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "n = -42\n");
    }

    #[test]
    fn test_stringify_uinteger_large() {
        let mut map = HashMap::new();
        map.insert("n".to_string(), Node::Number(Numeric::UInteger(u64::MAX)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("n = {}\n", u64::MAX));
    }

    #[test]
    fn test_stringify_float_value() {
        let mut map = HashMap::new();
        map.insert("pi".to_string(), Node::Number(Numeric::Float(3.14)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        let s = dest.to_string();
        assert!(s.starts_with("pi = 3.14"), "got: {}", s);
    }

    #[test]
    fn test_stringify_byte_value() {
        let mut map = HashMap::new();
        map.insert("b".to_string(), Node::Number(Numeric::Byte(255)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "b = 255\n");
    }

    #[test]
    fn test_stringify_int32_value() {
        let mut map = HashMap::new();
        map.insert("i".to_string(), Node::Number(Numeric::Int32(i32::MIN)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("i = {}\n", i32::MIN));
    }

    #[test]
    fn test_stringify_uint32_value() {
        let mut map = HashMap::new();
        map.insert("u".to_string(), Node::Number(Numeric::UInt32(u32::MAX)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), format!("u = {}\n", u32::MAX));
    }

    // Homogeneous arrays
    #[test]
    fn test_stringify_array_of_strings() {
        let mut map = HashMap::new();
        map.insert(
            "tags".to_string(),
            Node::Array(vec![Node::Str("a".to_string()), Node::Str("b".to_string())]),
        );
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "tags = [\"a\", \"b\"]\n");
    }

    #[test]
    fn test_stringify_array_of_booleans() {
        let mut map = HashMap::new();
        map.insert(
            "flags".to_string(),
            Node::Array(vec![Node::Boolean(true), Node::Boolean(false)]),
        );
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "flags = [true, false]\n");
    }

    #[test]
    fn test_stringify_array_of_integers() {
        let mut map = HashMap::new();
        map.insert(
            "nums".to_string(),
            Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Number(Numeric::Integer(2)),
                Node::Number(Numeric::Integer(3)),
            ]),
        );
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "nums = [1, 2, 3]\n");
    }

    // Mixed array → error
    #[test]
    fn test_stringify_mixed_array_returns_error() {
        let mut map = HashMap::new();
        map.insert(
            "mixed".to_string(),
            Node::Array(vec![
                Node::Number(Numeric::Integer(1)),
                Node::Str("oops".to_string()),
            ]),
        );
        let mut dest = BufferDestination::new();
        let result = stringify(&Node::Object(map), &mut dest);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "TOML arrays must contain elements of the same type"
        );
    }

    // Key sort order
    #[test]
    fn test_stringify_keys_sorted() {
        let mut map = HashMap::new();
        map.insert("z".to_string(), Node::Number(Numeric::Integer(3)));
        map.insert("a".to_string(), Node::Number(Numeric::Integer(1)));
        map.insert("m".to_string(), Node::Number(Numeric::Integer(2)));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "a = 1\nm = 2\nz = 3\n");
    }

    // Multiple root-level scalars of different types
    #[test]
    fn test_stringify_multiple_root_scalars() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), Node::Str("test".to_string()));
        map.insert("count".to_string(), Node::Number(Numeric::Integer(5)));
        map.insert("active".to_string(), Node::Boolean(true));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        // Keys are sorted: active, count, name
        assert_eq!(
            dest.to_string(),
            "active = true\ncount = 5\nname = \"test\"\n"
        );
    }

    // Empty string value
    #[test]
    fn test_stringify_empty_string_value() {
        let mut map = HashMap::new();
        map.insert("k".to_string(), Node::Str("".to_string()));
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "k = \"\"\n");
    }

    // Single-element arrays
    #[test]
    fn test_stringify_single_element_number_array() {
        let mut map = HashMap::new();
        map.insert(
            "ids".to_string(),
            Node::Array(vec![Node::Number(Numeric::Integer(99))]),
        );
        let mut dest = BufferDestination::new();
        stringify(&Node::Object(map), &mut dest).unwrap();
        assert_eq!(dest.to_string(), "ids = [99]\n");
    }

    // calculate_prefix via empty vs non-empty
    #[test]
    fn test_calculate_prefix_empty() {
        let result = calculate_prefix("", &"key".to_string());
        assert_eq!(result, "key");
    }

    #[test]
    fn test_calculate_prefix_nonempty() {
        let result = calculate_prefix("outer", &"inner".to_string());
        assert_eq!(result, "outer.inner");
    }

    #[test]
    fn test_calculate_prefix_deep() {
        let result = calculate_prefix("a.b", &"c".to_string());
        assert_eq!(result, "a.b.c");
    }

    // get_node_type
    #[test]
    fn test_get_node_type_variants() {
        assert_eq!(get_node_type(&Node::Str("".to_string())), "string");
        assert_eq!(get_node_type(&Node::Boolean(false)), "boolean");
        assert_eq!(get_node_type(&Node::Number(Numeric::Integer(0))), "number");
        assert_eq!(get_node_type(&Node::Array(vec![])), "array");
        assert_eq!(get_node_type(&Node::Object(HashMap::new())), "object");
        assert_eq!(get_node_type(&Node::None), "null");
    }
}
