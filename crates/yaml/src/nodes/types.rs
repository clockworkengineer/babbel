//! YAML Node Type Definitions
//!
//! Defines the core `Node` enum, `QuoteType`, `BlockStyle`, and conversion traits.
//!
//! Copyright (c) 2026 YAML Library Developers

use alloc::format;
use alloc::string::ToString;

#[allow(dead_code)]
/// Shared trait for node/string conversion and cloning
pub trait NodeStringConvert {
    /// Convert node to string (lossy)
    fn to_string_lossy(&self) -> alloc::string::String;
    /// Get string value if node is a string
    fn as_str(&self) -> Option<&str>;
    /// Clone node as a string node
    fn clone_as_string(&self) -> Option<Node>;
}

/// Represents different numeric types that can be stored in a YAML node
pub use babbel_core::num::Numeric;

/// Represents how a string was quoted in the source YAML
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub enum QuoteType {
    Unquoted,
    Single,
    Double,
}

/// Represents block/folded style for YAML string nodes
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub enum BlockStyle {
    None,
    Literal,
    Folded,
}

/// A node in the YAML data structure that can represent different types of values.
///
/// This is the main enum for representing YAML data in memory.
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// Represents a boolean value (true/false)
    Boolean(bool),
    /// Represents a numeric value (various integer and float types)
    Number(Numeric),
    /// Represents a string value and how it was quoted in the source
    Str(alloc::string::String, QuoteType, BlockStyle),
    /// Represents an array of other nodes
    Array(alloc::vec::Vec<Node>),
    /// Represents a set of unique nodes
    Set(alloc::vec::Vec<Node>),
    /// Represents a mapping where keys are Nodes (allowing quoted metadata)
    Mapping(alloc::vec::Vec<(Node, Node)>),
    /// Represents a comment
    Comment(alloc::string::String),
    /// Represents a document node
    Document(alloc::vec::Vec<Node>),
    /// Represents an anchored node: a node with an associated anchor name
    Anchored(alloc::boxed::Box<Node>, alloc::string::String),
    /// Represents a tagged node using YAML tag syntax (e.g., !!str, !mytag)
    Tagged(alloc::boxed::Box<Node>, alloc::string::String),
    /// Represents an alias node that references a previously anchored node
    Alias(alloc::string::String),
    /// Represents a sequence of documents
    Documents(alloc::vec::Vec<Node>),
    /// Represents a null value or uninitialized node
    None,
}

/// Minimal node for no-alloc environments (embedded only)
#[cfg(not(feature = "alloc"))]
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Boolean(bool),
    Number(Numeric),
    None,
}

impl NodeStringConvert for Node {
    #[inline]
    fn to_string_lossy(&self) -> alloc::string::String {
        match self {
            Node::Str(s, _, _) => s.clone(),
            Node::Number(n) => n.to_string_lossy(),
            Node::Boolean(b) => b.to_string(),
            Node::None => "null".to_string(),
            _ => format!("{:?}", self),
        }
    }
    #[inline]
    fn as_str(&self) -> Option<&str> {
        match self {
            Node::Str(s, _, _) => Some(s.as_str()),
            _ => None,
        }
    }
    #[inline]
    fn clone_as_string(&self) -> Option<Node> {
        self.as_str().map(|s| Node::from(s))
    }
}
