//! # XML Runtime Creation Example
//!
//! Demonstrates constructing an XML DOM tree dynamically at runtime using `Document::new()`,
//! `doc.add_node()`, `Attribute::new()`, and `doc.append_child()`.

use xml_lib_rust::{stringify, Attribute, Document, NodeKind};

fn main() {
    println!("--- XML Create At Runtime Example ---");
    let mut doc = Document::new();

    // Set XML Declaration (<?xml version="1.0" encoding="UTF-8" standalone="yes"?>)
    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".into(),
        encoding: Some("UTF-8".into()),
        standalone: Some(true),
    });
    doc.set_declaration_id(decl_id);

    // Create Root Element (<library location="Main Campus">)
    let root_elem_id = doc.add_node(NodeKind::Element {
        name: "library".into(),
        attributes: vec![Attribute::new("location", "Main Campus")],
    });
    let root_id = doc.root_id().unwrap();
    doc.append_child(root_id, root_elem_id).unwrap();

    // Create Child Element (<book isbn="978-0-306-40615-7">)
    let book_id = doc.add_node(NodeKind::Element {
        name: "book".into(),
        attributes: vec![Attribute::new("isbn", "978-0-306-40615-7")],
    });
    doc.append_child(root_elem_id, book_id).unwrap();

    // Create Title Element (<title>Rust Programming Language</title>)
    let title_id = doc.add_node(NodeKind::Element {
        name: "title".into(),
        attributes: Vec::new(),
    });
    let title_text_id = doc.add_node(NodeKind::Text("Rust Programming Language".into()));
    doc.append_child(title_id, title_text_id).unwrap();
    doc.append_child(book_id, title_id).unwrap();

    // Serialize DOM tree to formatted XML string
    println!("Constructed DOM Document:\n{}", stringify(&doc));
}
