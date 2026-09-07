//! Extensible format parser, emitter, and codec traits adhering to OCP and DIP.


use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;

/// Abstract parser converting text or byte inputs into a universal `Value` AST.
pub trait FormatParser {
    /// Parses a UTF-8 string into a universal `Value`.
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError>;

    /// Parses a raw byte slice into a universal `Value`.
    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        match core::str::from_utf8(input) {
            Ok(s) => self.parse_str(s),
            Err(_) => Err(BabbelError::encoding("input is not valid UTF-8")),
        }
    }
}

/// Abstract emitter writing a universal `Value` AST to an output destination.
pub trait FormatEmitter {
    /// Serializes a `Value` to the output destination in compact/standard format.
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError>;

    /// Serializes a `Value` to the output destination with pretty printing.
    fn emit_pretty(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        indent: usize,
    ) -> Result<(), BabbelError> {
        let _ = indent;
        self.emit(value, dest)
    }
}

/// A combined format codec providing both parsing and emission capabilities.
pub trait FormatCodec: FormatParser + FormatEmitter {
    /// Returns the name of the format (e.g., "json", "yaml", "xml", "bencode").
    fn format_name(&self) -> &'static str;
}

/// Standard built-in JSON format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonEmitter;

impl FormatEmitter for JsonEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_json(dest);
        Ok(())
    }
}

/// Standard built-in YAML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlEmitter;

impl FormatEmitter for YamlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_yaml(dest, 0);
        Ok(())
    }

    fn emit_pretty(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        indent: usize,
    ) -> Result<(), BabbelError> {
        value.serialize_yaml(dest, indent);
        Ok(())
    }
}

/// Standard built-in XML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct XmlEmitter;

impl FormatEmitter for XmlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_xml(dest, None);
        Ok(())
    }
}

/// Standard built-in Bencode format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct BencodeEmitter;

impl FormatEmitter for BencodeEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_bencode(dest);
        Ok(())
    }
}
