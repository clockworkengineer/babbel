//! Example demonstrating format conversions between bencode and various output formats.
//! This shows how to convert bencode data to JSON, TOML, XML, YAML and back to bencode.

use bencode_lib::{
    make_node, parse_str, stringify_to_string, to_json, to_toml, to_xml, to_yaml,
    BufferDestination, Node,
};
use std::collections::HashMap;

fn main() {
    println!("=== Format Conversion Examples ===\n");

    // Example 1: Simple conversions
    demonstrate_simple_conversions();

    // Example 2: Complex structure conversions
    demonstrate_complex_conversions();

    // Example 3: Round-trip conversions
    demonstrate_round_trips();

    // Example 4: Format comparison
    demonstrate_format_comparison();

    // Example 5: Practical use case - torrent metadata
    demonstrate_torrent_conversion();
}

/// Demonstrates converting simple values to different formats
fn demonstrate_simple_conversions() {
    println!("--- Simple Conversions ---");

    // Integer
    let integer = Node::Integer(42);
    print_all_formats("Integer (42)", &integer);

    // String
    let string = Node::Str("hello world".to_string());
    print_all_formats("String ('hello world')", &string);

    // Simple list
    let list = make_node(vec![make_node(1), make_node(2), make_node(3)]);
    print_all_formats("List [1, 2, 3]", &list);

    // Simple dictionary
    let mut dict = HashMap::new();
    dict.insert("name".to_string(), make_node("Alice"));
    dict.insert("age".to_string(), make_node(30));
    let dictionary = Node::Dictionary(dict);
    print_all_formats("Dictionary {name: 'Alice', age: 30}", &dictionary);

    println!();
}

/// Helper function to print a node in all supported formats
fn print_all_formats(description: &str, node: &Node) {
    println!("\n{}:", description);

    // Bencode (default)
    if let Ok(bencode) = stringify_to_string(node) {
        println!("  Bencode: {}", bencode);
    }

    // JSON
    let mut json_dest = BufferDestination::new();
    let _ = to_json(node, &mut json_dest).expect("Failed to convert to JSON");
    println!(
        "  JSON:    {}",
        String::from_utf8_lossy(&json_dest.buffer)
    );

    // TOML
    let mut toml_dest = BufferDestination::new();
    let _ = to_toml(node, &mut toml_dest).expect("Failed to convert to TOML");
    println!(
        "  TOML:    {}",
        String::from_utf8_lossy(&toml_dest.buffer)
    );

    // XML
    let mut xml_dest = BufferDestination::new();
    let _ = to_xml(node, &mut xml_dest).expect("Failed to convert to XML");
    let xml_output = String::from_utf8_lossy(&xml_dest.buffer);
    // Compact display for readability
    let xml_compact = xml_output.replace('\n', "").replace("  ", "");
    println!("  XML:     {}", xml_compact);

    // YAML
    let mut yaml_dest = BufferDestination::new();
    let _ = to_yaml(node, &mut yaml_dest).expect("Failed to convert to YAML");
    let yaml_output = String::from_utf8_lossy(&yaml_dest.buffer);
    // Show first line only for compact display
    let yaml_first_line = yaml_output.lines().next().unwrap_or("");
    println!("  YAML:    {}", yaml_first_line);
}

/// Demonstrates converting complex nested structures
fn demonstrate_complex_conversions() {
    println!("--- Complex Structure Conversions ---");

    // Build a complex nested structure
    let mut person = HashMap::new();
    person.insert("name".to_string(), make_node("Bob Smith"));
    person.insert("age".to_string(), make_node(35));
    person.insert("active".to_string(), make_node(1));

    let hobbies = make_node(vec![
        make_node("reading"),
        make_node("coding"),
        make_node("hiking"),
    ]);
    person.insert("hobbies".to_string(), hobbies);

    let mut address = HashMap::new();
    address.insert("street".to_string(), make_node("123 Main St"));
    address.insert("city".to_string(), make_node("Anytown"));
    address.insert("zip".to_string(), make_node("12345"));
    person.insert("address".to_string(), Node::Dictionary(address));

    let complex = Node::Dictionary(person);

    println!("\nComplex structure:");
    println!("{:#?}", complex);

    println!("\n=== JSON Format ===");
    let mut json_dest = BufferDestination::new();
    let _ = to_json(&complex, &mut json_dest).expect("Failed to convert to JSON");
    println!("{}", String::from_utf8_lossy(&json_dest.buffer));

    println!("=== TOML Format ===");
    let mut toml_dest = BufferDestination::new();
    let _ = to_toml(&complex, &mut toml_dest).expect("Failed to convert to TOML");
    println!("{}", String::from_utf8_lossy(&toml_dest.buffer));

    println!("=== XML Format ===");
    let mut xml_dest = BufferDestination::new();
    let _ = to_xml(&complex, &mut xml_dest).expect("Failed to convert to XML");
    println!("{}", String::from_utf8_lossy(&xml_dest.buffer));

    println!("=== YAML Format ===");
    let mut yaml_dest = BufferDestination::new();
    let _ = to_yaml(&complex, &mut yaml_dest).expect("Failed to convert to YAML");
    println!("{}", String::from_utf8_lossy(&yaml_dest.buffer));
}

/// Demonstrates round-trip conversions through different formats
fn demonstrate_round_trips() {
    println!("--- Round-Trip Conversions ---");

    let original = make_node(vec![
        make_node("item1"),
        make_node(42),
        make_node(vec![make_node("nested"), make_node("items")]),
    ]);

    println!("Original: {}", original);

    // Bencode -> Bencode
    if let Ok(bencode_str) = stringify_to_string(&original) {
        println!("\nBencode representation: {}", bencode_str);

        match parse_str(&bencode_str) {
            Ok(parsed) => {
                println!("Parsed back from bencode: {}", parsed);
                println!("Matches original: {}", original == parsed);
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }

    // Note: JSON/TOML/XML/YAML are output-only formats in this library
    // They cannot be parsed back into Node structures
    println!("\nNote: This library provides conversion TO JSON/TOML/XML/YAML,");
    println!("but does not parse FROM these formats back to Node structures.");
    println!("Only bencode format supports bidirectional conversion.");
}

/// Demonstrates comparing output sizes across formats
fn demonstrate_format_comparison() {
    println!("--- Format Size Comparison ---");

    let mut data = HashMap::new();
    data.insert("tracker".to_string(), make_node("udp://example.com:6969"));
    data.insert("length".to_string(), make_node(1048576));
    data.insert("name".to_string(), make_node("example-file.txt"));
    data.insert(
        "tags".to_string(),
        make_node(vec![make_node("media"), make_node("document")]),
    );

    let node = Node::Dictionary(data);

    println!("\nSample data: {}", node);
    println!("\nFormat size comparison:");

    // Bencode
    if let Ok(bencode) = stringify_to_string(&node) {
        println!("  Bencode: {} bytes", bencode.len());
    }

    // JSON
    let mut json_dest = BufferDestination::new();
    let _ = to_json(&node, &mut json_dest).expect("Failed to convert to JSON");
    println!("  JSON:    {} bytes", &json_dest.buffer.len());

    // TOML
    let mut toml_dest = BufferDestination::new();
    let _ = to_toml(&node, &mut toml_dest).expect("Failed to convert to TOML");
    println!("  TOML:    {} bytes", &toml_dest.buffer.len());

    // XML
    let mut xml_dest = BufferDestination::new();
    let _ = to_xml(&node, &mut xml_dest).expect("Failed to convert to XML");
    println!("  XML:     {} bytes", &xml_dest.buffer.len());

    // YAML
    let mut yaml_dest = BufferDestination::new();
    let _ = to_yaml(&node, &mut yaml_dest).expect("Failed to convert to YAML");
    println!("  YAML:    {} bytes", &yaml_dest.buffer.len());

    println!("\nNote: Bencode is typically the most compact format.");
    println!();
}

/// Demonstrates converting torrent metadata to various formats
fn demonstrate_torrent_conversion() {
    println!("--- Torrent Metadata Conversion ---");

    // Build simplified torrent structure
    let mut file_entry = HashMap::new();
    file_entry.insert("length".to_string(), make_node(524288));
    file_entry.insert(
        "path".to_string(),
        make_node(vec![make_node("docs"), make_node("readme.txt")]),
    );

    let mut info = HashMap::new();
    info.insert("name".to_string(), make_node("example-torrent"));
    info.insert("piece length".to_string(), make_node(262144));
    info.insert("pieces".to_string(), make_node("dummy_hash_string"));
    info.insert(
        "files".to_string(),
        make_node(vec![Node::Dictionary(file_entry)]),
    );

    let mut torrent = HashMap::new();
    torrent.insert(
        "announce".to_string(),
        make_node("udp://tracker.example.com:6969/announce"),
    );
    torrent.insert("created by".to_string(), make_node("bencode_lib"));
    torrent.insert("creation date".to_string(), make_node(1699564800));
    torrent.insert("info".to_string(), Node::Dictionary(info));

    let torrent_node = Node::Dictionary(torrent);

    println!("Torrent metadata in different formats:\n");

    println!("=== Original Bencode ===");
    if let Ok(bencode) = stringify_to_string(&torrent_node) {
        println!("{}\n", bencode);
    }

    println!("=== As JSON (human-readable) ===");
    let mut json_dest = BufferDestination::new();
    let _ = to_json(&torrent_node, &mut json_dest).expect("Failed to convert to JSON");
    println!("{}\n", String::from_utf8_lossy(&json_dest.buffer));

    println!("=== As YAML (human-readable) ===");
    let mut yaml_dest = BufferDestination::new();
    let _ = to_yaml(&torrent_node, &mut yaml_dest).expect("Failed to convert to YAML");
    println!("{}", String::from_utf8_lossy(&yaml_dest.buffer));

    println!("These conversions are useful for:");
    println!("  - Debugging torrent files");
    println!("  - Generating human-readable metadata reports");
    println!("  - Converting to formats for web APIs");
    println!("  - Data analysis and inspection");
}
