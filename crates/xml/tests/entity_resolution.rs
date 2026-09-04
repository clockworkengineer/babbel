use xml_lib_rust::{parse, EntityMapper};

#[test]
fn test_predefined_entities() {
    let xml = r#"<note message="&quot;Hello &amp; World&quot;">5 &gt; 3 &amp;&amp; 2 &lt; 4</note>"#;
    let doc = parse(xml).expect("Parsing predefined entities should succeed");

    let root_id = doc.root_element_id().expect("Root element exists");
    let children = doc.get_children(root_id);
    assert!(!children.is_empty());

    let text_content = doc.get_text_content(root_id);
    assert_eq!(text_content, "5 > 3 && 2 < 4");
}

#[test]
fn test_numeric_character_references() {
    let mapper = EntityMapper::default();
    
    // Hexadecimal references
    let hex_res = mapper.expand("&#x41;&#x42;&#x43;").expect("Hex entity expansion should work");
    assert_eq!(hex_res, "ABC");

    // Decimal references
    let dec_res = mapper.expand("&#65;&#66;&#67;").expect("Decimal entity expansion should work");
    assert_eq!(dec_res, "ABC");

    // Unicode character euro sign
    let euro_res = mapper.expand("&#x20AC;").expect("Euro symbol expansion should work");
    assert_eq!(euro_res, "€");
}

#[test]
fn test_custom_entity_mapper_registration() {
    let mut mapper = EntityMapper::default();
    mapper.register("company", "Acme Corp");
    mapper.register("trade", "&company; Trading");

    let expanded = mapper.expand("Welcome to &trade;!").expect("Recursive entity expansion should work");
    assert_eq!(expanded, "Welcome to Acme Corp Trading!");
}

#[test]
fn test_undeclared_entity_error() {
    let mapper = EntityMapper::default();
    let res = mapper.expand("Hello &unknown_entity;");
    assert!(res.is_err(), "Undeclared entity reference should fail");
}
