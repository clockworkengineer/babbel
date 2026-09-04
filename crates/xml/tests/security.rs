use xml_lib_rust::{parse_bytes, XmlError, ParseOptions, XmlParser, XmlSource};

#[test]
fn test_nesting_depth_limit() {
    let deep_xml = "<a>".repeat(1050) + &"</a>".repeat(1050);
    let source = XmlSource::from_string(&deep_xml);
    let mut options = ParseOptions::default();
    options.max_nesting_depth = 100;
    let mut parser = XmlParser::new(source, options);
    
    let res = parser.parse();
    assert!(matches!(res, Err(XmlError::SecurityLimitExceeded(_))));
}

#[test]
fn test_billion_laughs_entity_depth_limit() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE lolz [
  <!ENTITY lol "lol">
  <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
]>
<lolz>&lol1;</lolz>"#;

    let res = parse_bytes(xml.as_bytes());
    assert!(res.is_ok());
}
