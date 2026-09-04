use xml_lib_rust::{Attribute, EntityMapper};

#[test]
fn test_attribute_creation_and_search() {
    let attr1 = Attribute::new("test1", "value1");
    let attr2 = Attribute::new("test2", "value2");
    let attrs = vec![attr1.clone(), attr2.clone()];

    assert_eq!(&*attr1.name, "test1");
    assert_eq!(&*attr1.value, "value1");

    assert!(attrs.iter().any(|a| &*a.name == "test1"));
    assert!(attrs.iter().any(|a| &*a.name == "test2"));
    assert!(!attrs.iter().any(|a| &*a.name == "test3"));
}

#[test]
fn test_entity_mapper_default_entities() {
    let mapper = EntityMapper::default();

    assert_eq!(mapper.expand("&amp;").unwrap(), "&");
    assert_eq!(mapper.expand("&lt;").unwrap(), "<");
    assert_eq!(mapper.expand("&gt;").unwrap(), ">");
    assert_eq!(mapper.expand("&quot;").unwrap(), "\"");
    assert_eq!(mapper.expand("&apos;").unwrap(), "'");
}

#[test]
fn test_entity_mapper_custom_registrations() {
    let mut mapper = EntityMapper::default();
    mapper.register("author", "Jane Austen");
    mapper.register("book", "Pride and Prejudice");

    assert_eq!(mapper.expand("&author;").unwrap(), "Jane Austen");
    assert_eq!(mapper.expand("Book: &book; by &author;").unwrap(), "Book: Pride and Prejudice by Jane Austen");
}

#[test]
fn test_entity_mapper_recursive_limit() {
    let mut mapper = EntityMapper::default();
    mapper.register("a", "&b;");
    mapper.register("b", "&a;");

    let res = mapper.expand("&a;");
    assert!(res.is_err(), "Recursive entity loop should trigger recursion error");
}

#[test]
fn test_entity_mapper_numeric_references() {
    let mapper = EntityMapper::default();

    // Decimal
    assert_eq!(mapper.expand("&#65;&#66;&#67;").unwrap(), "ABC");
    // Hexadecimal
    assert_eq!(mapper.expand("&#x41;&#x42;&#x43;").unwrap(), "ABC");
    // Unicode symbol Euro
    assert_eq!(mapper.expand("&#x20AC;").unwrap(), "€");
}
