//! `FormatEngine` implementation for Apache Avro.

use babbel_core::{BabbelError, FormatEngine, FormatOptions, Value};
use babbel_core::io::IDestination;
use crate::{from_bytes, to_vec};

/// Format engine for Apache Avro binary payloads (`.avro`).
#[derive(Debug, Clone, Copy, Default)]
pub struct AvroEngine;

impl FormatEngine for AvroEngine {
    fn format_id(&self) -> &'static str {
        "avro"
    }

    fn mime_type(&self) -> &'static str {
        "application/avro"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["avro"]
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        self.parse_bytes(input.as_bytes())
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        if input.starts_with(&crate::OCF_MAGIC) {
            crate::from_bytes_ocf(input).map_err(Into::into)
        } else {
            from_bytes(input).map_err(Into::into)
        }
    }

    fn serialize(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let bytes = to_vec(value).map_err(BabbelError::from)?;
        dest.add_raw_bytes(&bytes);
        Ok(())
    }

    fn is_binary(&self) -> bool {
        true
    }
}
