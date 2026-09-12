//! XML Format Engine adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

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
        Ok(doc.to_value())
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

