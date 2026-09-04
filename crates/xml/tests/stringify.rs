use xml_lib_rust::{parse, stringify, SerializeOptions, XmlSerializer};

#[test]
fn test_roundtrip_stringify() {
    let xml = r#"<data attr="value &amp; &quot;quote&quot;"><item>Text &lt;content&gt;</item></data>"#;
    let doc = parse(xml).expect("Should parse XML");
    let output = stringify(&doc);
    assert!(output.contains("attr=\"value &amp; &quot;quote&quot;\""));
    assert!(output.contains("<item>Text &lt;content&gt;</item>"));
}

#[test]
fn test_serialize_options_pretty_print() {
    let xml = "<root><child1/><child2>val</child2></root>";
    let doc = parse(xml).expect("Should parse");

    let options = SerializeOptions {
        pretty_print: true,
        indent_step: 4,
        ..Default::default()
    };

    let serialized = XmlSerializer::serialize_to_string(&doc, &options);
    assert!(serialized.contains("<root>"));
    assert!(serialized.contains("</root>"));
}

#[test]
fn test_parse_stringify_reparse_equivalence() {
    let xml = r#"<catalog><book id="1"><title>Rust</title></book></catalog>"#;
    let doc1 = parse(xml).expect("Initial parse should succeed");
    let serialized = stringify(&doc1);
    let doc2 = parse(&serialized).expect("Reparse should succeed");

    assert_eq!(doc1.root_element_id().is_some(), doc2.root_element_id().is_some());
}
