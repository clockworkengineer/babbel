use xml_lib_rust::{parse, stringify, NodeKind};

#[test]
fn test_remove_child() {
    let mut doc = parse("<root><a/><b/><c/></root>").unwrap();
    let root = doc.root_element_id().unwrap();
    let children = doc.get_children(root);
    assert_eq!(children.len(), 3);

    let removed = doc.remove_child(root, children[1]).unwrap();
    assert_eq!(removed, children[1]);
    assert_eq!(doc.parent_id(removed), None);

    let new_children = doc.get_children(root);
    assert_eq!(new_children.len(), 2);
    assert_eq!(new_children[0], children[0]);
    assert_eq!(new_children[1], children[2]);
}

#[test]
fn test_insert_before() {
    let mut doc = parse("<root><a/><c/></root>").unwrap();
    let root = doc.root_element_id().unwrap();
    let children = doc.get_children(root);
    assert_eq!(children.len(), 2);

    let b_id = doc.add_node(NodeKind::Element {
        name: "b".into(),
        attributes: vec![],
    });

    doc.insert_before(root, b_id, children[1]).unwrap();
    let updated = doc.get_children(root);
    assert_eq!(updated.len(), 3);
    assert_eq!(updated[0], children[0]);
    assert_eq!(updated[1], b_id);
    assert_eq!(updated[2], children[1]);
    assert_eq!(doc.parent_id(b_id), Some(root));
}

#[test]
fn test_replace_child() {
    let mut doc = parse("<root><a/><old/><c/></root>").unwrap();
    let root = doc.root_element_id().unwrap();
    let children = doc.get_children(root);

    let new_node = doc.add_node(NodeKind::Element {
        name: "new".into(),
        attributes: vec![],
    });

    let replaced = doc.replace_child(root, new_node, children[1]).unwrap();
    assert_eq!(replaced, children[1]);
    assert_eq!(doc.parent_id(replaced), None);

    let updated = doc.get_children(root);
    assert_eq!(updated[1], new_node);
    assert_eq!(doc.parent_id(new_node), Some(root));
}

#[test]
fn test_detach() {
    let mut doc = parse("<root><child>data</child></root>").unwrap();
    let root = doc.root_element_id().unwrap();
    let child = doc.get_children(root)[0];

    assert_eq!(doc.parent_id(child), Some(root));
    doc.detach(child).unwrap();
    assert_eq!(doc.parent_id(child), None);
    assert_eq!(doc.get_children(root).len(), 0);
}

#[test]
fn test_attribute_mutations() {
    let mut doc = parse("<item id='1' category='books'/>").unwrap();
    let item = doc.root_element_id().unwrap();

    assert!(doc.has_attribute(item, "id"));
    assert!(doc.has_attribute(item, "category"));
    assert!(!doc.has_attribute(item, "price"));

    // Update existing
    doc.set_attribute(item, "id", "42").unwrap();
    assert_eq!(doc.get_attribute(item, "id"), Some("42"));

    // Add new
    doc.set_attribute(item, "price", "19.99").unwrap();
    assert_eq!(doc.get_attribute(item, "price"), Some("19.99"));

    // Remove
    assert!(doc.remove_attribute(item, "category"));
    assert!(!doc.has_attribute(item, "category"));
    assert!(!doc.remove_attribute(item, "category"));
}

#[test]
fn test_set_text_content() {
    let mut doc = parse("<p>Hello <b>bold</b> world!</p>").unwrap();
    let p = doc.root_element_id().unwrap();
    assert_eq!(doc.get_text_content(p), "Hello bold world!");

    doc.set_text_content(p, "Replaced simple text").unwrap();
    assert_eq!(doc.get_text_content(p), "Replaced simple text");
    let children = doc.get_children(p);
    assert_eq!(children.len(), 1);
    assert!(matches!(doc.get_node(children[0]).unwrap().kind, NodeKind::Text(_)));
}

#[test]
fn test_traversal_navigation() {
    let doc = parse("<list><item id='1'/><item id='2'/><item id='3'/></list>").unwrap();
    let list = doc.root_element_id().unwrap();

    let first = doc.first_child(list).unwrap();
    let last = doc.last_child(list).unwrap();
    assert_eq!(doc.get_attribute(first, "id"), Some("1"));
    assert_eq!(doc.get_attribute(last, "id"), Some("3"));

    let second = doc.next_sibling(first).unwrap();
    assert_eq!(doc.get_attribute(second, "id"), Some("2"));
    assert_eq!(doc.next_sibling(second), Some(last));
    assert_eq!(doc.next_sibling(last), None);

    assert_eq!(doc.previous_sibling(last), Some(second));
    assert_eq!(doc.previous_sibling(second), Some(first));
    assert_eq!(doc.previous_sibling(first), None);
}

#[test]
fn test_clone_node_shallow_and_deep() {
    let mut doc = parse("<catalog><book id='1'><title>Rust</title></book></catalog>").unwrap();
    let catalog = doc.root_element_id().unwrap();
    let book = doc.first_child(catalog).unwrap();

    // Shallow clone
    let shallow = doc.clone_node(book, false).unwrap();
    assert_eq!(doc.get_node(shallow).unwrap().kind.name(), "book");
    assert_eq!(doc.get_attribute(shallow, "id"), Some("1"));
    assert_eq!(doc.get_children(shallow).len(), 0);

    // Deep clone
    let deep = doc.clone_node(book, true).unwrap();
    assert_eq!(doc.get_children(deep).len(), 1);
    assert_eq!(doc.get_text_content(deep), "Rust");
    doc.append_child(catalog, deep).unwrap();

    assert_eq!(doc.get_children(catalog).len(), 2);
}

#[test]
fn test_query_selectors() {
    let doc = parse("<catalog><book id='b1'><chapter id='c1'/><chapter id='c2'/></book><book id='b2'/></catalog>").unwrap();

    let all_books = doc.get_elements_by_tag_name("book");
    assert_eq!(all_books.len(), 2);

    let all_elements = doc.get_elements_by_tag_name("*");
    assert_eq!(all_elements.len(), 5); // catalog, book, chapter, chapter, book

    let c2 = doc.get_element_by_id("c2");
    assert!(c2.is_some());
    assert_eq!(doc.get_node(c2.unwrap()).unwrap().kind.name(), "chapter");

    assert!(doc.get_element_by_id("nonexistent").is_none());
}

#[test]
fn test_arena_compact() {
    let mut doc = parse("<root><a/><b/><c/></root>").unwrap();
    let root = doc.root_element_id().unwrap();
    let b = doc.get_children(root)[1];

    let initial_len = doc.len();
    doc.remove_child(root, b).unwrap();

    // Compact arena
    doc.compact().unwrap();
    assert_eq!(doc.len(), initial_len - 1);

    let xml = stringify(&doc);
    assert!(xml.contains("<a/>") || xml.contains("<a>"));
    assert!(!xml.contains("<b"));
    assert!(xml.contains("<c/>") || xml.contains("<c>"));
}
