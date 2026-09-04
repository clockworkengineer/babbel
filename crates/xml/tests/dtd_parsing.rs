use xml_lib_rust::{parse, DtdValidator};

#[test]
fn test_dtd_element_content_models() {
    // EMPTY element valid case
    let xml_empty_valid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ELEMENT note (to)>
  <!ELEMENT to EMPTY>
]>
<note><to/></note>"#;
    let doc = parse(xml_empty_valid).expect("XML parse clean");
    let validator = DtdValidator::new();
    assert!(validator.validate(&doc).is_ok());

    // EMPTY element invalid case (contains text)
    let xml_empty_invalid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ELEMENT to EMPTY>
]>
<to>Some Text</to>"#;
    let doc_inv = parse(xml_empty_invalid).expect("XML parse clean");
    assert!(validator.validate(&doc_inv).is_err());
}

#[test]
fn test_dtd_required_attributes() {
    let xml_valid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ATTLIST note category CDATA #REQUIRED>
  <!ATTLIST note id CDATA #IMPLIED>
]>
<note category="reminder"/>"#;
    let doc = parse(xml_valid).expect("XML parse clean");
    let validator = DtdValidator::new();
    assert!(validator.validate(&doc).is_ok());

    let xml_missing_required = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ATTLIST note category CDATA #REQUIRED>
]>
<note id="123"/>"#;
    let doc_missing = parse(xml_missing_required).expect("XML parse clean");
    assert!(validator.validate(&doc_missing).is_err());
}

#[test]
fn test_dtd_entity_declarations_and_parsing() {
    let xml_entity = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ENTITY author "John Doe">
  <!ENTITY copy "Copyright 2026">
]>
<note>&author; - &copy;</note>"#;

    let doc = parse(xml_entity).expect("XML parse with internal DTD entities clean");
    let root_id = doc.root_element_id().unwrap();
    let text = doc.get_text_content(root_id);
    assert_eq!(text, "John Doe - Copyright 2026");
}

#[test]
fn test_dtd_element_any_and_mixed() {
    let xml_any = r#"<?xml version="1.0"?>
<!DOCTYPE root [
  <!ELEMENT root ANY>
  <!ELEMENT child ANY>
]>
<root><child>Hello <b>World</b></child></root>"#;
    let doc = parse(xml_any).expect("XML ANY model parse clean");
    let validator = DtdValidator::new();
    assert!(validator.validate(&doc).is_ok());
}
