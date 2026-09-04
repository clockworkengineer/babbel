//! # XML Entity Reference & Mapper Example
//!
//! Demonstrates parsing entity references (`&amp;`, `&lt;`, `&quot;`), numeric character references
//! (`&#65;`, `&#x42;`), and registering custom entity mappings via `EntityMapper`.

use xml_lib_rust::{parse, EntityMapper};

fn main() {
    println!("--- XML Entity Reference Example ---");
    let xml = r#"<data>AT&amp;T &lt;Corp&gt; &quot;Quote&quot; &#65; &#x42;</data>"#;

    // Parse document with standard entity decoding
    let doc = parse(xml).expect("Parse clean");
    let text = doc.get_text_content(doc.root_element_id().unwrap());
    println!("Parsed Raw Content: \"{text}\"");

    // Expand entities using EntityMapper
    let mut mapper = EntityMapper::default();
    let expanded = mapper.expand(&text).expect("Expand entities");
    println!("Expanded Content : \"{expanded}\"");

    // Register custom user-defined entity reference
    mapper.register("custom", "Special Product Value");
    let custom_expanded = mapper.expand("Product: &custom;").unwrap();
    println!("Custom Entity    : \"{custom_expanded}\"");
}
