//! # XML DTD Validation Example
//!
//! Demonstrates validating document structures against DTD content models
//! (`<!ELEMENT>`) and required attribute rules (`<!ATTLIST ... #REQUIRED>`) using `XmlValidator`.

use xml_lib_rust::{parse, DtdValidator};

fn main() {
    println!("--- XML DTD Validation Example ---");
    let xml_valid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ELEMENT note (to,from,heading,body)>
  <!ATTLIST note category CDATA #REQUIRED>
  <!ELEMENT to EMPTY>
  <!ELEMENT from EMPTY>
  <!ELEMENT heading EMPTY>
  <!ELEMENT body EMPTY>
]>
<note category="reminder">
  <to/>
  <from/>
  <heading/>
  <body/>
</note>"#;

    // Parse valid document with inline DTD subset
    let doc = parse(xml_valid).expect("Parse clean");
    let validator = DtdValidator::new();

    // Validate using the XmlValidator trait interface
    match validator.validate(&doc) {
        Ok(_) => println!("DTD Validation passed clean!"),
        Err(err) => eprintln!("DTD Validation failed: {err}"),
    }

    // Invalid XML document (missing required #REQUIRED 'category' attribute)
    let xml_invalid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ATTLIST note category CDATA #REQUIRED>
]>
<note>
  <to/>
</note>"#;

    let doc_inv = parse(xml_invalid).expect("Parse clean");
    match validator.validate(&doc_inv) {
        Ok(_) => println!("Unexpected pass"),
        Err(err) => println!("Validation correctly detected error: {err}"),
    }
}
