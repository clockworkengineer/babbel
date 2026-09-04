use xml_lib_rust::{
    parse, parse_with_options, Document, ParseOptions, XPathEngine, XPathValue, XmlError,
    XmlPullParser,
};

#[test]
fn test_unclosed_cdata_returns_error_no_panic() {
    let malformed = "<root><![CDATA[unclosed text without end marker";
    let res = parse(malformed);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SyntaxError { message, .. } => {
            assert!(message.contains("CDATA") || message.contains("EOF"));
        }
        other => panic!("Expected SyntaxError, got: {other:?}"),
    }
}

#[test]
fn test_unclosed_comment_returns_error_no_panic() {
    let malformed = "<root><!-- unclosed comment without closing delimiter";
    let res = parse(malformed);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SyntaxError { message, .. } => {
            assert!(message.contains("comment") || message.contains("EOF"));
        }
        other => panic!("Expected SyntaxError, got: {other:?}"),
    }
}

#[test]
fn test_unclosed_pi_returns_error_no_panic() {
    let malformed = "<root><?target data without closing question-mark";
    let res = parse(malformed);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SyntaxError { message, .. } => {
            assert!(message.contains("processing instruction") || message.contains("EOF"));
        }
        other => panic!("Expected SyntaxError, got: {other:?}"),
    }
}

#[test]
fn test_unclosed_doctype_subset_no_panic() {
    let malformed = "<!DOCTYPE root [ <!ELEMENT item EMPTY";
    let res = parse(malformed);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SyntaxError { .. } => {}
        other => panic!("Expected SyntaxError, got: {other:?}"),
    }
}

#[test]
fn test_pull_parser_unclosed_markup_terminates() {
    // Crucial regression test for SEC-01 infinite loop
    let malformed = "<root><unclosed";
    let mut parser = XmlPullParser::new(malformed);

    let mut iterations = 0;
    while let Ok(Some(_)) = parser.next_event() {
        iterations += 1;
        if iterations > 20 {
            panic!("XmlPullParser stuck in infinite loop on malformed tag!");
        }
    }
    // Pull parser must terminate safely in finite steps
    assert!(iterations < 10);
}

#[test]
fn test_pull_parser_unclosed_comment_terminates() {
    let malformed = "<!-- incomplete comment";
    let mut parser = XmlPullParser::new(malformed);

    let mut iterations = 0;
    while let Ok(Some(_)) = parser.next_event() {
        iterations += 1;
        if iterations > 20 {
            panic!("XmlPullParser stuck in infinite loop on unclosed comment!");
        }
    }
    assert!(iterations < 10);
}

#[test]
fn test_billion_laughs_expansion_size_limit() {
    // Construct exponential entity expansion XML (Billion Laughs)
    let xml = r#"
    <!DOCTYPE lolz [
        <!ENTITY lol0 "0123456789">
        <!ENTITY lol1 "&lol0;&lol0;">
        <!ENTITY lol2 "&lol1;&lol1;">
        <!ENTITY lol3 "&lol2;&lol2;">
        <!ENTITY lol4 "&lol3;&lol3;">
        <!ENTITY lol5 "&lol4;&lol4;">
        <!ENTITY lol6 "&lol5;&lol5;">
        <!ENTITY lol7 "&lol6;&lol6;">
        <!ENTITY lol8 "&lol7;&lol7;">
        <!ENTITY lol9 "&lol8;&lol8;">
        <!ENTITY lol10 "&lol9;&lol9;">
    ]>
    <lolz>&lol10;</lolz>
    "#;

    let mut opts = ParseOptions::default();
    // Cap total cumulative expansion to 500 bytes
    opts.max_total_entity_expansion_size = 500;

    let res = parse_with_options(xml, opts);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SecurityLimitExceeded(msg) => {
            assert!(msg.contains("expansion size") || msg.contains("Billion Laughs"));
        }
        other => panic!("Expected SecurityLimitExceeded for entity explosion, got: {other:?}"),
    }
}

#[test]
fn test_max_xml_size_enforced() {
    let xml = "<root><data>some content that exceeds the tiny 20-byte limit</data></root>";
    let mut opts = ParseOptions::default();
    opts.max_xml_size = 20;

    let res = parse_with_options(xml, opts);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SecurityLimitExceeded(msg) => {
            assert!(msg.contains("XML document size") && msg.contains("exceeded"));
        }
        other => panic!("Expected SecurityLimitExceeded, got: {other:?}"),
    }
}

#[test]
fn test_max_text_node_size_enforced() {
    let xml = "<root>This text string is definitely longer than twenty characters</root>";
    let mut opts = ParseOptions::default();
    opts.max_text_node_size = 20;

    let res = parse_with_options(xml, opts);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SecurityLimitExceeded(msg) => {
            assert!(msg.contains("text node size") && msg.contains("exceeded"));
        }
        other => panic!("Expected SecurityLimitExceeded, got: {other:?}"),
    }
}

#[test]
fn test_max_total_attributes_enforced() {
    let xml = r#"<root a="1" b="2"><child c="3" d="4"/></root>"#;
    let mut opts = ParseOptions::default();
    opts.max_total_attribute_count = 3;

    let res = parse_with_options(xml, opts);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SecurityLimitExceeded(msg) => {
            assert!(msg.contains("total attribute count") && msg.contains("exceeded"));
        }
        other => panic!("Expected SecurityLimitExceeded, got: {other:?}"),
    }
}

#[test]
fn test_disallowed_external_entities_in_doctype() {
    let xml = r#"
    <!DOCTYPE doc [
        <!ENTITY xxe SYSTEM "http://127.0.0.1:8080/secret">
    ]>
    <doc>&xxe;</doc>
    "#;

    let mut opts = ParseOptions::default();
    opts.allow_external_entities = false;

    let res = parse_with_options(xml, opts);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SecurityLimitExceeded(msg) => {
            assert!(msg.contains("External entity references in DOCTYPE are forbidden"));
        }
        other => panic!("Expected SecurityLimitExceeded, got: {other:?}"),
    }
}

#[test]
fn test_xpath_substring_negative_and_nan_no_panic() {
    let doc = Document::parse_str("<root><val>Hello World</val></root>").unwrap();
    let engine = XPathEngine::new(&doc);

    // Negative start offset
    let res1 = engine.evaluate("substring(//val, -5, 4)", None).unwrap();
    assert_eq!(res1, XPathValue::String("".into()));

    // Negative length
    let res2 = engine.evaluate("substring(//val, 1, -2)", None).unwrap();
    assert_eq!(res2, XPathValue::String("".into()));

    // Out of bound length
    let res3 = engine.evaluate("substring(//val, 1, 999999)", None).unwrap();
    assert_eq!(res3, XPathValue::String("Hello World".into()));

    // Zero length
    let res4 = engine.evaluate("substring(//val, 3, 0)", None).unwrap();
    assert_eq!(res4, XPathValue::String("".into()));
}

#[test]
fn test_xml_source_slice_range_safe_on_invalid_boundaries() {
    // "€" is 3 bytes (0xE2 0x82 0xAC)
    let source = xml_lib_rust::XmlSource::from_string("€uro");
    assert_eq!(source.len(), 6);

    // Slicing inside the multibyte character (byte 1 to 2)
    let slice = source.slice_range(1, 2);
    // Must return safe empty slice rather than panicking
    assert_eq!(slice, "");

    // Slicing on valid boundary
    let valid_slice = source.slice_range(0, 3);
    assert_eq!(valid_slice, "€");

    // Slicing past EOF
    let oob_slice = source.slice_range(10, 20);
    assert_eq!(oob_slice, "");
}

#[test]
fn test_dom_hierarchy_cycle_prevention() {
    let mut doc = Document::new();
    let root = doc.create_element("root");
    let child = doc.create_element("child");
    let grandchild = doc.create_element("grandchild");

    doc.append_child(root, child).unwrap();
    doc.append_child(child, grandchild).unwrap();

    // 1. Cannot append node into itself
    let self_append = doc.append_child(root, root);
    assert!(self_append.is_err());
    assert!(self_append.unwrap_err().to_string().contains("HierarchyRequestError"));

    // 2. Cannot append ancestor as child of descendant (would create cycle)
    let cycle_append = doc.append_child(grandchild, root);
    assert!(cycle_append.is_err());
    assert!(cycle_append.unwrap_err().to_string().contains("HierarchyRequestError"));

    // 3. Cannot insert ancestor before reference child
    let leaf = doc.create_element("leaf");
    doc.append_child(grandchild, leaf).unwrap();
    let cycle_insert = doc.insert_before(grandchild, root, leaf);
    assert!(cycle_insert.is_err());
    assert!(cycle_insert.unwrap_err().to_string().contains("HierarchyRequestError"));
}

#[test]
fn test_streaming_io_limit_enforced() {
    use std::io::Read;
    // 2 KB stream with a 1 KB limit
    let stream = std::io::repeat(b'x').take(2048);
    let res = xml_lib_rust::XmlSource::from_reader_with_limit(stream, 1024);
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::SecurityLimitExceeded(msg) => {
            assert!(msg.contains("Input stream exceeds maximum allowed XML stream size"));
        }
        other => panic!("Expected SecurityLimitExceeded, got: {other:?}"),
    }

    // Valid stream within limit
    let valid_stream = "<root><data>123</data></root>".as_bytes();
    let res_ok = xml_lib_rust::XmlSource::from_reader_with_limit(valid_stream, 1024);
    assert!(res_ok.is_ok());
}

#[test]
fn test_xpath_parser_depth_limit() {
    // Generate 150 nested parentheses: (((... 42 ...)))
    let mut expr = String::new();
    for _ in 0..150 {
        expr.push('(');
    }
    expr.push_str("42");
    for _ in 0..150 {
        expr.push(')');
    }

    let mut parser = xml_lib_rust::xpath::parser::XPathParser::new(&expr).unwrap();
    let res = parser.parse_expression();
    assert!(res.is_err());
    match res.unwrap_err() {
        XmlError::XPathError(msg) => {
            assert!(msg.contains("maximum nesting depth"));
        }
        other => panic!("Expected XPathError, got: {other:?}"),
    }
}

#[test]
fn test_pull_parser_attribute_iteration_clean_termination() {
    use xml_lib_rust::XmlPullParser;
    // Malformed attribute with unclosed quote
    let xml = r#"<tag valid="yes" unclosed="no_end_quote"#;
    let mut parser = XmlPullParser::new(xml);
    let event = parser.next_event();
    // Pull parser next_event returns syntax error on unclosed tag
    assert!(event.is_err());

    // Well-formed element with malformed attribute trailer
    let xml2 = r#"<tag valid="yes" trailing_garbage >"#;
    let mut parser2 = XmlPullParser::new(xml2);
    if let Ok(Some(ev)) = parser2.next_event() {
        let mut iter = ev.attributes();
        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().name, "valid");
        // Next attribute is malformed -> must return None cleanly
        assert!(iter.next().is_none());
        // Subsequent calls must also return None cleanly
        assert!(iter.next().is_none());
    }
}

#[test]
fn test_validator_and_serializer_depth_constants() {
    use xml_lib_rust::dtd::validator::DtdValidator;
    use xml_lib_rust::xsd::validator::XsdValidator;
    use xml_lib_rust::stringify::serializer::XmlSerializer;
    use xml_lib_rust::stringify::canonical::CanonicalSerializer;

    assert_eq!(DtdValidator::MAX_VALIDATION_DEPTH, 512);
    assert_eq!(XsdValidator::MAX_VALIDATION_DEPTH, 512);
    assert_eq!(XmlSerializer::MAX_SERIALIZE_DEPTH, 512);
    assert_eq!(CanonicalSerializer::MAX_CANONICAL_DEPTH, 512);
}

