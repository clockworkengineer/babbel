use xml_lib_rust::{parse, NamespaceScope, QName};

#[test]
fn test_qname_parsing() {
    let (prefix, local) = QName::split_prefix("xs:element");
    assert_eq!(prefix, Some("xs"));
    assert_eq!(local, "element");

    let (prefix, local) = QName::split_prefix("book");
    assert_eq!(prefix, None);
    assert_eq!(local, "book");

    let qn = QName::new(Some("env"), "Body", Some("http://schemas.xmlsoap.org/soap/envelope/"));
    assert_eq!(qn.prefix.as_deref(), Some("env"));
    assert_eq!(&*qn.local_name, "Body");
    assert_eq!(qn.namespace_uri.as_deref(), Some("http://schemas.xmlsoap.org/soap/envelope/"));
}

#[test]
fn test_namespace_scope_stack() {
    let mut scope = NamespaceScope::new();
    // Default xml prefix
    assert_eq!(
        scope.resolve_prefix(Some("xml")),
        Some("http://www.w3.org/XML/1998/namespace")
    );

    // Root scope
    scope.declare(None, "http://example.com/default");
    scope.declare(Some("p"), "http://example.com/p1");
    assert_eq!(scope.resolve_prefix(None), Some("http://example.com/default"));
    assert_eq!(scope.resolve_prefix(Some("p")), Some("http://example.com/p1"));

    // Nested scope with shadowing
    scope.push_scope();
    scope.declare(Some("p"), "http://example.com/p2");
    assert_eq!(scope.resolve_prefix(Some("p")), Some("http://example.com/p2"));

    // Pop scope reverts to outer
    scope.pop_scope();
    assert_eq!(scope.resolve_prefix(Some("p")), Some("http://example.com/p1"));
}

#[test]
fn test_document_namespace_inspection() {
    let xml = r#"
    <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                   xmlns:m="http://example.org/math">
        <soap:Body>
            <m:Add>
                <m:x>5</m:x>
            </m:Add>
        </soap:Body>
    </soap:Envelope>
    "#;

    let doc = parse(xml).unwrap();
    let root = doc.root_element_id().unwrap();

    assert_eq!(doc.get_prefix(root), Some("soap"));
    assert_eq!(doc.get_local_name(root), "Envelope");
    assert_eq!(
        doc.get_namespace_uri(root).as_deref(),
        Some("http://schemas.xmlsoap.org/soap/envelope/")
    );

    assert_eq!(
        doc.lookup_prefix(root, "http://example.org/math").as_deref(),
        Some("m")
    );
    assert_eq!(
        doc.lookup_namespace_uri(root, "soap").as_deref(),
        Some("http://schemas.xmlsoap.org/soap/envelope/")
    );

    // Find children in namespace
    let add_nodes = doc.get_elements_by_tag_name_ns("http://example.org/math", "Add");
    assert_eq!(add_nodes.len(), 1);

    // Wildcard namespace match
    let math_all = doc.get_elements_by_tag_name_ns("http://example.org/math", "*");
    assert_eq!(math_all.len(), 2); // Add and x

    // Wildcard local name match
    let soap_all = doc.get_elements_by_tag_name_ns("http://schemas.xmlsoap.org/soap/envelope/", "*");
    assert_eq!(soap_all.len(), 2); // Envelope and Body
}

#[test]
fn test_default_namespace_inheritance() {
    let xml = r#"
    <feed xmlns="http://www.w3.org/2005/Atom">
        <title>Sample Feed</title>
        <entry>
            <title>First Entry</title>
        </entry>
    </feed>
    "#;

    let doc = parse(xml).unwrap();
    let feed = doc.root_element_id().unwrap();

    assert_eq!(doc.get_prefix(feed), None);
    assert_eq!(doc.get_local_name(feed), "feed");
    assert_eq!(
        doc.get_namespace_uri(feed).as_deref(),
        Some("http://www.w3.org/2005/Atom")
    );

    let entries = doc.get_elements_by_tag_name_ns("http://www.w3.org/2005/Atom", "entry");
    assert_eq!(entries.len(), 1);

    let titles = doc.get_elements_by_tag_name_ns("http://www.w3.org/2005/Atom", "title");
    assert_eq!(titles.len(), 2);
}
