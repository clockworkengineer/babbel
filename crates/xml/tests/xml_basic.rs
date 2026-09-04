use xml_lib_rust::{parse, stringify, NodeKind};

#[test]
fn test_basic_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root attr="test">
  <child>Hello World</child>
</root>"#;

    let doc = parse(xml).expect("Parsing should succeed");
    assert!(!doc.is_empty());

    let root_elem_id = doc.root_element_id().expect("Should have root element");
    let root_node = doc.get_node(root_elem_id).expect("Root node exists");
    
    if let NodeKind::Element { name, attributes } = &root_node.kind {
        assert_eq!(&**name, "root");
        assert_eq!(attributes.len(), 1);
        assert_eq!(&*attributes[0].name, "attr");
        assert_eq!(&*attributes[0].value, "test");
    } else {
        panic!("Expected Element node");
    }

    let stringified = stringify(&doc);
    assert!(stringified.contains("<root attr=\"test\">"));
    assert!(stringified.contains("<child>Hello World</child>"));
}

#[test]
fn test_parsing_comments_and_pi() {
    let xml = r#"<?xml version="1.0"?>
<!-- This is a comment -->
<?target action="run"?>
<root>Text</root>"#;

    let doc = parse(xml).expect("Parsing comments and PI should succeed");
    let mut has_comment = false;
    let mut has_pi = false;

    for node in doc.nodes() {
        match &node.kind {
            NodeKind::Comment(c) => {
                assert_eq!(c.trim(), "This is a comment");
                has_comment = true;
            }
            NodeKind::ProcessingInstruction { target, data } => {
                assert_eq!(&**target, "target");
                assert_eq!(&**data, "action=\"run\"");
                has_pi = true;
            }
            _ => {}
        }
    }

    assert!(has_comment, "Should find comment node");
    assert!(has_pi, "Should find PI node");
}

#[test]
fn test_parsing_cdata() {
    let xml = r#"<root><![CDATA[<unescaped & content>]]></root>"#;

    let doc = parse(xml).expect("CDATA parsing should succeed");
    let mut found_cdata = false;

    for node in doc.nodes() {
        if let NodeKind::CData(content) = &node.kind {
            assert_eq!(&**content, "<unescaped & content>");
            found_cdata = true;
        }
    }

    assert!(found_cdata, "Should find CDATA node");
}

#[test]
fn test_malformed_xml_errors() {
    let unclosed = "<root><child>";
    assert!(parse(unclosed).is_err(), "Unclosed tag should produce error");

    let mismatched = "<root></other>";
    assert!(parse(mismatched).is_err(), "Mismatched closing tag should produce error");
}

#[test]
fn test_document_navigation() {
    let xml = r#"<parent><child1/><child2><sub/></child2></parent>"#;
    let doc = parse(xml).expect("Should parse parent-child structure");

    let root_id = doc.root_element_id().expect("Root element exists");
    let children = doc.get_children(root_id);
    assert_eq!(children.len(), 2, "Parent should have 2 children");

    let child2_id = children[1];
    assert_eq!(doc.parent_id(child2_id), Some(root_id));
}
