//! # XML File Parsing Example
//!
//! Demonstrates parsing an XML string into a DOM `Document`, inspecting node metadata,
//! and stringifying back to XML.

use xml_lib_rust::{parse, stringify};

fn main() {
    println!("--- XML Parse File Example ---");
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note>
  <to>Tove</to>
  <from>Jani</from>
  <heading>Reminder</heading>
  <body>Don't forget me this weekend!</body>
</note>"#;

    // Parse XML string into in-memory arena Document
    match parse(xml_content) {
        Ok(doc) => {
            println!("Successfully parsed XML document with {} nodes.", doc.len());

            // Inspect primary element tag
            if let Some(root_id) = doc.root_element_id() {
                if let Some(node) = doc.get_node(root_id) {
                    println!("Root element name: <{}>", node.kind.name());
                }
            }

            // Stringify document back to formatted XML output
            println!("Serialized Output:\n{}", stringify(&doc));
        }
        Err(err) => {
            eprintln!("Failed to parse XML: {err}");
        }
    }
}
