//! # Canonical XML (W3C C14N) Example
//!
//! Demonstrates standard W3C Canonical XML 1.0/1.1 transformation:
//! - Sorting attribute declarations (with `xmlns` declarations first)
//! - Expanding empty elements (`<item/>` -> `<item></item>`)
//! - Standardizing line breaks and character escaping for digital signatures (XMLDSig)

use xml_lib_rust::{canonicalize, parse, CanonicalOptions, CanonicalSerializer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Canonical XML (C14N) Example ---");

    // Non-canonical XML with unsorted attributes, empty elements, and inconsistent whitespace
    let raw_xml = r#"<doc z:attr="val2"   a:attr="val1" xmlns:z="http://z" xmlns:a="http://a">
      <!-- Signature Comment -->
      <empty_tag/>
    </doc>"#;

    println!("Original XML:\n{raw_xml}\n");

    let doc = parse(raw_xml)?;

    // 1. Standard C14N Canonicalization (omits comments, sorts xmlns then attributes)
    let c14n_default = canonicalize(&doc);
    println!("Canonical XML (default, without comments):\n{c14n_default}\n");

    // 2. C14N Canonicalization preserving comments (W3C Canonical XML with Comments)
    let c14n_with_comments = CanonicalSerializer::canonicalize(
        &doc,
        &CanonicalOptions {
            with_comments: true,
        },
    );
    println!("Canonical XML (with comments):\n{c14n_with_comments}\n");

    println!("Canonical transformation complete.");
    Ok(())
}
