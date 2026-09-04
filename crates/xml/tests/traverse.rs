use xml_lib_rust::{parse, NodeKind};

#[derive(Default, Debug, PartialEq, Eq)]
struct NodeStats {
    total_nodes: usize,
    elements: usize,
    texts: usize,
    cdatas: usize,
    comments: usize,
    pis: usize,
    declarations: usize,
    entity_refs: usize,
}

#[test]
fn test_document_node_traversal_and_counting() {
    let xml = r#"<?xml version="1.0"?>
<!-- comment 1 -->
<root>
  <child>test content</child>
  <![CDATA[raw cdata]]>
  <!-- comment 2 -->
  <?pi action="run"?>
</root>"#;

    let doc = parse(xml).expect("Parse clean");
    let mut stats = NodeStats::default();

    for node in doc.nodes() {
        stats.total_nodes += 1;
        match &node.kind {
            NodeKind::Element { .. } => stats.elements += 1,
            NodeKind::Text(_) => stats.texts += 1,
            NodeKind::CData(_) => stats.cdatas += 1,
            NodeKind::Comment(_) => stats.comments += 1,
            NodeKind::ProcessingInstruction { .. } => stats.pis += 1,
            NodeKind::Declaration { .. } => stats.declarations += 1,
            NodeKind::EntityReference(_) => stats.entity_refs += 1,
            _ => {}
        }
    }

    assert!(stats.total_nodes >= 6);
    assert_eq!(stats.elements, 2); // root + child
    assert_eq!(stats.cdatas, 1);
    assert_eq!(stats.comments, 2);
    assert_eq!(stats.pis, 1);
    assert_eq!(stats.declarations, 1);
}
