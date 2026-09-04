//! # XML Mixed Content Example
//!
//! Demonstrates parsing mixed content element tags containing interspersed text nodes,
//! child element tags (`<b>`, `<i>`), and CDATA sections (`<![CDATA[...]]>`).

use xml_lib_rust::{parse, NodeKind};

fn main() {
    println!("--- XML Mixed Content Example ---");
    let xml =
        r#"<p>This is <b>bold</b> text with an <i>italic</i> word and <![CDATA[raw CDATA content]]>.</p>"#;

    // Parse XML string into DOM arena
    let doc = parse(xml).expect("Parse clean");
    let root_id = doc.root_element_id().unwrap();

    // Extract concatenated text content recursively
    println!(
        "Full extracted text content:\n\"{}\"\n",
        doc.get_text_content(root_id)
    );

    // Inspect individual child node variants
    println!("Child node breakdown:");
    for &child_id in &doc.get_children(root_id) {
        if let Some(child) = doc.get_node(child_id) {
            match &child.kind {
                NodeKind::Text(t) => println!("  - Text: \"{t}\""),
                NodeKind::Element { name, .. } => println!("  - Element: <{name}>"),
                NodeKind::CData(c) => println!("  - CDATA: <![CDATA[{c}]]>"),
                _ => {}
            }
        }
    }
}
