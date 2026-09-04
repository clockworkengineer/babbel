use xml_lib_rust::{parse, DtdValidator};

#[test]
fn test_dtd_default_attribute_injection() {
    let dtd = r#"
        <!ELEMENT root (item)*>
        <!ELEMENT item EMPTY>
        <!ATTLIST item id CDATA #REQUIRED>
        <!ATTLIST item status CDATA "active">
        <!ATTLIST item env CDATA #FIXED "prod">
    "#;

    let xml = r#"<root><item id="1"/><item id="2" status="inactive"/></root>"#;
    let mut doc = parse(xml).expect("Clean parse");

    let mut validator = DtdValidator::new();
    validator.parse_subset(dtd).expect("Valid DTD subset");

    let injected_count = validator.apply_defaults(&mut doc).expect("Apply defaults");
    assert_eq!(injected_count, 3); // item 1 status + env, item 2 env

    let root_id = doc.root_element_id().unwrap();
    let children = doc.get_children(root_id);

    // item 1
    assert_eq!(doc.get_attribute(children[0], "status"), Some("active"));
    assert_eq!(doc.get_attribute(children[0], "env"), Some("prod"));

    // item 2
    assert_eq!(doc.get_attribute(children[1], "status"), Some("inactive"));
    assert_eq!(doc.get_attribute(children[1], "env"), Some("prod"));
}

#[test]
fn test_dtd_id_uniqueness() {
    let dtd = r#"
        <!ELEMENT root (user)*>
        <!ELEMENT user EMPTY>
        <!ATTLIST user id ID #REQUIRED>
    "#;

    let mut validator = DtdValidator::new();
    validator.parse_subset(dtd).unwrap();

    let xml_valid = r#"<root><user id="u1"/><user id="u2"/></root>"#;
    let doc_valid = parse(xml_valid).unwrap();
    assert!(validator.validate(&doc_valid).is_ok());

    let xml_duplicate = r#"<root><user id="u1"/><user id="u1"/></root>"#;
    let doc_dup = parse(xml_duplicate).unwrap();
    let res = validator.validate(&doc_dup);
    assert!(res.is_err(), "Duplicate ID must fail validation");
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Duplicate ID"));
}

#[test]
fn test_dtd_idref_referential_integrity() {
    let dtd = r#"
        <!ELEMENT catalog (author*, book*)>
        <!ELEMENT author EMPTY>
        <!ATTLIST author id ID #REQUIRED>
        <!ELEMENT book EMPTY>
        <!ATTLIST book author_id IDREF #REQUIRED>
    "#;

    let mut validator = DtdValidator::new();
    validator.parse_subset(dtd).unwrap();

    let xml_valid = r#"<catalog><author id="auth_1"/><book author_id="auth_1"/></catalog>"#;
    let doc_valid = parse(xml_valid).unwrap();
    assert!(validator.validate(&doc_valid).is_ok());

    let xml_broken_ref = r#"<catalog><author id="auth_1"/><book author_id="auth_99"/></catalog>"#;
    let doc_broken = parse(xml_broken_ref).unwrap();
    let res = validator.validate(&doc_broken);
    assert!(res.is_err(), "Invalid IDREF reference must be rejected");
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("does not match any declared ID"));
}

#[test]
fn test_dtd_external_subset_resolver() {
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE root SYSTEM "system_catalog.dtd">
    <root><item flag="ok">Hello</item></root>
    "#;

    let doc = parse(xml).expect("Parse DOCTYPE");

    let mut validator = DtdValidator::new();
    validator.set_external_resolver(|system_id, _| {
        if system_id == "system_catalog.dtd" {
            Some(r#"
                <!ELEMENT root (item)*>
                <!ELEMENT item (#PCDATA)>
                <!ATTLIST item flag CDATA #REQUIRED>
            "#.to_string())
        } else {
            None
        }
    });

    assert!(validator.validate(&doc).is_ok());

    let invalid_xml = r#"<?xml version="1.0"?>
    <!DOCTYPE root SYSTEM "system_catalog.dtd">
    <root><item>Missing flag</item></root>
    "#;
    let invalid_doc = parse(invalid_xml).unwrap();
    let res = validator.validate(&invalid_doc);
    assert!(res.is_err(), "Missing required attribute resolved from external DTD must fail");
}
