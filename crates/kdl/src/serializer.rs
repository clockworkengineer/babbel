//! KDL serializer supporting compact and pretty-printed formatting.

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

use babbel_core::{Value, io::IDestination};

use crate::ast::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use crate::error::KdlError;

/// Serialize a universal `Value` into a compact KDL document string.
pub fn to_string(value: &Value) -> Result<String, KdlError> {
    let doc = KdlDocument::from_value(value);
    let mut dest = babbel_core::BufferDestination::new();
    serialize_document(&doc, &mut dest, false, 2)?;
    dest.into_string()
        .map_err(|_| KdlError::Serialization("Output contains invalid UTF-8".to_string()))
}

/// Serialize a universal `Value` into a pretty-printed KDL document string with indentation.
pub fn to_string_pretty(value: &Value, indent: usize) -> Result<String, KdlError> {
    let doc = KdlDocument::from_value(value);
    let mut dest = babbel_core::BufferDestination::new();
    serialize_document(&doc, &mut dest, true, indent)?;
    dest.into_string()
        .map_err(|_| KdlError::Serialization("Output contains invalid UTF-8".to_string()))
}

/// Serialize a `KdlDocument` AST into destination.
pub fn serialize_document(
    doc: &KdlDocument,
    dest: &mut dyn IDestination,
    pretty: bool,
    indent_size: usize,
) -> Result<(), KdlError> {
    for node in &doc.nodes {
        serialize_node(node, dest, 0, pretty, indent_size)?;
    }
    Ok(())
}

fn serialize_node(
    node: &KdlNode,
    dest: &mut dyn IDestination,
    current_indent: usize,
    pretty: bool,
    indent_size: usize,
) -> Result<(), KdlError> {
    if pretty {
        write_indent(dest, current_indent);
    }

    if let Some(ref ty) = node.type_annotation {
        dest.add_byte(b'(');
        dest.add_bytes(ty);
        dest.add_byte(b')');
    }

    write_identifier(&node.name, dest);

    for entry in &node.entries {
        dest.add_byte(b' ');
        match entry {
            KdlEntry::Arg(ty, val) => {
                if let Some(t) = ty {
                    dest.add_byte(b'(');
                    dest.add_bytes(t);
                    dest.add_byte(b')');
                }
                serialize_value(val, dest);
            }
            KdlEntry::Prop(k, ty, val) => {
                write_identifier(k, dest);
                dest.add_byte(b'=');
                if let Some(t) = ty {
                    dest.add_byte(b'(');
                    dest.add_bytes(t);
                    dest.add_byte(b')');
                }
                serialize_value(val, dest);
            }
        }
    }

    if !node.children.is_empty() {
        dest.add_bytes(" {");
        if pretty {
            dest.add_byte(b'\n');
            for child in &node.children {
                serialize_node(
                    child,
                    dest,
                    current_indent + indent_size,
                    pretty,
                    indent_size,
                )?;
            }
            write_indent(dest, current_indent);
            dest.add_byte(b'}');
        } else {
            dest.add_byte(b' ');
            for child in &node.children {
                serialize_node(child, dest, 0, false, 0)?;
            }
            dest.add_byte(b'}');
        }
    }

    if pretty {
        dest.add_byte(b'\n');
    } else {
        dest.add_byte(b'\n');
    }

    Ok(())
}

fn serialize_value(val: &KdlValue, dest: &mut dyn IDestination) {
    match val {
        KdlValue::String(s) => {
            write_quoted_string(s, dest);
        }
        KdlValue::Integer(i) => {
            let s = format!("{}", i);
            dest.add_bytes(&s);
        }
        KdlValue::Float(f) => {
            let s = format!("{}", f);
            if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                dest.add_bytes(&s);
                dest.add_bytes(".0");
            } else {
                dest.add_bytes(&s);
            }
        }
        KdlValue::Bool(b) => {
            dest.add_bytes(if *b { "true" } else { "false" });
        }
        KdlValue::Null => {
            dest.add_bytes("null");
        }
    }
}

fn write_identifier(name: &str, dest: &mut dyn IDestination) {
    if is_safe_bare_ident(name) {
        dest.add_bytes(name);
    } else {
        write_quoted_string(name, dest);
    }
}

fn is_safe_bare_ident(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // Cannot be keywords true, false, null if bare identifier
    if name == "true" || name == "false" || name == "null" {
        return false;
    }
    // Cannot start with digit
    if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        return false;
    }
    name.chars().all(|ch| {
        !matches!(
            ch,
            '(' | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '/'
                | '\\'
                | '"'
                | '='
                | ';'
                | ','
                | ' '
                | '\t'
                | '\r'
                | '\n'
        ) && !ch.is_control()
    })
}

fn write_quoted_string(s: &str, dest: &mut dyn IDestination) {
    dest.add_byte(b'"');
    for ch in s.chars() {
        match ch {
            '"' => dest.add_bytes("\\\""),
            '\\' => dest.add_bytes("\\\\"),
            '\n' => dest.add_bytes("\\n"),
            '\r' => dest.add_bytes("\\r"),
            '\t' => dest.add_bytes("\\t"),
            '\x08' => dest.add_bytes("\\b"),
            '\x0C' => dest.add_bytes("\\f"),
            c if c.is_control() => {
                let u = format!("\\u{{{:x}}}", c as u32);
                dest.add_bytes(&u);
            }
            other => {
                let mut buf = [0u8; 4];
                let enc = other.encode_utf8(&mut buf);
                dest.add_bytes(enc);
            }
        }
    }
    dest.add_byte(b'"');
}

fn write_indent(dest: &mut dyn IDestination, spaces: usize) {
    for _ in 0..spaces {
        dest.add_byte(b' ');
    }
}
