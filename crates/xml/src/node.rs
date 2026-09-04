use crate::alloc_prelude::*;

/// Unique index identifier for a node within the arena [`Vec<NodeData>`].
/// Default `u32` supports up to 4.2 billion nodes. Enabling `features = ["small_nodes"]` uses `u16` (max 65,535 nodes) for ultra-low memory microcontrollers.
#[cfg(not(feature = "small_nodes"))]
pub type NodeId = u32;

/// Compact 16-bit node index identifier for microcontrollers with <64K nodes.
#[cfg(feature = "small_nodes")]
pub type NodeId = u16;

/// Key-value attribute pair on an XML Element tag stored with compact boxed strings (`Box<str>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    /// Attribute name/key (e.g., `id`, `xmlns:xs`, `category`).
    pub name: Box<str>,
    /// Attribute string value.
    pub value: Box<str>,
}

impl Attribute {
    /// Instantiates a new [`Attribute`] from name and value.
    pub fn new(name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Enum representing all supported XML node variants stored with memory-compact boxed types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// Top-level Prolog virtual container.
    Prolog,
    /// XML Declaration (`<?xml version="..." encoding="..." standalone="..."?>`).
    Declaration {
        version: Box<str>,
        encoding: Option<Box<str>>,
        standalone: Option<bool>,
    },
    /// Top-level Root virtual container.
    Root,
    /// XML Element tag (`<name attr="val">...</name>`).
    Element {
        name: Box<str>,
        attributes: Vec<Attribute>,
    },
    /// Text node content.
    Text(Box<str>),
    /// CDATA section (`<!CDATA[...]]>`).
    CData(Box<str>),
    /// XML Comment (`<!-- ... -->`).
    Comment(Box<str>),
    /// Processing Instruction (`<?target data?>`).
    ProcessingInstruction {
        target: Box<str>,
        data: Box<str>,
    },
    /// DTD DOCTYPE Definition (`<!DOCTYPE name ...>`).
    DocTypeDefinition {
        name: Box<str>,
        public_id: Option<Box<str>>,
        system_id: Option<Box<str>>,
        internal_subset: Option<Box<str>>,
    },
    /// Entity Reference (`&name;`).
    EntityReference(Box<str>),
}

impl NodeKind {
    /// Returns the element name or node kind label.
    pub fn name(&self) -> &str {
        match self {
            NodeKind::Element { name, .. } => name,
            NodeKind::ProcessingInstruction { target, .. } => target,
            NodeKind::DocTypeDefinition { name, .. } => name,
            NodeKind::EntityReference(name) => name,
            NodeKind::Declaration { .. } => "xml",
            NodeKind::CData(_) => "#cdata",
            NodeKind::Comment(_) => "#comment",
            NodeKind::Text(_) => "#text",
            NodeKind::Prolog => "#prolog",
            NodeKind::Root => "#document",
        }
    }

    /// Looks up an attribute value by name if this node is an [`NodeKind::Element`].
    pub fn get_attribute<'a>(&'a self, target: &str) -> Option<&'a str> {
        if let NodeKind::Element { attributes, .. } = self {
            attributes.iter().find(|a| &*a.name == target).map(|a| &*a.value)
        } else {
            None
        }
    }

    /// Checks if this node is an [`NodeKind::Element`] and has an attribute with the given name.
    pub fn has_attribute(&self, target: &str) -> bool {
        self.get_attribute(target).is_some()
    }

    /// Sets an attribute value on this node if it is an [`NodeKind::Element`].
    /// If the attribute already exists, its value is overwritten; otherwise, a new attribute is appended.
    pub fn set_attribute(&mut self, name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> bool {
        if let NodeKind::Element { attributes, .. } = self {
            let name_box = name.into();
            if let Some(attr) = attributes.iter_mut().find(|a| a.name == name_box) {
                attr.value = value.into();
            } else {
                attributes.push(Attribute::new(name_box, value));
            }
            true
        } else {
            false
        }
    }

    /// Removes an attribute by name if this node is an [`NodeKind::Element`]. Returns `true` if removed.
    pub fn remove_attribute(&mut self, target: &str) -> bool {
        if let NodeKind::Element { attributes, .. } = self {
            if let Some(pos) = attributes.iter().position(|a| &*a.name == target) {
                attributes.remove(pos);
                return true;
            }
        }
        false
    }

    /// Returns `true` if this node is an [`NodeKind::Element`].
    pub fn is_element(&self) -> bool {
        matches!(self, NodeKind::Element { .. })
    }

    /// Returns `true` if this node is a [`NodeKind::Text`].
    pub fn is_text(&self) -> bool {
        matches!(self, NodeKind::Text(_))
    }

    /// Returns `true` if this node is a [`NodeKind::Comment`].
    pub fn is_comment(&self) -> bool {
        matches!(self, NodeKind::Comment(_))
    }

    /// Returns `true` if this node is a [`NodeKind::CData`].
    pub fn is_cdata(&self) -> bool {
        matches!(self, NodeKind::CData(_))
    }

    /// Returns `true` if this node is a [`NodeKind::ProcessingInstruction`].
    pub fn is_processing_instruction(&self) -> bool {
        matches!(self, NodeKind::ProcessingInstruction { .. })
    }
}

/// Node container storing node identity, parent link, children links, and node payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeData {
    /// Arena Node ID index.
    pub id: NodeId,
    /// Parent Node ID in arena if linked.
    pub parent: Option<NodeId>,
    /// List of child Node IDs in arena.
    pub children: Vec<NodeId>,
    /// Variant payload describing node type and data.
    pub kind: NodeKind,
}

impl NodeData {
    /// Instantiates a new [`NodeData`] node for a given ID and [`NodeKind`].
    pub fn new(id: NodeId, kind: NodeKind) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            kind,
        }
    }
}
