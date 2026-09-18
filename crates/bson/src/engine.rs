//! BSON Format Engine adhering to OCP and DIP.

use babbel_core::{
    BabbelError, FormatEngine, FormatOptions, Value,
    io::{IDestination, ISource},
};

/// BSON format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BsonEngine;

impl FormatEngine for BsonEngine {
    fn format_id(&self) -> &'static str {
        "bson"
    }

    fn mime_type(&self) -> &'static str {
        "application/bson"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["bson"]
    }

    fn is_binary(&self) -> bool {
        true
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let bytes = babbel_core::io::read_all_bytes(source);
        self.parse_bytes(&bytes)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        crate::parser::from_bytes(input).map_err(|err| BabbelError::from(err).with_format("bson"))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        crate::serializer::serialize_to_dest(value, destination)
            .map_err(|err| BabbelError::from(err).with_format("bson"))
    }
}
