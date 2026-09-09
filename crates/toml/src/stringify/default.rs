//! Canonical TOML emission into arbitrary byte destinations.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

use babbel_core::escape::{is_valid_toml_bare_key, write_toml_escaped_string};
use babbel_core::io::traits::IDestination;

use crate::error::TomlError;
use crate::nodes::Node;

/// Serializes a `Node` structure to a TOML formatted destination.
pub fn emit_to(node: &Node, dest: &mut dyn IDestination) -> Result<(), TomlError> {
    match node {
        Node::Table(entries) => {
            serialize_table(entries, "", dest);
            Ok(())
        }
        Node::Array(items) if !items.is_empty() && items.iter().all(Node::is_table) => {
            for item in items {
                dest.add_bytes("[[item]]\n");
                if let Node::Table(entries) = item {
                    serialize_table(entries, "item", dest);
                }
            }
            Ok(())
        }
        _ => Err(TomlError::custom(
            "TOML specification requires a Table at the root level",
        )),
    }
}

fn format_key(key: &str, dest: &mut dyn IDestination) {
    if is_valid_toml_bare_key(key) {
        dest.add_bytes(key);
    } else {
        write_toml_escaped_string(key, dest);
    }
}

fn format_table_path(full_path: &str, dest: &mut dyn IDestination) {
    let mut first = true;
    for part in full_path.split('.') {
        if !first {
            dest.add_byte(b'.');
        }
        first = false;
        format_key(part, dest);
    }
}

fn is_array_of_tables(node: &Node) -> bool {
    match node {
        Node::Array(arr) => !arr.is_empty() && arr.iter().all(Node::is_table),
        _ => false,
    }
}

fn serialize_table(entries: &[(String, Node)], prefix: &str, dest: &mut dyn IDestination) {
    // 1. Emit simple scalars & inline arrays
    for (k, v) in entries {
        if matches!(v, Node::Table(_)) || is_array_of_tables(v) {
            continue;
        }
        format_key(k, dest);
        dest.add_bytes(" = ");
        serialize_value(v, dest);
        dest.add_byte(b'\n');
    }

    // 2. Emit nested tables
    for (k, v) in entries {
        if let Node::Table(sub_entries) = v {
            let full_key = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{}.{}", prefix, k)
            };
            dest.add_bytes("\n[");
            format_table_path(&full_key, dest);
            dest.add_bytes("]\n");
            serialize_table(sub_entries, &full_key, dest);
        }
    }

    // 3. Emit array of tables
    for (k, v) in entries {
        if is_array_of_tables(v) {
            if let Node::Array(items) = v {
                let full_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                for item in items {
                    dest.add_bytes("\n[[");
                    format_table_path(&full_key, dest);
                    dest.add_bytes("]]\n");
                    if let Node::Table(sub_entries) = item {
                        serialize_table(sub_entries, &full_key, dest);
                    }
                }
            }
        }
    }
}

fn serialize_value(val: &Node, dest: &mut dyn IDestination) {
    match val {
        Node::String(s) => write_toml_escaped_string(s, dest),
        Node::Integer(i) => {
            let mut buf = itoa::Buffer::new();
            dest.add_bytes(buf.format(*i));
        }
        Node::Float(f) => {
            if f.is_nan() {
                dest.add_bytes("nan");
            } else if f.is_infinite() {
                dest.add_bytes(if *f < 0.0 { "-inf" } else { "inf" });
            } else {
                let mut buf = dtoa::Buffer::new();
                let s = buf.format(*f);
                dest.add_bytes(s);
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    dest.add_bytes(".0");
                }
            }
        }
        Node::Boolean(b) => dest.add_bytes(if *b { "true" } else { "false" }),
        Node::Datetime(dt) => dest.add_bytes(dt.as_str()),
        Node::Array(items) => {
            dest.add_byte(b'[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    dest.add_bytes(", ");
                }
                serialize_value(item, dest);
            }
            dest.add_byte(b']');
        }
        Node::Table(entries) => {
            dest.add_bytes("{ ");
            for (idx, (k, v)) in entries.iter().enumerate() {
                if idx > 0 {
                    dest.add_bytes(", ");
                }
                format_key(k, dest);
                dest.add_bytes(" = ");
                serialize_value(v, dest);
            }
            dest.add_bytes(" }");
        }
    }
}
