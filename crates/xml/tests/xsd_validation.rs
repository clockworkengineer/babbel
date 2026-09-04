use xml_lib_rust::{parse, XsdValidator};

#[test]
fn test_xsd_simple_string_validation() {
    let xsd_schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="note" type="xs:string"/>
</xs:schema>"#;

    let doc_valid = parse("<note>Hello World</note>").expect("Parse clean");
    let mut validator = XsdValidator::new();
    validator.parse_schema(xsd_schema).unwrap();
    assert!(validator.validate(&doc_valid).is_ok());
}

#[test]
fn test_xsd_complex_type_sequence() {
    let xsd_schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="to" type="xs:string"/>
  <xs:element name="from" type="xs:string"/>
</xs:schema>"#;

    let doc_valid = parse("<note><to>Alice</to><from>Bob</from></note>").expect("Parse clean");
    let mut validator = XsdValidator::new();
    validator.parse_schema(xsd_schema).unwrap();
    assert!(validator.validate(&doc_valid).is_ok());
}

#[test]
fn test_xsd_boolean_simple_type() {
    let xsd_schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="flag" type="xs:boolean"/>
</xs:schema>"#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(xsd_schema).unwrap();

    let doc_true = parse("<flag>true</flag>").unwrap();
    assert!(validator.validate(&doc_true).is_ok());

    let doc_false = parse("<flag>false</flag>").unwrap();
    assert!(validator.validate(&doc_false).is_ok());

    let doc_1 = parse("<flag>1</flag>").unwrap();
    assert!(validator.validate(&doc_1).is_ok());

    let doc_invalid = parse("<flag>yes</flag>").unwrap();
    assert!(validator.validate(&doc_invalid).is_err());
}

#[test]
fn test_xsd_integer_simple_type() {
    let xsd_schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="count" type="xs:integer"/>
</xs:schema>"#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(xsd_schema).unwrap();

    let doc_valid = parse("<count>42</count>").unwrap();
    assert!(validator.validate(&doc_valid).is_ok());

    let doc_negative = parse("<count>-7</count>").unwrap();
    assert!(validator.validate(&doc_negative).is_ok());

    let doc_float = parse("<count>3.14</count>").unwrap();
    assert!(validator.validate(&doc_float).is_err());
}
