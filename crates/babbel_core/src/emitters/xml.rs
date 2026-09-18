use crate::codec::FormatEmitter;
use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;

/// Standard built-in XML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct XmlEmitter;

impl XmlEmitter {
    /// Serializes value to XML representation into `dest` with an optional root tag.
    pub fn emit_with_tag(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        root_tag: Option<&str>,
    ) {
        let tag = root_tag.unwrap_or("root");
        match value {
            Value::Null => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes("/>");
            }
            Value::Bool(b) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                dest.add_bytes(if *b { "true" } else { "false" });
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Integer(i) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Float(f) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                let mut buf = dtoa::Buffer::new();
                dest.add_bytes(buf.format(*f));
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::String(s) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                crate::escape::write_xml_escaped_string(s, dest);
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Bytes(b) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                crate::escape::write_xml_escaped_string(
                    &alloc::string::String::from_utf8_lossy(b),
                    dest,
                );
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Array(items) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                for item in items {
                    self.emit_with_tag(item, dest, Some("item"));
                }
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
            Value::Object(entries) => {
                dest.add_bytes("<");
                dest.add_bytes(tag);
                dest.add_bytes(">");
                for (k, v) in entries {
                    self.emit_with_tag(v, dest, Some(k));
                }
                dest.add_bytes("</");
                dest.add_bytes(tag);
                dest.add_bytes(">");
            }
        }
    }
}

impl FormatEmitter for XmlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        self.emit_with_tag(value, dest, None);
        Ok(())
    }
}
