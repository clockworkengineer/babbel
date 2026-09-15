//! # XML Parser Engine
//!
//! Recursive descent XML parser turning character streams ([`XmlSource`]) into DOM trees ([`Document`]).

pub mod content;
pub mod dtd;
pub mod element;
pub mod entity;
pub mod prolog;
pub mod validation;

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::entity::EntityMapper;
use crate::error::{Result, XmlError};
use crate::io::XmlSource;
use crate::namespace::NamespaceScope;
use crate::options::ParseOptions;

/// Main recursive descent XML parser implementation.
#[derive(Debug)]
pub struct XmlParser<'a> {
    pub(crate) source: XmlSource,
    pub(crate) options: ParseOptions,
    pub(crate) entity_mapper: EntityMapper,
    pub(crate) namespace_scope: NamespaceScope,
    pub(crate) element_count: usize,
    pub(crate) total_attribute_count: usize,
    pub(crate) standalone: Option<bool>,
    pub(crate) external_entities: Vec<String>,
    pub(crate) unparsed_entities: Vec<String>,
    pub(crate) _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> XmlParser<'a> {
    /// Instantiates a new [`XmlParser`] with an input source and parse options.
    pub fn new(source: XmlSource, options: ParseOptions) -> Self {
        let mapper = EntityMapper::with_limits(
            options.max_entity_expansion_depth,
            options.max_total_entity_expansion_size,
        );
        Self {
            source,
            options,
            entity_mapper: mapper,
            namespace_scope: NamespaceScope::new(),
            element_count: 0,
            total_attribute_count: 0,
            standalone: None,
            external_entities: Vec::new(),
            unparsed_entities: Vec::new(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Parses the entire input source into a DOM [`Document`].
    pub fn parse(&mut self) -> Result<Document> {
        self.options.check_xml_size(self.source.len())?;
        let mut doc = Document::new();

        // XML 1.0 §2.8 [22]: XML declaration must appear at the very start of document if present.
        // No whitespace or comments can precede it.
        if self.source.starts_with("<?xml") {
            self.parse_declaration(&mut doc)?;
        }

        self.parse_prolog(&mut doc)?;

        let root_container_id = doc.root_id().unwrap_or(0);
        self.parse_element(&mut doc, root_container_id, 0)?;

        self.parse_epilog(&mut doc)?;

        self.source.skip_whitespace();
        if !self.source.is_eof() {
            if self.source.starts_with("<") {
                return Err(XmlError::SyntaxError {
                    message: "Multiple root elements forbidden in well-formed XML".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            } else {
                return Err(XmlError::SyntaxError {
                    message: "Unexpected trailing content after root element".into(),
                    line: self.source.line(),
                    col: self.source.col(),
                });
            }
        }
        Ok(doc)
    }
}
