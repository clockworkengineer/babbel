//! # Babbel TOML
//!
//! Fast, modular, pure-Rust TOML v1.0.0 parser, serializer, streaming pull parser, and DOM.
//!
//! Complies with the TOML v1.0.0 specification and integrates with `babbel_core` and the Babbel ecosystem.

extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

pub mod error;
pub mod nodes;
pub mod parser;
pub mod stringify;

// Re-exports
pub use error::TomlError;
pub use nodes::{DatetimeKind, Node, TomlDatetime};
pub use parser::{Lexer, Parser, TomlPullEvent, TomlPullParser};
pub use stringify::{emit_pretty_to, emit_to, PrettyOptions};

use babbel_core::io::destinations::BufferDestination;
use babbel_core::io::traits::{ICharStream, IDestination};

/// Parses a TOML formatted UTF-8 string into a `Node::Table`.
pub fn from_str(input: &str) -> Result<Node, TomlError> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

/// Parses a TOML formatted byte slice into a `Node::Table`.
pub fn from_slice(bytes: &[u8]) -> Result<Node, TomlError> {
    let s = core::str::from_utf8(bytes).map_err(|_| {
        TomlError::custom("Input is not valid UTF-8")
    })?;
    from_str(s)
}

/// Parses a character stream into a `Node::Table`.
pub fn from_stream<S: ICharStream>(stream: &mut S) -> Result<Node, TomlError> {
    let mut s = String::new();
    while stream.more() {
        if let Some(ch) = stream.current() {
            s.push(ch);
        }
        stream.next();
    }
    from_str(&s)
}

/// Serializes a `Node` structure to a TOML string.
pub fn to_string(node: &Node) -> Result<String, TomlError> {
    let mut dest = BufferDestination::new();
    emit_to(node, &mut dest)?;
    Ok(dest.to_string())
}

/// Serializes a `Node` structure to a pretty-printed TOML string.
pub fn to_string_pretty(node: &Node) -> Result<String, TomlError> {
    let mut dest = BufferDestination::new();
    emit_pretty_to(node, &mut dest, &PrettyOptions::default())?;
    Ok(dest.to_string())
}

/// Serializes a `Node` structure into a destination buffer or stream.
pub fn to_writer(node: &Node, dest: &mut dyn IDestination) -> Result<(), TomlError> {
    emit_to(node, dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse_and_emit() {
        let input = r#"
# Server configuration
title = "TOML Example"

[owner]
name = "Tom Preston-Werner"
dob = 1979-05-27T07:32:00-08:00

[database]
enabled = true
ports = [ 8000, 8001, 8002 ]
temp_targets = { cpu = 79.5, case = 72.0 }
"#;
        let node = from_str(input).expect("Failed to parse TOML");
        assert_eq!(node.get("title").and_then(|n| n.as_str()), Some("TOML Example"));

        let owner = node.get("owner").expect("Owner table missing");
        assert_eq!(owner.get("name").and_then(|n| n.as_str()), Some("Tom Preston-Werner"));

        let db = node.get("database").expect("Database table missing");
        assert_eq!(db.get("enabled").and_then(|n| n.as_bool()), Some(true));

        let ports = db.get("ports").and_then(|n| n.as_array()).expect("Ports array missing");
        assert_eq!(ports.len(), 3);
        assert_eq!(ports[0].as_integer(), Some(8000));

        let out = to_string(&node).expect("Failed to emit TOML");
        assert!(out.contains("title = \"TOML Example\""));
        assert!(out.contains("[owner]"));
        assert!(out.contains("[database]"));
    }

    #[test]
    fn test_array_of_tables() {
        let input = r#"
[[products]]
name = "Hammer"
sku = 738594937

[[products]]
name = "Nail"
sku = 284758393
color = "gray"
"#;
        let node = from_str(input).expect("Failed to parse array of tables");
        let products = node.get("products").and_then(|n| n.as_array()).expect("Products array missing");
        assert_eq!(products.len(), 2);
        assert_eq!(products[0].get("name").and_then(|n| n.as_str()), Some("Hammer"));
        assert_eq!(products[1].get("name").and_then(|n| n.as_str()), Some("Nail"));
    }

    #[test]
    fn test_dotted_keys() {
        let input = r#"
fruit.apple.color = "red"
fruit.apple.taste.sweet = true
"#;
        let node = from_str(input).expect("Failed to parse dotted keys");
        let fruit = node.get("fruit").expect("fruit missing");
        let apple = fruit.get("apple").expect("apple missing");
        assert_eq!(apple.get("color").and_then(|n| n.as_str()), Some("red"));
        let taste = apple.get("taste").expect("taste missing");
        assert_eq!(taste.get("sweet").and_then(|n| n.as_bool()), Some(true));
    }
}
