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
