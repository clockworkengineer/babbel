//! # XSD Schema Validator
//!
//! Parses XSD schema documents (`xs:schema`) and validates DOM documents against elements, types, and restriction facets.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use crate::validator::XmlValidator;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// XSD simple type restriction facets.
#[derive(Debug, Clone, Default)]
pub struct XsdRestriction {
    /// Minimum inclusive numeric bound (`xs:minInclusive`).
    pub min_inclusive: Option<i64>,
    /// Maximum inclusive numeric bound (`xs:maxInclusive`).
    pub max_inclusive: Option<i64>,
    /// Minimum string character length (`xs:minLength`).
    pub min_length: Option<usize>,
    /// Maximum string character length (`xs:maxLength`).
    pub max_length: Option<usize>,
    /// Allowed enumeration values (`xs:enumeration`).
    pub enumerations: Vec<String>,
    /// Pattern regex/string restriction (`xs:pattern`).
    pub pattern: Option<String>,
}

/// Model group compositors supported by XSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compositor {
    /// `<xs:sequence>`
    Sequence,
    /// `<xs:choice>`
    Choice,
    /// `<xs:all>`
    All,
}

/// Attribute declaration rule in XSD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XsdAttributeRule {
    /// Attribute name.
    pub name: String,
    /// Attribute type (`xs:string`, `xs:integer`, etc.).
    pub attr_type: String,
    /// Whether attribute is required (`use="required"`).
    pub required: bool,
    /// Default literal value if any.
    pub default: Option<String>,
}

/// Complex type definition containing compositors, child elements, and attributes.
#[derive(Debug, Clone, Default)]
pub struct XsdComplexType {
    /// Type name (empty for anonymous/inline complex types).
    pub name: String,
    /// Model compositor (`Sequence`, `Choice`, or `All`).
    pub compositor: Option<Compositor>,
    /// Child element rules within the compositor.
    pub elements: Vec<XsdElementRule>,
    /// Attribute rules on the element.
    pub attributes: Vec<XsdAttributeRule>,
}

/// XSD schema element declaration rule.
#[derive(Debug, Clone)]
pub struct XsdElementRule {
    /// Element name.
    pub name: String,
    /// Type declaration (`xs:string`, `xs:integer`, `xs:boolean`, or complex type name).
    pub elem_type: String,
    /// Minimum occurrences.
    pub min_occurs: usize,
    /// Maximum occurrences (None = unbounded).
    pub max_occurs: Option<usize>,
    /// Associated restriction facets.
    pub restriction: XsdRestriction,
    /// Associated inline or referenced complex type.
    pub complex_type: Option<XsdComplexType>,
}

/// XSD Schema Validator.
///
/// # Examples
///
/// ```
/// use xml_lib_rust::{parse, XsdValidator};
///
/// let schema = r#"
/// <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
///   <xs:element name="note" type="xs:string"/>
/// </xs:schema>
/// "#;
/// let mut validator = XsdValidator::new();
/// validator.parse_schema(schema).unwrap();
/// let doc = parse("<note>hello</note>").unwrap();
/// assert!(validator.validate(&doc).is_ok());
/// ```
#[derive(Debug, Clone, Default)]
pub struct XsdValidator {
    elements: HashMap<String, XsdElementRule>,
    complex_types: HashMap<String, XsdComplexType>,
}

impl XsdValidator {
    /// Instantiates a new [`XsdValidator`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses an XSD schema string slice (`<xs:schema ...>`).
    pub fn parse_schema(&mut self, schema_xml: &str) -> Result<()> {
        let source = crate::io::source::XmlSource::from_string(schema_xml);
        let options = crate::options::ParseOptions::default();
        let mut parser = crate::parser::XmlParser::new(source, options);
        let schema_doc = parser.parse()?;

        if let Some(root_elem_id) = schema_doc.root_element_id() {
            self.collect_schema_components(&schema_doc, root_elem_id);
        }
        Ok(())
    }

    fn parse_occurs(min_str: Option<&str>, max_str: Option<&str>) -> (usize, Option<usize>) {
        let min = min_str.and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
        let max = match max_str {
            Some("unbounded") => None,
            Some(s) => s.parse::<usize>().ok(),
            None => Some(1),
        };
        (min, max)
    }

    fn collect_schema_components(&mut self, schema_doc: &Document, root_id: NodeId) {
        if let Some(root_node) = schema_doc.get_node(root_id) {
            for &c_id in &root_node.children {
                if let Some(c) = schema_doc.get_node(c_id) {
                    if let NodeKind::Element { name, .. } = &c.kind {
                        let local = name.split(':').last().unwrap_or(name);
                        if local == "complexType" {
                            if let Some(ct_name) = schema_doc.get_attribute(c_id, "name") {
                                let mut ct = self.parse_complex_type(schema_doc, c_id);
                                ct.name = ct_name.to_string();
                                for elem in &ct.elements {
                                    self.elements.entry(elem.name.clone()).or_insert_with(|| elem.clone());
                                }
                                self.complex_types.insert(ct_name.to_string(), ct);
                            }
                        } else if local == "element" {
                            let rule = self.parse_element_node(schema_doc, c_id);
                            if !rule.name.is_empty() {
                                if let Some(ct) = &rule.complex_type {
                                    for elem in &ct.elements {
                                        self.elements.entry(elem.name.clone()).or_insert_with(|| elem.clone());
                                    }
                                }
                                self.elements.insert(rule.name.clone(), rule);
                            }
                        }
                    }
                }
            }
        }
    }

    fn parse_element_node(&self, schema_doc: &Document, node_id: NodeId) -> XsdElementRule {
        let elem_name = schema_doc.get_attribute(node_id, "name").unwrap_or_default().to_string();
        let elem_type = schema_doc.get_attribute(node_id, "type").unwrap_or("").to_string();
        let min_str = schema_doc.get_attribute(node_id, "minOccurs");
        let max_str = schema_doc.get_attribute(node_id, "maxOccurs");
        let (min_occurs, max_occurs) = Self::parse_occurs(min_str, max_str);

        let restriction = self.extract_restriction(schema_doc, node_id);

        let mut complex_type = None;
        if let Some(node) = schema_doc.get_node(node_id) {
            for &c_id in &node.children {
                if let Some(c) = schema_doc.get_node(c_id) {
                    if let NodeKind::Element { name, .. } = &c.kind {
                        if name.split(':').last().unwrap_or(name) == "complexType" {
                            complex_type = Some(self.parse_complex_type(schema_doc, c_id));
                            break;
                        }
                    }
                }
            }
        }

        XsdElementRule {
            name: elem_name,
            elem_type: if elem_type.is_empty() { "xs:string".to_string() } else { elem_type },
            min_occurs,
            max_occurs,
            restriction,
            complex_type,
        }
    }

    fn parse_complex_type(&self, schema_doc: &Document, ct_id: NodeId) -> XsdComplexType {
        let mut ct = XsdComplexType::default();

        if let Some(node) = schema_doc.get_node(ct_id) {
            for &c_id in &node.children {
                if let Some(c) = schema_doc.get_node(c_id) {
                    if let NodeKind::Element { name, .. } = &c.kind {
                        let local = name.split(':').last().unwrap_or(name);
                        match local {
                            "sequence" => {
                                ct.compositor = Some(Compositor::Sequence);
                                ct.elements = self.collect_group_elements(schema_doc, c_id);
                            }
                            "choice" => {
                                ct.compositor = Some(Compositor::Choice);
                                ct.elements = self.collect_group_elements(schema_doc, c_id);
                            }
                            "all" => {
                                ct.compositor = Some(Compositor::All);
                                ct.elements = self.collect_group_elements(schema_doc, c_id);
                            }
                            "attribute" => {
                                ct.attributes.push(self.parse_attribute_node(schema_doc, c_id));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        ct
    }

    fn collect_group_elements(&self, schema_doc: &Document, group_id: NodeId) -> Vec<XsdElementRule> {
        let mut elements = Vec::new();
        if let Some(node) = schema_doc.get_node(group_id) {
            for &c_id in &node.children {
                if let Some(c) = schema_doc.get_node(c_id) {
                    if let NodeKind::Element { name, .. } = &c.kind {
                        if name.split(':').last().unwrap_or(name) == "element" {
                            elements.push(self.parse_element_node(schema_doc, c_id));
                        }
                    }
                }
            }
        }
        elements
    }

    fn parse_attribute_node(&self, schema_doc: &Document, attr_id: NodeId) -> XsdAttributeRule {
        let name = schema_doc.get_attribute(attr_id, "name").unwrap_or_default().to_string();
        let attr_type = schema_doc.get_attribute(attr_id, "type").unwrap_or("xs:string").to_string();
        let use_val = schema_doc.get_attribute(attr_id, "use").unwrap_or("optional");
        let default = schema_doc.get_attribute(attr_id, "default").map(|s| s.to_string());

        XsdAttributeRule {
            name,
            attr_type,
            required: use_val == "required",
            default,
        }
    }

    fn extract_restriction(&self, doc: &Document, elem_id: NodeId) -> XsdRestriction {
        let mut rest = XsdRestriction::default();
        let mut stack = vec![elem_id];

        while let Some(nid) = stack.pop() {
            if let Some(node) = doc.get_node(nid) {
                if let NodeKind::Element { name, .. } = &node.kind {
                    let local_name = name.split(':').last().unwrap_or(name);
                    let get_val = || doc.get_attribute(nid, "value");

                    match local_name {
                        "minInclusive" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<i64>().ok()) {
                                rest.min_inclusive = Some(v);
                            }
                        }
                        "maxInclusive" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<i64>().ok()) {
                                rest.max_inclusive = Some(v);
                            }
                        }
                        "minLength" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<usize>().ok()) {
                                rest.min_length = Some(v);
                            }
                        }
                        "maxLength" => {
                            if let Some(v) = get_val().and_then(|s| s.parse::<usize>().ok()) {
                                rest.max_length = Some(v);
                            }
                        }
                        "enumeration" => {
                            if let Some(v) = get_val() {
                                rest.enumerations.push(v.to_string());
                            }
                        }
                        "pattern" => {
                            if let Some(v) = get_val() {
                                rest.pattern = Some(v.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                stack.extend(node.children.iter().copied());
            }
        }
        rest
    }

    /// Maximum element nesting depth allowed during validation (512 frames).
    pub const MAX_VALIDATION_DEPTH: usize = 512;

    /// Validates a DOM [`Document`] against loaded XSD schema rules.
    pub fn validate(&self, doc: &Document) -> Result<()> {
        if let Some(root_id) = doc.root_element_id() {
            self.validate_element(doc, root_id, 0)?;
        }
        Ok(())
    }

    fn validate_element(&self, doc: &Document, elem_id: NodeId, depth: usize) -> Result<()> {
        if depth > Self::MAX_VALIDATION_DEPTH {
            return Err(XmlError::XsdError(
                "Document exceeds maximum validation nesting depth (512)".into(),
            ));
        }
        let node = doc
            .get_node(elem_id)
            .ok_or_else(|| XmlError::XsdError("Invalid node".into()))?;

        if let NodeKind::Element { name, .. } = &node.kind {
            if let Some(rule) = self.elements.get(&**name) {
                // Resolve complex type
                let ct_opt = rule.complex_type.as_ref().or_else(|| {
                    self.complex_types.get(&rule.elem_type)
                });

                if let Some(ct) = ct_opt {
                    // Attribute validation
                    for attr_rule in &ct.attributes {
                        let attr_val = doc.get_attribute(elem_id, &attr_rule.name);
                        if attr_rule.required && attr_val.is_none() {
                            return Err(XmlError::XsdError(format!(
                                "Required attribute '{}' missing on <{name}>",
                                attr_rule.name
                            )));
                        }
                        if let Some(val) = attr_val {
                            if attr_rule.attr_type == "xs:integer" && val.parse::<i64>().is_err() {
                                return Err(XmlError::XsdError(format!(
                                    "Attribute '{}' on <{name}> value '{val}' is not a valid integer",
                                    attr_rule.name
                                )));
                            }
                        }
                    }

                    // Child elements and compositor validation
                    let child_elements: Vec<(NodeId, String)> = node.children.iter().filter_map(|&c_id| {
                        doc.get_node(c_id).and_then(|c| match &c.kind {
                            NodeKind::Element { name: child_name, .. } => Some((c_id, child_name.to_string())),
                            _ => None,
                        })
                    }).collect();

                    match ct.compositor {
                        Some(Compositor::Choice) => {
                            if child_elements.is_empty() && ct.elements.iter().any(|e| e.min_occurs > 0) {
                                return Err(XmlError::XsdError(format!(
                                    "Element <{name}> requires at least one choice child element"
                                )));
                            }
                            for (_, c_name) in &child_elements {
                                if !ct.elements.iter().any(|e| e.name == *c_name) {
                                    return Err(XmlError::XsdError(format!(
                                        "Child element <{c_name}> is not allowed in choice group of <{name}>"
                                    )));
                                }
                            }
                        }
                        Some(Compositor::Sequence) => {
                            for expected in &ct.elements {
                                let count = child_elements.iter().filter(|(_, cn)| *cn == expected.name).count();
                                if count < expected.min_occurs {
                                    return Err(XmlError::XsdError(format!(
                                        "Element <{name}> missing required sequence child <{}> (expected {}, found {})",
                                        expected.name, expected.min_occurs, count
                                    )));
                                }
                                if let Some(max) = expected.max_occurs {
                                    if count > max {
                                        return Err(XmlError::XsdError(format!(
                                            "Element <{name}> exceeds maxOccurs ({max}) for child <{}> (found {count})",
                                            expected.name
                                        )));
                                    }
                                }
                            }
                        }
                        Some(Compositor::All) => {
                            for expected in &ct.elements {
                                let count = child_elements.iter().filter(|(_, cn)| *cn == expected.name).count();
                                if count < expected.min_occurs {
                                    return Err(XmlError::XsdError(format!(
                                        "Element <{name}> missing required child <{}> in xs:all group",
                                        expected.name
                                    )));
                                }
                                if count > expected.max_occurs.unwrap_or(1) {
                                    return Err(XmlError::XsdError(format!(
                                        "Element <{name}> exceeds maxOccurs for child <{}> in xs:all group",
                                        expected.name
                                    )));
                                }
                            }
                        }
                        None => {}
                    }
                }

                // Simple type restrictions
                let text = self.get_element_text(doc, elem_id);
                let trimmed = text.trim();

                // Primitive Type Validation
                if rule.elem_type == "xs:integer" || rule.elem_type == "xsd:integer" {
                    if !trimmed.is_empty() {
                        let parsed = trimmed.parse::<i64>().map_err(|_| {
                            XmlError::XsdError(format!("Element <{name}> value '{trimmed}' is not a valid integer"))
                        })?;

                        if let Some(min) = rule.restriction.min_inclusive {
                            if parsed < min {
                                return Err(XmlError::XsdError(format!(
                                    "Element <{name}> value '{parsed}' violates minInclusive ({min})"
                                )));
                            }
                        }
                        if let Some(max) = rule.restriction.max_inclusive {
                            if parsed > max {
                                return Err(XmlError::XsdError(format!(
                                    "Element <{name}> value '{parsed}' violates maxInclusive ({max})"
                                )));
                            }
                        }
                    }
                } else if rule.elem_type == "xs:boolean" || rule.elem_type == "xsd:boolean" {
                    if !trimmed.is_empty() && trimmed != "true" && trimmed != "false" && trimmed != "1" && trimmed != "0" {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> value '{trimmed}' is not a valid boolean"
                        )));
                    }
                }

                // String Restrictions
                if let Some(min_len) = rule.restriction.min_length {
                    if trimmed.chars().count() < min_len {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> length violates minLength ({min_len})"
                        )));
                    }
                }
                if let Some(max_len) = rule.restriction.max_length {
                    if trimmed.chars().count() > max_len {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> length violates maxLength ({max_len})"
                        )));
                    }
                }
                if !rule.restriction.enumerations.is_empty() {
                    if !rule.restriction.enumerations.iter().any(|e| e == trimmed) {
                        return Err(XmlError::XsdError(format!(
                            "Element <{name}> value '{trimmed}' not in enumeration list"
                        )));
                    }
                }
            }

            for &c_id in &node.children {
                if let Some(c) = doc.get_node(c_id) {
                    if matches!(c.kind, NodeKind::Element { .. }) {
                        self.validate_element(doc, c_id, depth + 1)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn get_element_text(&self, doc: &Document, elem_id: NodeId) -> String {
        let mut text = String::new();
        if let Some(node) = doc.get_node(elem_id) {
            for &c_id in &node.children {
                if let Some(c) = doc.get_node(c_id) {
                    if let NodeKind::Text(t) = &c.kind {
                        text.push_str(t);
                    }
                }
            }
        }
        text
    }
}

impl XmlValidator for XsdValidator {
    fn validate(&self, doc: &Document) -> Result<()> {
        self.validate(doc)
    }
}
