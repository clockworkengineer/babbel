//! `FormatEngine` implementation for HCL v2.

use crate::{from_str, to_string, to_string_pretty};
use babbel_core::io::IDestination;
use babbel_core::{BabbelError, FormatEngine, FormatOptions, Value};

/// Format engine for HashiCorp HCL v2 (`.hcl`, `.tf`, `.tfvars`).
#[derive(Debug, Clone, Copy, Default)]
pub struct HclEngine;

impl FormatEngine for HclEngine {
    fn format_id(&self) -> &'static str {
        "hcl"
    }

    fn mime_type(&self) -> &'static str {
        "application/x-hcl"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["hcl", "tf", "tfvars"]
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        from_str(input).map_err(Into::into)
    }

    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        let s = core::str::from_utf8(input)
            .map_err(|_| BabbelError::encoding("invalid UTF-8 in HCL input").with_format("hcl"))?;
        self.parse_str(s)
    }

    fn serialize(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let s = if options.pretty {
            to_string_pretty(value, options.indent).map_err(BabbelError::from)?
        } else {
            to_string(value).map_err(BabbelError::from)?
        };
        dest.add_bytes(&s);
        Ok(())
    }

    fn is_binary(&self) -> bool {
        false
    }
}
