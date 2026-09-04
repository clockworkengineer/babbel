//! # XML Parser Engine
//!
//! Recursive descent XML parser turning character streams ([`XmlSource`]) into DOM trees ([`Document`]).

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::entity::EntityMapper;
use crate::error::{Result, XmlError};
use crate::io::{is_valid_xml_char, is_xml_name_char, XmlSource};
use crate::node::{Attribute, NodeId, NodeKind};
use crate::options::ParseOptions;

/// Main recursive descent XML parser implementation.
#[derive(Debug)]
pub struct XmlParser<'a> {
    source: XmlSource,
    options: ParseOptions,
    entity_mapper: EntityMapper,
    element_count: usize,
    total_attribute_count: usize,
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> XmlParser<'a> {
    /// Instantiates a new [`XmlParser`] with an input source and parse options.
    pub fn new(source: XmlSource, options: ParseOptions) -> Self {
        let mapper = EntityMapper::with_limits(
            options.max_entity_expansion_depth,
            options.max_total_entity_expansion_size,
        );
        Self {
            source,
            options,
            entity_mapper: mapper,
            element_count: 0,
            total_attribute_count: 0,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Parses the entire input source into a DOM [`Document`].
    pub fn parse(&mut self) -> Result<Document> {
        self.options.check_xml_size(self.source.len())?;
        let mut doc = Document::new();

        self.source.skip_whitespace();
        if self.source.starts_with("<?xml") {
            self.parse_declaration(&mut doc)?;
        }

        self.parse_prolog(&mut doc)?;

        let root_container_id = doc.root_id().unwrap_or(0);
        self.parse_element(&mut doc, root_container_id, 0)?;

        self.parse_epilog(&mut doc)?;

        self.source.skip_whitespace();
        if !self.source.is_eof() {
            if self.source.starts_with("<") {
                return Err(XmlError::SyntaxError {
                    message: "Multiple root elements forbidden in well-formed XML".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Unexpected trailing content after root element".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }
        Ok(doc)
    }

    /// Parses XML declaration (`<?xml version="..." encoding="..."?>`).
    fn parse_declaration(&mut self, doc: &mut Document) -> Result<()> {
        self.source.consume("<?xml");
        self.source.skip_whitespace();

        let mut version = String::from("1.0");
        let mut encoding = None;
        let mut standalone = None;

        while !self.source.is_eof() && !self.source.starts_with("?>") {
            let (key, val) = self.parse_attribute()?;
            match key.as_str() {
                "version" => version = val,
                "encoding" => encoding = Some(val),
                "standalone" => standalone = Some(val == "yes"),
                _ => {}
            }
            self.source.skip_whitespace();
        }

        if !self.source.consume("?>") {
            return Err(XmlError::SyntaxError {
                message: "Unclosed XML declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        let decl_id = doc.add_node(NodeKind::Declaration {
            version: version.into_boxed_str(),
            encoding: encoding.map(String::into_boxed_str),
            standalone,
        });
        doc.set_declaration_id(decl_id);

        let prolog_id = doc.prolog_id().unwrap_or(0);
        doc.append_child(prolog_id, decl_id)?;

        Ok(())
    }

    /// Parses prolog items (comments, PIs, DOCTYPE) prior to the root element tag.
    fn parse_prolog(&mut self, doc: &mut Document) -> Result<()> {
        let prolog_id = doc.prolog_id().unwrap_or(0);

        loop {
            self.source.skip_whitespace();
            if self.source.starts_with("<!--") {
                let comment_id = self.parse_comment(doc)?;
                doc.append_child(prolog_id, comment_id)?;
            } else if self.source.starts_with("<?") {
                let pi_id = self.parse_pi(doc)?;
                doc.append_child(prolog_id, pi_id)?;
            } else if self.source.starts_with("<!DOCTYPE") {
                let dtd_id = self.parse_doctype(doc)?;
                doc.set_dtd_id(dtd_id);
                doc.append_child(prolog_id, dtd_id)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Parses epilog items (comments, PIs) following the root element tag.
    fn parse_epilog(&mut self, doc: &mut Document) -> Result<()> {
        let root_container_id = doc.root_id().unwrap_or(0);
        loop {
            self.source.skip_whitespace();
            if self.source.starts_with("<!--") {
                let comment_id = self.parse_comment(doc)?;
                doc.append_child(root_container_id, comment_id)?;
            } else if self.source.starts_with("<?") {
                let pi_id = self.parse_pi(doc)?;
                doc.append_child(root_container_id, pi_id)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Parses a single XML element tag, its attributes, and child content recursively.
    fn parse_element(&mut self, doc: &mut Document, parent_id: NodeId, depth: usize) -> Result<()> {
        self.options.check_nesting_depth(depth)?;

        self.element_count += 1;
        self.options.check_element_count(self.element_count)?;

        if !self.source.consume("<") {
            return Err(XmlError::SyntaxError {
                message: "Expected '<' at start of element".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        let name = self.parse_name()?;
        if name.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Empty element tag name".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        self.source.skip_whitespace();
        let mut attributes = Vec::new();

        while !self.source.is_eof() && !self.source.starts_with(">") && !self.source.starts_with("/>") {
            let (attr_name, attr_value) = self.parse_attribute()?;
            
            // Check duplicate attribute error
            if attributes.iter().any(|a: &Attribute| *a.name == attr_name) {
                return Err(XmlError::SyntaxError {
                    message: format!("Duplicate attribute '{attr_name}' on element <{name}>"),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }

            attributes.push(Attribute::new(attr_name, attr_value));

            self.total_attribute_count += 1;
            self.options.check_attribute_count(attributes.len())?;
            self.options.check_total_attribute_count(self.total_attribute_count)?;

            self.source.skip_whitespace();
        }

        let elem_id = doc.add_node(NodeKind::Element {
            name: name.clone().into_boxed_str(),
            attributes,
        });
        doc.append_child(parent_id, elem_id)?;

        if self.source.consume("/>") {
            return Ok(());
        }

        if !self.source.consume(">") {
            return Err(XmlError::SyntaxError {
                message: format!("Expected '>' or '/>' for element <{name}>"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        // Parse element body / child content
        loop {
            if self.source.is_eof() {
                return Err(XmlError::SyntaxError {
                    message: format!("Unclosed element <{name}>"),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }

            if self.source.starts_with("</") {
                self.source.consume("</");
                let end_name = self.parse_name()?;
                self.source.skip_whitespace();

                if end_name != name {
                    return Err(XmlError::SyntaxError {
                        message: format!("Mismatched closing tag: expected </{name}>, found </{end_name}>"),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }

                if !self.source.consume(">") {
                    return Err(XmlError::SyntaxError {
                        message: format!("Expected '>' after closing tag </{name}>"),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }

                break;
            } else if self.source.starts_with("<![CDATA[") {
                let cdata_id = self.parse_cdata(doc)?;
                doc.append_child(elem_id, cdata_id)?;
            } else if self.source.starts_with("<!--") {
                let comment_id = self.parse_comment(doc)?;
                doc.append_child(elem_id, comment_id)?;
            } else if self.source.starts_with("<?") {
                let pi_id = self.parse_pi(doc)?;
                doc.append_child(elem_id, pi_id)?;
            } else if self.source.starts_with("<") {
                self.parse_element(doc, elem_id, depth + 1)?;
            } else {
                let text_id = self.parse_text(doc)?;
                if let Some(t_node) = doc.get_node(text_id) {
                    if let NodeKind::Text(t) = &t_node.kind {
                        if !t.is_empty() {
                            doc.append_child(elem_id, text_id)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Parses attribute key-value pair (`key="value"`).
    fn parse_attribute(&mut self) -> Result<(String, String)> {
        let key = self.parse_name()?;
        self.source.skip_whitespace();

        if !self.source.consume("=") {
            return Err(XmlError::SyntaxError {
                message: format!("Expected '=' after attribute '{key}'"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        self.source.skip_whitespace();
        let quote = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
            message: format!("Expected quote after '=' for attribute '{key}'"),
            line: self.source.line(),
            col: self.source.col(),
        })?;

        if quote != '"' && quote != '\'' {
            return Err(XmlError::SyntaxError {
                message: format!("Invalid attribute quote character '{quote}'"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        let mut raw_val = String::new();
        while let Some(ch) = self.source.next_char() {
            if ch == quote {
                let expanded_val = self.entity_mapper.expand(&raw_val)?;
                return Ok((key, expanded_val));
            }
            raw_val.push(ch);
        }

        Err(XmlError::SyntaxError {
            message: format!("Unterminated value for attribute '{key}'"),
            line: self.source.line(),
            col: self.source.col(),
        })
    }

    /// Parses text content up to the next `<` tag start.
    fn parse_text(&mut self, doc: &mut Document) -> Result<NodeId> {
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

        let expanded = self.entity_mapper.expand(&raw_text)?;
        Ok(doc.add_node(NodeKind::Text(expanded.into_boxed_str())))
    }

    /// Parses CDATA section (`<![CDATA[...]]>`).
    fn parse_cdata(&mut self, doc: &mut Document) -> Result<NodeId> {
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
    fn parse_comment(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume("<!--");
        let mut comment = String::new();

        while !self.source.is_eof() {
            if self.source.starts_with("-->") {
                self.source.consume("-->");
                return Ok(doc.add_node(NodeKind::Comment(comment.into_boxed_str())));
            }
            let ch = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
                message: "Unterminated XML comment".into(),
                line: self.source.line(),
                col: self.source.col(),
            })?;
            comment.push(ch);
        }

        Err(XmlError::SyntaxError {
            message: "Unterminated XML comment".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }

    /// Parses processing instruction (`<?target data?>`).
    fn parse_pi(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume("<?");
        let target = self.parse_name()?;
        self.source.skip_whitespace();

        let mut data = String::new();
        while !self.source.is_eof() {
            if self.source.starts_with("?>") {
                self.source.consume("?>");
                return Ok(doc.add_node(NodeKind::ProcessingInstruction {
                    target: target.into_boxed_str(),
                    data: data.into_boxed_str(),
                }));
            }
            let ch = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
                message: "Unterminated processing instruction".into(),
                line: self.source.line(),
                col: self.source.col(),
            })?;
            data.push(ch);
        }

        Err(XmlError::SyntaxError {
            message: "Unterminated processing instruction".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }

    /// Parses DTD DOCTYPE definition (`<!DOCTYPE name ...>`).
    fn parse_doctype(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume("<!DOCTYPE");
        self.source.skip_whitespace();

        let name = self.parse_name()?;
        self.source.skip_whitespace();

        let mut public_id = None;
        let mut system_id = None;
        let mut internal_subset = None;

        if self.source.consume("PUBLIC") {
            self.source.skip_whitespace();
            public_id = Some(self.parse_quoted_string()?);
            self.source.skip_whitespace();
            system_id = Some(self.parse_quoted_string()?);
        } else if self.source.consume("SYSTEM") {
            self.source.skip_whitespace();
            system_id = Some(self.parse_quoted_string()?);
        }

        self.source.skip_whitespace();
        if self.source.consume("[") {
            let mut subset = String::new();
            while !self.source.is_eof() && !self.source.starts_with("]") {
                let ch = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
                    message: "Unterminated DOCTYPE subset".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                })?;
                subset.push(ch);
            }
            self.source.consume("]");
            internal_subset = Some(subset);
        }

        self.source.skip_whitespace();
        if !self.source.consume(">") {
            return Err(XmlError::SyntaxError {
                message: "Unclosed DOCTYPE declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        // Register entity declarations from internal subset
        if let Some(subset) = &internal_subset {
            for line in subset.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("<!ENTITY") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let ent_name = parts[1];
                        if (trimmed.contains("SYSTEM") || trimmed.contains("PUBLIC")) && !self.options.allow_external_entities {
                            return Err(XmlError::SecurityLimitExceeded(
                                "External entity references in DOCTYPE are forbidden by security policy".into(),
                            ));
                        }
                        let raw_val = parts[2..].join(" ");
                        let val = raw_val.trim_matches(|c| c == '"' || c == '\'' || c == '>');
                        self.entity_mapper.register(ent_name, val);
                    }
                }
            }
        }

        Ok(doc.add_node(NodeKind::DocTypeDefinition {
            name: name.into_boxed_str(),
            public_id: public_id.map(String::into_boxed_str),
            system_id: system_id.map(String::into_boxed_str),
            internal_subset: internal_subset.map(String::into_boxed_str),
        }))
    }

    /// Helper parsing an identifier / tag name string.
    fn parse_name(&mut self) -> Result<String> {
        let start = self.source.position();
        while let Some(ch) = self.source.peek() {
            if is_xml_name_char(ch) {
                self.source.next_char();
            } else {
                break;
            }
        }
        let end = self.source.position();
        Ok(self.source.slice_range(start, end).to_string())
    }

    /// Helper parsing a single- or double-quoted string.
    fn parse_quoted_string(&mut self) -> Result<String> {
        let quote = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
            message: "Expected quote for string literal".into(),
            line: self.source.line(),
            col: self.source.col(),
        })?;

        let mut s = String::new();
        while let Some(ch) = self.source.next_char() {
            if ch == quote {
                return Ok(s);
            }
            s.push(ch);
        }

        Err(XmlError::SyntaxError {
            message: "Unterminated string literal".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }
}
