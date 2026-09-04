//! # Entity Mapper & XXE Security Guard
//!
//! Provides predefined entity mapping (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`), numeric reference decoding (`&#65;`, `&#x41;`), and recursive expansion limits.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Standard XML 1.0 predefined entity table mappings `(name, replacement)`.
pub const PREDEFINED_ENTITIES: &[(&str, &str)] = &[
    ("lt", "<"),
    ("gt", ">"),
    ("amp", "&"),
    ("quot", "\""),
    ("apos", "'"),
];

/// Entity table mapping entity reference names to replacement text with depth recursion and byte size tracking.
#[derive(Debug, Clone)]
pub struct EntityMapper {
    entities: HashMap<String, String>,
    max_depth: usize,
    max_expansion_size: usize,
}

impl Default for EntityMapper {
    fn default() -> Self {
        let mut map = HashMap::new();
        for &(name, value) in PREDEFINED_ENTITIES {
            map.insert(name.into(), value.into());
        }
        Self {
            entities: map,
            max_depth: 512,
            max_expansion_size: 10 * 1024 * 1024,
        }
    }
}

impl EntityMapper {
    /// Instantiates a new [`EntityMapper`] with a given max recursion depth limit.
    pub fn new(max_depth: usize) -> Self {
        let mut s = Self::default();
        s.max_depth = max_depth;
        s
    }

    /// Instantiates a new [`EntityMapper`] with custom recursion depth and total expansion size limits.
    pub fn with_limits(max_depth: usize, max_expansion_size: usize) -> Self {
        let mut s = Self::default();
        s.max_depth = max_depth;
        s.max_expansion_size = max_expansion_size;
        s
    }

    /// Registers a custom entity reference name and replacement value.
    pub fn register(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.entities.insert(name.into(), value.into());
    }

    /// Looks up an entity reference replacement value by name.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.entities.get(name)
    }

    /// Expands entity references and numeric character references within input string slice.
    ///
    /// # Errors
    /// Returns [`XmlError::SecurityLimitExceeded`] if expansion depth or total expansion size exceeds configured thresholds (XML bomb guard).
    pub fn expand(&self, input: &str) -> Result<String> {
        if !input.contains('&') {
            return Ok(input.to_string());
        }
        let mut total_expanded = 0;
        self.expand_with_depth(input, 0, &mut total_expanded)
    }

    fn expand_with_depth(&self, input: &str, depth: usize, total_expanded: &mut usize) -> Result<String> {
        if depth > self.max_depth {
            return Err(XmlError::SecurityLimitExceeded(
                "Maximum entity expansion depth exceeded (possible XML Bomb/Billion Laughs)".into(),
            ));
        }

        let mut result = String::with_capacity(input.len());
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            if bytes[pos] == b'&' {
                if let Some(semi_offset) = input[pos..].find(';') {
                    let semi_idx = pos + semi_offset;
                    let entity_ref = &input[pos + 1..semi_idx];

                    if entity_ref.starts_with('#') {
                        // Numeric reference (dec or hex)
                        let code_str = &entity_ref[1..];
                        let codepoint = if code_str.starts_with('x') || code_str.starts_with('X') {
                            u32::from_str_radix(&code_str[1..], 16)
                        } else {
                            code_str.parse::<u32>()
                        };

                        match codepoint {
                            Ok(cp) => {
                                if let Some(ch) = char::from_u32(cp) {
                                    *total_expanded += ch.len_utf8();
                                    if *total_expanded > self.max_expansion_size {
                                        return Err(XmlError::SecurityLimitExceeded(format!(
                                            "Maximum cumulative entity expansion size of {} bytes exceeded",
                                            self.max_expansion_size
                                        )));
                                    }
                                    result.push(ch);
                                } else {
                                    return Err(XmlError::EntityError(format!(
                                        "Invalid character reference code point: {code_str}"
                                    )));
                                }
                            }
                            Err(_) => {
                                return Err(XmlError::EntityError(format!(
                                    "Malformed numeric entity reference: &{entity_ref};"
                                )));
                            }
                        }
                    } else {
                        // Named reference
                        match entity_ref {
                            "lt" => { *total_expanded += 1; result.push('<'); }
                            "gt" => { *total_expanded += 1; result.push('>'); }
                            "amp" => { *total_expanded += 1; result.push('&'); }
                            "quot" => { *total_expanded += 1; result.push('"'); }
                            "apos" => { *total_expanded += 1; result.push('\''); }
                            _ => {
                                if let Some(val) = self.entities.get(entity_ref) {
                                    let expanded_val = self.expand_with_depth(val, depth + 1, total_expanded)?;
                                    result.push_str(&expanded_val);
                                } else {
                                    return Err(XmlError::EntityError(format!(
                                        "Undeclared entity reference: &{entity_ref};"
                                    )));
                                }
                            }
                        }
                        if *total_expanded > self.max_expansion_size {
                            return Err(XmlError::SecurityLimitExceeded(format!(
                                "Maximum cumulative entity expansion size of {} bytes exceeded",
                                self.max_expansion_size
                            )));
                        }
                    }

                    pos = semi_idx + 1;
                } else {
                    return Err(XmlError::EntityError(
                        "Unterminated entity reference missing ';'".into(),
                    ));
                }
            } else {
                let ch = input[pos..].chars().next().ok_or_else(|| {
                    XmlError::EntityError("Unexpected EOF reading character in entity expansion".into())
                })?;
                *total_expanded += ch.len_utf8();
                if *total_expanded > self.max_expansion_size {
                    return Err(XmlError::SecurityLimitExceeded(format!(
                        "Maximum cumulative entity expansion size of {} bytes exceeded",
                        self.max_expansion_size
                    )));
                }
                result.push(ch);
                pos += ch.len_utf8();
            }
        }

        Ok(result)
    }
}
