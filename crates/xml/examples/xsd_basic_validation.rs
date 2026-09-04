//! # XSD Basic Validation Example
//!
//! Demonstrates parsing a simple XSD schema definition (`<xs:element name="note" type="xs:string"/>`)
//! and validating document instance nodes using `XsdValidator` and `XmlValidator`.

use xml_lib_rust::{parse, XsdValidator};

fn main() {
    println!("--- XSD Basic Validation Example ---");
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="note" type="xs:string"/>
</xs:schema>"#;

    // Parse XSD schema document
    let mut validator = XsdValidator::new();
    validator.parse_schema(schema).unwrap();

    // Parse XML instance document
    let doc = parse("<note>Hello World</note>").unwrap();

    // Validate using the XmlValidator trait interface
    match validator.validate(&doc) {
        Ok(_) => println!("XSD Validation passed clean!"),
        Err(err) => eprintln!("XSD Validation failed: {err}"),
    }
}
