//! RON Format Engine adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

use crate::serializer::RonSerializerConfig;

/// RON format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RonEngine;

impl FormatEngine for RonEngine {
    fn format_id(&self) -> &'static str {
        "ron"
    }

    fn mime_type(&self) -> &'static str {
        "application/ron"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["ron"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut text = alloc::string::String::new();
        while source.more() {
            if let Some(ch) = source.current() {
                text.push(ch);
            }
            source.next();
        }
        self.parse_str(&text)
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        crate::parser::from_str(input)
            .map_err(|err| BabbelError::from(err).with_format("ron"))
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        crate::parser::from_bytes(input)
            .map_err(|err| BabbelError::from(err).with_format("ron"))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let config = RonSerializerConfig {
            pretty: options.pretty,
            indent_spaces: options.indent,
        };
        crate::serializer::serialize_to_dest(value, destination, &config)
            .map_err(|err| BabbelError::from(err).with_format("ron"))
    }
}
