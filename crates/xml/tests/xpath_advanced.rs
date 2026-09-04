use xml_lib_rust::{parse, XPathEngine, XPathValue};

#[test]
fn test_xpath_position_and_last() {
    let xml = r#"
    <catalog>
        <book id="1"><title>First</title></book>
        <book id="2"><title>Second</title></book>
        <book id="3"><title>Third</title></book>
    </catalog>
    "#;
    let doc = parse(xml).unwrap();
    let engine = XPathEngine::new(&doc);

    // position() = 1
    let first = engine.evaluate_nodes("//book[position() = 1]", None).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(doc.get_attribute(first[0], "id"), Some("1"));

    // position() = last()
    let last = engine.evaluate_nodes("//book[position() = last()]", None).unwrap();
    assert_eq!(last.len(), 1);
    assert_eq!(doc.get_attribute(last[0], "id"), Some("3"));

    // position() <= 2
    let first_two = engine.evaluate_nodes("//book[position() <= 2]", None).unwrap();
    assert_eq!(first_two.len(), 2);
}

#[test]
fn test_xpath_id_function() {
    let xml = r#"
    <catalog>
        <book id="b1"><title>Rust</title></book>
        <book id="b2"><title>C++</title></book>
    </catalog>
    "#;
    let doc = parse(xml).unwrap();
    let engine = XPathEngine::new(&doc);

    let res = engine.evaluate("id('b2')", None).unwrap();
    if let XPathValue::NodeSet(ns) = res {
        assert_eq!(ns.len(), 1);
        assert_eq!(doc.get_attribute(ns[0], "id"), Some("b2"));
    } else {
        panic!("Expected NodeSet");
    }
}

#[test]
fn test_xpath_namespace_uri_function() {
    let xml = r#"
    <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
        <soap:Body/>
    </soap:Envelope>
    "#;
    let doc = parse(xml).unwrap();
    let engine = XPathEngine::new(&doc);

    let res = engine.evaluate("namespace-uri(//soap:Envelope)", None).unwrap();
    assert_eq!(res, XPathValue::String("http://schemas.xmlsoap.org/soap/envelope/".into()));
}

#[test]
fn test_xpath_lang_function() {
    let xml = r#"
    <doc>
        <p id="p1" xml:lang="en-US">English</p>
        <p id="p2" xml:lang="fr">French</p>
    </doc>
    "#;
    let doc = parse(xml).unwrap();
    let engine = XPathEngine::new(&doc);

    let p1 = doc.get_element_by_id("p1").unwrap();
    let p2 = doc.get_element_by_id("p2").unwrap();

    let res1 = engine.evaluate("lang('en')", Some(p1)).unwrap();
    assert_eq!(res1, XPathValue::Boolean(true));

    let res2 = engine.evaluate("lang('fr')", Some(p1)).unwrap();
    assert_eq!(res2, XPathValue::Boolean(false));

    let res3 = engine.evaluate("lang('fr')", Some(p2)).unwrap();
    assert_eq!(res3, XPathValue::Boolean(true));
}

#[test]
fn test_xpath_modern_string_functions() {
    let doc = parse("<root/>").unwrap();
    let engine = XPathEngine::new(&doc);

    // ends-with
    assert_eq!(
        engine.evaluate("ends-with('document.xml', '.xml')", None).unwrap(),
        XPathValue::Boolean(true)
    );
    assert_eq!(
        engine.evaluate("ends-with('document.xml', '.json')", None).unwrap(),
        XPathValue::Boolean(false)
    );

    // lower-case & upper-case
    assert_eq!(
        engine.evaluate("lower-case('XML_LIB')", None).unwrap(),
        XPathValue::String("xml_lib".into())
    );
    assert_eq!(
        engine.evaluate("upper-case('rust')", None).unwrap(),
        XPathValue::String("RUST".into())
    );

    // replace
    assert_eq!(
        engine.evaluate("replace('banana', 'a', 'o')", None).unwrap(),
        XPathValue::String("bonono".into())
    );
}

#[test]
fn test_xpath_variable_bindings() {
    let xml = r#"
    <items>
        <item id="1" price="10"/>
        <item id="2" price="25"/>
        <item id="3" price="50"/>
    </items>
    "#;
    let doc = parse(xml).unwrap();
    let mut engine = XPathEngine::new(&doc);

    engine.set_variable("limit", XPathValue::Number(20.0));

    let nodes = engine.evaluate_nodes("//item[@price > $limit]", None).unwrap();
    assert_eq!(nodes.len(), 2);
    assert_eq!(doc.get_attribute(nodes[0], "id"), Some("2"));
    assert_eq!(doc.get_attribute(nodes[1], "id"), Some("3"));
}

#[test]
fn test_xpath_custom_function() {
    let doc = parse("<root/>").unwrap();
    let mut engine = XPathEngine::new(&doc);

    engine.register_function("square", |args| {
        if args.len() != 1 {
            return Err(xml_lib_rust::XmlError::XPathError("square() takes 1 argument".into()));
        }
        let n = match args[0] {
            XPathValue::Number(num) => num,
            _ => 0.0,
        };
        Ok(XPathValue::Number(n * n))
    });

    let res = engine.evaluate("square(5)", None).unwrap();
    assert_eq!(res, XPathValue::Number(25.0));
}
