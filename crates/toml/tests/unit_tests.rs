//! Comprehensive test suite for babbel_toml validating TOML v1.0.0 features.

use babbel_toml::{from_str, DatetimeKind, Node, TomlPullEvent, TomlPullParser};
use babbel_core::model::Value;

#[test]
fn test_string_escapes_and_multiline() {
    let input = r#"
basic = "Hello\tWorld\nNew\"Line\""
literal = 'C:\Users\Node\path'
multiline_basic = """
The quick brown \
fox jumps over \
the lazy dog."""
multiline_literal = '''
Roses are red
Violets are blue'''
"#;
    let node = from_str(input).expect("Failed to parse string types");
    assert_eq!(
        node.get("basic").and_then(|n| n.as_str()),
        Some("Hello\tWorld\nNew\"Line\"")
    );
    assert_eq!(
        node.get("literal").and_then(|n| n.as_str()),
        Some(r"C:\Users\Node\path")
    );
    assert_eq!(
        node.get("multiline_basic").and_then(|n| n.as_str()),
        Some("The quick brown fox jumps over the lazy dog.")
    );
    assert_eq!(
        node.get("multiline_literal").and_then(|n| n.as_str()),
        Some("Roses are red\nViolets are blue")
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn test_numbers_bases_and_special_floats() {
    let input = r#"
dec_int = 1_000_000
neg_int = -17
hex_int = 0xDEADBEEF
oct_int = 0o755
bin_int = 0b11010110

pi = 3.14159
exp = 5e+22
pos_inf = inf
neg_inf = -inf
not_a_num = nan
"#;
    let node = from_str(input).expect("Failed to parse numbers");
    assert_eq!(node.get("dec_int").and_then(|n| n.as_integer()), Some(1_000_000));
    assert_eq!(node.get("neg_int").and_then(|n| n.as_integer()), Some(-17));
    assert_eq!(node.get("hex_int").and_then(|n| n.as_integer()), Some(0xDEADBEEF));
    assert_eq!(node.get("oct_int").and_then(|n| n.as_integer()), Some(0o755));
    assert_eq!(node.get("bin_int").and_then(|n| n.as_integer()), Some(0b11010110));

    assert_eq!(node.get("pi").and_then(|n| n.as_float()), Some(3.14159));
    assert_eq!(node.get("exp").and_then(|n| n.as_float()), Some(5e22));
    assert_eq!(node.get("pos_inf").and_then(|n| n.as_float()), Some(f64::INFINITY));
    assert_eq!(node.get("neg_inf").and_then(|n| n.as_float()), Some(f64::NEG_INFINITY));
    assert!(node.get("not_a_num").and_then(|n| n.as_float()).unwrap().is_nan());
}

#[test]
fn test_rfc3339_datetimes() {
    let input = r#"
odt1 = 1979-05-27T07:32:00Z
odt2 = 1979-05-27T00:32:00-07:00
ldt = 1979-05-27T07:32:00
ld = 1979-05-27
lt = 07:32:00.999999
"#;
    let node = from_str(input).expect("Failed to parse datetimes");

    let odt1 = node.get("odt1").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(odt1.kind, DatetimeKind::OffsetDateTime);
    assert_eq!(odt1.as_str(), "1979-05-27T07:32:00Z");

    let odt2 = node.get("odt2").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(odt2.kind, DatetimeKind::OffsetDateTime);

    let ldt = node.get("ldt").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(ldt.kind, DatetimeKind::LocalDateTime);

    let ld = node.get("ld").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(ld.kind, DatetimeKind::LocalDate);
    assert_eq!(ld.as_str(), "1979-05-27");

    let lt = node.get("lt").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(lt.kind, DatetimeKind::LocalTime);
    assert_eq!(lt.as_str(), "07:32:00.999999");
}

#[test]
fn test_inline_tables_and_nested_tables() {
    let input = r#"
name = { first = "Tom", last = "Preston-Werner" }
point = { x = 1, y = 2 }

[server.alpha]
ip = "10.0.0.1"
dc = "eqdc10"

[server.beta]
ip = "10.0.0.2"
dc = "eqdc10"
"#;
    let node = from_str(input).expect("Failed to parse tables");

    let name = node.get("name").expect("name missing");
    assert_eq!(name.get("first").and_then(|n| n.as_str()), Some("Tom"));
    assert_eq!(name.get("last").and_then(|n| n.as_str()), Some("Preston-Werner"));

    let server = node.get("server").expect("server missing");
    let alpha = server.get("alpha").expect("alpha missing");
    assert_eq!(alpha.get("ip").and_then(|n| n.as_str()), Some("10.0.0.1"));

    let beta = server.get("beta").expect("beta missing");
    assert_eq!(beta.get("ip").and_then(|n| n.as_str()), Some("10.0.0.2"));
}

#[test]
fn test_core_value_conversions() {
    let input = r#"
[package]
name = "babbel"
version = "0.2.0"
edition = 2024
features = [ "json", "toml", "yaml" ]
"#;
    let node = from_str(input).expect("Failed to parse");

    // Convert to babbel_core::Value
    let value = Value::from(&node);
    assert!(matches!(value, Value::Object(_)));

    // Verify Value contents
    if let Value::Object(entries) = &value {
        assert_eq!(entries[0].0, "package");
        if let Value::Object(pkg_entries) = &entries[0].1 {
            assert!(pkg_entries.iter().any(|(k, v)| k == "name" && v.as_str() == Some("babbel")));
        }
    }

    // Convert back to Node
    let roundtrip_node = Node::from(&value);
    let pkg = roundtrip_node.get("package").unwrap();
    assert_eq!(pkg.get("name").and_then(|n| n.as_str()), Some("babbel"));
}

#[test]
fn test_streaming_pull_parser() {
    let input = r#"
title = "Pull Test"
count = 42
[details]
active = true
"#;
    let mut pull_parser = TomlPullParser::new(input);
    let mut events = Vec::new();

    while let Ok(Some(ev)) = pull_parser.next_event() {
        events.push(ev);
    }

    assert!(events.contains(&TomlPullEvent::StartDocument));
    assert!(events.contains(&TomlPullEvent::Key("title".to_string())));
    assert!(events.contains(&TomlPullEvent::ValueString("Pull Test".to_string())));
    assert!(events.contains(&TomlPullEvent::Key("count".to_string())));
    assert!(events.contains(&TomlPullEvent::ValueInteger(42)));
    assert!(events.contains(&TomlPullEvent::EndDocument));
}

#[test]
fn test_toml_1_1_escapes() {
    let input = r#"
esc_char = "\e[31mRed\e[0m"
hex_a = "\x61"
hex_null = "\x00"
hex_accent = "\xE9"
"#;
    let node = from_str(input).expect("Failed to parse TOML 1.1 escapes");
    assert_eq!(
        node.get("esc_char").and_then(|n| n.as_str()),
        Some("\x1B[31mRed\x1B[0m")
    );
    assert_eq!(
        node.get("hex_a").and_then(|n| n.as_str()),
        Some("a")
    );
    assert_eq!(
        node.get("hex_null").and_then(|n| n.as_str()),
        Some("\0")
    );
    assert_eq!(
        node.get("hex_accent").and_then(|n| n.as_str()),
        Some("é")
    );
}

#[test]
fn test_toml_1_1_inline_tables() {
    let input = r#"
tbl = {
    key = "a string",
    # comment inside inline table
    moar-tbl = {
        key = 1,
    },
}
empty_trailing = { }
"#;
    let node = from_str(input).expect("Failed to parse multiline inline tables with trailing commas");
    let tbl = node.get("tbl").expect("tbl missing");
    assert_eq!(tbl.get("key").and_then(|n| n.as_str()), Some("a string"));
    let moar = tbl.get("moar-tbl").expect("moar-tbl missing");
    assert_eq!(moar.get("key").and_then(|n| n.as_integer()), Some(1));
    assert!(node.get("empty_trailing").is_some());
}

#[test]
fn test_toml_1_1_optional_seconds_datetime() {
    let input = r#"
odt1 = 1979-05-27 07:32Z
odt2 = 1979-05-27 07:32-07:00
ldt1 = 1979-05-27T07:32
ldt2 = 1979-05-27 07:32
lt1 = 07:32
lt2 = 14:15
"#;
    let node = from_str(input).expect("Failed to parse datetimes with optional seconds");

    let odt1 = node.get("odt1").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(odt1.kind, DatetimeKind::OffsetDateTime);
    assert_eq!(odt1.as_str(), "1979-05-27 07:32Z");

    let odt2 = node.get("odt2").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(odt2.kind, DatetimeKind::OffsetDateTime);
    assert_eq!(odt2.as_str(), "1979-05-27 07:32-07:00");

    let ldt1 = node.get("ldt1").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(ldt1.kind, DatetimeKind::LocalDateTime);
    assert_eq!(ldt1.as_str(), "1979-05-27T07:32");

    let ldt2 = node.get("ldt2").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(ldt2.kind, DatetimeKind::LocalDateTime);
    assert_eq!(ldt2.as_str(), "1979-05-27 07:32");

    let lt1 = node.get("lt1").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(lt1.kind, DatetimeKind::LocalTime);
    assert_eq!(lt1.as_str(), "07:32");

    let lt2 = node.get("lt2").and_then(|n| n.as_datetime()).unwrap();
    assert_eq!(lt2.kind, DatetimeKind::LocalTime);
    assert_eq!(lt2.as_str(), "14:15");
}

#[test]
fn test_toml_1_1_crlf_normalization() {
    let input = "lit = '''\r\nline1\r\nline2'''\r\nbasic = \"\"\"\r\nline1\r\nline2\"\"\"\r\n";
    let node = from_str(input).expect("Failed to parse CRLF multiline strings");
    assert_eq!(
        node.get("lit").and_then(|n| n.as_str()),
        Some("line1\nline2")
    );
    assert_eq!(
        node.get("basic").and_then(|n| n.as_str()),
        Some("line1\nline2")
    );
}
