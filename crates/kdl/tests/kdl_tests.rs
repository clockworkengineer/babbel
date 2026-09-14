use babbel_core::Value;
use babbel_kdl::{from_str, to_string_pretty};

#[test]
fn test_basic_nodes_and_arguments() {
    let kdl = r#"
    title "Babbel KDL"
    author "Alice"
    stars 100
    active true
    optional null
    "#;

    let val = from_str(kdl).expect("failed to parse basic kdl");
    assert_eq!(val.get("title").and_then(|v| v.as_str()), Some("Babbel KDL"));
    assert_eq!(val.get("author").and_then(|v| v.as_str()), Some("Alice"));
    assert_eq!(val.get("stars").and_then(|v| v.as_i64()), Some(100));
    assert_eq!(val.get("active").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(val.get("optional"), Some(&Value::Null));
}

#[test]
fn test_properties() {
    let kdl = r#"
    server host="127.0.0.1" port=8080 timeout=30.5
    "#;

    let val = from_str(kdl).expect("failed to parse properties");
    let server = val.get("server").expect("server node missing");
    assert_eq!(server.get("host").and_then(|v| v.as_str()), Some("127.0.0.1"));
    assert_eq!(server.get("port").and_then(|v| v.as_i64()), Some(8080));
    assert_eq!(server.get("timeout").and_then(|v| v.as_f64()), Some(30.5));
}

#[test]
fn test_nested_children() {
    let kdl = r#"
    package {
        name "babbel"
        version "0.2.1"
        dependencies {
            serde version="1.0"
        }
    }
    "#;

    let val = from_str(kdl).expect("failed to parse children");
    let pkg = val.get("package").expect("package missing");
    assert_eq!(pkg.get("name").and_then(|v| v.as_str()), Some("babbel"));
    assert_eq!(pkg.get("version").and_then(|v| v.as_str()), Some("0.2.1"));

    let deps = pkg.get("dependencies").expect("dependencies missing");
    let serde = deps.get("serde").expect("serde missing");
    assert_eq!(serde.get("version").and_then(|v| v.as_str()), Some("1.0"));
}

#[test]
fn test_comments_and_slashdash() {
    let kdl = r#"
    // Single line comment
    /* Multi
       line
       /* Nested block comment */
    */
    /- skipped_node "arg" {
        inner "ignored"
    }

    node /- "skipped_arg" "kept_arg" /- prop="skipped" active=true
    "#;

    let val = from_str(kdl).expect("failed to parse comments");
    assert!(val.get("skipped_node").is_none());

    let node = val.get("node").expect("node missing");
    assert_eq!(node.get("active").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(node.get("@arg").and_then(|v| v.as_str()), Some("kept_arg"));
}

#[test]
fn test_numbers_and_raw_strings() {
    let kdl = r##"
    hex_num 0x1A2F
    oct_num 0o755
    bin_num 0b1010
    big_num 1_000_000
    raw_str #"C:\Program Files\Rust"#
    "##;

    let val = from_str(kdl).expect("failed to parse numbers and raw strings");
    assert_eq!(val.get("hex_num").and_then(|v| v.as_i64()), Some(0x1A2F));
    assert_eq!(val.get("oct_num").and_then(|v| v.as_i64()), Some(0o755));
    assert_eq!(val.get("bin_num").and_then(|v| v.as_i64()), Some(0b1010));
    assert_eq!(val.get("big_num").and_then(|v| v.as_i64()), Some(1000000));
    assert_eq!(
        val.get("raw_str").and_then(|v| v.as_str()),
        Some(r#"C:\Program Files\Rust"#)
    );
}

#[test]
fn test_pretty_serialize_roundtrip() {
    let kdl = r#"
    app {
        name "editor"
        window width=1920 height=1080 fullscreen=true
    }
    "#;

    let val = from_str(kdl).expect("failed to parse initial kdl");
    let serialized = to_string_pretty(&val, 2).expect("failed to serialize");
    assert!(serialized.contains("name \"editor\"") || serialized.contains("name"));

    let roundtrip = from_str(&serialized).expect("failed to parse serialized");
    let app = roundtrip.get("app").expect("app missing");
    assert_eq!(app.get("name").and_then(|v| v.as_str()), Some("editor"));
    let window = app.get("window").expect("window missing");
    assert_eq!(window.get("width").and_then(|v| v.as_i64()), Some(1920));
    assert_eq!(window.get("height").and_then(|v| v.as_i64()), Some(1080));
    assert_eq!(window.get("fullscreen").and_then(|v| v.as_bool()), Some(true));
}
