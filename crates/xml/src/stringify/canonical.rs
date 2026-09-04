//! # Canonical XML (C14N) Serializer
//!
//! Implementation of W3C Canonical XML (C14N 1.0 / 1.1) formatting for XML Digital Signatures and document hashing.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::io::destination::XmlDestination;
use crate::node::{NodeId, NodeKind};

/// Configuration options for Canonical XML (C14N) serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalOptions {
    /// Retain XML comments in canonical output (default: false).
    pub with_comments: bool,
}

impl Default for CanonicalOptions {
    fn default() -> Self {
        Self {
            with_comments: false,
        }
    }
}

/// Serializer producing W3C Canonical XML (C14N) output.
pub struct CanonicalSerializer;

impl CanonicalSerializer {
    /// Maximum element nesting depth allowed during canonicalization (512 frames).
    pub const MAX_CANONICAL_DEPTH: usize = 512;

    /// Serializes a [`Document`] to a canonical XML string according to W3C C14N rules.
    pub fn canonicalize(doc: &Document, options: &CanonicalOptions) -> String {
        let mut dest = XmlDestination::new();
        if let Some(root_elem_id) = doc.root_element_id() {
            Self::serialize_canonical_node(doc, root_elem_id, options, &mut dest, 0);
        }
        dest.into_string()
    }

    fn serialize_canonical_node(
        doc: &Document,
        node_id: NodeId,
        options: &CanonicalOptions,
        dest: &mut XmlDestination,
        depth: usize,
    ) {
        if depth > Self::MAX_CANONICAL_DEPTH {
            return;
        }

        let node = match doc.get_node(node_id) {
            Some(n) => n,
            None => return,
        };

        match &node.kind {
            NodeKind::Element { name, attributes } => {
                dest.write_str("<");
                dest.write_str(name);

                // Sort attributes in lexicographical order:
                // Primary key: namespace URI; Secondary key: local name
                let mut sorted_attrs = attributes.clone();
                sorted_attrs.sort_by(|a, b| {
                    let a_is_xmlns = a.name.starts_with("xmlns");
                    let b_is_xmlns = b.name.starts_with("xmlns");

                    if a_is_xmlns && !b_is_xmlns {
                        core::cmp::Ordering::Less
                    } else if !a_is_xmlns && b_is_xmlns {
                        core::cmp::Ordering::Greater
                    } else {
                        a.name.cmp(&b.name)
                    }
                });

                for attr in &sorted_attrs {
                    dest.write_str(" ");
                    dest.write_str(&attr.name);
                    dest.write_str("=\"");
                    Self::write_canonical_attr(&attr.value, dest);
                    dest.write_str("\"");
                }

                dest.write_str(">");

                // Serialize children
                for &child_id in &node.children {
                    Self::serialize_canonical_node(doc, child_id, options, dest, depth + 1);
                }

                // C14N: Empty elements are NEVER self-closing (<elem></elem>)
                dest.write_str("</");
                dest.write_str(name);
                dest.write_str(">");
            }
            NodeKind::Text(t) => {
                Self::write_canonical_text(t, dest);
            }
            NodeKind::CData(c) => {
                // In C14N, CDATA sections are replaced with character data
                Self::write_canonical_text(c, dest);
            }
            NodeKind::Comment(com) => {
                if options.with_comments {
                    dest.write_str("<!--");
                    dest.write_str(com);
                    dest.write_str("-->");
                }
            }
            NodeKind::ProcessingInstruction { target, data } => {
                dest.write_str("<?");
                dest.write_str(target);
                if !data.is_empty() {
                    dest.write_str(" ");
                    dest.write_str(data);
                }
                dest.write_str("?>");
            }
            _ => {}
        }
    }

    fn write_canonical_text(text: &str, dest: &mut XmlDestination) {
        for ch in text.chars() {
            match ch {
                '&' => dest.write_str("&amp;"),
                '<' => dest.write_str("&lt;"),
                '>' => dest.write_str("&gt;"),
                '\r' => dest.write_str("&#xD;"),
                _ => dest.write_char(ch),
            }
        }
    }

    fn write_canonical_attr(value: &str, dest: &mut XmlDestination) {
        for ch in value.chars() {
            match ch {
                '&' => dest.write_str("&amp;"),
                '<' => dest.write_str("&lt;"),
                '"' => dest.write_str("&quot;"),
                '\t' => dest.write_str("&#x9;"),
                '\n' => dest.write_str("&#xA;"),
                '\r' => dest.write_str("&#xD;"),
                _ => dest.write_char(ch),
            }
        }
    }
}
