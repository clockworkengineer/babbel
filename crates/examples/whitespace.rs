//! # XML Whitespace Handling Example
//!
//! Demonstrates how `xml_lib` preserves text whitespace and distinguishes between
//! raw child nodes (including indentation text nodes) and element-only children.

use xml_lib_rust::parse;

fn main() {
    println!("--- XML Whitespace Handling Example ---");
    let xml = "<root>\n  <item>  spaced content  </item>\n</root>";

    // Parse XML string with default options
    let doc = parse(xml).expect("Parse clean");
    let root_id = doc.root_element_id().unwrap();

    // Get all direct child nodes (includes raw whitespace Text nodes between element tags)
    let children = doc.get_children(root_id);
    println!(
        "Total direct children of <root> (including whitespace text nodes): {}",
        children.len()
    );

    // Get element-only child nodes using the Document helper method
    let element_children = doc.get_element_children(root_id);
    println!("Element-only children count: {}", element_children.len());
}
