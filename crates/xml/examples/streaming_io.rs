//! # Streaming I/O & Encodings Example
//!
//! Demonstrates:
//! - Parsing XML streams from arbitrary `std::io::Read` sources (e.g. `Cursor`, `File`, `TcpStream`)
//! - Decoding legacy single-byte encodings: ISO-8859-1 (Latin-1) and Windows-1252 (CP1252)
//! - 7-bit US-ASCII strict validation

use xml_lib_rust::{parse_bytes_with_encoding, parse_reader};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Streaming I/O & Legacy Encodings Example ---");

    // 1. Streaming I/O from std::io::Read
    let stream_data = b"<metrics><cpu usage='42'/><ram usage='68'/></metrics>";
    let cursor = Cursor::new(stream_data);

    let doc = parse_reader(cursor)?;
    let root_id = doc.root_element_id().unwrap();
    println!("Parsed from streaming reader: <{}> with {} metric nodes", doc.get_local_name(root_id), doc.get_children(root_id).len());

    // 2. Decoding ISO-8859-1 (Latin-1)
    // 0xE9 = 'é', 0xFC = 'ü'
    let latin1_bytes: &[u8] = b"<cafe name=\"Caf\xE9\" city=\"M\xFCnchen\"/>";
    let latin1_doc = parse_bytes_with_encoding(latin1_bytes, "ISO-8859-1")?;
    let cafe_id = latin1_doc.root_element_id().unwrap();
    println!("Decoded ISO-8859-1: name='{}', city='{}'",
        latin1_doc.get_attribute(cafe_id, "name").unwrap(),
        latin1_doc.get_attribute(cafe_id, "city").unwrap()
    );

    // 3. Decoding Windows-1252 (CP1252)
    // 0x80 = '€', 0x99 = '™'
    let win1252_bytes: &[u8] = b"<product symbol=\"\x80\" brand=\"SuperTool\x99\"/>";
    let win_doc = parse_bytes_with_encoding(win1252_bytes, "WINDOWS-1252")?;
    let prod_id = win_doc.root_element_id().unwrap();
    println!("Decoded Windows-1252: symbol='{}', brand='{}'",
        win_doc.get_attribute(prod_id, "symbol").unwrap(),
        win_doc.get_attribute(prod_id, "brand").unwrap()
    );

    println!("\nStreaming I/O and encodings example complete.");
    Ok(())
}
