//! Universal polyglot intermediate data model (`Value`) and visitor abstractions.

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Universal hierarchical data value across XML, JSON, YAML, and Bencode representations.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null / empty / none value
    Null,
    /// Boolean flag
    Bool(bool),
    /// Signed or unsigned integer
    Integer(i128),
    /// Floating point number
    Float(f64),
    /// UTF-8 Text String
    String(String),
    /// Raw binary byte payload (e.g. Bencode byte strings or base64 data)
    Bytes(Vec<u8>),
    /// Sequence / Array / List of values
    Array(Vec<Value>),
    /// Map / Object / Dictionary of key-value pairs (ordered to preserve deterministic serialization)
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Check if the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Returns string reference if value is `Value::String`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns integer if value is `Value::Integer`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => (*i).try_into().ok(),
            _ => None,
        }
    }

    /// Returns boolean if value is `Value::Bool`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns slice of elements if value is `Value::Array`.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    /// Returns key-value pairs if value is `Value::Object`.
    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Object(obj) => Some(obj.as_slice()),
            _ => None,
        }
    }

    /// Emits this value using a pluggable format emitter adhering to OCP.
    pub fn emit<E: crate::codec::FormatEmitter + ?Sized>(
        &self,
        emitter: &E,
        dest: &mut dyn crate::io::IDestination,
    ) -> Result<(), crate::error::BabbelError> {
        emitter.emit(self, dest)
    }

    /// Emits this value with pretty printing using a pluggable format emitter adhering to OCP.
    pub fn emit_pretty<E: crate::codec::FormatEmitter + ?Sized>(
        &self,
        emitter: &E,
        dest: &mut dyn crate::io::IDestination,
        indent: usize,
    ) -> Result<(), crate::error::BabbelError> {
        emitter.emit_pretty(self, dest, indent)
    }

    /// Serializes value to JSON representation into `dest`.
    pub fn serialize_json(&self, dest: &mut dyn crate::io::IDestination) {
        match self {
            Value::Null => dest.add_bytes("null"),
            Value::Bool(b) => dest.add_bytes(if *b { "true" } else { "false" }),
            Value::Integer(i) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
            }
            Value::Float(f) => {
                let mut buf = dtoa::Buffer::new();
                dest.add_bytes(buf.format(*f));
            }
            Value::String(s) => {
                crate::escape::write_json_escaped_string(s, dest);
            }
            Value::Bytes(bytes) => {
                crate::escape::write_json_escaped_string(&alloc::string::String::from_utf8_lossy(bytes), dest);
            }
            Value::Array(items) => {
                dest.add_bytes("[");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        dest.add_bytes(",");
                    }
                    item.serialize_json(dest);
                }
                dest.add_bytes("]");
            }
            Value::Object(entries) => {
                dest.add_bytes("{");
                for (idx, (k, v)) in entries.iter().enumerate() {
                    if idx > 0 {
                        dest.add_bytes(",");
                    }
                    crate::escape::write_json_escaped_string(k, dest);
                    dest.add_bytes(":");
                    v.serialize_json(dest);
                }
                dest.add_bytes("}");
            }
        }
    }

    /// Serializes value to YAML representation into `dest` with indentation.
    pub fn serialize_yaml(&self, dest: &mut dyn crate::io::IDestination, indent: usize) {
        match self {
            Value::Null => dest.add_bytes("null"),
            Value::Bool(b) => dest.add_bytes(if *b { "true" } else { "false" }),
            Value::Integer(i) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
            }
            Value::Float(f) => {
                let mut buf = dtoa::Buffer::new();
                dest.add_bytes(buf.format(*f));
            }
            Value::String(s) => {
                if s.contains('\n') || s.contains('"') {
                    dest.add_bytes("|\n");
                    for line in s.lines() {
                        for _ in 0..(indent + 2) {
                            dest.add_byte(b' ');
                        }
                        dest.add_bytes(line);
                        dest.add_bytes("\n");
                    }
                } else {
                    dest.add_bytes(s);
                }
            }
            Value::Bytes(b) => {
                dest.add_bytes(&alloc::string::String::from_utf8_lossy(b));
            }
            Value::Array(items) => {
                if items.is_empty() {
                    dest.add_bytes("[]");
                    return;
                }
                dest.add_bytes("\n");
                for item in items {
                    for _ in 0..indent {
                        dest.add_byte(b' ');
                    }
                    dest.add_bytes("- ");
                    item.serialize_yaml(dest, indent + 2);
                    dest.add_bytes("\n");
                }
            }
            Value::Object(entries) => {
                if entries.is_empty() {
                    dest.add_bytes("{}");
                    return;
                }
                dest.add_bytes("\n");
                for (k, v) in entries {
                    for _ in 0..indent {
                        dest.add_byte(b' ');
                    }
                    dest.add_bytes(k);
                    dest.add_bytes(": ");
                    v.serialize_yaml(dest, indent + 2);
                    dest.add_bytes("\n");
                }
            }
        }
    }

    /// Serializes value to Bencode representation into `dest`.
    pub fn serialize_bencode(&self, dest: &mut dyn crate::io::IDestination) {
        match self {
            Value::Null => {}
            Value::Bool(b) => {
                dest.add_bytes(if *b { "i1e" } else { "i0e" });
            }
            Value::Integer(i) => {
                dest.add_bytes("i");
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
                dest.add_bytes("e");
            }
            Value::Float(f) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes("i");
                let rounded = if *f >= 0.0 { (*f + 0.5) as i64 } else { (*f - 0.5) as i64 };
                dest.add_bytes(buf.format(rounded));
                dest.add_bytes("e");
            }
            Value::String(s) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(s.len()));
                dest.add_bytes(":");
                dest.add_bytes(s);
            }
            Value::Bytes(b) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(b.len()));
                dest.add_bytes(":");
                if let Ok(s) = core::str::from_utf8(b) {
                    dest.add_bytes(s);
                } else {
                    for &byte in b {
                        dest.add_byte(byte);
                    }
                }
            }
            Value::Array(items) => {
                dest.add_bytes("l");
                for item in items {
                    item.serialize_bencode(dest);
                }
                dest.add_bytes("e");
            }
            Value::Object(entries) => {
                dest.add_bytes("d");
                for (k, v) in entries {
                    let mut buf = itoa::Buffer::new();
                    dest.add_bytes(buf.format(k.len()));
                    dest.add_bytes(":");
                    dest.add_bytes(k);
                    v.serialize_bencode(dest);
                }
                dest.add_bytes("e");
            }
        }
    }

    /// Serializes value to XML representation into `dest`.
    pub fn serialize_xml(&self, dest: &mut dyn crate::io::IDestination, root_tag: Option<&str>) {
        let tag = root_tag.unwrap_or("root");
        match self {
            Value::Null => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes("/>");
            }
            Value::Bool(b) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                dest.add_bytes(if *b { "true" } else { "false" });
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Integer(i) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Float(f) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                let mut buf = dtoa::Buffer::new();
                dest.add_bytes(buf.format(*f));
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::String(s) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                crate::escape::write_xml_escaped_string(s, dest);
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Bytes(b) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                crate::escape::write_xml_escaped_string(&alloc::string::String::from_utf8_lossy(b), dest);
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Array(items) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                for item in items {
                    item.serialize_xml(dest, Some("item"));
                }
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Object(entries) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                for (k, v) in entries {
                    v.serialize_xml(dest, Some(k));
                }
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
        }
    }
}

/// Visitor pattern for streaming serialization or traversal across polyglot formats.
pub trait FormatVisitor {
    type Error;

    fn visit_null(&mut self) -> Result<(), Self::Error>;
    fn visit_bool(&mut self, val: bool) -> Result<(), Self::Error>;
    fn visit_integer(&mut self, val: i128) -> Result<(), Self::Error>;
    fn visit_float(&mut self, val: f64) -> Result<(), Self::Error>;
    fn visit_str(&mut self, val: &str) -> Result<(), Self::Error>;
    fn visit_bytes(&mut self, val: &[u8]) -> Result<(), Self::Error>;
    fn visit_array_start(&mut self) -> Result<(), Self::Error>;
    fn visit_array_end(&mut self) -> Result<(), Self::Error>;
    fn visit_object_start(&mut self) -> Result<(), Self::Error>;
    fn visit_key(&mut self, key: &str) -> Result<(), Self::Error>;
    fn visit_object_end(&mut self) -> Result<(), Self::Error>;
}
