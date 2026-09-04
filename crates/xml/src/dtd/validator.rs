//! # DTD Validation Engine
//!
//! Parses DTD internal subsets (`<!ELEMENT>`, `<!ATTLIST>`) and validates document structure and required attributes.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use crate::validator::XmlValidator;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Representation of DTD element content models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentModel {
    /// Declared `EMPTY` (no child elements or text content allowed).
    Empty,
    /// Declared `ANY` (any element or text child allowed).
    Any,
    /// Mixed content model (`(#PCDATA | a | b)*`).
    Mixed(Vec<String>),
    /// Child element sequence model.
    Children(String),
}

/// DTD element content declaration rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtdElementRule {
    /// Element tag name.
    pub name: String,
    /// Content model rule.
    pub model: ContentModel,
}

/// DTD attribute declaration rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DtdAttributeRule {
    /// Associated element tag name.
    pub element_name: String,
    /// Attribute key name.
    pub attr_name: String,
    /// Attribute type (`CDATA`, `ID`, etc.).
    pub attr_type: String,
    /// Default declaration (`#REQUIRED`, `#IMPLIED`, `#FIXED`).
    pub default_decl: String,
    /// Extracted default literal value, if any.
    pub default_value: Option<String>,
}

/// External DTD subset resolution callback.
pub type ExternalSubsetResolver = Arc<dyn Fn(&str, Option<&str>) -> Option<String> + Send + Sync>;

/// DTD validator enforcing element content model, attribute constraints, and ID/IDREF integrity.
///
/// # Examples
///
/// ```
/// use xml_lib_rust::{parse, DtdValidator};
///
/// let mut validator = DtdValidator::new();
/// let dtd = "<!ELEMENT root (item*)>\n<!ELEMENT item EMPTY>";
/// validator.parse_subset(dtd).unwrap();
/// let doc = parse("<root><item/></root>").unwrap();
/// assert!(validator.validate(&doc).is_ok());
/// ```
#[derive(Clone, Default)]
pub struct DtdValidator {
    elements: HashMap<String, DtdElementRule>,
    attributes: HashMap<String, Vec<DtdAttributeRule>>,
    external_resolver: Option<ExternalSubsetResolver>,
}

impl core::fmt::Debug for DtdValidator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DtdValidator")
            .field("elements", &self.elements)
            .field("attributes", &self.attributes)
            .field("has_external_resolver", &self.external_resolver.is_some())
            .finish()
    }
}

impl DtdValidator {
    /// Instantiates a new empty [`DtdValidator`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom resolver for external DTD subsets (`SYSTEM` and optional `PUBLIC` IDs).
    pub fn set_external_resolver<F>(&mut self, resolver: F)
    where
        F: Fn(&str, Option<&str>) -> Option<String> + Send + Sync + 'static,
    {
        self.external_resolver = Some(Arc::new(resolver));
    }

    /// Parses internal DTD subset string (`<!ELEMENT ...>` and `<!ATTLIST ...>`).
    pub fn parse_subset(&mut self, subset: &str) -> Result<()> {
        let lines = subset.lines();
        for raw_line in lines {
            let line = raw_line.trim();
            if line.starts_with("<!ELEMENT") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let elem_name = parts[1].to_string();
                    let model_str = parts[2..].join(" ");
                    let model = if model_str.contains("EMPTY") {
                        ContentModel::Empty
                    } else if model_str.contains("ANY") {
                        ContentModel::Any
                    } else {
                        ContentModel::Children(model_str)
                    };
                    self.elements.insert(
                        elem_name.clone(),
                        DtdElementRule {
                            name: elem_name,
                            model,
                        },
                    );
                }
            } else if line.starts_with("<!ATTLIST") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let elem_name = parts[1].to_string();
                    let attr_name = parts[2].to_string();
                    let attr_type = parts[3].to_string();
                    let (default_decl, default_value) = if parts.len() >= 5 {
                        let raw_decl = parts[4..].join(" ").trim_end_matches('>').trim().to_string();
                        let def_val = if raw_decl.contains("#FIXED") {
                            raw_decl.split_whitespace().nth(1).map(|v| v.trim_matches(|c| c == '"' || c == '\'').to_string())
                        } else if raw_decl.starts_with('"') || raw_decl.starts_with('\'') {
                            Some(raw_decl.trim_matches(|c| c == '"' || c == '\'').to_string())
                        } else {
                            None
                        };
                        (raw_decl, def_val)
                    } else {
                        ("#IMPLIED".to_string(), None)
                    };
                    self.attributes.entry(elem_name.clone()).or_default().push(
                        DtdAttributeRule {
                            element_name: elem_name,
                            attr_name,
                            attr_type,
                            default_decl,
                            default_value,
                        },
                    );
                }
            }
        }
        Ok(())
    }

    /// Injects default attribute values defined in DTD into element nodes where the attribute is absent.
    ///
    /// Returns the total number of attributes injected.
    pub fn apply_defaults(&self, doc: &mut Document) -> Result<usize> {
        let mut count = 0;
        if let Some(root_id) = doc.root_id() {
            let mut stack = vec![root_id];
            let mut to_insert = Vec::new();

            while let Some(nid) = stack.pop() {
                if let Some(node) = doc.get_node(nid) {
                    if let NodeKind::Element { name, attributes } = &node.kind {
                        if let Some(rules) = self.attributes.get(&**name) {
                            for rule in rules {
                                if let Some(def_val) = &rule.default_value {
                                    if !attributes.iter().any(|a| *a.name == rule.attr_name) {
                                        to_insert.push((nid, rule.attr_name.clone(), def_val.clone()));
                                    }
                                }
                            }
                        }
                    }
                    stack.extend(node.children.iter().copied());
                }
            }

            for (nid, attr_name, attr_val) in to_insert {
                doc.set_attribute(nid, attr_name, attr_val)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Validates a [`Document`] against loaded DTD rules.
    pub fn validate(&self, doc: &Document) -> Result<()> {
        if let Some(dtd_id) = doc.dtd_id() {
            if let Some(node) = doc.get_node(dtd_id) {
                if let NodeKind::DocTypeDefinition {
                    name: _,
                    public_id,
                    system_id,
                    internal_subset,
                } = &node.kind
                {
                    let mut mut_self = self.clone();
                    if let Some(sys) = system_id {
                        if let Some(resolver) = &self.external_resolver {
                            if let Some(ext_subset) = resolver(sys, public_id.as_deref()) {
                                mut_self.parse_subset(&ext_subset)?;
                            }
                        }
                    }
                    if let Some(subset) = internal_subset {
                        mut_self.parse_subset(subset)?;
                    }
                    return mut_self.validate_doc(doc);
                }
            }
        }
        self.validate_doc(doc)
    }

    /// Maximum element nesting depth allowed during validation (512 frames).
    pub const MAX_VALIDATION_DEPTH: usize = 512;

    fn validate_doc(&self, doc: &Document) -> Result<()> {
        let mut id_set = HashMap::new();
        let mut idref_list = Vec::new();

        if let Some(root_elem_id) = doc.root_element_id() {
            self.collect_ids_and_idrefs(doc, root_elem_id, &mut id_set, &mut idref_list, 0)?;
            self.validate_element(doc, root_elem_id, 0)?;
        }

        for (ref_val, elem_name, attr_name) in idref_list {
            if !id_set.contains_key(&ref_val) {
                return Err(XmlError::DtdError(format!(
                    "IDREF '{ref_val}' in attribute '{attr_name}' of <{elem_name}> does not match any declared ID"
                )));
            }
        }

        Ok(())
    }

    fn collect_ids_and_idrefs(
        &self,
        doc: &Document,
        elem_id: NodeId,
        id_set: &mut HashMap<String, NodeId>,
        idref_list: &mut Vec<(String, String, String)>,
        depth: usize,
    ) -> Result<()> {
        if depth > Self::MAX_VALIDATION_DEPTH {
            return Err(XmlError::DtdError(
                "Document exceeds maximum validation nesting depth (512)".into(),
            ));
        }

        if let Some(node) = doc.get_node(elem_id) {
            if let NodeKind::Element { name, attributes } = &node.kind {
                if let Some(attr_rules) = self.attributes.get(&**name) {
                    for attr in attributes {
                        if let Some(rule) = attr_rules.iter().find(|r| r.attr_name == *attr.name) {
                            if rule.attr_type == "ID" {
                                let id_val = attr.value.to_string();
                                if id_set.contains_key(&id_val) {
                                    return Err(XmlError::DtdError(format!(
                                        "Duplicate ID value '{id_val}' in element <{name}>"
                                    )));
                                }
                                id_set.insert(id_val, elem_id);
                            } else if rule.attr_type == "IDREF" {
                                idref_list.push((attr.value.to_string(), name.to_string(), attr.name.to_string()));
                            } else if rule.attr_type == "IDREFS" {
                                for single_ref in attr.value.split_whitespace() {
                                    idref_list.push((single_ref.to_string(), name.to_string(), attr.name.to_string()));
                                }
                            }
                        }
                    }
                }
            }

            for &child_id in &node.children {
                if let Some(child) = doc.get_node(child_id) {
                    if matches!(child.kind, NodeKind::Element { .. }) {
                        self.collect_ids_and_idrefs(doc, child_id, id_set, idref_list, depth + 1)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_element(&self, doc: &Document, elem_id: NodeId, depth: usize) -> Result<()> {
        if depth > Self::MAX_VALIDATION_DEPTH {
            return Err(XmlError::DtdError(
                "Document exceeds maximum validation nesting depth (512)".into(),
            ));
        }
        let node = doc
            .get_node(elem_id)
            .ok_or_else(|| XmlError::DtdError("Invalid node".into()))?;

        if let NodeKind::Element { name, attributes } = &node.kind {
            // Check Element Rule
            if let Some(rule) = self.elements.get(&**name) {
                match &rule.model {
                    ContentModel::Empty => {
                        let has_child_elems = node.children.iter().any(|&c_id| {
                            doc.get_node(c_id).map_or(false, |c| {
                                matches!(c.kind, NodeKind::Element { .. } | NodeKind::Text(_))
                            })
                        });
                        if has_child_elems {
                            return Err(XmlError::DtdError(format!(
                                "Element <{name}> is declared EMPTY but contains child content"
                            )));
                        }
                    }
                    ContentModel::Children(spec) => {
                        let child_names: Vec<String> = node.children.iter().filter_map(|&c_id| {
                            doc.get_node(c_id).and_then(|c| match &c.kind {
                                NodeKind::Element { name, .. } => Some(name.to_string()),
                                _ => None,
                            })
                        }).collect();

                        if spec.contains(',') && !spec.contains('*') && !spec.contains('?') {
                            let required_names: Vec<String> = spec
                                .trim_matches(|c| c == '(' || c == ')' || c == '>' || c == ' ')
                                .split(',')
                                .map(|s| s.trim().trim_matches('>').trim().to_string())
                                .filter(|s| !s.is_empty() && !s.starts_with('#'))
                                .collect();

                            for req in &required_names {
                                if !child_names.contains(req) {
                                    return Err(XmlError::DtdError(format!(
                                        "Element <{name}> does not conform to content specification {spec}: missing required child <{req}>"
                                    )));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check Attribute Rules
            if let Some(attr_rules) = self.attributes.get(&**name) {
                for rule in attr_rules {
                    if rule.default_decl.contains("#REQUIRED") {
                        let present = attributes.iter().any(|a| *a.name == rule.attr_name);
                        if !present {
                            return Err(XmlError::DtdError(format!(
                                "Required attribute '{}' missing on element <{name}>",
                                rule.attr_name
                            )));
                        }
                    }
                }
            }

            // Recurse on child elements
            for &child_id in &node.children {
                if let Some(child) = doc.get_node(child_id) {
                    if matches!(child.kind, NodeKind::Element { .. }) {
                        self.validate_element(doc, child_id, depth + 1)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl XmlValidator for DtdValidator {
    fn validate(&self, doc: &Document) -> Result<()> {
        self.validate(doc)
    }
}
