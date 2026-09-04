//! # XML Serializer / Stringifier
//!
//! Formats and serializes a [`Document`] DOM tree back into valid XML string output with optional pretty printing.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::io::destination::XmlDestination;
use crate::node::{NodeId, NodeKind};

/// Formatting options for XML serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializeOptions {
    /// Enable pretty-printing indentation (default: true).
    pub pretty_print: bool,
    /// Indentation step size in spaces (default: 2).
    pub indent_step: usize,
    /// Omit the XML declaration even if present in Document (default: false).
    pub omit_xml_declaration: bool,
    /// Quote character for attributes (`"` or `'`, default: `"`).
    pub quote_char: char,
    /// Self-close empty elements like `<item/>` vs `<item></item>` (default: true).
    pub self_close_empty: bool,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            pretty_print: true,
            indent_step: 2,
            omit_xml_declaration: false,
            quote_char: '"',
            self_close_empty: true,
        }
    }
}

/// Serializer traversing the DOM tree and emitting formatted XML output.
pub struct XmlSerializer;

impl XmlSerializer {
    /// Maximum element nesting depth allowed during serialization (512 frames).
    pub const MAX_SERIALIZE_DEPTH: usize = 512;

    /// Serializes a [`Document`] DOM tree into a formatted string.
    pub fn serialize_to_string(doc: &Document, options: &SerializeOptions) -> String {
        let mut dest = XmlDestination::new();

        if !options.omit_xml_declaration {
            if let Some(decl_id) = doc.declaration_id() {
                if let Some(node) = doc.get_node(decl_id) {
                    if let NodeKind::Declaration {
                        version,
                        encoding,
                        standalone,
                    } = &node.kind
                    {
                        dest.write_str("<?xml version=");
                        dest.write_char(options.quote_char);
                        dest.write_str(version);
                        dest.write_char(options.quote_char);
                        if let Some(enc) = encoding {
                            dest.write_str(" encoding=");
                            dest.write_char(options.quote_char);
                            dest.write_str(enc);
                            dest.write_char(options.quote_char);
                        }
                        if let Some(st) = standalone {
                            dest.write_str(" standalone=");
                            dest.write_char(options.quote_char);
                            dest.write_str(if *st { "yes" } else { "no" });
                            dest.write_char(options.quote_char);
                        }
                        dest.write_str("?>\n");
                    }
                }
            }
        }

        if let Some(prolog_id) = doc.prolog_id() {
            Self::serialize_children(doc, prolog_id, 0, options, &mut dest);
        }

        if let Some(root_id) = doc.root_id() {
            Self::serialize_children(doc, root_id, 0, options, &mut dest);
        }

        dest.into_string()
    }

    /// Serializes a [`Document`] directly to an I/O writer.
    #[cfg(feature = "std")]
    pub fn serialize_to_writer<W: std::io::Write>(
        doc: &Document,
        writer: &mut W,
        options: &SerializeOptions,
    ) -> std::io::Result<()> {
        let s = Self::serialize_to_string(doc, options);
        writer.write_all(s.as_bytes())
    }

    /// Serializes a [`Document`] directly to a formatting writer.
    pub fn serialize_to_fmt<W: core::fmt::Write>(
        doc: &Document,
        writer: &mut W,
        options: &SerializeOptions,
    ) -> core::fmt::Result {
        let s = Self::serialize_to_string(doc, options);
        writer.write_str(&s)
    }

    fn serialize_children(
        doc: &Document,
        parent_id: NodeId,
        indent_level: usize,
        options: &SerializeOptions,
        dest: &mut XmlDestination,
    ) {
        if let Some(node) = doc.get_node(parent_id) {
            for &c_id in &node.children {
                if doc.declaration_id() == Some(c_id) {
                    continue; // Already processed XML Declaration
                }
                Self::serialize_node(doc, c_id, indent_level, options, dest);
            }
        }
    }

    fn serialize_node(
        doc: &Document,
        node_id: NodeId,
        indent_level: usize,
        options: &SerializeOptions,
        dest: &mut XmlDestination,
    ) {
        if indent_level > Self::MAX_SERIALIZE_DEPTH {
            return;
        }

        let node = match doc.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        let indent = if options.pretty_print {
            " ".repeat(indent_level * options.indent_step)
        } else {
            String::new()
        };

        match &node.kind {
            NodeKind::Element { name, attributes } => {
                dest.write_str(&indent);
                dest.write_str("<");
                dest.write_str(name);

                for attr in attributes {
                    dest.write_str(" ");
                    dest.write_str(&attr.name);
                    dest.write_str("=");
                    dest.write_char(options.quote_char);
                    Self::write_escaped_attr(&attr.value, options.quote_char, dest);
                    dest.write_char(options.quote_char);
                }

                if node.children.is_empty() {
                    if options.self_close_empty {
                        dest.write_str("/>");
                    } else {
                        dest.write_str("></");
                        dest.write_str(name);
                        dest.write_str(">");
                    }
                    if options.pretty_print {
                        dest.write_char('\n');
                    }
                } else {
                    dest.write_str(">");
                    let has_only_text = node.children.iter().all(|&c_id| {
                        doc.get_node(c_id).map_or(false, |c| {
                            matches!(c.kind, NodeKind::Text(_) | NodeKind::CData(_))
                        })
                    });

                    if has_only_text {
                        for &c_id in &node.children {
                            if let Some(child) = doc.get_node(c_id) {
                                match &child.kind {
                                    NodeKind::Text(t) => Self::write_escaped_text(t, dest),
                                    NodeKind::CData(c) => {
                                        dest.write_str("<![CDATA[");
                                        dest.write_str(c);
                                        dest.write_str("]]>");
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        if options.pretty_print {
                            dest.write_char('\n');
                        }
                        Self::serialize_children(doc, node_id, indent_level + 1, options, dest);
                        dest.write_str(&indent);
                    }

                    dest.write_str("</");
                    dest.write_str(name);
                    dest.write_str(">");
                    if options.pretty_print {
                        dest.write_char('\n');
                    }
                }
            }
            NodeKind::Text(t) => {
                dest.write_str(&indent);
                Self::write_escaped_text(t, dest);
                if options.pretty_print {
                    dest.write_char('\n');
                }
            }
            NodeKind::CData(c) => {
                dest.write_str(&indent);
                dest.write_str("<![CDATA[");
                dest.write_str(c);
                dest.write_str("]]>");
                if options.pretty_print {
                    dest.write_char('\n');
                }
            }
            NodeKind::Comment(c) => {
                dest.write_str(&indent);
                dest.write_str("<!--");
                dest.write_str(c);
                dest.write_str("-->");
                if options.pretty_print {
                    dest.write_char('\n');
                }
            }
            NodeKind::ProcessingInstruction { target, data } => {
                dest.write_str(&indent);
                dest.write_str("<?");
                dest.write_str(target);
                if !data.is_empty() {
                    dest.write_str(" ");
                    dest.write_str(data);
                }
                dest.write_str("?>");
                if options.pretty_print {
                    dest.write_char('\n');
                }
            }
            _ => {}
        }
    }

    fn write_escaped(s: &str, quote_char: Option<char>, dest: &mut XmlDestination) {
        let mut last = 0;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let esc = match b {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'"' if quote_char == Some('"') => "&quot;",
                b'\'' if quote_char == Some('\'') => "&apos;",
                _ => continue,
            };
            dest.write_str(&s[last..i]);
            dest.write_str(esc);
            last = i + 1;
        }
        dest.write_str(&s[last..]);
    }

    fn write_escaped_text(s: &str, dest: &mut XmlDestination) {
        Self::write_escaped(s, None, dest);
    }

    fn write_escaped_attr(s: &str, quote_char: char, dest: &mut XmlDestination) {
        Self::write_escaped(s, Some(quote_char), dest);
    }
}
