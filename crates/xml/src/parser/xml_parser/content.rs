//! XML character data, CDATA, and comment parsing implementation.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::io::is_valid_xml_char;
use crate::node::{NodeId, NodeKind};

use super::XmlParser;

impl<'a> XmlParser<'a> {
    /// Parses text content up to the next `<` tag start.
    pub(crate) fn parse_text(&mut self, doc: &mut Document) -> Result<NodeId> {
        let mut raw_text = String::new();
        while let Some(ch) = self.source.peek() {
            if ch == '<' {
                break;
            }
            if !is_valid_xml_char(ch) {
                return Err(XmlError::SyntaxError {
                    message: format!("Forbidden XML character '\\u{{{:x}}}'", ch as u32),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let ch = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
                message: "Unexpected EOF while reading text".into(),
                line: self.source.line(),
                col: self.source.col(),
            })?;
            raw_text.push(ch);
            self.options.check_text_node_size(raw_text.len())?;
        }

        if raw_text.contains("]]>") {
            return Err(XmlError::SyntaxError {
                message: "The sequence ']]>' is forbidden in character data".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        let mut pos = 0;
        let bytes = raw_text.as_bytes();
        while pos < bytes.len() {
            if bytes[pos] == b'&' {
                if let Some(semi_offset) = raw_text[pos..].find(';') {
                    let semi_idx = pos + semi_offset;
                    let ref_name = &raw_text[pos + 1..semi_idx];
                    if !ref_name.starts_with('#') {
                        if self.unparsed_entities.iter().any(|u| u == ref_name) {
                            return Err(XmlError::SyntaxError {
                                message: format!("Unparsed entity '{ref_name}' cannot be referenced in content (WFC: Parsed Entity)"),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                        if ref_name != "lt" && ref_name != "gt" && ref_name != "amp" && ref_name != "quot" && ref_name != "apos" {
                            if !self.external_entities.iter().any(|e| e == ref_name) {
                                if let Some(val) = self.entity_mapper.get(ref_name) {
                                    self.validate_entity_replacement_text(ref_name, val)?;
                                }
                            }
                        }
                    }
                    pos = semi_idx + 1;
                } else {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }

        let expanded = self.entity_mapper.expand(&raw_text)?;
        Ok(doc.add_node(NodeKind::Text(expanded.into_boxed_str())))
    }

    /// Parses CDATA section (`<![CDATA[...]]>`).
    pub(crate) fn parse_cdata(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume("<![CDATA[");
        let mut content = String::new();

        while !self.source.is_eof() {
            if self.source.starts_with("]]>") {
                self.source.consume("]]>");
                return Ok(doc.add_node(NodeKind::CData(content.into_boxed_str())));
            }
            let ch = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
                message: "Unterminated CDATA section".into(),
                line: self.source.line(),
                col: self.source.col(),
            })?;
            if !is_valid_xml_char(ch) {
                return Err(XmlError::SyntaxError {
                    message: format!("Forbidden XML character '\\u{{{:x}}}' in CDATA", ch as u32),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            content.push(ch);
            self.options.check_text_node_size(content.len())?;
        }

        Err(XmlError::SyntaxError {
            message: "Unterminated CDATA section".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }

    /// Parses XML comment (`<!-- ... -->`).
    pub(crate) fn parse_comment(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume("<!--");
        let mut comment = String::new();

        while !self.source.is_eof() {
            if self.source.starts_with("-->") {
                if comment.ends_with('-') {
                    return Err(XmlError::SyntaxError {
                        message: "Comment must not end in '--->'".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                self.source.consume("-->");
                return Ok(doc.add_node(NodeKind::Comment(comment.into_boxed_str())));
            }
            let ch = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
                message: "Unterminated XML comment".into(),
                line: self.source.line(),
                col: self.source.col(),
            })?;
            if !is_valid_xml_char(ch) {
                return Err(XmlError::SyntaxError {
                    message: format!("Forbidden XML character '\\u{{{:x}}}' in comment", ch as u32),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            if comment.ends_with('-') && ch == '-' {
                return Err(XmlError::SyntaxError {
                    message: "The string '--' is forbidden inside comments".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            comment.push(ch);
        }

        Err(XmlError::SyntaxError {
            message: "Unterminated XML comment".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }
}
