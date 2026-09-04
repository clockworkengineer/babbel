//! # XML Deep Nesting & Security Limit Example
//!
//! Demonstrates configuring `ParseOptions::max_nesting_depth` to protect applications
//! against stack overflow and DoS attacks from deeply nested element structures.

use xml_lib_rust::{parse_with_options, ParseOptions};

fn main() {
    println!("--- XML Deep Nesting Example ---");

    // Generate nested XML: <level1><level2>...</level2></level1>
    let depth = 50;
    let mut xml = String::new();
    for i in 1..=depth {
        xml.push_str(&format!("<level{i}>"));
    }
    xml.push_str("Deep Content");
    for i in (1..=depth).rev() {
        xml.push_str(&format!("</level{i}>"));
    }

    // Configure generous depth limit (max_nesting_depth = 100)
    let mut options = ParseOptions::default();
    options.max_nesting_depth = 100;

    match parse_with_options(&xml, options) {
        Ok(doc) => println!(
            "Successfully parsed document with nesting depth {depth}. Total nodes: {}",
            doc.len()
        ),
        Err(err) => eprintln!("Parse failed: {err}"),
    }

    // Configure strict depth limit (max_nesting_depth = 20)
    let mut strict_options = ParseOptions::default();
    strict_options.max_nesting_depth = 20;

    match parse_with_options(&xml, strict_options) {
        Ok(_) => println!("Unexpected success"),
        Err(err) => println!("Correctly rejected deep nesting with security limit: {err}"),
    }
}
