//! Configuration Text Engine (INI / Java .properties / .env) parser and emitter.
//!
//! Supports sectioned INI files (`[section]`), flat property/environment files,
//! customizable comment characters (`#`, `;`), key-value delimiters (`=`, `:`),
//! and automatic type inference into the universal [`Value`] model.

use crate::error::{BabbelError, ErrorCode};
use crate::io::traits::IDestination;
use crate::model::Value;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Configuration options for INI and property file parsing and emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IniOptions {
    /// Characters indicating line comments (default: `['#', ';']`).
    pub comment_chars: &'static [char],
    /// Delimiter characters separating keys from values (default: `['=', ':']`).
    pub delimiters: &'static [char],
    /// Whether to trim leading/trailing whitespace around keys and unquoted values (default: `true`).
    pub trim: bool,
    /// Whether to infer booleans, numbers, and nulls from unquoted values (default: `true`).
    pub infer_types: bool,
    /// Section name to use for top-level keys before any section header (default: `""` meaning flat/root level).
    pub default_section: &'static str,
    /// Primary delimiter to use when emitting key-value pairs (default: `"="`).
    pub emit_delimiter: &'static str,
}

impl Default for IniOptions {
    fn default() -> Self {
        Self {
            comment_chars: &['#', ';'],
            delimiters: &['=', ':'],
            trim: true,
            infer_types: true,
            default_section: "",
            emit_delimiter: " = ",
        }
    }
}

impl IniOptions {
    /// Options tailored for `.env` files (delimiters `=`, comments `#`, no sections).
    pub fn env() -> Self {
        Self {
            comment_chars: &['#'],
            delimiters: &['='],
            emit_delimiter: "=",
            ..Default::default()
        }
    }

    /// Options tailored for Java `.properties` files (delimiters `=`, `:`, comments `#`, `!`).
    pub fn properties() -> Self {
        Self {
            comment_chars: &['#', '!'],
            delimiters: &['=', ':'],
            emit_delimiter: "=",
            ..Default::default()
        }
    }
}

/// Parses an INI or properties document into a universal [`Value::Object`].
///
/// If section headers `[section]` are present, the result is an object where each section is a nested
/// `Value::Object`. Any keys defined before the first section header are placed at the root level
/// (or under `options.default_section` if specified).
/// If no section headers are present, a flat `Value::Object` of key-value pairs is returned.
///
/// # Examples
/// ```
/// use babbel_core::ini::{parse_ini, IniOptions};
///
/// let doc = "[server]\nhost = localhost\nport = 8080\nenabled = true\n";
/// let value = parse_ini(doc, &IniOptions::default()).unwrap();
/// assert!(value.as_object().is_some());
/// ```
pub fn parse_ini(input: &str, options: &IniOptions) -> Result<Value, BabbelError> {
    let mut root_entries: Vec<(String, Value)> = Vec::new();
    let mut sections: Vec<(String, Vec<(String, Value)>)> = Vec::new();
    let mut current_section: Option<String> = if options.default_section.is_empty() {
        None
    } else {
        Some(options.default_section.to_string())
    };

    for line in input.split('\n') {
        let line_clean = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = line_clean.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check for comment line
        if options.comment_chars.iter().any(|&c| trimmed.starts_with(c)) {
            continue;
        }

        // Check for section header [section]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section_name = trimmed[1..trimmed.len() - 1].trim().to_string();
            current_section = Some(section_name);
            continue;
        }

        // Parse key-value pair
        let delim_pos = find_first_delimiter(trimmed, options.delimiters);
        if let Some(pos) = delim_pos {
            let raw_key = &trimmed[..pos];
            let raw_val = &trimmed[pos + 1..];

            let key = if options.trim {
                raw_key.trim().to_string()
            } else {
                raw_key.to_string()
            };

            let val = parse_ini_value(raw_val, options);

            match &current_section {
                None => {
                    root_entries.push((key, val));
                }
                Some(sec) => {
                    if let Some((_, entries)) = sections.iter_mut().find(|(s, _)| s == sec) {
                        entries.push((key, val));
                    } else {
                        sections.push((sec.clone(), vec![(key, val)]));
                    }
                }
            }
        } else {
            // Key without value (boolean flag or error)
            let key = if options.trim {
                trimmed.to_string()
            } else {
                line_clean.to_string()
            };
            let val = Value::Bool(true);

            match &current_section {
                None => root_entries.push((key, val)),
                Some(sec) => {
                    if let Some((_, entries)) = sections.iter_mut().find(|(s, _)| s == sec) {
                        entries.push((key, val));
                    } else {
                        sections.push((sec.clone(), vec![(key, val)]));
                    }
                }
            }
        }
    }

    if sections.is_empty() {
        Ok(Value::Object(root_entries))
    } else {
        let mut final_obj = root_entries;
        for (sec_name, sec_entries) in sections {
            final_obj.push((sec_name, Value::Object(sec_entries)));
        }
        Ok(Value::Object(final_obj))
    }
}

fn find_first_delimiter(s: &str, delims: &[char]) -> Option<usize> {
    let mut earliest: Option<usize> = None;
    for &d in delims {
        if let Some(pos) = s.find(d) {
            earliest = Some(match earliest {
                Some(prev) => prev.min(pos),
                None => pos,
            });
        }
    }
    earliest
}

fn parse_ini_value(raw: &str, options: &IniOptions) -> Value {
    let trimmed = if options.trim { raw.trim() } else { raw };

    // Strip comments at end of unquoted line if any
    let clean = if !trimmed.starts_with('"') && !trimmed.starts_with('\'') {
        let mut end = trimmed.len();
        for &c in options.comment_chars {
            if let Some(pos) = trimmed.find(c) {
                end = end.min(pos);
            }
        }
        trimmed[..end].trim()
    } else {
        trimmed
    };

    // Check for quoted strings
    if (clean.starts_with('"') && clean.ends_with('"') && clean.len() >= 2)
        || (clean.starts_with('\'') && clean.ends_with('\'') && clean.len() >= 2)
    {
        return Value::String(clean[1..clean.len() - 1].to_string());
    }

    if !options.infer_types {
        return Value::String(clean.to_string());
    }

    if clean.is_empty() || clean.eq_ignore_ascii_case("null") || clean == "~" {
        return Value::Null;
    }

    if clean.eq_ignore_ascii_case("true") || clean.eq_ignore_ascii_case("yes") || clean.eq_ignore_ascii_case("on") {
        return Value::Bool(true);
    }
    if clean.eq_ignore_ascii_case("false") || clean.eq_ignore_ascii_case("no") || clean.eq_ignore_ascii_case("off") {
        return Value::Bool(false);
    }

    if let Ok(i) = clean.parse::<i128>() {
        return Value::Integer(i);
    }

    if (clean.contains('.') || clean.contains('e') || clean.contains('E'))
        && let Ok(f) = clean.parse::<f64>()
        && f.is_finite()
    {
        return Value::Float(f);
    }

    Value::String(clean.to_string())
}

/// Serializes a [`Value`] to an INI or properties document.
pub fn emit_ini(value: &Value, options: &IniOptions) -> Result<String, BabbelError> {
    let mut dest = crate::io::StringDestination::default();
    emit_ini_to(value, options, &mut dest)?;
    Ok(dest.into_string())
}

/// Serializes a [`Value`] to an INI destination.
pub fn emit_ini_to(
    value: &Value,
    options: &IniOptions,
    destination: &mut dyn IDestination,
) -> Result<(), BabbelError> {
    let Value::Object(fields) = value else {
        return Err(BabbelError::new(
            ErrorCode::UnsupportedType,
            "Top-level value for INI serialization must be an Object",
        ));
    };

    // First pass: write top-level scalar / non-object fields
    let mut first_written = false;
    for (k, v) in fields {
        if !matches!(v, Value::Object(_)) {
            emit_key_value(k, v, options, destination);
            first_written = true;
        }
    }

    // Second pass: write sections
    for (k, v) in fields {
        if let Value::Object(sec_fields) = v {
            if first_written {
                destination.add_bytes("\n");
            }
            destination.add_bytes("[");
            destination.add_bytes(k);
            destination.add_bytes("]\n");

            for (sec_k, sec_v) in sec_fields {
                emit_key_value(sec_k, sec_v, options, destination);
            }
            first_written = true;
        }
    }

    Ok(())
}

fn emit_key_value(key: &str, val: &Value, options: &IniOptions, destination: &mut dyn IDestination) {
    destination.add_bytes(key);
    destination.add_bytes(options.emit_delimiter);

    match val {
        Value::Null => {}
        Value::Bool(b) => destination.add_bytes(if *b { "true" } else { "false" }),
        Value::Integer(i) => destination.add_bytes(&format!("{}", i)),
        Value::Float(f) => destination.add_bytes(&format!("{}", f)),
        Value::String(s) => {
            let quote = s.contains(' ') || s.contains('#') || s.contains(';') || s.contains('=') || s.contains(':');
            if quote {
                destination.add_bytes("\"");
                destination.add_bytes(s);
                destination.add_bytes("\"");
            } else {
                destination.add_bytes(s);
            }
        }
        Value::Bytes(_) => destination.add_bytes("<bytes>"),
        Value::Array(_) => destination.add_bytes("<array>"),
        Value::Object(_) => destination.add_bytes("<object>"),
    }

    destination.add_bytes("\n");
}

// ==========================================
// Zero-Allocation Streaming Pull Parser
// ==========================================

/// Streaming event emitted by [`IniPullParser`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IniEvent<'a> {
    /// Section header `[section_name]`
    Section(&'a str),
    /// Key-value property pair `key = value`
    Entry { key: &'a str, val: &'a str },
    /// Line comment starting with `#` or `;`
    Comment(&'a str),
}

/// Zero-allocation streaming INI and property file pull parser.
///
/// Scans line-by-line over a borrowed string slice with $O(1)$ stack memory.
#[derive(Debug, Clone)]
pub struct IniPullParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> IniPullParser<'a> {
    /// Creates a new streaming INI pull parser over a borrowed string slice.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Pulls the next event from the INI input stream. Returns `None` at EOF.
    pub fn next_event(&mut self) -> Option<IniEvent<'a>> {
        while self.pos < self.input.len() {
            let start = self.pos;
            let bytes = self.input.as_bytes();
            while self.pos < bytes.len() && bytes[self.pos] != b'\n' && bytes[self.pos] != b'\r' {
                self.pos += 1;
            }
            let line = &self.input[start..self.pos];
            if self.pos < bytes.len() && bytes[self.pos] == b'\r' {
                self.pos += 1;
            }
            if self.pos < bytes.len() && bytes[self.pos] == b'\n' {
                self.pos += 1;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                return Some(IniEvent::Comment(trimmed));
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = &trimmed[1..trimmed.len() - 1].trim();
                return Some(IniEvent::Section(section_name));
            }

            if let Some((k, v)) = trimmed.split_once('=') {
                return Some(IniEvent::Entry {
                    key: k.trim(),
                    val: v.trim(),
                });
            } else if let Some((k, v)) = trimmed.split_once(':') {
                return Some(IniEvent::Entry {
                    key: k.trim(),
                    val: v.trim(),
                });
            } else {
                return Some(IniEvent::Entry {
                    key: trimmed,
                    val: "",
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ini_sections() {
        let doc = r#"
        # Global comment
        app_name = Babbel

        [database]
        host = localhost
        port = 5432
        ssl = true

        [cache]
        enabled = false
        ttl = 3600
        "#;

        let val = parse_ini(doc, &IniOptions::default()).unwrap();
        let obj = val.as_object().unwrap();

        assert_eq!(obj[0].0, "app_name");
        assert_eq!(obj[0].1, Value::String("Babbel".into()));

        let db = &obj[1].1;
        let db_obj = db.as_object().unwrap();
        assert_eq!(db_obj[0], ("host".to_string(), Value::String("localhost".into())));
        assert_eq!(db_obj[1], ("port".to_string(), Value::Integer(5432)));
        assert_eq!(db_obj[2], ("ssl".to_string(), Value::Bool(true)));
    }

    #[test]
    fn test_parse_env_file() {
        let doc = "DATABASE_URL=postgres://user:pass@localhost:5432/db\nPORT=3000\nDEBUG=true\n";
        let val = parse_ini(doc, &IniOptions::env()).unwrap();
        let obj = val.as_object().unwrap();

        assert_eq!(obj[0], ("DATABASE_URL".to_string(), Value::String("postgres://user:pass@localhost:5432/db".into())));
        assert_eq!(obj[1], ("PORT".to_string(), Value::Integer(3000)));
        assert_eq!(obj[2], ("DEBUG".to_string(), Value::Bool(true)));
    }

    #[test]
    fn test_emit_ini_roundtrip() {
        let doc = "app_name = Babbel\n\n[database]\nhost = localhost\nport = 5432\n";
        let val = parse_ini(doc, &IniOptions::default()).unwrap();
        let emitted = emit_ini(&val, &IniOptions::default()).unwrap();
        assert_eq!(emitted, doc);
    }

    #[test]
    fn test_ini_pull_parser() {
        let doc = "# Embedded config\nbaud_rate = 115200\n\n[wifi]\nssid = IoT_Network\npass = secret123\n";
        let mut parser = IniPullParser::new(doc);

        assert_eq!(parser.next_event(), Some(IniEvent::Comment("# Embedded config")));
        assert_eq!(parser.next_event(), Some(IniEvent::Entry { key: "baud_rate", val: "115200" }));
        assert_eq!(parser.next_event(), Some(IniEvent::Section("wifi")));
        assert_eq!(parser.next_event(), Some(IniEvent::Entry { key: "ssid", val: "IoT_Network" }));
        assert_eq!(parser.next_event(), Some(IniEvent::Entry { key: "pass", val: "secret123" }));
        assert_eq!(parser.next_event(), None);
    }
}
