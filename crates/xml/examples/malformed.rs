//! # XML Malformed Syntax & Error Handling Example
//!
//! Demonstrates error handling for invalid XML syntax inputs (mismatched closing tags,
//! unclosed attributes, unclosed tags) using `XmlError::SyntaxError`.

use xml_lib_rust::parse;

fn main() {
    println!("--- XML Malformed Errors Example ---");
    let malformed_xmls = [
        "<note><to>Alice</from></note>", // mismatched closing tag
        "<note attr=\"unclosed>",        // unclosed attribute quote
        "<root><child></root>",          // unclosed inner element tag
    ];

    // Iterate across malformed XML test cases and capture line/column syntax errors
    for (i, xml) in malformed_xmls.iter().enumerate() {
        match parse(xml) {
            Ok(_) => println!("Case {}: Unexpected pass", i + 1),
            Err(err) => println!("Case {}: Correctly caught syntax error:\n  -> {err}", i + 1),
        }
    }
}
