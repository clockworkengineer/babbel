use xml_lib_rust::{
    Attribute, Document, NodeKind, parse,
    EntityMapper, DtdValidator,
};

#[test]
fn test_element_node_creation_and_attributes() {
    let mut doc = Document::new();
    let elem_id = doc.add_node(NodeKind::Element {
        name: "test".into(),
        attributes: vec![
            Attribute::new("attr1", "value1"),
            Attribute::new("attr2", "value2"),
            Attribute::new("attr3", "value3"),
        ],
    });

    let node = doc.get_node(elem_id).expect("Element node exists");
    assert_eq!(node.kind.name(), "test");
    if let NodeKind::Element { name, attributes } = &node.kind {
        assert_eq!(&**name, "test");
        assert_eq!(attributes.len(), 3);
        assert_eq!(&*attributes[0].name, "attr1");
        assert_eq!(&*attributes[0].value, "value1");
        assert_eq!(&*attributes[1].name, "attr2");
        assert_eq!(&*attributes[1].value, "value2");
        assert_eq!(&*attributes[2].name, "attr3");
        assert_eq!(&*attributes[2].value, "value3");
    } else {
        panic!("Expected NodeKind::Element");
    }
}

#[test]
fn test_element_node_with_children() {
    let mut doc = Document::new();
    let parent_id = doc.add_node(NodeKind::Element {
        name: "element".into(),
        attributes: vec![],
    });
    let child_id = doc.add_node(NodeKind::Text("child content".into()));
    doc.append_child(parent_id, child_id).unwrap();

    let children = doc.get_children(parent_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], child_id);

    let child_node = doc.get_node(child_id).unwrap();
    assert_eq!(child_node.kind, NodeKind::Text("child content".into()));
    assert_eq!(doc.get_text_content(parent_id), "child content");
}

#[test]
fn test_element_xml_parsing_integration() {
    let xml = r#"<?xml version="1.0"?>
<root><element attr='1'>data</element></root>"#;
    let doc = parse(xml).expect("XML parse clean");

    let root_elem_id = doc.root_element_id().expect("Has root element");
    let root_node = doc.get_node(root_elem_id).unwrap();
    assert_eq!(root_node.kind.name(), "root");

    let children = doc.get_children(root_elem_id);
    assert_eq!(children.len(), 1);
    let elem_node = doc.get_node(children[0]).unwrap();
    assert_eq!(elem_node.kind.name(), "element");
    assert_eq!(doc.get_text_content(children[0]), "data");
}

#[test]
fn test_content_text_node() {
    let mut doc = Document::new();
    let text_id = doc.add_node(NodeKind::Text("Sample Text Node".into()));
    let node = doc.get_node(text_id).unwrap();

    assert_eq!(node.kind, NodeKind::Text("Sample Text Node".into()));
    assert_eq!(node.kind.name(), "#text");
}

#[test]
fn test_cdata_node_creation_and_attributes() {
    let mut doc = Document::new();
    let cdata_id = doc.add_node(NodeKind::CData("Some CDATA Content".into()));
    let node = doc.get_node(cdata_id).unwrap();

    assert_eq!(node.kind, NodeKind::CData("Some CDATA Content".into()));
    assert_eq!(node.kind.name(), "#cdata");
}

#[test]
fn test_cdata_stringify_output() {
    let mut doc = Document::new();
    let root_id = doc.root_id().unwrap();
    let elem_id = doc.add_node(NodeKind::Element {
        name: "root".into(),
        attributes: vec![],
    });
    let cdata_id = doc.add_node(NodeKind::CData("<raw>&data</raw>".into()));
    doc.append_child(elem_id, cdata_id).unwrap();
    doc.append_child(root_id, elem_id).unwrap();

    let output = xml_lib_rust::stringify(&doc);
    assert!(output.contains("<![CDATA[<raw>&data</raw>]]>"));
}

#[test]
fn test_comment_node_creation_and_attributes() {
    let mut doc = Document::new();
    let comment_id = doc.add_node(NodeKind::Comment("This is a test comment".into()));
    let node = doc.get_node(comment_id).unwrap();

    assert_eq!(node.kind, NodeKind::Comment("This is a test comment".into()));
    assert_eq!(node.kind.name(), "#comment");
}

#[test]
fn test_comment_stringify_output() {
    let mut doc = Document::new();
    let root_id = doc.root_id().unwrap();
    let comment_id = doc.add_node(NodeKind::Comment("Header comment".into()));
    let prolog_id = doc.prolog_id().unwrap();
    doc.append_child(prolog_id, comment_id).unwrap();

    let elem_id = doc.add_node(NodeKind::Element {
        name: "data".into(),
        attributes: vec![],
    });
    doc.append_child(root_id, elem_id).unwrap();

    let output = xml_lib_rust::stringify(&doc);
    assert!(output.contains("<!--Header comment-->"));
}

#[test]
fn test_processing_instruction_node() {
    let mut doc = Document::new();
    let pi_id = doc.add_node(NodeKind::ProcessingInstruction {
        target: "php".into(),
        data: "echo 'hello';".into(),
    });

    let node = doc.get_node(pi_id).unwrap();
    assert_eq!(
        node.kind,
        NodeKind::ProcessingInstruction {
            target: "php".into(),
            data: "echo 'hello';".into(),
        }
    );
    assert_eq!(node.kind.name(), "php");
}

#[test]
fn test_declaration_node() {
    let mut doc = Document::new();
    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".into(),
        encoding: Some("UTF-8".into()),
        standalone: Some(true),
    });

    let node = doc.get_node(decl_id).unwrap();
    assert_eq!(node.kind.name(), "xml");

    if let NodeKind::Declaration {
        version,
        encoding,
        standalone,
    } = &node.kind
    {
        assert_eq!(&**version, "1.0");
        assert_eq!(encoding.as_deref(), Some("UTF-8"));
        assert_eq!(*standalone, Some(true));
    } else {
        panic!("Expected NodeKind::Declaration");
    }
}

#[test]
fn test_declaration_stringify_output() {
    let mut doc = Document::new();
    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".into(),
        encoding: Some("UTF-8".into()),
        standalone: Some(true),
    });
    doc.set_declaration_id(decl_id);

    let root_id = doc.root_id().unwrap();
    let elem_id = doc.add_node(NodeKind::Element {
        name: "root".into(),
        attributes: vec![],
    });
    doc.append_child(root_id, elem_id).unwrap();

    let output = xml_lib_rust::stringify(&doc);
    assert!(output.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"));
}

#[test]
fn test_prolog_node_and_children() {
    let mut doc = Document::new();
    let prolog_id = doc.prolog_id().expect("Prolog container exists");
    let prolog_node = doc.get_node(prolog_id).unwrap();
    assert_eq!(prolog_node.kind, NodeKind::Prolog);

    let comment_id = doc.add_node(NodeKind::Comment("Prolog comment".into()));
    doc.append_child(prolog_id, comment_id).unwrap();

    let children = doc.get_children(prolog_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], comment_id);
}

#[test]
fn test_prolog_xml_integration() {
    let xml = r#"<?xml version="1.0"?>
<!-- Prolog Comment -->
<?pi target="test"?>
<root/>"#;
    let doc = parse(xml).expect("XML parse clean");

    let prolog_id = doc.prolog_id().expect("Prolog exists");
    let children = doc.get_children(prolog_id);
    assert!(children.len() >= 2, "Prolog should contain declaration, comment, and PI");
}

#[test]
fn test_root_node_and_children() {
    let mut doc = Document::new();
    let root_container_id = doc.root_id().expect("Document has root container");
    assert_eq!(doc.get_node(root_container_id).unwrap().kind, NodeKind::Root);

    let root_elem_id = doc.add_node(NodeKind::Element {
        name: "root".into(),
        attributes: vec![Attribute::new("attr1", "1")],
    });
    doc.append_child(root_container_id, root_elem_id).unwrap();

    assert_eq!(doc.root_element_id(), Some(root_elem_id));
    let children = doc.get_children(root_container_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], root_elem_id);
}

#[test]
fn test_root_xml_integration() {
    let xml = r#"<?xml version="1.0"?>
<root attr1='1'><child>data</child></root>"#;
    let doc = parse(xml).expect("XML parse clean");

    let root_elem_id = doc.root_element_id().expect("Root element present");
    let root_node = doc.get_node(root_elem_id).unwrap();
    assert_eq!(root_node.kind.name(), "root");

    if let NodeKind::Element { attributes, .. } = &root_node.kind {
        assert_eq!(attributes.len(), 1);
        assert_eq!(&*attributes[0].name, "attr1");
        assert_eq!(&*attributes[0].value, "1");
    }

    let children = doc.get_children(root_elem_id);
    assert_eq!(children.len(), 1);
    let child_node = doc.get_node(children[0]).unwrap();
    assert_eq!(child_node.kind.name(), "child");
    assert_eq!(doc.get_text_content(children[0]), "data");
}

#[test]
fn test_dtd_node_creation_and_elements() {
    let validator = DtdValidator::new();
    assert!(validator.validate(&Document::new()).is_ok());
}

#[test]
fn test_entity_reference_node() {
    let mut doc = Document::new();

    // Entity Reference Node
    let entity_id = doc.add_node(NodeKind::EntityReference("foo".into()));
    let node = doc.get_node(entity_id).unwrap();
    assert_eq!(node.kind, NodeKind::EntityReference("foo".into()));
    assert_eq!(node.kind.name(), "foo");

    // EntityMapper resolution
    let mut mapper = EntityMapper::default();
    mapper.register("foo", "bar");
    assert_eq!(mapper.expand("&foo;").unwrap(), "bar");
    assert!(mapper.expand("&unknown;").is_err());
}

#[test]
fn test_variant_node_types() {
    let mut doc = Document::new();

    let prolog = doc.add_node(NodeKind::Prolog);
    let elem = doc.add_node(NodeKind::Element { name: "test".into(), attributes: vec![] });
    let text = doc.add_node(NodeKind::Text("text".into()));

    assert_eq!(doc.get_node(prolog).unwrap().kind.name(), "#prolog");
    assert_eq!(doc.get_node(elem).unwrap().kind.name(), "test");
    assert_eq!(doc.get_node(text).unwrap().kind.name(), "#text");

    // Parent / Child relationship
    doc.append_child(elem, text).unwrap();
    assert_eq!(doc.get_children(elem), vec![text]);
    assert_eq!(doc.parent_id(text), Some(elem));
}
