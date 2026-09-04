use xml_lib_rust::{
    parse, stringify, NodeKind,
};

// --- Parse Attributes Tests ---

#[test]
fn test_parse_attributes_single_and_multiple() {
    let xml1 = r#"<?xml version="1.0"?>
<AddressBook number='15'>
</AddressBook>"#;
    let doc1 = parse(xml1).expect("Parsing single attribute should succeed");
    let root1_id = doc1.root_element_id().unwrap();
    let root1 = doc1.get_node(root1_id).unwrap();
    if let NodeKind::Element { name, attributes } = &root1.kind {
        assert_eq!(&**name, "AddressBook");
        assert_eq!(attributes.len(), 1);
        assert_eq!(&*attributes[0].name, "number");
        assert_eq!(&*attributes[0].value, "15");
    } else {
        panic!("Expected Element node");
    }

    let xml2 = r#"<?xml version="1.0"?>
<AddressBook number='15' away="yes" flat='no'>
</AddressBook>"#;
    let doc2 = parse(xml2).expect("Parsing multiple attributes should succeed");
    let root2_id = doc2.root_element_id().unwrap();
    let root2 = doc2.get_node(root2_id).unwrap();
    if let NodeKind::Element { attributes, .. } = &root2.kind {
        assert_eq!(attributes.len(), 3);
        assert_eq!(&*attributes[0].name, "number");
        assert_eq!(&*attributes[0].value, "15");
        assert_eq!(&*attributes[1].name, "away");
        assert_eq!(&*attributes[1].value, "yes");
        assert_eq!(&*attributes[2].name, "flat");
        assert_eq!(&*attributes[2].value, "no");
    }
}

#[test]
fn test_parse_attributes_quotes_and_entities() {
    let xml_dq = r#"<gangster name='George "Shotgun" Ziegler'></gangster>"#;
    let doc_dq = parse(xml_dq).expect("Double quotes inside single quotes should succeed");
    let root_dq_id = doc_dq.root_element_id().unwrap();
    if let NodeKind::Element { attributes, .. } = &doc_dq.get_node(root_dq_id).unwrap().kind {
        assert_eq!(&*attributes[0].value, r#"George "Shotgun" Ziegler"#);
    }

    let xml_sq = r#"<gangster name="George 'Shotgun' Ziegler"></gangster>"#;
    let doc_sq = parse(xml_sq).expect("Single quotes inside double quotes should succeed");
    let root_sq_id = doc_sq.root_element_id().unwrap();
    if let NodeKind::Element { attributes, .. } = &doc_sq.get_node(root_sq_id).unwrap().kind {
        assert_eq!(&*attributes[0].value, "George 'Shotgun' Ziegler");
    }

    let xml_ws = r#"<AddressBook   number   =   '  15  '   ></AddressBook>"#;
    let doc_ws = parse(xml_ws).expect("Whitespace around attribute equals should succeed");
    let root_ws_id = doc_ws.root_element_id().unwrap();
    if let NodeKind::Element { attributes, .. } = &doc_ws.get_node(root_ws_id).unwrap().kind {
        assert_eq!(&*attributes[0].value, "  15  ");
    }
}

#[test]
fn test_parse_attributes_errors() {
    let duplicate = r#"<AddressBook number='15' colour='red' number='16'></AddressBook>"#;
    assert!(parse(duplicate).is_err(), "Duplicate attributes should fail");

    let no_val = r#"<AddressBook number=></AddressBook>"#;
    assert!(parse(no_val).is_err(), "Attribute without value should fail");

    let unquoted = r#"<AddressBook number=15></AddressBook>"#;
    assert!(parse(unquoted).is_err(), "Unquoted attribute value should fail");
}

// --- Parse Namespaces & QName Tests ---

#[test]
fn test_parse_namespaces_declarations() {
    let xml = r#"<root>
<h:table xmlns:h="http://www.w3.org/TR/html4/">
<h:tr>
<h:td>Apples</h:td>
<h:td>Bananas</h:td>
</h:tr>
</h:table>
<f:table xmlns:f="https://www.w3schools.com/furniture">
<f:name>African Coffee Table</f:name>
<f:width>80</f:width>
<f:length>120</f:length>
</f:table>
</root>"#;

    let doc = parse(xml).expect("Namespace parsing should succeed");
    let root_id = doc.root_element_id().unwrap();
    let elem_children: Vec<_> = doc.get_children(root_id).into_iter().filter(|&id| {
        matches!(doc.get_node(id).unwrap().kind, NodeKind::Element { .. })
    }).collect();

    assert_eq!(elem_children.len(), 2);
    assert_eq!(doc.get_node(elem_children[0]).unwrap().kind.name(), "h:table");
    assert_eq!(doc.get_node(elem_children[1]).unwrap().kind.name(), "f:table");
}

#[test]
fn test_parse_namespaces_root_declaration() {
    let xml = r#"<root xmlns:h="http://www.w3.org/TR/html4/" xmlns:f="https://www.w3schools.com/furniture">
<h:table>
<h:tr><h:td>Apples</h:td></h:tr>
</h:table>
<f:table>
<f:name>African Coffee Table</f:name>
</f:table>
</root>"#;

    let doc = parse(xml).expect("Root namespace declaration should succeed");
    let root_id = doc.root_element_id().unwrap();
    let elem_children: Vec<_> = doc.get_children(root_id).into_iter().filter(|&id| {
        matches!(doc.get_node(id).unwrap().kind, NodeKind::Element { .. })
    }).collect();

    assert_eq!(elem_children.len(), 2);
    assert_eq!(doc.get_node(elem_children[0]).unwrap().kind.name(), "h:table");
    assert_eq!(doc.get_node(elem_children[1]).unwrap().kind.name(), "f:table");
}

#[test]
fn test_parse_namespaces_default_and_override() {
    let xml = r#"<root xmlns="http://www.w3.org/TR/html4/">
<table xmlns="https://www.w3schools.com/furniture">
<tr><td>Apples</td></tr>
</table>
</root>"#;

    let doc = parse(xml).expect("Default namespace override should succeed");
    let root_id = doc.root_element_id().unwrap();
    let elem_children: Vec<_> = doc.get_children(root_id).into_iter().filter(|&id| {
        matches!(doc.get_node(id).unwrap().kind, NodeKind::Element { .. })
    }).collect();

    assert_eq!(elem_children.len(), 1);
    assert_eq!(doc.get_node(elem_children[0]).unwrap().kind.name(), "table");
}

// --- Roundtrip Stringify Tests ---

#[test]
fn test_stringify_roundtrip_basic() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root attr="1"><child>Hello World</child></root>"#;
    let doc = parse(xml).expect("Parsing should succeed");
    let out = stringify(&doc);
    let doc2 = parse(&out).expect("Reparsing stringified XML should succeed");

    assert_eq!(
        doc.get_node(doc.root_element_id().unwrap()).unwrap().kind.name(),
        doc2.get_node(doc2.root_element_id().unwrap()).unwrap().kind.name()
    );
}

#[test]
fn test_stringify_roundtrip_namespaces() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:h="http://www.w3.org/TR/html4/"><h:table><h:tr><h:td>Apples</h:td></h:tr></h:table></root>"#;
    let doc = parse(xml).expect("Parsing namespace XML should succeed");
    let out = stringify(&doc);
    assert!(out.contains("xmlns:h=\"http://www.w3.org/TR/html4/\""));
    assert!(out.contains("<h:table>"));
}

// --- Unicode & Security Tests ---

#[test]
fn test_unicode_character_parsing() {
    let xml = r#"<note title="测试">Unicode content: 𝄞𝄢𝄫 and 💖</note>"#;
    let doc = parse(xml).expect("Unicode XML parsing should succeed");
    let root_id = doc.root_element_id().unwrap();
    let text = doc.get_text_content(root_id);
    assert!(text.contains("𝄞𝄢𝄫"));
    assert!(text.contains("💖"));
}

#[test]
fn test_security_depth_limits() {
    use xml_lib_rust::{parse_with_options, ParseOptions};

    let mut options = ParseOptions::default();
    options.max_nesting_depth = 20;

    let mut deep_xml = String::new();
    for i in 0..50 {
        deep_xml.push_str(&format!("<tag{}>", i));
    }
    deep_xml.push_str("deep text");
    for i in (0..50).rev() {
        deep_xml.push_str(&format!("</tag{}>", i));
    }

    let res = parse_with_options(&deep_xml, options);
    assert!(res.is_err(), "Deeply nested XML exceeding depth limit should fail cleanly");
}
