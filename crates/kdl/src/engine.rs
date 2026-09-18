//! KDL Format Engine implementation adhering to OCP and DIP.

use babbel_core::{BabbelError, FormatEngine, FormatOptions, Value, io::IDestination};

use crate::ast::KdlDocument;
use crate::serializer::serialize_document;

/// KDL format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct KdlEngine;

impl FormatEngine for KdlEngine {
    fn format_id(&self) -> &'static str {
        "kdl"
    }

    fn mime_type(&self) -> &'static str {
        "application/kdl"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["kdl"]
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        crate::from_str(input).map_err(|err| BabbelError::from(err).with_format("kdl"))
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        crate::from_bytes(input).map_err(|err| BabbelError::from(err).with_format("kdl"))
    }

    fn serialize(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let doc = KdlDocument::from_value(value);
        serialize_document(&doc, dest, options.pretty, options.indent)?;
        Ok(())
    }
}
