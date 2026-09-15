//! XML element and attribute parsing implementation.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::io::{is_valid_xml_char, is_xml_name_char, is_xml_name_start};
use crate::namespace::QName;
use crate::node::{Attribute, NodeId, NodeKind};

use super::XmlParser;

impl<'a> XmlParser<'a> {
    /// Parses a single XML element tag, its attributes, and child content recursively.
    pub(crate) fn parse_element(&mut self, doc: &mut Document, parent_id: NodeId, depth: usize) -> Result<()> {
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
    pub(crate) fn parse_attribute(&mut self) -> Result<(String, String)> {
        let key = self.parse_name()?;
        if key.is_empty() {
            return Err(XmlError::SyntaxError {
                message: "Attribute name cannot be empty".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
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
                self.check_attribute_entities(&raw_val)?;
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

    pub(crate) fn check_qname(&self, name: &str) -> Result<()> {
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
    pub(crate) fn parse_name(&mut self) -> Result<String> {
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
    pub(crate) fn parse_quoted_string(&mut self) -> Result<String> {
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
            if !is_valid_xml_char(ch) {
                return Err(XmlError::SyntaxError {
                    message: format!("Forbidden XML character '\\u{{{:x}}}' in string literal", ch as u32),
                    line: self.source.line(),
                    col: self.source.col(),
                });
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
