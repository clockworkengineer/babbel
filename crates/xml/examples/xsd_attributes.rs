//! # XSD Attribute & Type Constraint Example
//!
//! Demonstrates parsing XSD attribute declarations (`xs:attribute`) and validating
//! attribute primitive data types (`xs:integer`) using `XsdValidator` and `XmlValidator`.

use xml_lib_rust::{parse, XsdValidator};

fn main() {
    println!("--- XSD Attributes Example ---");
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="person">
    <xs:complexType>
      <xs:attribute name="id" type="xs:integer" use="required"/>
    </xs:complexType>
  </xs:element>
</xs:schema>"#;

    // Parse XSD schema definition
    let mut validator = XsdValidator::new();
    validator.parse_schema(schema).unwrap();

    // Valid XML instance (integer attribute value id="42")
    let valid_xml = r#"<person id="42"/>"#;
    let doc_valid = parse(valid_xml).unwrap();
    println!(
        "Valid XML attribute validation: {:?}",
        validator.validate(&doc_valid)
    );

    // Invalid XML instance (non-integer string attribute value id="abc")
    let invalid_xml = r#"<person id="abc"/>"#;
    let doc_invalid = parse(invalid_xml).unwrap();
    println!(
        "Invalid attribute integer validation error: {:?}",
        validator.validate(&doc_invalid)
    );
}
