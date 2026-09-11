//! JSON Format Engine adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// JSON format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonEngine;

impl FormatEngine for JsonEngine {
    fn format_id(&self) -> &'static str {
        "json"
    }

    fn mime_type(&self) -> &'static str {
        "application/json"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["json", "jsonl"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let node = crate::parser::default::parse(source)
            .map_err(|err| BabbelError::syntax(err).with_format("json"))?;
        Ok(Value::from(&node))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        value.serialize_json(destination);
        Ok(())
    }
}
