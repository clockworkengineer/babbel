//! DTD subset, element, attribute list, and notation declaration parsing.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::io::is_xml_name_char;

use super::XmlParser;

impl<'a> XmlParser<'a> {
    pub(crate) fn parse_nmtoken(&mut self) -> Result<String> {
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

    pub(crate) fn parse_mixed(&mut self) -> Result<()> {
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

    pub(crate) fn parse_cp(&mut self) -> Result<()> {
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

    pub(crate) fn parse_children_group(&mut self) -> Result<()> {
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

    pub(crate) fn parse_element_contentspec(&mut self) -> Result<()> {
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

    pub(crate) fn parse_element_decl(&mut self, _subset: &mut String) -> Result<()> {
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

    pub(crate) fn parse_attlist_decl(&mut self, _subset: &mut String) -> Result<()> {
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
                let val = self.parse_quoted_string()?;
                self.validate_attribute_default_value(&val)?;
            } else if let Some(ch) = self.source.peek() {
                if ch == '"' || ch == '\'' {
                    let val = self.parse_quoted_string()?;
                    self.validate_attribute_default_value(&val)?;
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

    pub(crate) fn parse_notation_decl(&mut self, _subset: &mut String) -> Result<()> {
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

    pub(crate) fn parse_internal_subset(&mut self) -> Result<String> {
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
