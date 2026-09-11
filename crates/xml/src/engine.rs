//! XML Format Engine adhering to OCP and DIP.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};
use crate::document::Document;
use crate::node::{NodeId, NodeKind};

/// XML format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct XmlEngine;

impl FormatEngine for XmlEngine {
    fn format_id(&self) -> &'static str {
        "xml"
    }

    fn mime_type(&self) -> &'static str {
        "application/xml"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["xml"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let doc = crate::parse_source(source)?;
        if let Some(root_id) = doc.root_element_id() {
            let name = doc.get_node(root_id).map(|n| n.kind.name().to_string()).unwrap_or_else(|| "xml".into());
            let val = element_to_value(&doc, root_id);
            Ok(Value::Object(vec![(name, val)]))
        } else {
            Ok(Value::Null)
        }
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        value.serialize_xml(destination, None);
        Ok(())
    }
}

fn element_to_value(doc: &Document, id: NodeId) -> Value {
    if let Some(node) = doc.get_node(id) {
        if let NodeKind::Element { attributes, .. } = &node.kind {
            let mut entries: Vec<(String, Value)> = Vec::new();
            for attr in attributes {
                entries.push((format!("@{}", attr.name), Value::String(attr.value.to_string())));
            }
            let child_elements = doc.get_element_children(id);
            if child_elements.is_empty() {
                let text = doc.get_text_content(id);
                if !text.is_empty() {
                    if entries.is_empty() {
                        return Value::String(text);
                    } else {
                        entries.push(("#text".into(), Value::String(text)));
                    }
                }
            } else {
                for child_id in child_elements {
                    if let Some(child_node) = doc.get_node(child_id) {
                        let child_val = element_to_value(doc, child_id);
                        entries.push((child_node.kind.name().into(), child_val));
                    }
                }
            }
            return Value::Object(entries);
        }
    }
    Value::Null
}
