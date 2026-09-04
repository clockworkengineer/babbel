//! # Large Tag Attributes & Security Policy Example
//!
//! Demonstrates parsing tags with numerous attributes while configuring
//! `ParseOptions::max_attribute_count` threshold protection.

use xml_lib_rust::{parse_with_options, NodeKind, ParseOptions};

fn main() {
    println!("--- XML Large Attributes Example ---");

    // Construct element tag with 100 attributes
    let mut xml = String::from("<widget ");
    for i in 0..100 {
        xml.push_str(&format!("attr{i}=\"value{i}\" "));
    }
    xml.push_str("/>");

    // Configure attribute threshold limit (max_attribute_count = 500)
    let mut options = ParseOptions::default();
    options.max_attribute_count = 500;

    let doc = parse_with_options(&xml, options).expect("Parse clean");
    if let Some(root_id) = doc.root_element_id() {
        if let Some(node) = doc.get_node(root_id) {
            if let NodeKind::Element { attributes, .. } = &node.kind {
                println!(
                    "Successfully parsed element with {} attributes.",
                    attributes.len()
                );
            }
        }
    }
}
