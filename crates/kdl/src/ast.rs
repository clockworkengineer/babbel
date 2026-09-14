//! KDL Document Abstract Syntax Tree and conversion to Babbel `Value`.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use babbel_core::Value;

/// A complete KDL document consisting of a sequence of nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KdlDocument {
    pub nodes: Vec<KdlNode>,
}

impl KdlDocument {
    /// Create a new empty KDL document.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a node to the document.
    pub fn with_node(mut self, node: KdlNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Convert KdlDocument into universal Babbel `Value`.
    pub fn to_value(&self) -> Value {
        let mut entries: Vec<(String, Value)> = Vec::new();

        for node in &self.nodes {
            let node_val = node.to_value();
            // Check if key already exists to group repeated nodes into an Array
            if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == &node.name) {
                match existing {
                    Value::Array(arr) => {
                        arr.push(node_val);
                    }
                    single => {
                        let prev = core::mem::replace(single, Value::Null);
                        *single = Value::Array(alloc::vec![prev, node_val]);
                    }
                }
            } else {
                entries.push((node.name.clone(), node_val));
            }
        }

        Value::Object(entries)
    }

    /// Convert universal Babbel `Value` into KdlDocument.
    pub fn from_value(value: &Value) -> Self {
        let mut doc = KdlDocument::new();

        match value {
            Value::Object(obj) => {
                for (key, val) in obj {
                    match val {
                        Value::Array(arr) => {
                            for item in arr {
                                doc.nodes.push(node_from_key_val(key, item));
                            }
                        }
                        _ => {
                            doc.nodes.push(node_from_key_val(key, val));
                        }
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    doc.nodes.push(node_from_key_val("item", item));
                }
            }
            scalar => {
                doc.nodes.push(node_from_key_val("value", scalar));
            }
        }

        doc
    }
}

fn node_from_key_val(key: &str, val: &Value) -> KdlNode {
    match val {
        Value::Object(inner) => {
            let mut entries = Vec::new();
            let mut children = Vec::new();

            for (k, v) in inner {
                if k == "@arg" {
                    entries.push(KdlEntry::Arg(None, KdlValue::from_value(v)));
                } else if k == "@args" {
                    if let Value::Array(args) = v {
                        for a in args {
                            entries.push(KdlEntry::Arg(None, KdlValue::from_value(a)));
                        }
                    }
                } else {
                    match v {
                        Value::Array(arr) => {
                            for item in arr {
                                children.push(node_from_key_val(k, item));
                            }
                        }
                        _ => {
                            children.push(node_from_key_val(k, v));
                        }
                    }
                }
            }

            KdlNode {
                type_annotation: None,
                name: key.to_string(),
                entries,
                children,
            }
        }
        Value::Array(arr) => {
            let mut entries = Vec::new();
            for item in arr {
                entries.push(KdlEntry::Arg(None, KdlValue::from_value(item)));
            }
            KdlNode {
                type_annotation: None,
                name: key.to_string(),
                entries,
                children: Vec::new(),
            }
        }
        _ => {
            let kdl_val = KdlValue::from_value(val);
            KdlNode {
                type_annotation: None,
                name: key.to_string(),
                entries: alloc::vec![KdlEntry::Arg(None, kdl_val)],
                children: Vec::new(),
            }
        }
    }
}

/// A single KDL node.
#[derive(Debug, Clone, PartialEq)]
pub struct KdlNode {
    pub type_annotation: Option<String>,
    pub name: String,
    pub entries: Vec<KdlEntry>,
    pub children: Vec<KdlNode>,
}

impl KdlNode {
    /// Create a new node with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            type_annotation: None,
            name: name.into(),
            entries: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Convert node to universal `Value`.
    pub fn to_value(&self) -> Value {
        let has_children = !self.children.is_empty();
        let has_entries = !self.entries.is_empty();

        if !has_children && !has_entries {
            // Bare flag e.g. `fullscreen` -> boolean true
            return Value::Bool(true);
        }

        // Separate positional arguments and properties
        let mut args: Vec<Value> = Vec::new();
        let mut props: Vec<(String, Value)> = Vec::new();

        for entry in &self.entries {
            match entry {
                KdlEntry::Arg(_ty, val) => args.push(val.to_value()),
                KdlEntry::Prop(k, _ty, val) => props.push((k.clone(), val.to_value())),
            }
        }

        // Simplest and most common configuration case:
        // single argument, no props, no children -> directly that value!
        // e.g. `version "1.0.0"` -> Value::String("1.0.0")
        if args.len() == 1 && props.is_empty() && !has_children {
            return args.pop().unwrap();
        }

        // Multiple arguments without props or children -> array
        // e.g. `dimensions 1920 1080` -> Value::Array([1920, 1080])
        if args.len() > 1 && props.is_empty() && !has_children {
            return Value::Array(args);
        }

        // If it only has children and no args/props:
        // e.g. `server { port 8080 }` -> Value::Object of children
        if args.is_empty() && props.is_empty() && has_children {
            let child_doc = KdlDocument {
                nodes: self.children.clone(),
            };
            return child_doc.to_value();
        }

        // General case: combine props, args, and children into an Object
        let mut obj_entries = Vec::new();
        for (k, v) in props {
            obj_entries.push((k, v));
        }

        if !args.is_empty() {
            if args.len() == 1 {
                obj_entries.push(("@arg".into(), args.pop().unwrap()));
            } else {
                obj_entries.push(("@args".into(), Value::Array(args)));
            }
        }

        if has_children {
            let child_doc = KdlDocument {
                nodes: self.children.clone(),
            };
            if let Value::Object(child_entries) = child_doc.to_value() {
                for (k, v) in child_entries {
                    obj_entries.push((k, v));
                }
            }
        }

        Value::Object(obj_entries)
    }
}

/// A positional argument or named property entry on a node.
#[derive(Debug, Clone, PartialEq)]
pub enum KdlEntry {
    /// Positional argument: `(type)value`
    Arg(Option<String>, KdlValue),
    /// Named property: `key=(type)value`
    Prop(String, Option<String>, KdlValue),
}

/// KDL scalar value types.
#[derive(Debug, Clone, PartialEq)]
pub enum KdlValue {
    String(String),
    Integer(i128),
    Float(f64),
    Bool(bool),
    Null,
}

impl KdlValue {
    /// Convert KdlValue to universal `Value`.
    pub fn to_value(&self) -> Value {
        match self {
            KdlValue::String(s) => Value::String(s.clone()),
            KdlValue::Integer(i) => Value::Integer(*i),
            KdlValue::Float(f) => Value::Float(*f),
            KdlValue::Bool(b) => Value::Bool(*b),
            KdlValue::Null => Value::Null,
        }
    }

    /// Convert universal `Value` to KdlValue.
    pub fn from_value(value: &Value) -> Self {
        match value {
            Value::String(s) => KdlValue::String(s.clone()),
            Value::Integer(i) => KdlValue::Integer(*i),
            Value::Float(f) => KdlValue::Float(*f),
            Value::Bool(b) => KdlValue::Bool(*b),
            Value::Null => KdlValue::Null,
            Value::Bytes(b) => {
                // Encode bytes as base64 string or hex string
                KdlValue::String(alloc::format!("0x{}", hex_encode(b)))
            }
            Value::Array(_) | Value::Object(_) => {
                // Complex values in scalar positions fall back to debug string representation
                KdlValue::String(alloc::format!("{:?}", value))
            }
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use core::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}
