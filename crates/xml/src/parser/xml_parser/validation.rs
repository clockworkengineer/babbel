//! DTD and entity attribute validation rules.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};
use crate::io::{is_xml_name_char, is_xml_name_start};

use super::XmlParser;

impl<'a> XmlParser<'a> {
    pub(crate) fn check_attribute_entities(&self, text: &str) -> Result<()> {
        let mut pos = 0;
        let bytes = text.as_bytes();
        while pos < bytes.len() {
            if bytes[pos] == b'&' {
                if let Some(semi_offset) = text[pos..].find(';') {
                    let semi_idx = pos + semi_offset;
                    let ref_name = &text[pos + 1..semi_idx];
                    if !ref_name.starts_with('#') {
                        self.check_entity_for_attribute(ref_name, 0)?;
                    }
                    pos = semi_idx + 1;
                } else {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }
        Ok(())
    }

    pub(crate) fn check_entity_for_attribute(&self, name: &str, depth: usize) -> Result<()> {
        if depth > 100 {
            return Err(XmlError::SyntaxError {
                message: "Circular entity reference in attribute".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.unparsed_entities.iter().any(|u| u == name) {
            return Err(XmlError::SyntaxError {
                message: format!("Unparsed entity '{name}' cannot be referenced in attribute"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if self.external_entities.iter().any(|e| e == name) {
            return Err(XmlError::SyntaxError {
                message: format!(
                    "Attribute values must not contain references to external entity '&{name};'"
                ),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        if name != "lt" && name != "gt" && name != "amp" && name != "quot" && name != "apos" {
            if let Some(val) = self.entity_mapper.get(name) {
                if val.contains('<') {
                    return Err(XmlError::SyntaxError {
                        message: format!(
                            "Replacement text of entity '{name}' in attribute contains '<' (WFC: No < in Attribute Values)"
                        ),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
                let mut pos = 0;
                let bytes = val.as_bytes();
                while pos < bytes.len() {
                    if bytes[pos] == b'&' {
                        if let Some(semi_offset) = val[pos..].find(';') {
                            let semi_idx = pos + semi_offset;
                            let inner_ref = &val[pos + 1..semi_idx];
                            if !inner_ref.starts_with('#') {
                                self.check_entity_for_attribute(inner_ref, depth + 1)?;
                            }
                            pos = semi_idx + 1;
                        } else {
                            pos += 1;
                        }
                    } else {
                        pos += 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_attribute_default_value(&self, val: &str) -> Result<()> {
        if val.contains('<') {
            return Err(XmlError::SyntaxError {
                message: "Attribute default value cannot contain '<'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        self.check_attribute_entities(val)?;
        let has_pe = self.entity_mapper.allow_undeclared;
        if !has_pe {
            let mut pos = 0;
            let bytes = val.as_bytes();
            while pos < bytes.len() {
                if bytes[pos] == b'&' {
                    if let Some(semi_offset) = val[pos..].find(';') {
                        let semi_idx = pos + semi_offset;
                        let ref_name = &val[pos + 1..semi_idx];
                        if !ref_name.starts_with('#') {
                            match ref_name {
                                "lt" | "gt" | "amp" | "quot" | "apos" => {}
                                other => {
                                    if self.entity_mapper.get(other).is_none() {
                                        return Err(XmlError::SyntaxError {
                                            message: format!(
                                                "Entity '{other}' referenced in attribute default before it was declared"
                                            ),
                                            line: self.source.line(),
                                            col: self.source.col(),
                                        });
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
        }
        let expanded = self.entity_mapper.expand(val)?;
        if expanded.contains('<') {
            return Err(XmlError::SyntaxError {
                message: "Replacement text of entity in attribute default contains '<'".into(),
                line: self.source.line(),
                col: self.source.col(),
            });
        }
        Ok(())
    }

    pub(crate) fn validate_entity_replacement_text(&self, name: &str, val: &str) -> Result<()> {
        if val.contains("<?xml") {
            return Err(XmlError::SyntaxError {
                message: format!("'<?xml' forbidden in internal entity '{name}' replacement text"),
                line: self.source.line(),
                col: self.source.col(),
            });
        }

        let mut i = 0;
        let bytes = val.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'&' {
                if let Some(semi_pos) = val[i..].find(';') {
                    let ref_content = &val[i + 1..i + semi_pos];
                    if ref_content.starts_with('#') {
                        let code_str = &ref_content[1..];
                        let valid = if let Some(hex_digits) = code_str.strip_prefix('x') {
                            !hex_digits.is_empty()
                                && hex_digits.chars().all(|c| c.is_ascii_hexdigit())
                        } else {
                            !code_str.is_empty() && code_str.chars().all(|c| c.is_ascii_digit())
                        };
                        if !valid {
                            return Err(XmlError::SyntaxError {
                                message: format!(
                                    "Invalid numeric character reference in entity '{name}'"
                                ),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else {
                        let mut chars = ref_content.chars();
                        let valid = match chars.next() {
                            Some(first) => is_xml_name_start(first) && chars.all(is_xml_name_char),
                            None => false,
                        };
                        if !valid {
                            return Err(XmlError::SyntaxError {
                                message: format!(
                                    "Invalid entity reference name in entity '{name}'"
                                ),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    }
                    i += semi_pos + 1;
                } else {
                    return Err(XmlError::SyntaxError {
                        message: format!(
                            "Unclosed '&' in entity '{name}' replacement text (WFC: Parsed Entity)"
                        ),
                        line: self.source.line(),
                        col: self.source.col(),
                    });
                }
            } else {
                i += 1;
            }
        }

        if val.contains('<') {
            let mut pos = 0;
            let mut tag_stack: Vec<String> = Vec::new();
            while pos < bytes.len() {
                if bytes[pos] == b'<' {
                    let rest = &val[pos..];
                    if rest.starts_with("<!--") {
                        if let Some(end_comment) = rest.find("-->") {
                            pos += end_comment + 3;
                            continue;
                        } else {
                            return Err(XmlError::SyntaxError {
                                message: format!(
                                    "Unclosed comment in entity '{name}' (WFC: Parsed Entity)"
                                ),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else if rest.starts_with("<?") {
                        if let Some(end_pi) = rest.find("?>") {
                            pos += end_pi + 2;
                            continue;
                        } else {
                            return Err(XmlError::SyntaxError {
                                message: format!(
                                    "Unclosed PI in entity '{name}' (WFC: Parsed Entity)"
                                ),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else if rest.starts_with("<![CDATA[") {
                        if let Some(end_cdata) = rest.find("]]>") {
                            pos += end_cdata + 3;
                            continue;
                        } else {
                            return Err(XmlError::SyntaxError {
                                message: format!(
                                    "Unclosed CDATA in entity '{name}' (WFC: Parsed Entity)"
                                ),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else if rest.starts_with("</") {
                        if let Some(gt) = rest.find('>') {
                            let tag_name = rest[2..gt].trim();
                            if let Some(expected) = tag_stack.pop() {
                                if expected != tag_name {
                                    return Err(XmlError::SyntaxError {
                                        message: format!(
                                            "Mismatched end tag '</{tag_name}>' in entity '{name}', expected '</{expected}>'"
                                        ),
                                        line: self.source.line(),
                                        col: self.source.col(),
                                    });
                                }
                            } else {
                                return Err(XmlError::SyntaxError {
                                    message: format!(
                                        "End tag '</{tag_name}>' in entity '{name}' has no matching start tag (WFC: Parsed Entity)"
                                    ),
                                    line: self.source.line(),
                                    col: self.source.col(),
                                });
                            }
                            pos += gt + 1;
                            continue;
                        } else {
                            return Err(XmlError::SyntaxError {
                                message: format!("Unclosed end tag in entity '{name}'"),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    } else {
                        if let Some(gt) = rest.find('>') {
                            let is_self_closing = rest[..gt].ends_with('/');
                            let tag_body = if is_self_closing {
                                &rest[1..gt - 1]
                            } else {
                                &rest[1..gt]
                            };
                            let elem_tag = tag_body.split_whitespace().next().unwrap_or("");
                            if let Some(first_space) = tag_body.find(|c: char| c.is_whitespace()) {
                                let attrs_part = &tag_body[first_space..];
                                if attrs_part.contains('<') {
                                    return Err(XmlError::SyntaxError {
                                        message: format!(
                                            "Attribute value cannot contain '<' in entity '{name}'"
                                        ),
                                        line: self.source.line(),
                                        col: self.source.col(),
                                    });
                                }
                                let mut a_pos = 0;
                                let a_bytes = attrs_part.as_bytes();
                                while a_pos < a_bytes.len() {
                                    if a_bytes[a_pos] == b'&' {
                                        if let Some(semi) = attrs_part[a_pos..].find(';') {
                                            a_pos += semi + 1;
                                        } else {
                                            return Err(XmlError::SyntaxError {
                                                message: format!(
                                                    "Unescaped '&' in attribute in entity '{name}'"
                                                ),
                                                line: self.source.line(),
                                                col: self.source.col(),
                                            });
                                        }
                                    } else {
                                        a_pos += 1;
                                    }
                                }
                            }
                            if !is_self_closing && !elem_tag.is_empty() {
                                tag_stack.push(elem_tag.to_string());
                            }
                            pos += gt + 1;
                            continue;
                        } else {
                            return Err(XmlError::SyntaxError {
                                message: format!(
                                    "Unclosed start tag in entity '{name}' (WFC: Parsed Entity)"
                                ),
                                line: self.source.line(),
                                col: self.source.col(),
                            });
                        }
                    }
                }
                pos += 1;
            }
            if !tag_stack.is_empty() {
                return Err(XmlError::SyntaxError {
                    message: format!(
                        "Unclosed element in entity '{name}' replacement text (WFC: Parsed Entity)"
                    ),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }

        Ok(())
    }
}
