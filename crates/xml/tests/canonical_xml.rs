use xml_lib_rust::{
    canonicalize, parse, CanonicalOptions, CanonicalSerializer, SerializeOptions, XmlSerializer,
};

#[test]
fn test_serialize_options_omit_declaration() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root><item/></root>"#;
    let doc = parse(xml).unwrap();

    let mut opts = SerializeOptions::default();
    opts.omit_xml_declaration = true;
    opts.pretty_print = false;

    let output = XmlSerializer::serialize_to_string(&doc, &opts);
    assert!(!output.starts_with("<?xml"));
    assert!(output.starts_with("<root"));
}

#[test]
fn test_serialize_options_single_quotes() {
    let xml = r#"<item name="book" id="101"/>"#;
    let doc = parse(xml).unwrap();

    let mut opts = SerializeOptions::default();
    opts.quote_char = '\'';
    opts.omit_xml_declaration = true;
    opts.pretty_print = false;

    let output = XmlSerializer::serialize_to_string(&doc, &opts);
    assert!(output.contains("name='book'"));
    assert!(output.contains("id='101'"));
}

#[test]
fn test_serialize_options_no_self_closing() {
    let xml = r#"<root><empty/></root>"#;
    let doc = parse(xml).unwrap();

    let mut opts = SerializeOptions::default();
    opts.self_close_empty = false;
    opts.omit_xml_declaration = true;
    opts.pretty_print = false;

    let output = XmlSerializer::serialize_to_string(&doc, &opts);
    assert!(output.contains("<empty></empty>"));
    assert!(!output.contains("<empty/>"));
}

#[test]
fn test_serialize_to_writer() {
    let xml = r#"<data value="42"/>"#;
    let doc = parse(xml).unwrap();

    let mut buffer = Vec::new();
    XmlSerializer::serialize_to_writer(&doc, &mut buffer, &SerializeOptions::default()).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains(r#"<data value="42"/>"#));
}

#[test]
fn test_canonical_xml_attribute_sorting_and_empty_tags() {
    // Unsorted attributes, self-closing tags
    let xml = r#"<item z="last" a="first" m="middle"><empty/></item>"#;
    let doc = parse(xml).unwrap();

    let canonical = canonicalize(&doc);

    // C14N requires:
    // 1. Attributes sorted alphabetically: a="first" m="middle" z="last"
    // 2. Empty tags expanded to <empty></empty>
    // 3. No declaration
    assert_eq!(
        canonical,
        r#"<item a="first" m="middle" z="last"><empty></empty></item>"#
    );
}

#[test]
fn test_canonical_xml_comments_handling() {
    let xml = r#"<root><!-- Ignored by default --><data>hello</data></root>"#;
    let doc = parse(xml).unwrap();

    // Without comments
    let c1 = canonicalize(&doc);
    assert_eq!(c1, "<root><data>hello</data></root>");

    // With comments
    let mut opts = CanonicalOptions::default();
    opts.with_comments = true;
    let c2 = CanonicalSerializer::canonicalize(&doc, &opts);
    assert_eq!(c2, "<root><!-- Ignored by default --><data>hello</data></root>");
}
