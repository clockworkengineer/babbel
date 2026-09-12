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

    /// Returns 64-bit float if value is `Value::Float` or `Value::Integer`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Returns unsigned 64-bit integer if value is non-negative `Value::Integer`.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Integer(i) => (*i).try_into().ok(),
            _ => None,
        }
    }

    /// Returns 128-bit integer if value is `Value::Integer`.
    pub fn as_i128(&self) -> Option<i128> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns raw byte slice if value is `Value::Bytes` or `Value::String`.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b.as_slice()),
            Value::String(s) => Some(s.as_bytes()),
            _ => None,
        }
    }

    /// Check if the value is a boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    /// Check if the value is an integer.
    pub fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// Check if the value is a floating point number.
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    /// Check if the value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if the value is raw binary bytes.
    pub fn is_bytes(&self) -> bool {
        matches!(self, Value::Bytes(_))
    }

    /// Check if the value is an array / sequence.
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Check if the value is an object / mapping.
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Infallible lookup of an object property by key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(entries) => {
                entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Mutable lookup of an object property by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Object(entries) => {
                entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// Infallible index-based lookup in an array.
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        match self {
            Value::Array(items) => items.get(index),
            _ => None,
        }
    }

    /// Mutable index-based lookup in an array.
    pub fn get_index_mut(&mut self, index: usize) -> Option<&mut Value> {
        match self {
            Value::Array(items) => items.get_mut(index),
            _ => None,
        }
    }

    /// Infallible dot-separated path navigation (e.g. `"server.database.port"` or `"users.0.name"`).
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        if path.is_empty() {
            return Some(self);
        }
        let mut current = self;
        for segment in path.split('.') {
            if segment.is_empty() {
                continue;
            }
            if let Ok(idx) = segment.parse::<usize>() {
                if let Some(next) = current.get_index(idx) {
                    current = next;
                    continue;
                }
            }
            match current.get(segment) {
                Some(next) => current = next,
                None => return None,
            }
        }
        Some(current)
    }

    /// Infallible RFC 6901 JSON pointer navigation (e.g. `"/server/port"` or `"/users/0/name"`).
    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        if pointer.is_empty() {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        let mut current = self;
        for part in pointer.split('/').skip(1) {
            let unescaped = if part.contains('~') {
                part.replace("~1", "/").replace("~0", "~")
            } else {
                alloc::string::String::from(part)
            };
            if let Ok(idx) = unescaped.parse::<usize>() {
                if let Some(next) = current.get_index(idx) {
                    current = next;
                    continue;
                }
            }
            match current.get(&unescaped) {
                Some(next) => current = next,
                None => return None,
            }
        }
        Some(current)
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

    /// Serializes value to TOML representation into `dest`.
    pub fn serialize_toml(&self, dest: &mut dyn crate::io::IDestination) {
        match self {
            Value::Object(entries) => {
                serialize_toml_table(entries, "", dest);
            }
            Value::Array(items) if !items.is_empty() && items.iter().all(|x| matches!(x, Value::Object(_))) => {
                for item in items {
                    dest.add_bytes("[[item]]\n");
                    if let Value::Object(entries) = item {
                        serialize_toml_table(entries, "item", dest);
                    }
                }
            }
            single => {
                dest.add_bytes("value = ");
                serialize_toml_value(single, dest);
                dest.add_bytes("\n");
            }
        }
    }
}

fn format_toml_key(key: &str, dest: &mut dyn crate::io::IDestination) {
    if crate::escape::is_valid_toml_bare_key(key) {
        dest.add_bytes(key);
    } else {
        crate::escape::write_toml_escaped_string(key, dest);
    }
}

fn format_toml_table_path(full_path: &str, dest: &mut dyn crate::io::IDestination) {
    let mut first = true;
    for part in full_path.split('.') {
        if !first {
            dest.add_byte(b'.');
        }
        first = false;
        format_toml_key(part, dest);
    }
}

fn is_array_of_objects(val: &Value) -> bool {
    match val {
        Value::Array(arr) => !arr.is_empty() && arr.iter().all(|it| matches!(it, Value::Object(_))),
        _ => false,
    }
}

fn serialize_toml_table(
    entries: &[(alloc::string::String, Value)],
    prefix: &str,
    dest: &mut dyn crate::io::IDestination,
) {
    let mut has_written_direct = false;
    // 1. Emit direct scalar/inline properties
    for (k, v) in entries {
        if matches!(v, Value::Object(_)) || is_array_of_objects(v) {
            continue;
        }
        if !prefix.is_empty() && !has_written_direct {
            dest.add_bytes("[");
            format_toml_table_path(prefix, dest);
            dest.add_bytes("]\n");
        }
        format_toml_key(k, dest);
        dest.add_bytes(" = ");
        serialize_toml_value(v, dest);
        dest.add_byte(b'\n');
        has_written_direct = true;
    }

    // 2. Emit sub-tables
    for (k, v) in entries {
        if let Value::Object(sub_entries) = v {
            let full_key = if prefix.is_empty() {
                k.clone()
            } else {
                alloc::format!("{}.{}", prefix, k)
            };
            if sub_entries.is_empty() {
                dest.add_bytes("[");
                format_toml_table_path(&full_key, dest);
                dest.add_bytes("]\n");
            } else {
                serialize_toml_table(sub_entries, &full_key, dest);
            }
        }
    }

    // 3. Emit array of tables
    for (k, v) in entries {
        if is_array_of_objects(v) {
            if let Value::Array(items) = v {
                let full_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    alloc::format!("{}.{}", prefix, k)
                };
                for item in items {
                    dest.add_bytes("[[");
                    format_toml_table_path(&full_key, dest);
                    dest.add_bytes("]]\n");
                    if let Value::Object(sub_entries) = item {
                        serialize_toml_table(sub_entries, &full_key, dest);
                    }
                }
            }
        }
    }
}

fn serialize_toml_value(val: &Value, dest: &mut dyn crate::io::IDestination) {
    match val {
        Value::Null => dest.add_bytes("\"\""),
        Value::Bool(b) => dest.add_bytes(if *b { "true" } else { "false" }),
        Value::Integer(i) => {
            let mut buf = itoa::Buffer::new();
            dest.add_bytes(buf.format(*i));
        }
        Value::Float(f) => {
            if f.is_nan() {
                dest.add_bytes("nan");
            } else if f.is_infinite() {
                dest.add_bytes(if *f < 0.0 { "-inf" } else { "inf" });
            } else {
                let mut buf = dtoa::Buffer::new();
                let s = buf.format(*f);
                dest.add_bytes(s);
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    dest.add_bytes(".0");
                }
            }
        }
        Value::String(s) => crate::escape::write_toml_escaped_string(s, dest),
        Value::Bytes(b) => {
            crate::escape::write_toml_escaped_string(&alloc::string::String::from_utf8_lossy(b), dest);
        }
        Value::Array(items) => {
            dest.add_byte(b'[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    dest.add_bytes(", ");
                }
                serialize_toml_value(item, dest);
            }
            dest.add_byte(b']');
        }
        Value::Object(entries) => {
            dest.add_bytes("{ ");
            for (idx, (k, v)) in entries.iter().enumerate() {
                if idx > 0 {
                    dest.add_bytes(", ");
                }
                format_toml_key(k, dest);
                dest.add_bytes(" = ");
                serialize_toml_value(v, dest);
            }
            dest.add_bytes(" }");
        }
    }
}

/// Visitor pattern for streaming serialization or traversal across polyglot formats.
/// Provides default no-op implementations (ISP compliant) so implementers only need to
/// override the callbacks relevant to their use case.
pub trait FormatVisitor {
    type Error;

    /// Called when visiting a null value.
    fn visit_null(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a boolean value.
    fn visit_bool(&mut self, _val: bool) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a signed integer value.
    fn visit_integer(&mut self, _val: i128) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a 64-bit floating point value.
    fn visit_float(&mut self, _val: f64) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a string slice.
    fn visit_str(&mut self, _val: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a raw byte slice.
    fn visit_bytes(&mut self, _val: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the start of an array sequence.
    fn visit_array_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the end of an array sequence.
    fn visit_array_end(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the start of a key-value mapping/object.
    fn visit_object_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting an object key.
    fn visit_key(&mut self, _key: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the end of a key-value mapping/object.
    fn visit_object_end(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i as i128)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Integer(i as i128)
    }
}

impl From<i128> for Value {
    fn from(i: i128) -> Self {
        Value::Integer(i)
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Self {
        Value::Integer(u as i128)
    }
}

impl From<u32> for Value {
    fn from(u: u32) -> Self {
        Value::Integer(u as i128)
    }
}

impl From<usize> for Value {
    fn from(u: usize) -> Self {
        Value::Integer(u as i128)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f as f64)
    }
}

impl From<alloc::string::String> for Value {
    fn from(s: alloc::string::String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(alloc::string::String::from(s))
    }
}

impl From<alloc::vec::Vec<u8>> for Value {
    fn from(b: alloc::vec::Vec<u8>) -> Self {
        Value::Bytes(b)
    }
}

impl From<&[u8]> for Value {
    fn from(b: &[u8]) -> Self {
        Value::Bytes(b.to_vec())
    }
}

impl From<alloc::vec::Vec<Value>> for Value {
    fn from(arr: alloc::vec::Vec<Value>) -> Self {
        Value::Array(arr)
    }
}

impl From<alloc::vec::Vec<(alloc::string::String, Value)>> for Value {
    fn from(obj: alloc::vec::Vec<(alloc::string::String, Value)>) -> Self {
        Value::Object(obj)
    }
}

static NULL_VALUE: Value = Value::Null;

impl core::ops::Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap_or(&NULL_VALUE)
    }
}

impl core::ops::Index<&alloc::string::String> for Value {
    type Output = Value;

    fn index(&self, index: &alloc::string::String) -> &Self::Output {
        self.get(index.as_str()).unwrap_or(&NULL_VALUE)
    }
}

impl core::ops::Index<usize> for Value {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        self.get_index(index).unwrap_or(&NULL_VALUE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;

    // Test visitor implementing ONLY visit_key and visit_str (testing ISP segregation)
    struct KeyCollector {
        keys: Vec<String>,
    }

    impl FormatVisitor for KeyCollector {
        type Error = ();

        fn visit_key(&mut self, key: &str) -> Result<(), Self::Error> {
            self.keys.push(String::from(key));
            Ok(())
        }
    }

    #[test]
    fn test_format_visitor_isp_defaults() {
        let mut collector = KeyCollector { keys: Vec::new() };

        // Test that unimplemented methods return Ok(()) without error or boilerplate
        assert_eq!(collector.visit_null(), Ok(()));
        assert_eq!(collector.visit_bool(true), Ok(()));
        assert_eq!(collector.visit_integer(42), Ok(()));
        assert_eq!(collector.visit_float(3.14), Ok(()));
        assert_eq!(collector.visit_str("hello"), Ok(()));
        assert_eq!(collector.visit_bytes(b"bytes"), Ok(()));
        assert_eq!(collector.visit_array_start(), Ok(()));
        assert_eq!(collector.visit_array_end(), Ok(()));
        assert_eq!(collector.visit_object_start(), Ok(()));
        assert_eq!(collector.visit_object_end(), Ok(()));

        // Test overridden method works as expected
        assert_eq!(collector.visit_key("name"), Ok(()));
        assert_eq!(collector.visit_key("version"), Ok(()));
        assert_eq!(collector.keys, vec!["name", "version"]);
    }

    #[test]
    fn test_value_navigation_and_accessors() {
        let val = Value::Object(vec![
            ("title".to_string(), Value::String("Babbel".to_string())),
            ("version".to_string(), Value::Float(0.2)),
            ("port".to_string(), Value::Integer(8080)),
            ("active".to_string(), Value::Bool(true)),
            (
                "server".to_string(),
                Value::Object(vec![
                    ("host".to_string(), Value::String("127.0.0.1".to_string())),
                    (
                        "tags".to_string(),
                        Value::Array(vec![
                            Value::String("api".to_string()),
                            Value::String("v1".to_string()),
                        ]),
                    ),
                ]),
            ),
        ]);

        assert_eq!(val.is_object(), true);
        assert_eq!(val.get("title").and_then(|v| v.as_str()), Some("Babbel"));
        assert_eq!(val.get("port").and_then(|v| v.as_i64()), Some(8080));
        assert_eq!(val.get("port").and_then(|v| v.as_u64()), Some(8080));
        assert_eq!(val.get("port").and_then(|v| v.as_i128()), Some(8080));
        assert_eq!(val.get("version").and_then(|v| v.as_f64()), Some(0.2));
        assert_eq!(val.get("active").and_then(|v| v.as_bool()), Some(true));

        // Path navigation
        assert_eq!(
            val.get_path("server.host").and_then(|v| v.as_str()),
            Some("127.0.0.1")
        );
        assert_eq!(
            val.get_path("server.tags.0").and_then(|v| v.as_str()),
            Some("api")
        );
        assert_eq!(
            val.get_path("server.tags.1").and_then(|v| v.as_str()),
            Some("v1")
        );
        assert_eq!(val.get_path("nonexistent.path"), None);

        // JSON Pointer navigation
        assert_eq!(
            val.pointer("/server/host").and_then(|v| v.as_str()),
            Some("127.0.0.1")
        );
        assert_eq!(
            val.pointer("/server/tags/0").and_then(|v| v.as_str()),
            Some("api")
        );

        // From conversions
        assert_eq!(Value::from(42_i64), Value::Integer(42));
        assert_eq!(Value::from("hello"), Value::String("hello".to_string()));
        assert_eq!(Value::from(true), Value::Bool(true));
    }
}


