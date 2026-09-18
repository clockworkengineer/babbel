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
            Value::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Mutable lookup of an object property by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Object(entries) => entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
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

    /// Evaluates an RFC 9535 JSONPath expression on this value, returning all matching node references.
    pub fn jsonpath<'a>(
        &'a self,
        expr: &str,
    ) -> Result<alloc::vec::Vec<&'a Value>, crate::error::BabbelError> {
        crate::query::query(self, expr)
    }

    /// Evaluates an RFC 9535 JSONPath expression on this value, returning all matching mutable references.
    pub fn jsonpath_mut<'a>(
        &'a mut self,
        expr: &str,
    ) -> Result<alloc::vec::Vec<&'a mut Value>, crate::error::BabbelError> {
        crate::query::query_mut(self, expr)
    }

    /// Applies an RFC 6902 JSON Patch to this value, returning a new transformed `Value`.
    pub fn patch(&self, patch: &crate::patch::Patch) -> Result<Value, crate::error::BabbelError> {
        patch.apply(self)
    }

    /// Applies an RFC 6902 JSON Patch in-place to this value.
    pub fn patch_inplace(
        &mut self,
        patch: &crate::patch::Patch,
    ) -> Result<(), crate::error::BabbelError> {
        patch.apply_inplace(self)
    }

    /// Applies an RFC 7396 JSON Merge Patch in-place to this value.
    pub fn merge_patch(&mut self, patch: &Value) {
        crate::patch::apply_merge_patch(self, patch);
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

    /// Accepts a visitor for streaming traversal across the universal AST (ISP / Visitor Pattern).
    pub fn accept<V: FormatVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            Value::Null => visitor.visit_null(),
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Integer(i) => visitor.visit_integer(*i),
            Value::Float(f) => visitor.visit_float(*f),
            Value::String(s) => visitor.visit_str(s),
            Value::Bytes(b) => visitor.visit_bytes(b),
            Value::Array(items) => {
                visitor.visit_array_start()?;
                for item in items {
                    item.accept(visitor)?;
                }
                visitor.visit_array_end()
            }
            Value::Object(entries) => {
                visitor.visit_object_start()?;
                for (k, v) in entries {
                    visitor.visit_key(k)?;
                    v.accept(visitor)?;
                }
                visitor.visit_object_end()
            }
        }
    }

    /// Serializes value to JSON representation into `dest` (delegates to [`crate::emitters::JsonEmitter`]).
    #[inline]
    pub fn serialize_json(&self, dest: &mut dyn crate::io::IDestination) {
        crate::emitters::JsonEmitter.emit_to_dest(self, dest);
    }

    /// Serializes value to YAML representation into `dest` with indentation (delegates to [`crate::emitters::YamlEmitter`]).
    #[inline]
    pub fn serialize_yaml(&self, dest: &mut dyn crate::io::IDestination, indent: usize) {
        crate::emitters::YamlEmitter.emit_with_indent(self, dest, indent);
    }

    /// Serializes value to Bencode representation into `dest` (delegates to [`crate::emitters::BencodeEmitter`]).
    #[inline]
    pub fn serialize_bencode(&self, dest: &mut dyn crate::io::IDestination) {
        crate::emitters::BencodeEmitter.emit_to_dest(self, dest);
    }

    /// Serializes value to XML representation into `dest` (delegates to [`crate::emitters::XmlEmitter`]).
    #[inline]
    pub fn serialize_xml(&self, dest: &mut dyn crate::io::IDestination, root_tag: Option<&str>) {
        crate::emitters::XmlEmitter.emit_with_tag(self, dest, root_tag);
    }

    /// Serializes value to TOML representation into `dest` (delegates to [`crate::emitters::TomlEmitter`]).
    #[inline]
    pub fn serialize_toml(&self, dest: &mut dyn crate::io::IDestination) {
        crate::emitters::TomlEmitter.emit_to_dest(self, dest);
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

        assert!(val.is_object());
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

    #[test]
    fn test_value_accept_visitor() {
        let val = Value::Object(vec![
            ("key1".to_string(), Value::String("val1".to_string())),
            ("key2".to_string(), Value::Integer(100)),
        ]);

        let mut collector = KeyCollector { keys: Vec::new() };
        assert!(val.accept(&mut collector).is_ok());
        assert_eq!(collector.keys, vec!["key1", "key2"]);
    }

    #[test]
    fn test_segregated_emitters() {
        let val = Value::Object(vec![
            ("name".to_string(), Value::String("test".to_string())),
            ("count".to_string(), Value::Integer(1)),
        ]);

        let mut json_buf = crate::io::Buffer::new();
        val.serialize_json(&mut json_buf);
        assert!(json_buf.to_string().contains("\"name\":\"test\""));

        let mut yaml_buf = crate::io::Buffer::new();
        val.serialize_yaml(&mut yaml_buf, 0);
        assert!(yaml_buf.to_string().contains("name: test"));

        let mut toml_buf = crate::io::Buffer::new();
        val.serialize_toml(&mut toml_buf);
        assert!(toml_buf.to_string().contains("name = \"test\""));

        let mut bencode_buf = crate::io::Buffer::new();
        val.serialize_bencode(&mut bencode_buf);
        assert!(bencode_buf.to_string().starts_with('d'));

        let mut xml_buf = crate::io::Buffer::new();
        val.serialize_xml(&mut xml_buf, Some("root"));
        assert!(xml_buf.to_string().starts_with("<root>"));
    }
}
