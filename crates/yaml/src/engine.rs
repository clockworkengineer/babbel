//! YAML Format Engine adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// YAML format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlEngine;

impl FormatEngine for YamlEngine {
    fn format_id(&self) -> &'static str {
        "yaml"
    }

    fn mime_type(&self) -> &'static str {
        "application/yaml"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["yaml", "yml"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut buf = alloc::string::String::new();
        while let Some(ch) = source.current() {
            buf.push(ch);
            source.next();
        }
        self.parse_str(&buf)
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let mut yaml_src = crate::io::sources::buffer::Buffer::new(input.as_bytes());
        let node = crate::parser::document::parse(&mut yaml_src)?;
        Ok(Value::from(&node))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let indent = if options.pretty { options.indent } else { 0 };
        value.serialize_yaml(destination, indent);
        Ok(())
    }
}
