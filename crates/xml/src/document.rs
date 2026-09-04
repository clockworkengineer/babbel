//! # Document (DOM Arena Model)
//!
//! Provides the primary [`Document`] container representing an XML DOM tree stored in a flat arena vector.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};
use crate::node::{NodeData, NodeId, NodeKind};

/// In-memory representation of an XML Document stored in a flat arena [`Vec<NodeData>`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    /// Arena vector storing all nodes indexed by [`NodeId`].
    nodes: Vec<NodeData>,
    /// Optional Node ID of top-level virtual Root container.
    root_id: Option<NodeId>,
    /// Optional Node ID of Prolog container.
    prolog_id: Option<NodeId>,
    /// Optional Node ID of XML Declaration node.
    declaration_id: Option<NodeId>,
    /// Optional Node ID of DTD DOCTYPE node.
    dtd_id: Option<NodeId>,
}

impl Document {
    /// Creates a new empty `Document` with initialized Prolog and Root virtual container nodes.
    pub fn new() -> Self {
        let mut doc = Self {
            nodes: Vec::new(),
            root_id: None,
            prolog_id: None,
            declaration_id: None,
            dtd_id: None,
        };
        // Create top-level Prolog container node
        let prolog_id = doc.add_node(NodeKind::Prolog);
        doc.prolog_id = Some(prolog_id);
        
        // Create root virtual container node
        let root_id = doc.add_node(NodeKind::Root);
        doc.root_id = Some(root_id);
        
        doc
    }

    /// Parses an XML string slice into a new [`Document`].
    ///
    /// # Examples
    ///
    /// ```
    /// use xml_lib_rust::Document;
    ///
    /// let doc = Document::parse_str("<root><child/></root>").unwrap();
    /// assert_eq!(doc.get_root_element_name(), Some("root"));
    /// ```
    pub fn parse_str(xml: &str) -> Result<Self> {
        crate::parse(xml)
    }

    /// Returns the tag name of the root element if one exists.
    pub fn get_root_element_name(&self) -> Option<&str> {
        self.root_element_id().and_then(|id| self.get_node(id)).map(|n| n.kind.name())
    }

    /// Appends a new node kind to the arena and returns its unique [`NodeId`].
    pub fn add_node(&mut self, kind: NodeKind) -> NodeId {
        assert!(
            self.nodes.len() < (NodeId::MAX as usize),
            "Document arena capacity exceeded for configured NodeId size"
        );
        let id = self.nodes.len() as NodeId;
        self.nodes.push(NodeData::new(id, kind));
        id
    }

    /// Validates hierarchy rules preventing self-insertion or cyclic ancestor-child relationships (W3C HierarchyRequestError).
    fn validate_hierarchy(&self, parent_id: NodeId, child_id: NodeId) -> Result<()> {
        if parent_id == child_id {
            return Err(XmlError::NodeError(
                "HierarchyRequestError: Cannot insert a node into itself".into(),
            ));
        }
        let mut curr = Some(parent_id);
        while let Some(ancestor_id) = curr {
            if ancestor_id == child_id {
                return Err(XmlError::NodeError(
                    "HierarchyRequestError: Cannot insert an ancestor node as a child of its descendant".into(),
                ));
            }
            curr = self.nodes.get(ancestor_id as usize).and_then(|n| n.parent);
        }
        Ok(())
    }

    /// Links a child node to a parent node within the arena.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<()> {
        let p_idx = parent_id as usize;
        let c_idx = child_id as usize;
        if p_idx >= self.nodes.len() || c_idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        self.validate_hierarchy(parent_id, child_id)?;
        if let Some(old_parent) = self.nodes[c_idx].parent {
            if old_parent != parent_id {
                self.remove_child(old_parent, child_id)?;
            }
        }
        self.nodes[c_idx].parent = Some(parent_id);
        if !self.nodes[p_idx].children.contains(&child_id) {
            self.nodes[p_idx].children.push(child_id);
        }
        Ok(())
    }

    /// Detaches a child node from its parent within the arena and resets the child's parent link to `None`.
    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<NodeId> {
        let p_idx = parent_id as usize;
        let c_idx = child_id as usize;
        if p_idx >= self.nodes.len() || c_idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        let pos = self.nodes[p_idx]
            .children
            .iter()
            .position(|&id| id == child_id)
            .ok_or_else(|| XmlError::NodeError(format!("Child node {} not found on parent {}", child_id, parent_id)))?;
        self.nodes[p_idx].children.remove(pos);
        self.nodes[c_idx].parent = None;
        Ok(child_id)
    }

    /// Inserts a new child node immediately before a reference child node under the specified parent.
    pub fn insert_before(&mut self, parent_id: NodeId, new_child_id: NodeId, ref_child_id: NodeId) -> Result<()> {
        let p_idx = parent_id as usize;
        let n_idx = new_child_id as usize;
        let r_idx = ref_child_id as usize;
        if p_idx >= self.nodes.len() || n_idx >= self.nodes.len() || r_idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        self.validate_hierarchy(parent_id, new_child_id)?;
        if let Some(old_parent) = self.nodes[n_idx].parent {
            self.remove_child(old_parent, new_child_id)?;
        }
        let pos = self.nodes[p_idx]
            .children
            .iter()
            .position(|&id| id == ref_child_id)
            .ok_or_else(|| XmlError::NodeError(format!("Reference child {} not found on parent {}", ref_child_id, parent_id)))?;
        self.nodes[p_idx].children.insert(pos, new_child_id);
        self.nodes[n_idx].parent = Some(parent_id);
        Ok(())
    }

    /// Replaces an existing child node with a new child node under the specified parent.
    pub fn replace_child(&mut self, parent_id: NodeId, new_child_id: NodeId, old_child_id: NodeId) -> Result<NodeId> {
        let p_idx = parent_id as usize;
        let n_idx = new_child_id as usize;
        let o_idx = old_child_id as usize;
        if p_idx >= self.nodes.len() || n_idx >= self.nodes.len() || o_idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        self.validate_hierarchy(parent_id, new_child_id)?;
        if let Some(old_parent) = self.nodes[n_idx].parent {
            self.remove_child(old_parent, new_child_id)?;
        }
        let pos = self.nodes[p_idx]
            .children
            .iter()
            .position(|&id| id == old_child_id)
            .ok_or_else(|| XmlError::NodeError(format!("Old child {} not found on parent {}", old_child_id, parent_id)))?;
        self.nodes[p_idx].children[pos] = new_child_id;
        self.nodes[o_idx].parent = None;
        self.nodes[n_idx].parent = Some(parent_id);
        Ok(old_child_id)
    }

    /// Detaches a node from its parent if one exists.
    pub fn detach(&mut self, node_id: NodeId) -> Result<()> {
        let n_idx = node_id as usize;
        if n_idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        if let Some(parent_id) = self.nodes[n_idx].parent {
            self.remove_child(parent_id, node_id)?;
        }
        Ok(())
    }

    /// Detaches a node from its parent and unlinks it from the DOM tree.
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<()> {
        self.detach(node_id)
    }

    /// Creates a new unattached element node in the document arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use xml_lib_rust::Document;
    ///
    /// let mut doc = Document::new();
    /// let elem = doc.create_element("item");
    /// assert_eq!(doc.get_node(elem).unwrap().kind.name(), "item");
    /// ```
    pub fn create_element(&mut self, name: impl Into<Box<str>>) -> NodeId {
        self.add_node(NodeKind::Element {
            name: name.into(),
            attributes: Vec::new(),
        })
    }

    /// Creates a new unattached text node in the document arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use xml_lib_rust::Document;
    ///
    /// let mut doc = Document::new();
    /// let text = doc.create_text_node("Hello, XML!");
    /// assert_eq!(doc.get_text_content(text), "Hello, XML!");
    /// ```
    pub fn create_text_node(&mut self, text: impl Into<Box<str>>) -> NodeId {
        self.add_node(NodeKind::Text(text.into()))
    }

    /// Sets or updates an attribute on an [`NodeKind::Element`] node.
    pub fn set_attribute(&mut self, elem_id: NodeId, name: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Result<()> {
        let idx = elem_id as usize;
        if idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        if !self.nodes[idx].kind.set_attribute(name, value) {
            return Err(XmlError::NodeError(format!("Node {} is not an Element", elem_id)));
        }
        Ok(())
    }

    /// Removes an attribute by name from an [`NodeKind::Element`] node. Returns `true` if removed.
    pub fn remove_attribute(&mut self, elem_id: NodeId, name: &str) -> bool {
        let idx = elem_id as usize;
        if idx >= self.nodes.len() {
            return false;
        }
        self.nodes[idx].kind.remove_attribute(name)
    }

    /// Checks if an [`NodeKind::Element`] node has an attribute with the given name.
    pub fn has_attribute(&self, elem_id: NodeId, name: &str) -> bool {
        self.get_attribute(elem_id, name).is_some()
    }

    /// Sets the text content of a node.
    /// For an Element, this removes all existing child nodes and replaces them with a single Text node.
    /// For Text/CData/Comment, this updates the content in place.
    pub fn set_text_content(&mut self, node_id: NodeId, text: impl Into<Box<str>>) -> Result<()> {
        let idx = node_id as usize;
        if idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        let text_box = text.into();
        match &mut self.nodes[idx].kind {
            NodeKind::Text(t) | NodeKind::CData(t) | NodeKind::Comment(t) => {
                *t = text_box;
                Ok(())
            }
            NodeKind::Element { .. } => {
                let old_children = self.nodes[idx].children.clone();
                for child_id in old_children {
                    self.nodes[child_id as usize].parent = None;
                }
                self.nodes[idx].children.clear();
                let text_node_id = self.add_node(NodeKind::Text(text_box));
                self.append_child(node_id, text_node_id)?;
                Ok(())
            }
            _ => Err(XmlError::NodeError("Cannot set text content on this node type".into())),
        }
    }

    /// Returns the first child node ID of a given node, if any.
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id)?.children.first().copied()
    }

    /// Returns the last child node ID of a given node, if any.
    pub fn last_child(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id)?.children.last().copied()
    }

    /// Returns the next sibling node ID of a given node, if any.
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        let parent_id = self.parent_id(id)?;
        let parent = self.get_node(parent_id)?;
        let pos = parent.children.iter().position(|&child| child == id)?;
        parent.children.get(pos + 1).copied()
    }

    /// Returns the previous sibling node ID of a given node, if any.
    pub fn previous_sibling(&self, id: NodeId) -> Option<NodeId> {
        let parent_id = self.parent_id(id)?;
        let parent = self.get_node(parent_id)?;
        let pos = parent.children.iter().position(|&child| child == id)?;
        if pos > 0 {
            parent.children.get(pos - 1).copied()
        } else {
            None
        }
    }

    /// Returns the first child node ID that is an [`NodeKind::Element`].
    pub fn first_element_child(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id)?
            .children
            .iter()
            .copied()
            .find(|&c_id| self.get_node(c_id).map_or(false, |c| c.kind.is_element()))
    }

    /// Returns the last child node ID that is an [`NodeKind::Element`].
    pub fn last_element_child(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id)?
            .children
            .iter()
            .copied()
            .rfind(|&c_id| self.get_node(c_id).map_or(false, |c| c.kind.is_element()))
    }

    /// Creates a clone of a node, optionally cloning its child subtree recursively (`deep = true`).
    /// Cloned nodes are inserted into the arena as unlinked root nodes (`parent = None`).
    pub fn clone_node(&mut self, node_id: NodeId, deep: bool) -> Result<NodeId> {
        let idx = node_id as usize;
        if idx >= self.nodes.len() {
            return Err(XmlError::NodeError("Invalid Node ID".into()));
        }
        let kind = self.nodes[idx].kind.clone();
        let new_id = self.add_node(kind);

        if deep {
            let children = self.nodes[idx].children.clone();
            for child_id in children {
                let cloned_child = self.clone_node(child_id, true)?;
                self.append_child(new_id, cloned_child)?;
            }
        }

        Ok(new_id)
    }

    /// Returns all descendant [`NodeKind::Element`] node IDs with a given tag name (or all elements if `name == "*"`).
    pub fn get_elements_by_tag_name(&self, name: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        let match_all = name == "*";

        for node in &self.nodes {
            if let NodeKind::Element { name: elem_name, .. } = &node.kind {
                if match_all || &**elem_name == name {
                    results.push(node.id);
                }
            }
        }
        results
    }

    /// Finds the first element with an attribute `id` matching the specified string.
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        self.nodes.iter().find_map(|node| {
            if node.kind.is_element() && node.kind.get_attribute("id") == Some(id) {
                Some(node.id)
            } else {
                None
            }
        })
    }

    /// Returns the namespace prefix for a node if present (e.g. `"xs"` in `<xs:element>`).
    pub fn get_prefix(&self, id: NodeId) -> Option<&str> {
        let node = self.get_node(id)?;
        crate::namespace::QName::split_prefix(node.kind.name()).0
    }

    /// Returns the local name for a node (e.g. `"element"` in `<xs:element>`).
    pub fn get_local_name(&self, id: NodeId) -> &str {
        if let Some(node) = self.get_node(id) {
            crate::namespace::QName::split_prefix(node.kind.name()).1
        } else {
            ""
        }
    }

    /// Resolves the active namespace URI for a given element node by walking up ancestor declaration scopes.
    pub fn get_namespace_uri(&self, id: NodeId) -> Option<String> {
        let prefix = self.get_prefix(id);
        if let Some(p) = prefix {
            if p == "xml" {
                return Some("http://www.w3.org/XML/1998/namespace".to_string());
            }
            if p == "xmlns" {
                return Some("http://www.w3.org/2000/xmlns/".to_string());
            }
            self.lookup_namespace_uri(id, p)
        } else {
            self.lookup_namespace_uri(id, "")
        }
    }

    /// Finds the namespace prefix mapped to the given URI in the scope of the specified node.
    pub fn lookup_prefix(&self, id: NodeId, uri: &str) -> Option<String> {
        let mut curr = Some(id);
        while let Some(nid) = curr {
            if let Some(node) = self.get_node(nid) {
                if let NodeKind::Element { attributes, .. } = &node.kind {
                    for attr in attributes {
                        if &*attr.value == uri {
                            if let Some(prefix) = attr.name.strip_prefix("xmlns:") {
                                return Some(prefix.to_string());
                            } else if &*attr.name == "xmlns" {
                                return Some(String::new());
                            }
                        }
                    }
                }
            }
            curr = self.parent_id(nid);
        }
        None
    }

    /// Finds the namespace URI mapped to the given prefix in the scope of the specified node.
    pub fn lookup_namespace_uri(&self, id: NodeId, prefix: &str) -> Option<String> {
        let mut curr = Some(id);
        let target_attr = if prefix.is_empty() {
            "xmlns".to_string()
        } else {
            format!("xmlns:{}", prefix)
        };

        while let Some(nid) = curr {
            if let Some(node) = self.get_node(nid) {
                if let NodeKind::Element { attributes, .. } = &node.kind {
                    for attr in attributes {
                        if &*attr.name == target_attr {
                            if attr.value.is_empty() {
                                return None;
                            }
                            return Some(attr.value.to_string());
                        }
                    }
                }
            }
            curr = self.parent_id(nid);
        }
        None
    }

    /// Returns all descendant [`NodeKind::Element`] node IDs matching a namespace URI and local name.
    /// Supports `"*"` as wildcard for either `uri` or `local_name`.
    pub fn get_elements_by_tag_name_ns(&self, uri: &str, local_name: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        let match_any_uri = uri == "*";
        let match_any_local = local_name == "*";

        for node in &self.nodes {
            if node.kind.is_element() {
                let elem_local = self.get_local_name(node.id);
                let local_matches = match_any_local || elem_local == local_name;
                if local_matches {
                    let elem_uri = self.get_namespace_uri(node.id);
                    let uri_matches = match_any_uri
                        || match (uri, elem_uri.as_deref()) {
                            ("", None) => true,
                            (u, Some(eu)) => u == eu,
                            _ => false,
                        };
                    if uri_matches {
                        results.push(node.id);
                    }
                }
            }
        }
        results
    }

    /// Garbage collects unreferenced nodes and compacts the arena vector.
    /// Traverses all reachable nodes starting from virtual containers (Root and Prolog),
    /// remaps all `NodeId` identifiers, and rebuilds the internal arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use xml_lib_rust::Document;
    ///
    /// let mut doc = Document::parse_str("<root><a/><b/></root>").unwrap();
    /// let b_id = doc.get_elements_by_tag_name("b")[0];
    /// doc.remove_node(b_id).unwrap();
    /// doc.compact().unwrap();
    /// assert_eq!(doc.get_elements_by_tag_name("b").len(), 0);
    /// ```
    pub fn compact(&mut self) -> Result<()> {
        let mut reachable = vec![false; self.nodes.len()];
        let mut stack = Vec::new();

        if let Some(r) = self.root_id {
            stack.push(r);
        }
        if let Some(p) = self.prolog_id {
            stack.push(p);
        }

        while let Some(nid) = stack.pop() {
            let idx = nid as usize;
            if idx < self.nodes.len() && !reachable[idx] {
                reachable[idx] = true;
                if let Some(node) = self.nodes.get(idx) {
                    for &child in &node.children {
                        stack.push(child);
                    }
                }
            }
        }

        let mut id_map = vec![None; self.nodes.len()];
        let mut new_nodes = Vec::new();

        for (old_idx, &is_reachable) in reachable.iter().enumerate() {
            if is_reachable {
                if new_nodes.len() >= (NodeId::MAX as usize) {
                    return Err(XmlError::NodeError("Compacted arena exceeds NodeId capacity".into()));
                }
                let new_id = new_nodes.len() as NodeId;
                id_map[old_idx] = Some(new_id);
                new_nodes.push(self.nodes[old_idx].clone());
            }
        }

        for node in &mut new_nodes {
            let old_id = node.id as usize;
            if let Some(Some(remapped)) = id_map.get(old_id) {
                node.id = *remapped;
            }
            node.parent = node.parent.and_then(|p| id_map.get(p as usize).copied().flatten());
            node.children = node
                .children
                .iter()
                .filter_map(|&c| id_map.get(c as usize).copied().flatten())
                .collect();
        }

        self.root_id = self.root_id.and_then(|r| id_map.get(r as usize).copied().flatten());
        self.prolog_id = self.prolog_id.and_then(|p| id_map.get(p as usize).copied().flatten());
        self.declaration_id = self.declaration_id.and_then(|d| id_map.get(d as usize).copied().flatten());
        self.dtd_id = self.dtd_id.and_then(|d| id_map.get(d as usize).copied().flatten());

        self.nodes = new_nodes;
        Ok(())
    }

    /// Immutable lookup for a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(id as usize)
    }

    /// Mutable lookup for a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(id as usize)
    }

    /// Returns the ID of the root virtual container node.
    pub fn root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    /// Returns the ID of the Prolog container node.
    pub fn prolog_id(&self) -> Option<NodeId> {
        self.prolog_id
    }

    /// Returns the ID of the XML Declaration node if present.
    pub fn declaration_id(&self) -> Option<NodeId> {
        self.declaration_id
    }

    /// Sets the XML Declaration node ID.
    pub fn set_declaration_id(&mut self, id: NodeId) {
        self.declaration_id = Some(id);
    }

    /// Returns the ID of the DTD DOCTYPE node if present.
    pub fn dtd_id(&self) -> Option<NodeId> {
        self.dtd_id
    }

    /// Sets the DTD DOCTYPE node ID.
    pub fn set_dtd_id(&mut self, id: NodeId) {
        self.dtd_id = Some(id);
    }

    /// Returns the ID of the primary document element node (first child Element of Root).
    pub fn root_element_id(&self) -> Option<NodeId> {
        let root_id = self.root_id?;
        let root_node = self.get_node(root_id)?;
        for &child_id in &root_node.children {
            if let Some(child) = self.get_node(child_id) {
                if matches!(child.kind, NodeKind::Element { .. }) {
                    return Some(child_id);
                }
            }
        }
        None
    }

    /// Returns a list of child node IDs for a given parent node ID.
    pub fn get_children(&self, id: NodeId) -> Vec<NodeId> {
        self.get_node(id).map(|n| n.children.clone()).unwrap_or_default()
    }

    /// Returns a list of child Node IDs that are [`NodeKind::Element`] variants.
    pub fn get_element_children(&self, id: NodeId) -> Vec<NodeId> {
        self.get_children(id)
            .into_iter()
            .filter(|&c_id| self.get_node(c_id).map_or(false, |c| matches!(c.kind, NodeKind::Element { .. })))
            .collect()
    }

    /// Looks up an attribute value by name for a given node ID.
    pub fn get_attribute<'a>(&'a self, id: NodeId, attr_name: &str) -> Option<&'a str> {
        self.get_node(id).and_then(|node| node.kind.get_attribute(attr_name))
    }

    /// Returns the parent node ID for a given node ID.
    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id).and_then(|n| n.parent)
    }

    /// Recursively extracts concatenated text content for a given node and its descendants.
    pub fn get_text_content(&self, id: NodeId) -> String {
        let mut text = String::new();
        if let Some(node) = self.get_node(id) {
            match &node.kind {
                NodeKind::Text(t) => text.push_str(t),
                _ => {
                    for &child_id in &node.children {
                        if let Some(child) = self.get_node(child_id) {
                            match &child.kind {
                                NodeKind::Text(t) => text.push_str(t),
                                NodeKind::Element { .. } => text.push_str(&self.get_text_content(child_id)),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        text
    }

    /// Returns a slice of all nodes stored in the arena.
    pub fn nodes(&self) -> &[NodeData] {
        &self.nodes
    }

    /// Returns the total number of nodes stored in the arena.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the arena contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
