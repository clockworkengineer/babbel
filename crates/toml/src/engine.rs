//! TOML Format Engine adhering to OCP and DIP.

use alloc::string::String;
use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// TOML format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct TomlEngine;

impl FormatEngine for TomlEngine {
    fn format_id(&self) -> &'static str {
        "toml"
    }

    fn mime_type(&self) -> &'static str {
        "application/toml"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["toml"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut s = String::new();
        while let Some(ch) = source.current() {
            s.push(ch);
            source.next();
        }
        self.parse_str(&s)
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let node = crate::from_str(input)?;
        Ok(Value::from(&node))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        value.serialize_toml(destination);
        Ok(())
    }
}
