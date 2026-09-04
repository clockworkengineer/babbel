use xml_lib_rust::{XmlPullEvent, XmlPullParser};

#[test]
fn test_zero_allocation_pull_parser_events() {
    let xml = r#"<sensor id="temp_01" location="lab"><val>24.5</val></sensor>"#;

    let mut parser = XmlPullParser::new(xml);

    let ev1 = parser.next_event().unwrap().unwrap();
    if let XmlPullEvent::StartElement { name, .. } = ev1 {
        assert_eq!(name, "sensor");
        let attrs: Vec<_> = ev1.attributes().collect();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].name, "id");
        assert_eq!(attrs[0].value, "temp_01");
        assert_eq!(attrs[1].name, "location");
        assert_eq!(attrs[1].value, "lab");
    } else {
        panic!("Expected StartElement sensor");
    }

    let ev2 = parser.next_event().unwrap().unwrap();
    if let XmlPullEvent::StartElement { name, .. } = ev2 {
        assert_eq!(name, "val");
        assert_eq!(ev2.attributes().count(), 0);
    } else {
        panic!("Expected StartElement val");
    }

    let ev3 = parser.next_event().unwrap().unwrap();
    assert_eq!(ev3, XmlPullEvent::Text("24.5"));

    let ev4 = parser.next_event().unwrap().unwrap();
    assert_eq!(ev4, XmlPullEvent::EndElement { name: "val" });

    let ev5 = parser.next_event().unwrap().unwrap();
    assert_eq!(ev5, XmlPullEvent::EndElement { name: "sensor" });

    assert!(parser.next_event().unwrap().is_none());
}
