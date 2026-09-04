//! # XML Analysis Example
//!
//! Demonstrates iterating over DOM arena nodes (`doc.nodes()`), categorizing node variants
//! (`Element`, `Text`, `Comment`), and counting metrics across a document structure.

use xml_lib_rust::{parse, NodeKind};

fn main() {
    println!("--- XML Analyze File Example ---");
    let xml = r#"<?xml version="1.0"?>
<!-- Document comment -->
<catalog>
  <book id="bk101">
    <author>Gambardella, Matthew</author>
    <title>XML Developer's Guide</title>
    <genre>Computer</genre>
    <price>44.95</price>
    <publish_date>2000-10-01</publish_date>
    <description>An in-depth look at creating applications with XML.</description>
  </book>
</catalog>"#;

    // Parse XML string into DOM arena
    let doc = parse(xml).expect("Parse clean");

    let mut element_count = 0;
    let mut text_count = 0;
    let mut comment_count = 0;

    // Traversal across all nodes in the Document arena
    for node in doc.nodes() {
        match &node.kind {
            NodeKind::Element { name, attributes } => {
                element_count += 1;
                println!("Element: <{name}> with {} attributes", attributes.len());
            }
            NodeKind::Text(t) => {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    text_count += 1;
                    println!("Text: \"{trimmed}\"");
                }
            }
            NodeKind::Comment(c) => {
                comment_count += 1;
                println!("Comment: <!-- {c} -->");
            }
            _ => {}
        }
    }

    println!("\nAnalysis Summary:");
    println!("  Total Elements : {element_count}");
    println!("  Total Text     : {text_count}");
    println!("  Total Comments : {comment_count}");
}
