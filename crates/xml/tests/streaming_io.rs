use xml_lib_rust::{parse_bytes_with_encoding, parse_reader};

#[test]
fn test_parse_reader_streaming() {
    let xml_data = b"<streaming><chunk>1</chunk><chunk>2</chunk></streaming>";
    let cursor = std::io::Cursor::new(xml_data);

    let doc = parse_reader(cursor).expect("Parse from io::Read cursor");
    let root_id = doc.root_element_id().unwrap();
    let children = doc.get_children(root_id);
    assert_eq!(children.len(), 2);
}

#[test]
fn test_iso_8859_1_decoding() {
    // In ISO-8859-1:
    // 'é' is 0xE9
    // 'ü' is 0xFC
    let raw_bytes: &[u8] = b"<cafe name=\"Caf\xE9\" city=\"M\xFCnchen\"/>";

    let doc = parse_bytes_with_encoding(raw_bytes, "ISO-8859-1").expect("Decode ISO-8859-1");
    let root_id = doc.root_element_id().unwrap();
    assert_eq!(doc.get_attribute(root_id, "name"), Some("Café"));
    assert_eq!(doc.get_attribute(root_id, "city"), Some("München"));
}

#[test]
fn test_windows_1252_decoding() {
    // In Windows-1252:
    // 0x80 is '€' (Euro)
    // 0x99 is '™' (Trademark)
    let raw_bytes: &[u8] = b"<price currency=\"\x80\" product=\"SuperApp\x99\"/>";

    let doc = parse_bytes_with_encoding(raw_bytes, "WINDOWS-1252").expect("Decode Windows-1252");
    let root_id = doc.root_element_id().unwrap();
    assert_eq!(doc.get_attribute(root_id, "currency"), Some("€"));
    assert_eq!(doc.get_attribute(root_id, "product"), Some("SuperApp™"));
}

#[test]
fn test_ascii_strict_validation() {
    let valid_ascii = b"<ascii>Hello World 123</ascii>";
    assert!(parse_bytes_with_encoding(valid_ascii, "US-ASCII").is_ok());

    let invalid_ascii = b"<ascii>Invalid \xFF byte</ascii>";
    let res = parse_bytes_with_encoding(invalid_ascii, "US-ASCII");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("Byte out of 7-bit US-ASCII range"));
}
