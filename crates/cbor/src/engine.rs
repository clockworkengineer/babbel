//! CBOR Format Engine adhering to OCP and DIP.

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

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

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut bytes = Vec::new();
        while source.more() {
            if let Some(ch) = source.current() {
                bytes.push(ch as u8);
            }
            source.next();
        }
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
