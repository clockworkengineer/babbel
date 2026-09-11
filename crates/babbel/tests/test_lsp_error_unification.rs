//! Integration tests for Phase 3: Liskov Substitution Principle (LSP)
//! Invariant hardening & universal error unification across format crates.

use babbel::{
    bencode::BencodeError,
    json::{JsonError, ParseError},
    toml::TomlError,
    xml::XmlError,
    yaml::{ErrorKind as YamlErrorKind, Node as YamlNode, YamlError},
    BabbelError,
};

#[test]
fn test_universal_error_substitution_to_babbel_error() {
    // 1. JSON error -> BabbelError
    let json_err: JsonError = JsonError::syntax("Unexpected token", Some(1), Some(5));
    let babbel_from_json: BabbelError = json_err.clone().into();
    assert_eq!(babbel_from_json.format, Some("json"));
    assert!(babbel_from_json.to_string().contains("Unexpected token"));

    // Verify JsonError alias matches ParseError
    let parse_err: ParseError = json_err;
    let babbel_from_parse: BabbelError = BabbelError::from(parse_err);
    assert_eq!(babbel_from_parse.format, Some("json"));

    // 2. YAML error -> BabbelError
    let yaml_err = YamlError::new(YamlErrorKind::SyntaxError, "unexpected yaml anchor");
    let babbel_from_yaml: BabbelError = yaml_err.into();
    assert_eq!(babbel_from_yaml.format, Some("yaml"));
    assert!(babbel_from_yaml.to_string().contains("unexpected yaml anchor"));

    // 3. XML error -> BabbelError
    let xml_err = XmlError::SyntaxError {
        message: "unclosed tag <item>".into(),
        line: 1,
        col: 10,
    };
    let babbel_from_xml: BabbelError = xml_err.into();
    assert_eq!(babbel_from_xml.format, Some("xml"));
    assert!(babbel_from_xml.to_string().contains("unclosed tag <item>"));

    // 4. TOML error -> BabbelError
    let toml_err = TomlError::syntax("invalid key-value pair", 2, 10, 20);
    let babbel_from_toml: BabbelError = toml_err.into();
    assert_eq!(babbel_from_toml.format, Some("toml"));
    assert!(babbel_from_toml.to_string().contains("invalid key-value pair"));

    // 5. Bencode error -> BabbelError
    let bencode_err = BencodeError::EmptyInput;
    let babbel_from_bencode: BabbelError = bencode_err.into();
    assert_eq!(babbel_from_bencode.format, Some("bencode"));
}

#[test]
fn test_all_format_errors_implement_std_error() {
    fn assert_std_error<E: std::error::Error + core::fmt::Display + 'static>(_err: &E) {}

    let json_err = JsonError::syntax("test", Some(1), Some(1));
    let yaml_err = YamlError::new(YamlErrorKind::SyntaxError, "test");
    let xml_err = XmlError::SyntaxError {
        message: "test".into(),
        line: 1,
        col: 1,
    };
    let toml_err = TomlError::syntax("test", 1, 1, 1);
    let bencode_err = BencodeError::InvalidInteger;
    let babbel_err = BabbelError::custom("test");

    assert_std_error(&json_err);
    assert_std_error(&yaml_err);
    assert_std_error(&xml_err);
    assert_std_error(&toml_err);
    assert_std_error(&bencode_err);
    assert_std_error(&babbel_err);
}

#[test]
fn test_lsp_safe_total_node_access() {
    // Array access
    let mut arr = YamlNode::Array(vec![YamlNode::from("a"), YamlNode::from("b")]);
    assert_eq!(arr.get(0), Some(&YamlNode::from("a")));
    assert_eq!(arr.get_index(1), Some(&YamlNode::from("b")));
    assert_eq!(arr.get(5), None);
    assert_eq!(arr.get_index(5), None);
    assert_eq!(arr.get("out-of-domain-key"), None);

    if let Some(item) = arr.get_mut(0) {
        *item = YamlNode::from("replaced");
    }
    assert_eq!(arr.get(0), Some(&YamlNode::from("replaced")));

    if let Some(item) = arr.get_index_mut(1) {
        *item = YamlNode::from("replaced2");
    }
    assert_eq!(arr.get_index(1), Some(&YamlNode::from("replaced2")));

    // Mapping access
    let mut map = YamlNode::Mapping(vec![
        (YamlNode::from("host"), YamlNode::from("localhost")),
        (YamlNode::from("port"), YamlNode::from(8080)),
    ]);
    assert_eq!(map.get("host"), Some(&YamlNode::from("localhost")));
    assert_eq!(map.get_key("port"), Some(&YamlNode::from(8080)));
    assert_eq!(map.get("missing"), None);
    assert_eq!(map.get(0), None);
    assert_eq!(map.get_index(0), None);

    if let Some(val) = map.get_mut("host") {
        *val = YamlNode::from("127.0.0.1");
    }
    assert_eq!(map.get("host"), Some(&YamlNode::from("127.0.0.1")));

    if let Some(val) = map.get_key_mut("port") {
        *val = YamlNode::from(9090);
    }
    assert_eq!(map.get("port"), Some(&YamlNode::from(9090)));

    // Scalar non-panicking access
    let scalar = YamlNode::from(12345);
    assert_eq!(scalar.get(0), None);
    assert_eq!(scalar.get("key"), None);
    assert_eq!(scalar.get_index(0), None);
}
