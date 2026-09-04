//! # XML Encoding & BOM Detection Example
//!
//! Demonstrates automatic Byte Order Mark (BOM) detection and parsing for raw byte streams
//! (`UTF-8`, `UTF-16 LE`, `UTF-16 BE`) using `parse_bytes`.

use xml_lib_rust::{parse_bytes, XmlSource};

fn main() {
    println!("--- XML Encoding & BOM Example ---");

    // Raw byte stream starting with UTF-8 BOM preamble (0xEF, 0xBB, 0xBF)
    let utf8_bom_bytes = [
        0xEF, 0xBB, 0xBF, b'<', b'r', b'o', b'o', b't', b'>', b'O', b'K', b'<', b'/', b'r', b'o',
        b'o', b't', b'>',
    ];

    // Detect format and strip preamble
    let source = XmlSource::from_bytes(&utf8_bom_bytes).expect("BOM detection");
    println!("Detected BOM format: {:?}", source.format());

    // Parse bytes directly into DOM Document
    let doc = parse_bytes(&utf8_bom_bytes).expect("Parse BOM bytes");
    println!(
        "Successfully parsed XML with BOM detection: {} nodes",
        doc.len()
    );
}
