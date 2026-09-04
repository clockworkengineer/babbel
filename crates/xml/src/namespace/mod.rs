//! # XML Namespaces Subsystem
//!
//! Provides data structures and scoped resolution for W3C Namespaces in XML 1.0/1.1.

use crate::alloc_prelude::*;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Representation of an XML namespace declaration binding a prefix to a URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    /// Optional prefix (`None` for default namespace `xmlns="..."`).
    pub prefix: Option<Box<str>>,
    /// Bound namespace URI.
    pub uri: Box<str>,
}

impl Namespace {
    /// Creates a new namespace binding.
    pub fn new(prefix: Option<impl Into<Box<str>>>, uri: impl Into<Box<str>>) -> Self {
        Self {
            prefix: prefix.map(Into::into),
            uri: uri.into(),
        }
    }
}

/// Qualified Name (`QName`) separating prefix, local name, and resolved namespace URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QName {
    /// Optional namespace prefix (e.g. `xs` in `xs:element`).
    pub prefix: Option<Box<str>>,
    /// Local part of the qualified name (e.g. `element` in `xs:element`).
    pub local_name: Box<str>,
    /// Resolved namespace URI (e.g. `http://www.w3.org/2001/XMLSchema`).
    pub namespace_uri: Option<Box<str>>,
}

impl QName {
    /// Creates a new qualified name.
    pub fn new(
        prefix: Option<impl Into<Box<str>>>,
        local_name: impl Into<Box<str>>,
        namespace_uri: Option<impl Into<Box<str>>>,
    ) -> Self {
        Self {
            prefix: prefix.map(Into::into),
            local_name: local_name.into(),
            namespace_uri: namespace_uri.map(Into::into),
        }
    }

    /// Parses a raw XML tag or attribute name into `(prefix, local_name)`.
    pub fn split_prefix(raw: &str) -> (Option<&str>, &str) {
        if let Some((prefix, local)) = raw.split_once(':') {
            (Some(prefix), local)
        } else {
            (None, raw)
        }
    }
}

/// Lexical scope stack managing active namespace prefix bindings during parsing.
///
/// # Examples
///
/// ```
/// use xml_lib_rust::NamespaceScope;
///
/// let mut scope = NamespaceScope::new();
/// scope.declare(Some("soap"), "http://schemas.xmlsoap.org/soap/envelope/");
/// assert_eq!(
///     scope.resolve_prefix(Some("soap")),
///     Some("http://schemas.xmlsoap.org/soap/envelope/")
/// );
/// ```
#[derive(Debug, Clone)]
pub struct NamespaceScope {
    scopes: Vec<HashMap<String, String>>,
}

impl Default for NamespaceScope {
    fn default() -> Self {
        let mut initial = HashMap::new();
        // Predefined 'xml' and 'xmlns' namespaces per XML Namespaces 1.0
        initial.insert("xml".to_string(), "http://www.w3.org/XML/1998/namespace".to_string());
        initial.insert("xmlns".to_string(), "http://www.w3.org/2000/xmlns/".to_string());
        Self {
            scopes: vec![initial],
        }
    }
}

impl NamespaceScope {
    /// Creates a new empty `NamespaceScope` with standard `xml` and `xmlns` bindings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a new nested lexical scope level (when entering an element).
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pops the current lexical scope level (when leaving an element).
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Declares a namespace prefix binding in the current innermost scope.
    /// Use `prefix = None` or `""` for the default namespace `xmlns="..."`.
    pub fn declare(&mut self, prefix: Option<&str>, uri: &str) {
        if let Some(current) = self.scopes.last_mut() {
            let key = prefix.unwrap_or("").to_string();
            current.insert(key, uri.to_string());
        }
    }

    /// Resolves a prefix to its active URI by searching from innermost to outermost scope.
    pub fn resolve_prefix(&self, prefix: Option<&str>) -> Option<&str> {
        let key = prefix.unwrap_or("");
        for scope in self.scopes.iter().rev() {
            if let Some(uri) = scope.get(key) {
                // In XML 1.1 / Namespaces 1.0 (2nd ed), undeclaring default namespace results in empty URI
                if uri.is_empty() {
                    return None;
                }
                return Some(uri.as_str());
            }
        }
        None
    }

    /// Resolves a raw name into a full [`QName`].
    pub fn resolve_qname(&self, raw_name: &str, is_attribute: bool) -> QName {
        let (prefix, local) = QName::split_prefix(raw_name);
        let uri: Option<Box<str>> = if is_attribute && prefix.is_none() {
            // Per XML Namespaces §6.2: Unprefixed attributes are NOT in any namespace
            None
        } else {
            self.resolve_prefix(prefix).map(Into::into)
        };

        QName::new(prefix, local, uri)
    }
}
