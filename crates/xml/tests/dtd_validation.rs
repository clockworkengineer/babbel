use xml_lib_rust::{parse, DtdValidator};

#[test]
fn test_dtd_validation() {
    let xml = r#"<?xml version="1.0"?>
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

    let doc = parse(xml).expect("Should parse XML with DTD");
    let dtd = DtdValidator::new();
    assert!(dtd.validate(&doc).is_ok());
}

#[test]
fn test_dtd_missing_required_attribute() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ATTLIST note category CDATA #REQUIRED>
]>
<note>
  <to/>
</note>"#;

    let doc = parse(xml).expect("Should parse XML");
    let dtd = DtdValidator::new();
    let res = dtd.validate(&doc);
    assert!(res.is_err(), "DTD validation should fail when required attribute is missing");
}
