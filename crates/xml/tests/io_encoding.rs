use xml_lib_rust::{parse_bytes, Format, XmlSource, XmlDestination};

#[test]
fn test_bom_detection_utf8() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(b"<root>UTF-8 BOM</root>");

    let source = XmlSource::from_bytes(&bytes).expect("BOM detection should succeed");
    assert_eq!(source.format(), Format::Utf8Bom);

    let doc = parse_bytes(&bytes).expect("Should parse UTF-8 with BOM");
    assert!(doc.root_element_id().is_some());
}

#[test]
fn test_bom_detection_utf16le() {
    // UTF-16 LE BOM is 0xFF, 0xFE
    let mut bytes = vec![0xFF, 0xFE];
    let utf16_str: Vec<u16> = "<root>UTF-16 LE</root>".encode_utf16().collect();
    for u in utf16_str {
        bytes.extend_from_slice(&u.to_le_bytes());
    }

    let source = XmlSource::from_bytes(&bytes).expect("UTF-16 LE detection should succeed");
    assert_eq!(source.format(), Format::Utf16Le);

    let doc = parse_bytes(&bytes).expect("Should parse UTF-16 LE");
    assert!(doc.root_element_id().is_some());
}

#[test]
fn test_bom_detection_utf16be() {
    // UTF-16 BE BOM is 0xFE, 0xFF
    let mut bytes = vec![0xFE, 0xFF];
    let utf16_str: Vec<u16> = "<root>UTF-16 BE</root>".encode_utf16().collect();
    for u in utf16_str {
        bytes.extend_from_slice(&u.to_be_bytes());
    }

    let source = XmlSource::from_bytes(&bytes).expect("UTF-16 BE detection should succeed");
    assert_eq!(source.format(), Format::Utf16Be);

    let doc = parse_bytes(&bytes).expect("Should parse UTF-16 BE");
    assert!(doc.root_element_id().is_some());
}

#[test]
fn test_line_ending_normalization() {
    let xml_crlf = "<root>\r\n  <line1>A</line1>\r  <line2>B</line2>\n</root>";
    let source = XmlSource::from_string(xml_crlf);
    assert!(!source.is_eof());
}

#[test]
fn test_xml_destination() {
    let mut dest = XmlDestination::new();
    dest.write_str("<test>");
    dest.write_str("Content");
    dest.write_str("</test>");

    let output = dest.into_string();
    assert_eq!(output, "<test>Content</test>");
}
