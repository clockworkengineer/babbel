//! # XML Parser Engine
//!
//! Recursive descent XML parser turning character streams ([`XmlSource`]) into DOM trees ([`Document`]).

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::entity::EntityMapper;
use crate::error::{Result, XmlError};
use crate::io::{is_valid_xml_char, is_xml_name_char, is_xml_name_start, XmlSource};
use crate::namespace::{NamespaceScope, QName};
use crate::node::{Attribute, NodeId, NodeKind};
use crate::options::ParseOptions;

/// Main recursive descent XML parser implementation.
#[derive(Debug)]
pub struct XmlParser<'a> {
    source: XmlSource,
    options: ParseOptions,
    entity_mapper: EntityMapper,
    namespace_scope: NamespaceScope,
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
            namespace_scope: NamespaceScope::new(),
            element_count: 0,
            total_attribute_count: 0,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Parses the entire input source into a DOM [`Document`].
    pub fn parse(&mut self) -> Result<Document> {
        self.options.check_xml_size(self.source.len())?;
        let mut doc = Document::new();

        // XML 1.0 §2.8 [22]: XML declaration must appear at the very start of document if present.
        // No whitespace or comments can precede it.
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
        // XML 1.0 §2.8 [23]: XMLDecl ::= '<?xml' VersionInfo ... where VersionInfo begins with S
        let ws = self.source.skip_whitespace();
        if ws == 0 {
            return Err(XmlError::SyntaxError {
                message: "Expected whitespace after '<?xml'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        // First attribute MUST be version
        let (k1, v1) = self.parse_attribute()?;
        if k1 != "version" {
            return Err(XmlError::SyntaxError {
                message: format!("XML declaration must begin with 'version', found '{k1}'"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        // XML 1.0 §2.8 [26]: VersionNum ::= '1.' [0-9]+
        if !v1.starts_with("1.") || v1.len() < 3 || !v1[2..].chars().all(|c| c.is_ascii_digit()) {
            return Err(XmlError::SyntaxError {
                message: format!("Invalid XML version format '{v1}'"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let version = v1;
        let mut encoding = None;
        let mut standalone = None;

        while !self.source.is_eof() && !self.source.starts_with("?>") {
            let ws_attr = self.source.skip_whitespace();
            if self.source.starts_with("?>") {
                break;
            }
            if ws_attr == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required before attribute in XML declaration".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }

            let (key, val) = self.parse_attribute()?;
            match key.as_str() {
                "encoding" => {
                    if standalone.is_some() {
                        return Err(XmlError::SyntaxError {
                            message: "'encoding' declaration must precede 'standalone' in XML declaration".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    if encoding.is_some() {
                        return Err(XmlError::SyntaxError {
                            message: "Duplicate 'encoding' in XML declaration".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    // Validate EncName: [A-Za-z] ([A-Za-z0-9._] | '-')*
                    let mut chars = val.chars();
                    let valid = match chars.next() {
                        Some(f) => f.is_ascii_alphabetic() && chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'),
                        None => false,
                    };
                    if !valid {
                        return Err(XmlError::SyntaxError {
                            message: format!("Invalid encoding name '{val}'"),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    encoding = Some(val);
                }
                "standalone" => {
                    if standalone.is_some() {
                        return Err(XmlError::SyntaxError {
                            message: "Duplicate 'standalone' in XML declaration".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    // XML 1.0 §2.9 [32]: standalone value must be strictly "yes" or "no" (lowercase)
                    if val != "yes" && val != "no" {
                        return Err(XmlError::SyntaxError {
                            message: format!("Invalid standalone value '{val}', must be 'yes' or 'no'"),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    standalone = Some(val == "yes");
                }
                other => {
                    return Err(XmlError::SyntaxError {
                        message: format!("Unexpected attribute '{other}' in XML declaration"),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            }
        }

        self.source.skip_whitespace();
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
        self.namespace_scope.push_scope();
        let res = self.parse_element_internal(doc, parent_id, depth);
        self.namespace_scope.pop_scope();
        res
    }

    fn parse_element_internal(&mut self, doc: &mut Document, parent_id: NodeId, depth: usize) -> Result<()> {
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

            // Register namespaces declared on this element
            if self.options.namespace_aware {
                if attr_name == "xmlns" {
                    self.namespace_scope.declare(None, attr_value.trim());
                } else if let Some(prefix) = attr_name.strip_prefix("xmlns:") {
                    self.namespace_scope.declare(Some(prefix), attr_value.trim());
                }
            }

            attributes.push(Attribute::new(attr_name, attr_value));

            self.total_attribute_count += 1;
            self.options.check_attribute_count(attributes.len())?;
            self.options.check_total_attribute_count(self.total_attribute_count)?;

            let ws = self.source.skip_whitespace();
            if self.source.starts_with(">") || self.source.starts_with("/>") {
                break;
            }
            if ws == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required between attributes in element tag".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }

        // Validate namespace prefixes and collisions
        if self.options.namespace_aware {
            // Check element prefix
            let (elem_pfx, _) = QName::split_prefix(&name);
            if let Some(pfx) = elem_pfx {
                if self.namespace_scope.resolve_prefix(Some(pfx)).is_none() {
                    return Err(XmlError::SyntaxError {
                        message: format!("Undeclared element namespace prefix '{pfx}'"),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            }

            // Check attribute prefixes and attribute namespace collisions
            let mut resolved_attrs: Vec<(Option<String>, String)> = Vec::new();
            for attr in &attributes {
                let (attr_pfx, local) = QName::split_prefix(&attr.name);
                let uri_opt = if let Some(pfx) = attr_pfx {
                    if pfx == "xmlns" {
                        None
                    } else {
                        match self.namespace_scope.resolve_prefix(Some(pfx)) {
                            Some(u) => Some(u.to_string()),
                            None => {
                                return Err(XmlError::SyntaxError {
                                    message: format!("Undeclared attribute namespace prefix '{pfx}'"),
                                    line: self.source.line(),
                                    col: self.source.col(),
                                });
                            }
                        }
                    }
                } else {
                    None
                };

                if let Some(uri) = &uri_opt {
                    if resolved_attrs.iter().any(|(u, l)| u.as_deref() == Some(uri.as_str()) && l == local) {
                        return Err(XmlError::SyntaxError {
                            message: format!("Attribute collision: multiple attributes with namespace '{uri}' and local name '{local}'"),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                }
                resolved_attrs.push((uri_opt, local.to_string()));
            }
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
            if ch == '<' {
                return Err(XmlError::SyntaxError {
                    message: format!("Unescaped '<' is forbidden in attribute value for '{key}'"),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            if !is_valid_xml_char(ch) {
                return Err(XmlError::SyntaxError {
                    message: format!("Forbidden XML character '\\u{{{:x}}}' in attribute value", ch as u32),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
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
                    if expanded_val == "http://www.w3.org/XML/1998/namespace" {
                        return Err(XmlError::SyntaxError {
                            message: "The xml namespace must not be bound to any other prefix".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
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

        if raw_text.contains("]]>") {
            return Err(XmlError::SyntaxError {
                message: "The sequence ']]>' is forbidden in character data".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
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
    fn parse_comment(&mut self, doc: &mut Document) -> Result<NodeId> {
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

    /// Parses processing instruction (`<?target data?>`).
    fn parse_pi(&mut self, doc: &mut Document) -> Result<NodeId> {
        self.source.consume("<?");
        let target = self.parse_name()?;
        if target.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Processing instruction target cannot be empty".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if target.eq_ignore_ascii_case("xml") {
            return Err(XmlError::SyntaxError {
                message: "Processing instruction target cannot be 'xml'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.options.namespace_aware && target.contains(':') {
            return Err(XmlError::SyntaxError {
                message: format!("Processing instruction target '{target}' cannot contain a colon"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.source.starts_with("?>") {
            self.source.consume("?>");
            return Ok(doc.add_node(NodeKind::ProcessingInstruction {
                target: target.into_boxed_str(),
                data: "".into(),
            }));
        }

        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required between processing instruction target and data".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

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
            if !is_valid_xml_char(ch) {
                return Err(XmlError::SyntaxError {
                    message: format!("Forbidden XML character '\\u{{{:x}}}' in processing instruction", ch as u32),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
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
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after 'PUBLIC'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            public_id = Some(self.parse_pubid_literal()?);
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required between Public ID and System ID".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            system_id = Some(self.parse_quoted_string()?);
        } else if self.source.consume("SYSTEM") {
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after 'SYSTEM'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            system_id = Some(self.parse_quoted_string()?);
        }

        self.source.skip_whitespace();
        if self.source.consume("[") {
            let subset = self.parse_internal_subset()?;
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
                    if self.options.namespace_aware && ent_name.contains(':') {
                        return Err(XmlError::SyntaxError {
                            message: format!("Entity name '{ent_name}' cannot contain a colon in namespace-aware XML"),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
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
            } else if trimmed.starts_with("<!NOTATION") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let not_name = parts[1];
                    if self.options.namespace_aware && not_name.contains(':') {
                        return Err(XmlError::SyntaxError {
                            message: format!("Notation name '{not_name}' cannot contain a colon in namespace-aware XML"),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
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

    fn check_qname(&self, name: &str) -> Result<()> {
        if !self.options.namespace_aware {
            return Ok(());
        }
        let colons = name.matches(':').count();
        if colons > 1 {
            return Err(XmlError::SyntaxError {
                message: format!("QName '{name}' cannot contain multiple colons"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if name.starts_with(':') || name.ends_with(':') {
            return Err(XmlError::SyntaxError {
                message: format!("QName '{name}' cannot start or end with a colon"),
                line: self.source.line(),
                col: self.source.col(),
            });
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
        let name = self.source.slice_range(start, end).to_string();
        if !name.is_empty() {
            self.check_qname(&name)?;
        }
        Ok(name)
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

    fn is_pubid_char(ch: char) -> bool {
        matches!(ch, ' ' | '\r' | '\n' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '\'' | '(' | ')' | '+' | ',' | '.' | '/' | ':' | '=' | '?' | ';' | '!' | '*' | '#' | '@' | '$' | '_' | '%')
    }

    fn parse_pubid_literal(&mut self) -> Result<String> {
        let quote = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
            message: "Expected quote for PubidLiteral".into(),
            line: self.source.line(),
            col: self.source.col(),
        })?;
        if quote != '"' && quote != '\'' {
            return Err(XmlError::SyntaxError {
                message: "PubidLiteral must start with single or double quote".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let mut s = String::new();
        while let Some(ch) = self.source.next_char() {
            if ch == quote {
                return Ok(s);
            }
            if !Self::is_pubid_char(ch) || (quote == '\'' && ch == '\'') {
                return Err(XmlError::SyntaxError {
                    message: format!("Illegal character in PubidLiteral: '{}'", ch),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            s.push(ch);
        }
        Err(XmlError::SyntaxError {
            message: "Unterminated PubidLiteral".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }

    fn parse_nmtoken(&mut self) -> Result<String> {
        let start = self.source.position();
        while let Some(ch) = self.source.peek() {
            if is_xml_name_char(ch) {
                self.source.next_char();
            } else {
                break;
            }
        }
        let end = self.source.position();
        if start == end {
            return Err(XmlError::SyntaxError {
                message: "Expected Nmtoken".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        Ok(self.source.slice_range(start, end).to_string())
    }

    fn parse_entity_value(&mut self) -> Result<String> {
        let quote = self.source.next_char().ok_or_else(|| XmlError::SyntaxError {
            message: "Expected quote for EntityValue".into(),
            line: self.source.line(),
            col: self.source.col(),
        })?;
        if quote != '"' && quote != '\'' {
            return Err(XmlError::SyntaxError {
                message: "EntityValue must be quoted".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let mut s = String::new();
        while let Some(ch) = self.source.peek() {
            if ch == quote {
                self.source.next_char();
                return Ok(s);
            }
            if ch == '%' {
                self.source.next_char();
                s.push('%');
                let name = self.parse_name()?;
                if name.is_empty() {
                    return Err(XmlError::SyntaxError {
                        message: "Expected Name in PEReference inside EntityValue".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                s.push_str(&name);
                if !self.source.consume(";") {
                    return Err(XmlError::SyntaxError {
                        message: "Expected ';' terminating PEReference inside EntityValue".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                s.push(';');
            } else if ch == '&' {
                self.source.next_char();
                s.push('&');
                if self.source.consume("#") {
                    s.push('#');
                    let hex = self.source.consume("x");
                    if hex {
                        s.push('x');
                    }
                    let mut num_str = String::new();
                    while let Some(nc) = self.source.peek() {
                        if (hex && nc.is_ascii_hexdigit()) || (!hex && nc.is_ascii_digit()) {
                            num_str.push(nc);
                            s.push(nc);
                            self.source.next_char();
                        } else {
                            break;
                        }
                    }
                    if num_str.is_empty() {
                        return Err(XmlError::SyntaxError {
                            message: "Expected digits in numeric character reference".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    if !self.source.consume(";") {
                        return Err(XmlError::SyntaxError {
                            message: "Expected ';' terminating numeric character reference".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    s.push(';');
                } else {
                    let name = self.parse_name()?;
                    if name.is_empty() {
                        return Err(XmlError::SyntaxError {
                            message: "Expected Name in entity reference inside EntityValue".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    s.push_str(&name);
                    if !self.source.consume(";") {
                        return Err(XmlError::SyntaxError {
                            message: "Expected ';' terminating entity reference inside EntityValue".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    s.push(';');
                }
            } else {
                self.source.next_char();
                s.push(ch);
            }
        }
        Err(XmlError::SyntaxError {
            message: "Unterminated EntityValue".into(),
            line: self.source.line(),
            col: self.source.col(),
        })
    }

    fn parse_mixed(&mut self) -> Result<()> {
        let mut count = 0;
        loop {
            self.source.skip_whitespace();
            if self.source.consume("|") {
                count += 1;
                self.source.skip_whitespace();
                let name = self.parse_name()?;
                if name.is_empty() {
                    return Err(XmlError::SyntaxError {
                        message: "Expected element name after '|' in Mixed content".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            } else {
                break;
            }
        }
        self.source.skip_whitespace();
        if !self.source.consume(")") {
            return Err(XmlError::SyntaxError {
                message: "Expected ')' terminating Mixed content".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if count > 0 {
            if !self.source.consume("*") {
                return Err(XmlError::SyntaxError {
                    message: "Mixed content with element names must end with ')*'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        } else {
            let _ = self.source.consume("*");
        }
        if let Some(ch) = self.source.peek() {
            if ch == '?' || ch == '+' || ch == '*' {
                return Err(XmlError::SyntaxError {
                    message: "Illegal quantifier on Mixed content".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }
        Ok(())
    }

    fn parse_cp(&mut self) -> Result<()> {
        self.source.skip_whitespace();
        if self.source.peek() == Some('(') {
            self.parse_children_group()?;
        } else {
            let name = self.parse_name()?;
            if name.is_empty() {
                return Err(XmlError::SyntaxError {
                    message: "Expected element name in content particle".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }
        if let Some(ch) = self.source.peek() {
            if ch == '?' || ch == '*' || ch == '+' {
                self.source.next_char();
            }
        }
        Ok(())
    }

    fn parse_children_group(&mut self) -> Result<()> {
        if !self.source.consume("(") {
            return Err(XmlError::SyntaxError {
                message: "Expected '(' starting children group".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        self.source.skip_whitespace();
        if self.source.peek() == Some(')') {
            return Err(XmlError::SyntaxError {
                message: "Empty children group '()' is not allowed".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        self.parse_cp()?;

        let mut sep_choice = false;
        let mut sep_seq = false;

        loop {
            self.source.skip_whitespace();
            if self.source.consume("|") {
                if sep_seq {
                    return Err(XmlError::SyntaxError {
                        message: "Cannot mix '|' and ',' in the same content group".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                sep_choice = true;
                self.source.skip_whitespace();
                self.parse_cp()?;
            } else if self.source.consume(",") {
                if sep_choice {
                    return Err(XmlError::SyntaxError {
                        message: "Cannot mix '|' and ',' in the same content group".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                sep_seq = true;
                self.source.skip_whitespace();
                self.parse_cp()?;
            } else if self.source.consume(")") {
                break;
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Expected '|', ',', or ')' in children group".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }

        if let Some(ch) = self.source.peek() {
            if ch == '?' || ch == '*' || ch == '+' {
                self.source.next_char();
            }
        }
        Ok(())
    }

    fn parse_element_contentspec(&mut self) -> Result<()> {
        if self.source.starts_with("EMPTY") {
            let next = self.source.peek_offset(5);
            if next.map_or(true, |c| !is_xml_name_char(c)) {
                self.source.consume("EMPTY");
                return Ok(());
            }
        }
        if self.source.starts_with("ANY") {
            let next = self.source.peek_offset(3);
            if next.map_or(true, |c| !is_xml_name_char(c)) {
                self.source.consume("ANY");
                return Ok(());
            }
        }
        if self.source.peek() == Some('(') {
            let mut offset = 1;
            while let Some(ch) = self.source.peek_offset(offset) {
                if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                    offset += 1;
                } else {
                    break;
                }
            }
            let is_pcdata = self.source.peek_offset(offset) == Some('#')
                && self.source.peek_offset(offset + 1) == Some('P')
                && self.source.peek_offset(offset + 2) == Some('C')
                && self.source.peek_offset(offset + 3) == Some('D')
                && self.source.peek_offset(offset + 4) == Some('A')
                && self.source.peek_offset(offset + 5) == Some('T')
                && self.source.peek_offset(offset + 6) == Some('A');
            if is_pcdata {
                self.source.consume("(");
                self.source.skip_whitespace();
                self.source.consume("#PCDATA");
                self.parse_mixed()
            } else {
                self.parse_children_group()
            }
        } else {
            Err(XmlError::SyntaxError {
                message: "Invalid element contentspec, expected EMPTY, ANY, or parenthesized model".into(),
                line: self.source.line(),
                col: self.source.col(),
            })
        }
    }

    fn parse_element_decl(&mut self, _subset: &mut String) -> Result<()> {
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required after '<!ELEMENT'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let name = self.parse_name()?;
        if name.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Expected element name in element declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required between element name and contentspec".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        self.parse_element_contentspec()?;
        self.source.skip_whitespace();
        if !self.source.consume(">") {
            return Err(XmlError::SyntaxError {
                message: "Unclosed ELEMENT declaration, expected '>'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        Ok(())
    }

    fn parse_attlist_decl(&mut self, _subset: &mut String) -> Result<()> {
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required after '<!ATTLIST'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let elem_name = self.parse_name()?;
        if elem_name.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Expected element name in ATTLIST declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        loop {
            let ws = self.source.skip_whitespace();
            if self.source.consume(">") {
                return Ok(());
            }
            if ws == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required before attribute definition".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let att_name = self.parse_name()?;
            if att_name.is_empty() {
                return Err(XmlError::SyntaxError {
                    message: "Expected attribute name in ATTLIST declaration".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after attribute name".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }

            if self.source.consume("CDATA") {
                // ok
            } else if self.source.consume("IDREFS") {
                // ok
            } else if self.source.consume("IDREF") {
                // ok
            } else if self.source.consume("ID") {
                // ok
            } else if self.source.consume("ENTITIES") {
                // ok
            } else if self.source.consume("ENTITY") {
                // ok
            } else if self.source.consume("NMTOKENS") {
                // ok
            } else if self.source.consume("NMTOKEN") {
                // ok
            } else if self.source.consume("NOTATION") {
                if self.source.skip_whitespace() == 0 {
                    return Err(XmlError::SyntaxError {
                        message: "Whitespace required after 'NOTATION'".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                if !self.source.consume("(") {
                    return Err(XmlError::SyntaxError {
                        message: "Expected '(' after NOTATION".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                self.source.skip_whitespace();
                let first_not = self.parse_name()?;
                if first_not.is_empty() {
                    return Err(XmlError::SyntaxError {
                        message: "Expected notation name in NOTATION enumeration".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                loop {
                    self.source.skip_whitespace();
                    if self.source.consume("|") {
                        self.source.skip_whitespace();
                        let not = self.parse_name()?;
                        if not.is_empty() {
                            return Err(XmlError::SyntaxError {
                                message: "Expected notation name after '|'".into(),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else if self.source.consume(")") {
                        break;
                    } else {
                        return Err(XmlError::SyntaxError {
                            message: "Expected '|' or ')' in NOTATION enumeration".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                }
            } else if self.source.peek() == Some('(') {
                self.source.next_char();
                self.source.skip_whitespace();
                let first_tok = self.parse_nmtoken()?;
                if first_tok.is_empty() {
                    return Err(XmlError::SyntaxError {
                        message: "Expected Nmtoken in enumeration".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                loop {
                    self.source.skip_whitespace();
                    if self.source.consume("|") {
                        self.source.skip_whitespace();
                        let tok = self.parse_nmtoken()?;
                        if tok.is_empty() {
                            return Err(XmlError::SyntaxError {
                                message: "Expected Nmtoken after '|' in enumeration".into(),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else if self.source.consume(")") {
                        break;
                    } else {
                        return Err(XmlError::SyntaxError {
                            message: "Expected '|' or ')' in enumeration".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                }
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Invalid attribute type in ATTLIST declaration".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }

            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required before DefaultDecl in ATTLIST".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            if self.source.consume("#REQUIRED") {
                // ok
            } else if self.source.consume("#IMPLIED") {
                // ok
            } else if self.source.consume("#FIXED") {
                if self.source.skip_whitespace() == 0 {
                    return Err(XmlError::SyntaxError {
                        message: "Whitespace required after '#FIXED'".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                let _val = self.parse_quoted_string()?;
            } else if let Some(ch) = self.source.peek() {
                if ch == '"' || ch == '\'' {
                    let _val = self.parse_quoted_string()?;
                } else {
                    return Err(XmlError::SyntaxError {
                        message: "Invalid DefaultDecl in ATTLIST declaration".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Unexpected end of input in ATTLIST declaration".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }
    }

    fn parse_entity_decl(&mut self, subset: &mut String) -> Result<()> {
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required after '<!ENTITY'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let is_param = if self.source.consume("%") {
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after '%' in parameter entity decl".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            true
        } else {
            false
        };
        let name = self.parse_name()?;
        if name.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Expected entity name in entity declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required after entity name".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        if self.source.consume("SYSTEM") {
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after 'SYSTEM'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let _sys_lit = self.parse_quoted_string()?;
            if !is_param {
                let ws = self.source.skip_whitespace();
                if self.source.consume("NDATA") {
                    if ws == 0 {
                        return Err(XmlError::SyntaxError {
                            message: "Whitespace required before 'NDATA'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    if self.source.skip_whitespace() == 0 {
                        return Err(XmlError::SyntaxError {
                            message: "Whitespace required after 'NDATA'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    let ndata_name = self.parse_name()?;
                    if ndata_name.is_empty() {
                        return Err(XmlError::SyntaxError {
                            message: "Expected notation name after 'NDATA'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                } else if self.source.consume("ndata") {
                    return Err(XmlError::SyntaxError {
                        message: "'NDATA' must be uppercase".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            } else {
                self.source.skip_whitespace();
                if self.source.consume("NDATA") || self.source.consume("ndata") {
                    return Err(XmlError::SyntaxError {
                        message: "Parameter entities cannot have NDataDecl".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            }
        } else if self.source.consume("PUBLIC") {
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after 'PUBLIC'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let _pub_lit = self.parse_pubid_literal()?;
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required between Public ID and System ID".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let _sys_lit = self.parse_quoted_string()?;
            if !is_param {
                let ws = self.source.skip_whitespace();
                if self.source.consume("NDATA") {
                    if ws == 0 {
                        return Err(XmlError::SyntaxError {
                            message: "Whitespace required before 'NDATA'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    if self.source.skip_whitespace() == 0 {
                        return Err(XmlError::SyntaxError {
                            message: "Whitespace required after 'NDATA'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    let ndata_name = self.parse_name()?;
                    if ndata_name.is_empty() {
                        return Err(XmlError::SyntaxError {
                            message: "Expected notation name after 'NDATA'".into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                } else if self.source.consume("ndata") {
                    return Err(XmlError::SyntaxError {
                        message: "'NDATA' must be uppercase".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            } else {
                self.source.skip_whitespace();
                if self.source.consume("NDATA") || self.source.consume("ndata") {
                    return Err(XmlError::SyntaxError {
                        message: "Parameter entities cannot have NDataDecl".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            }
        } else if let Some(ch) = self.source.peek() {
            if ch == '"' || ch == '\'' {
                let val = self.parse_entity_value()?;
                if !is_param {
                    subset.push_str(&format!("<!ENTITY {} \"{}\">", name, val.replace('"', "&quot;")));
                }
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Expected EntityValue, SYSTEM, or PUBLIC in entity declaration".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        } else {
            return Err(XmlError::SyntaxError {
                message: "Unexpected end of input in entity declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        self.source.skip_whitespace();
        if !self.source.consume(">") {
            return Err(XmlError::SyntaxError {
                message: "Unclosed ENTITY declaration, expected '>'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        Ok(())
    }

    fn parse_notation_decl(&mut self, _subset: &mut String) -> Result<()> {
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required after '<!NOTATION'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        let name = self.parse_name()?;
        if name.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Expected notation name".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.source.skip_whitespace() == 0 {
            return Err(XmlError::SyntaxError {
                message: "Whitespace required after notation name".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.source.consume("SYSTEM") {
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after 'SYSTEM'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let _sys = self.parse_quoted_string()?;
        } else if self.source.consume("PUBLIC") {
            if self.source.skip_whitespace() == 0 {
                return Err(XmlError::SyntaxError {
                    message: "Whitespace required after 'PUBLIC'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
            let _pub = self.parse_pubid_literal()?;
            if self.source.skip_whitespace() > 0 {
                if let Some(ch) = self.source.peek() {
                    if ch == '"' || ch == '\'' {
                        let _sys = self.parse_quoted_string()?;
                    }
                }
            }
        } else {
            return Err(XmlError::SyntaxError {
                message: "Expected SYSTEM or PUBLIC in notation declaration".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        self.source.skip_whitespace();
        if !self.source.consume(">") {
            return Err(XmlError::SyntaxError {
                message: "Unclosed NOTATION declaration, expected '>'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        Ok(())
    }

    fn parse_internal_subset(&mut self) -> Result<String> {
        let start = self.source.position();
        let mut subset = String::new();
        loop {
            self.source.skip_whitespace();
            if self.source.peek() == Some(']') {
                let end = self.source.position();
                self.source.next_char();
                return Ok(self.source.slice_range(start, end).to_string());
            }
            if self.source.starts_with("<!--") {
                let mut dummy_doc = Document::new();
                self.parse_comment(&mut dummy_doc)?;
            } else if self.source.starts_with("<?") {
                let mut dummy_doc = Document::new();
                self.parse_pi(&mut dummy_doc)?;
            } else if self.source.starts_with("<!ELEMENT") {
                self.source.consume("<!ELEMENT");
                self.parse_element_decl(&mut subset)?;
            } else if self.source.starts_with("<!ATTLIST") {
                self.source.consume("<!ATTLIST");
                self.parse_attlist_decl(&mut subset)?;
            } else if self.source.starts_with("<!ENTITY") {
                self.source.consume("<!ENTITY");
                self.parse_entity_decl(&mut subset)?;
            } else if self.source.starts_with("<!NOTATION") {
                self.source.consume("<!NOTATION");
                self.parse_notation_decl(&mut subset)?;
            } else if self.source.starts_with("%") {
                self.source.next_char();
                let name = self.parse_name()?;
                if name.is_empty() {
                    return Err(XmlError::SyntaxError {
                        message: "Expected Name in parameter entity reference".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                if !self.source.consume(";") {
                    return Err(XmlError::SyntaxError {
                        message: "Expected ';' in parameter entity reference".into(),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            } else if self.source.starts_with("<!") {
                return Err(XmlError::SyntaxError {
                    message: "Invalid markup declaration in internal subset".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            } else if self.source.is_eof() {
                return Err(XmlError::SyntaxError {
                    message: "Unclosed internal DTD subset, expected ']'".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Unexpected character in internal DTD subset".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }
    }
}
