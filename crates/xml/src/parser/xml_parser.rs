//! # XML Parser Engine
//!
//! Recursive descent XML parser turning character streams ([`XmlSource`]) into DOM trees ([`Document`]).

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::entity::EntityMapper;
use crate::error::{Result, XmlError};
use crate::io::{is_valid_xml_char, is_xml_name_char, is_xml_name_start, XmlSource};
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

        // XML 1.0 §4.3.3: Fatal error if declared encoding conflicts with actual stream encoding
        if let Some(enc) = &encoding {
            if (enc.eq_ignore_ascii_case("UTF-16")
                || enc.eq_ignore_ascii_case("UTF-16LE")
                || enc.eq_ignore_ascii_case("UTF-16BE"))
                && !self.options.is_utf16
            {
                return Err(XmlError::SyntaxError {
                    message: format!("Declared encoding '{enc}' does not match byte stream encoding"),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }

        let decl_id = doc.add_node(NodeKind::Declaration(Box::new(crate::node::DeclarationData {
            version: version.into_boxed_str(),
            encoding: encoding.map(String::into_boxed_str),
            standalone,
        })));
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

        // Namespaces in XML 1.0 §3: Elements must not have the prefix 'xmlns'
        if name.starts_with("xmlns:") {
            return Err(XmlError::SyntaxError {
                message: format!("Element <{name}> must not have prefix 'xmlns'"),
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

                // Namespaces in XML 1.0 validation checks
                if key == "xmlns" {
                    if expanded_val == "http://www.w3.org/XML/1998/namespace" {
                        return Err(XmlError::SyntaxError {
                            message: "The xml namespace must not be declared as the default namespace".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    if expanded_val == "http://www.w3.org/2000/xmlns/" {
                        return Err(XmlError::SyntaxError {
                            message: "The xmlns namespace must not be declared as a namespace".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                } else if key == "xmlns:xml" {
                    if expanded_val != "http://www.w3.org/XML/1998/namespace" {
                        return Err(XmlError::SyntaxError {
                            message: "The prefix 'xml' can only be bound to 'http://www.w3.org/XML/1998/namespace'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                } else if key == "xmlns:xmlns" {
                    return Err(XmlError::SyntaxError {
                        message: "The prefix 'xmlns' must not be declared".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                } else if key.starts_with("xmlns:") {
                    if expanded_val == "http://www.w3.org/2000/xmlns/" {
                        return Err(XmlError::SyntaxError {
                            message: "Prefix cannot be bound to the xmlns namespace".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    if expanded_val.is_empty() {
                        return Err(XmlError::SyntaxError {
                            message: "Empty namespace URI is illegal for prefixed namespace in XML 1.0".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                }

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

        // Register external DTD entity declarations if allowed
        #[cfg(feature = "std")]
        if let Some(sys_id) = &system_id {
            if self.options.allow_external_entities {
                let dtd_path = if let Some(base) = &self.options.base_dir {
                    std::path::Path::new(base).join(sys_id.as_str())
                } else {
                    std::path::PathBuf::from(sys_id.as_str())
                };
                if let Ok(content) = std::fs::read_to_string(&dtd_path) {
                    let _ = self.register_entities_from_text(&content);
                }
            }
        }

        // Register entity declarations from internal subset
        if let Some(subset) = &internal_subset {
            self.register_entities_from_text(subset)?;
        }

        // XML 1.0 §4.1: If DOCTYPE contains parameter entities or external subsets,
        // undeclared entity references are validity errors, not well-formedness errors.
        if self.options.allow_external_entities
            || system_id.is_some()
            || public_id.is_some()
            || internal_subset.as_ref().map(|s| s.contains('%')).unwrap_or(false)
        {
            self.entity_mapper.allow_undeclared = true;
        }

        Ok(doc.add_node(NodeKind::DocTypeDefinition(Box::new(crate::node::DocTypeData {
            name: name.into_boxed_str(),
            public_id: public_id.map(String::into_boxed_str),
            system_id: system_id.map(String::into_boxed_str),
            internal_subset: internal_subset.map(String::into_boxed_str),
        }))))
    }

    /// Helper registering entity declarations from a DTD text subset.
    fn register_entities_from_text(&mut self, text: &str) -> Result<()> {
        let mut external_entities: Vec<String> = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("<!ENTITY") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let is_param = parts[1] == "%";
                    let (name_idx, val_idx) = if is_param && parts.len() >= 4 {
                        (2, 3)
                    } else {
                        (1, 2)
                    };
                    let ent_name = parts[name_idx];
                    let is_external = trimmed.contains("SYSTEM") || trimmed.contains("PUBLIC");
                    if is_external {
                        external_entities.push(ent_name.to_string());
                    }
                    if is_external && !self.options.allow_external_entities {
                        return Err(XmlError::SecurityLimitExceeded(
                            "External entity references in DOCTYPE are forbidden by security policy".into(),
                        ));
                    }
                    if is_external && self.options.allow_external_entities && trimmed.contains("SYSTEM") && !trimmed.contains("NDATA") {
                        let raw_val = parts[val_idx..].join(" ");
                        let sys_val = raw_val.trim_matches(|c| c == '"' || c == '\'' || c == '>' || c == ';').trim();
                        let file_name = sys_val.strip_prefix("SYSTEM").unwrap_or(sys_val).trim().trim_matches(|c| c == '"' || c == '\'');
                        #[cfg(feature = "std")]
                        {
                            let file_path = if let Some(base) = &self.options.base_dir {
                                std::path::Path::new(base).join(file_name)
                            } else {
                                std::path::PathBuf::from(file_name)
                            };
                            if let Ok(bytes) = std::fs::read(&file_path) {
                                if let Ok((loaded_text, _)) = babbel_core::encoding::detect_encoding_and_strip_bom(&bytes) {
                                    // XML 1.0 §4.3.4: An XML 1.0 document cannot include external entity with version 1.1
                                    if loaded_text.contains("<?xml") && (loaded_text.contains("version=\"1.1\"") || loaded_text.contains("version='1.1'")) {
                                        return Err(XmlError::SyntaxError {
                                            message: "XML 1.0 document cannot include external entity with version 1.1".into(),
                                            line: self.source.line(),
                                            col: self.source.col(),
                                        });
                                    }
                                    if is_param {
                                        let _ = self.register_entities_from_text(&loaded_text);
                                    }
                                    self.entity_mapper.register(ent_name, &*loaded_text);
                                    continue;
                                }
                            }
                        }
                    }
                    let raw_val = parts[val_idx..].join(" ");
                    let val = raw_val.trim_matches(|c| c == '"' || c == '\'' || c == '>');
                    self.entity_mapper.register(ent_name, val);
                }
            } else if trimmed.starts_with("<!ATTLIST") {
                // W3C XML 1.0 §3.1: Attribute values must not contain references to external entities
                for ext in &external_entities {
                    let needle = format!("&{ext};");
                    if trimmed.contains(&needle) {
                        return Err(XmlError::SyntaxError {
                            message: format!("Attribute values must not contain references to external entity '&{ext};'"),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper parsing an identifier / tag name string.
    fn parse_name(&mut self) -> Result<String> {
        let start = self.source.position();
        if let Some(ch) = self.source.peek() {
            if !is_xml_name_start(ch) {
                return Ok(String::new());
            }
            self.source.next_char();
        }
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
