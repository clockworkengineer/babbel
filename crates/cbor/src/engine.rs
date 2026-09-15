//! CBOR Format Engine adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// CBOR format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CborEngine;

impl FormatEngine for CborEngine {
    fn format_id(&self) -> &'static str {
        "cbor"
    }

    fn mime_type(&self) -> &'static str {
        "application/cbor"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["cbor"]
    }

    fn is_binary(&self) -> bool {
        true
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let bytes = babbel_core::io::read_all_bytes(source);
        self.parse_bytes(&bytes)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        crate::parser::from_bytes(input)
            .map_err(|err| BabbelError::from(err).with_format("cbor"))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        crate::serializer::serialize_to_dest(value, destination)
            .map_err(|err| BabbelError::from(err).with_format("cbor"))
    }
}
