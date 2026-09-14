//! JSON and JSON5 Format Engines adhering to OCP and DIP.

use babbel_core::{
    io::{IDestination, ISource},
    BabbelError, FormatEngine, FormatOptions, Value,
};

/// JSON format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JsonEngine;

impl FormatEngine for JsonEngine {
    fn format_id(&self) -> &'static str {
        "json"
    }

    fn mime_type(&self) -> &'static str {
        "application/json"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["json"]
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

/// JSON Lines format engine implementing [`FormatEngine`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JsonLinesEngine;

impl FormatEngine for JsonLinesEngine {
    fn format_id(&self) -> &'static str {
        "jsonlines"
    }

    fn mime_type(&self) -> &'static str {
        "application/x-ndjson"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["jsonl", "ndjson"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut s = alloc::string::String::new();
        while source.more() {
            if let Some(ch) = source.current() {
                s.push(ch);
            }
            source.next();
        }
        self.parse_str(&s)
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let nodes = crate::lines::parse_json_lines(input)
            .map_err(|err| BabbelError::syntax(err).with_format("jsonlines"))?;
        let values: alloc::vec::Vec<Value> = nodes.iter().map(Value::from).collect();
        Ok(Value::Array(values))
    }

    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn IDestination,
        _options: &FormatOptions,
    ) -> Result<(), BabbelError> {
        let records = match value {
            Value::Array(arr) => arr.as_slice(),
            single => core::slice::from_ref(single),
        };
        for r in records {
            r.serialize_json(destination);
            destination.add_byte(b'\n');
        }
        Ok(())
    }
}

/// JSON5 and JSONC format engine implementing [`FormatEngine`].
///
/// Supports single/multi-line comments, unquoted keys, single-quoted strings,
/// trailing commas, hexadecimal numbers, explicit signs, and `Infinity`/`NaN`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Json5Engine;

impl FormatEngine for Json5Engine {
    fn format_id(&self) -> &'static str {
        "json5"
    }

    fn mime_type(&self) -> &'static str {
        "application/json5"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["json5", "jsonc"]
    }

    fn parse(&self, source: &mut dyn ISource) -> Result<Value, BabbelError> {
        let mut s = alloc::string::String::new();
        while source.more() {
            if let Some(ch) = source.current() {
                s.push(ch);
            }
            source.next();
        }
        self.parse_str(&s)
    }

    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let node = crate::parser::json5::parse_json5(input)
            .map_err(|err| BabbelError::syntax(err).with_format("json5"))?;
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
