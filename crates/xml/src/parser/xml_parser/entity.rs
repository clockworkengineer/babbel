//! DTD entity declaration and registration implementation.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};
use crate::io::is_valid_xml_char;

use super::XmlParser;

impl<'a> XmlParser<'a> {
    /// Helper registering entity declarations from a DTD text subset.
    pub(crate) fn register_entities_from_text(&mut self, text: &str) -> Result<()> {
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
                            message: format!(
                                "Entity name '{ent_name}' cannot contain a colon in namespace-aware XML"
                            ),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    let is_external = trimmed.contains("SYSTEM") || trimmed.contains("PUBLIC");
                    if is_external {
                        external_entities.push(ent_name.to_string());
                        self.external_entities.push(ent_name.to_string());
                    }
                    if trimmed.contains("NDATA") {
                        self.unparsed_entities.push(ent_name.to_string());
                    }
                    if is_external && !self.options.allow_external_entities {
                        return Err(XmlError::SecurityLimitExceeded(
                            "External entity references in DOCTYPE are forbidden by security policy".into(),
                        ));
                    }
                    if is_external
                        && self.options.allow_external_entities
                        && trimmed.contains("SYSTEM")
                        && !trimmed.contains("NDATA")
                    {
                        let raw_val = parts[val_idx..].join(" ");
                        let sys_val = raw_val
                            .trim_matches(|c| c == '"' || c == '\'' || c == '>' || c == ';')
                            .trim();
                        let file_name = sys_val
                            .strip_prefix("SYSTEM")
                            .unwrap_or(sys_val)
                            .trim()
                            .trim_matches(|c| c == '"' || c == '\'');
                        #[cfg(feature = "std")]
                        {
                            let file_path = if let Some(base) = &self.options.base_dir {
                                std::path::Path::new(base).join(file_name)
                            } else {
                                std::path::PathBuf::from(file_name)
                            };
                            if let Ok(bytes) = std::fs::read(&file_path) {
                                if let Ok((loaded_text, _)) =
                                    babbel_core::encoding::detect_encoding_and_strip_bom(&bytes)
                                {
                                    // XML 1.0 §4.3.4: An XML 1.0 document cannot include external entity with version 1.1
                                    if loaded_text.contains("<?xml")
                                        && (loaded_text.contains("version=\"1.1\"")
                                            || loaded_text.contains("version='1.1'"))
                                    {
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
                            message: format!(
                                "Notation name '{not_name}' cannot contain a colon in namespace-aware XML"
                            ),
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
                            message: format!(
                                "Attribute values must not contain references to external entity '&{ext};'"
                            ),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn parse_entity_value(&mut self) -> Result<String> {
        let quote = self
            .source
            .next_char()
            .ok_or_else(|| XmlError::SyntaxError {
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
                return Err(XmlError::SyntaxError {
                    message: "Parameter-entity references must not occur within markup declarations in the internal DTD subset (WFC: In Subset)".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            } else if ch == '&' {
                self.source.next_char();
                if self.source.consume("#") {
                    let hex = self.source.consume("x");
                    let mut num_str = String::new();
                    while let Some(nc) = self.source.peek() {
                        if (hex && nc.is_ascii_hexdigit()) || (!hex && nc.is_ascii_digit()) {
                            num_str.push(nc);
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
                    let codepoint = if hex {
                        u32::from_str_radix(&num_str, 16)
                    } else {
                        num_str.parse::<u32>()
                    };
                    match codepoint {
                        Ok(cp) => {
                            if let Some(c) = char::from_u32(cp) {
                                if !is_valid_xml_char(c) {
                                    return Err(XmlError::SyntaxError {
                                        message: format!(
                                            "Forbidden XML character '\\u{{{:x}}}' in character reference",
                                            cp
                                        ),
                                        line: self.source.line(),
                                        col: self.source.col(),
                                    });
                                }
                                s.push(c);
                            } else {
                                return Err(XmlError::SyntaxError {
                                    message: format!(
                                        "Invalid code point {cp} in character reference"
                                    ),
                                    line: self.source.line(),
                                    col: self.source.col(),
                                });
                            }
                        }
                        Err(_) => {
                            return Err(XmlError::SyntaxError {
                                message: "Malformed numeric character reference".into(),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    }
                } else {
                    s.push('&');
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
                            message: "Expected ';' terminating entity reference inside EntityValue"
                                .into(),
                            line: self.source.line(),
                            col: self.source.col(),
                        });
                    }
                    s.push(';');
                }
            } else {
                if !is_valid_xml_char(ch) {
                    return Err(XmlError::SyntaxError {
                        message: format!(
                            "Forbidden XML character '\\u{{{:x}}}' in EntityValue",
                            ch as u32
                        ),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
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

    pub(crate) fn parse_entity_decl(&mut self, subset: &mut String) -> Result<()> {
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
            self.external_entities.push(name.clone());
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
                    self.unparsed_entities.push(name.clone());
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
            self.external_entities.push(name.clone());
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
                    self.unparsed_entities.push(name.clone());
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
                    self.entity_mapper.register(&name, &val);
                    subset.push_str(&format!(
                        "<!ENTITY {} \"{}\">",
                        name,
                        val.replace('"', "&quot;")
                    ));
                } else {
                    subset.push_str(&format!(
                        "<!ENTITY % {} \"{}\">",
                        name,
                        val.replace('"', "&quot;")
                    ));
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
}
