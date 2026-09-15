//! XML prolog, epilog, declaration, processing instruction, and DOCTYPE parsing.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::io::is_valid_xml_char;
use crate::node::{NodeId, NodeKind};

use super::XmlParser;

impl<'a> XmlParser<'a> {
    /// Parses XML declaration (`<?xml version="..." encoding="..."?>`).
    pub(crate) fn parse_declaration(&mut self, doc: &mut Document) -> Result<()> {
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

        self.standalone = standalone;
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
    pub(crate) fn parse_prolog(&mut self, doc: &mut Document) -> Result<()> {
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
    pub(crate) fn parse_epilog(&mut self, doc: &mut Document) -> Result<()> {
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

    /// Parses processing instruction (`<?target data?>`).
    pub(crate) fn parse_pi(&mut self, doc: &mut Document) -> Result<NodeId> {
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
    pub(crate) fn parse_doctype(&mut self, doc: &mut Document) -> Result<NodeId> {
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
        // In a standalone document or one without external subset or parameter entities,
        // undeclared entity references are strictly well-formedness errors.
        let is_standalone = self.standalone == Some(true);
        if !is_standalone
            && (system_id.is_some()
                || public_id.is_some()
                || internal_subset.as_ref().map(|s| s.contains('%')).unwrap_or(false))
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

    pub(crate) fn is_pubid_char(ch: char) -> bool {
        matches!(ch, ' ' | '\r' | '\n' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '\'' | '(' | ')' | '+' | ',' | '.' | '/' | ':' | '=' | '?' | ';' | '!' | '*' | '#' | '@' | '$' | '_' | '%')
    }

    pub(crate) fn parse_pubid_literal(&mut self) -> Result<String> {
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
}
