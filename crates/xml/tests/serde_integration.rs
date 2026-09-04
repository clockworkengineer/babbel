#![cfg(feature = "serde")]

use serde::{Deserialize, Serialize};
use xml_lib_rust::serde_impl::{from_str, to_string, to_string_with_root};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Department {
    name: String,
    manager: User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Project {
    title: String,
    tags: Vec<String>,
}

#[test]
fn test_serde_deserialize_simple_struct() {
    let xml = r#"
        <user id="42">
            <name>Alice Smith</name>
            <active>true</active>
        </user>
    "#;

    let user: User = from_str(xml).expect("Deserialize User");
    assert_eq!(user.id, 42);
    assert_eq!(user.name, "Alice Smith");
    assert!(user.active);
}

#[test]
fn test_serde_deserialize_nested_struct() {
    let xml = r#"
        <department>
            <name>Engineering</name>
            <manager id="1">
                <name>Bob Jones</name>
                <active>false</active>
            </manager>
        </department>
    "#;

    let dept: Department = from_str(xml).expect("Deserialize Department");
    assert_eq!(dept.name, "Engineering");
    assert_eq!(dept.manager.id, 1);
    assert_eq!(dept.manager.name, "Bob Jones");
    assert!(!dept.manager.active);
}

#[test]
fn test_serde_serialize_struct() {
    let user = User {
        id: 99,
        name: "Charlie Brown".to_string(),
        active: true,
    };

    let xml_root = to_string(&user).expect("Serialize User with default root");
    assert!(xml_root.starts_with("<root>"));
    assert!(xml_root.ends_with("</root>"));

    let xml = to_string_with_root("user", &user).expect("Serialize User");
    assert!(xml.starts_with("<user>"));
    assert!(xml.contains("<id>99</id>"));
    assert!(xml.contains("<name>Charlie Brown</name>"));
    assert!(xml.contains("<active>true</active>"));
    assert!(xml.ends_with("</user>"));
}

#[test]
fn test_serde_roundtrip() {
    let user = User {
        id: 777,
        name: "Roundtrip Test".to_string(),
        active: true,
    };

    let xml = to_string_with_root("user", &user).expect("Serialize");
    let deserialized: User = from_str(&xml).expect("Deserialize");
    assert_eq!(user, deserialized);
}

#[test]
fn test_serde_sequence_and_collections() {
    let proj = Project {
        title: "Rust XML Lib".to_string(),
        tags: vec!["rust".to_string(), "xml".to_string(), "serde".to_string()],
    };

    let xml = to_string_with_root("project", &proj).expect("Serialize");
    assert!(xml.contains("<title>Rust XML Lib</title>"));
    assert!(xml.contains("<tags>"));
    assert!(xml.contains("<item>rust</item>"));
    assert!(xml.contains("<item>xml</item>"));
    assert!(xml.contains("<item>serde</item>"));
}
