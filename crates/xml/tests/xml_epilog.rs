use xml_lib_rust::{parse, XmlPullEvent, XmlPullParser};

#[test]
fn test_epilog_comments_and_pi() {
    let xml = r#"<root><item>val</item></root>
    <!-- Trailing epilog comment -->
    <?trailing_pi data="ok"?>
    "#;

    let doc = parse(xml).expect("Should parse with epilog comments and PIs");
    let root_id = doc.root_id().unwrap();
    let root_children = doc.get_children(root_id);

    // root element + comment + PI = 3 children under root virtual container
    assert_eq!(root_children.len(), 3);
}

#[test]
fn test_multiple_root_elements_error() {
    let xml = "<first/><second/>";
    let res = parse(xml);
    assert!(res.is_err(), "Multiple root elements must be rejected");
    if let Err(e) = res {
        let msg = e.to_string();
        assert!(msg.contains("Multiple root elements"));
    }
}

#[test]
fn test_invalid_xml_character_error() {
    let xml = "<root>\u{0001}</root>";
    let res = parse(xml);
    assert!(res.is_err(), "Control character U+0001 must be rejected");
    if let Err(e) = res {
        let msg = e.to_string();
        assert!(msg.contains("Forbidden XML character"));
    }
}

#[test]
fn test_pull_parser_self_closing_tag_pairing() {
    let xml = r#"<catalog><book id="b1"/><book id="b2"/></catalog>"#;
    let mut parser = XmlPullParser::new(xml);

    let mut events = Vec::new();
    while let Some(ev) = parser.next_event().unwrap() {
        if ev == XmlPullEvent::EndDocument {
            break;
        }
        events.push(ev);
    }

    assert_eq!(events.len(), 6);
    assert!(matches!(events[0], XmlPullEvent::StartElement { name: "catalog", .. }));
    assert!(matches!(events[1], XmlPullEvent::StartElement { name: "book", .. }));
    assert!(matches!(events[2], XmlPullEvent::EndElement { name: "book" }));
    assert!(matches!(events[3], XmlPullEvent::StartElement { name: "book", .. }));
    assert!(matches!(events[4], XmlPullEvent::EndElement { name: "book" }));
    assert!(matches!(events[5], XmlPullEvent::EndElement { name: "catalog" }));
}

#[test]
fn test_pull_parser_iterator_trait() {
    let xml = r#"<items><item>A</item><item>B</item></items>"#;
    let parser = XmlPullParser::new(xml);

    let events: Vec<XmlPullEvent> = parser.map(|res| res.unwrap()).collect();
    assert!(events.len() >= 6);
    assert!(events.iter().any(|e| matches!(e, XmlPullEvent::Text("A"))));
    assert!(events.iter().any(|e| matches!(e, XmlPullEvent::Text("B"))));
}
