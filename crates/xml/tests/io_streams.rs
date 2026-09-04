use xml_lib_rust::{
    io::{XmlSource, XmlDestination},
    parse, parse_bytes,
};
use std::io::Write;

#[test]
fn test_xml_source_line_ending_normalization() {
    let crlf_xml = "<?xml version=\"1.0\"?>\r\n<root>\r\n<child>Data</child>\r\n</root>";
    let doc = parse(crlf_xml).expect("Parsing CRLF XML should succeed");
    let text = doc.get_text_content(doc.root_element_id().unwrap());
    assert_eq!(text.trim(), "Data");

    let cr_xml = "<?xml version=\"1.0\"?>\r<root>\r<child>Data</child>\r</root>";
    let doc_cr = parse(cr_xml).expect("Parsing CR XML should succeed");
    let text_cr = doc_cr.get_text_content(doc_cr.root_element_id().unwrap());
    assert_eq!(text_cr.trim(), "Data");
}

#[test]
fn test_xml_source_from_bytes_bom_handling() {
    // UTF-8 with BOM
    let mut utf8_bom = vec![0xEF, 0xBB, 0xBF];
    utf8_bom.extend_from_slice(b"<root>UTF-8 BOM</root>");
    let doc_utf8 = parse_bytes(&utf8_bom).expect("UTF-8 BOM parse should succeed");
    assert_eq!(doc_utf8.get_text_content(doc_utf8.root_element_id().unwrap()), "UTF-8 BOM");

    // UTF-16 BE with BOM
    let mut utf16_be = vec![0xFE, 0xFF];
    for ch in "<root>UTF-16 BE</root>".encode_utf16() {
        utf16_be.extend_from_slice(&ch.to_be_bytes());
    }
    let doc_utf16be = parse_bytes(&utf16_be).expect("UTF-16 BE BOM parse should succeed");
    assert_eq!(doc_utf16be.get_text_content(doc_utf16be.root_element_id().unwrap()), "UTF-16 BE");

    // UTF-16 LE with BOM
    let mut utf16_le = vec![0xFF, 0xFE];
    for ch in "<root>UTF-16 LE</root>".encode_utf16() {
        utf16_le.extend_from_slice(&ch.to_le_bytes());
    }
    let doc_utf16le = parse_bytes(&utf16_le).expect("UTF-16 LE BOM parse should succeed");
    assert_eq!(doc_utf16le.get_text_content(doc_utf16le.root_element_id().unwrap()), "UTF-16 LE");
}

#[test]
fn test_xml_source_from_file() {
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join("test_xml_source_temp.xml");

    let xml_content = "<library><book>The Iliad</book></library>";
    {
        let mut file = std::fs::File::create(&temp_file_path).expect("Create temp file");
        file.write_all(xml_content.as_bytes()).expect("Write temp file");
    }

    let source = XmlSource::from_file(&temp_file_path).expect("Open file source");
    let mut parser = xml_lib_rust::XmlParser::new(source, xml_lib_rust::ParseOptions::default());
    let doc = parser.parse().expect("Parse file source clean");

    assert_eq!(doc.get_text_content(doc.root_element_id().unwrap()), "The Iliad");

    let _ = std::fs::remove_file(temp_file_path);
}

#[test]
fn test_parse_file_convenience_function() {
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join("test_parse_file_temp.xml");

    let xml_content = "<note><to>Tove</to></note>";
    std::fs::write(&temp_file_path, xml_content).expect("Write temp file");

    let doc = xml_lib_rust::parse_file(&temp_file_path).expect("parse_file convenience function should succeed");
    assert_eq!(doc.get_text_content(doc.root_element_id().unwrap()), "Tove");

    let _ = std::fs::remove_file(temp_file_path);
}

#[test]
fn test_xml_destination_buffer_output() {
    let mut dest = XmlDestination::new();
    dest.write_str("<root>");
    dest.write_str("Hello");
    dest.write_str("</root>");

    assert_eq!(dest.as_str(), "<root>Hello</root>");
}
