//! Bencode Format Engine adhering to OCP and DIP.

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// Bencode format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct BencodeEngine;

impl FormatEngine for BencodeEngine {
    fn format_id(&self) -> &'static str {
        "bencode"
    }

    fn mime_type(&self) -> &'static str {
        "application/x-bencode"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["torrent", "bencode"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut bytes = Vec::new();
        while let Some(ch) = source.current() {
            bytes.push(ch as u8);
            source.next();
        }
        self.parse_bytes(&bytes)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        let node = crate::parser::default::parse_bytes(input)
            .map_err(|err| BabbelError::syntax(err).with_format("bencode"))?;
        Ok(Value::from(&node))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        value.serialize_bencode(destination);
        Ok(())
    }
}
