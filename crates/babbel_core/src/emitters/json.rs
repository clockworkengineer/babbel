use crate::codec::FormatEmitter;
use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;

/// Standard built-in JSON format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct JsonEmitter;

impl JsonEmitter {
    /// Serializes a `Value` to JSON representation into `dest`.
    pub fn emit_to_dest(&self, value: &Value, dest: &mut dyn IDestination) {
        match value {
            Value::Null => dest.add_bytes("null"),
            Value::Bool(b) => dest.add_bytes(if *b { "true" } else { "false" }),
            Value::Integer(i) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
            }
            Value::Float(f) => {
                let mut buf = dtoa::Buffer::new();
                dest.add_bytes(buf.format(*f));
            }
            Value::String(s) => {
                crate::escape::write_json_escaped_string(s, dest);
            }
            Value::Bytes(bytes) => {
                crate::escape::write_json_escaped_string(
                    &alloc::string::String::from_utf8_lossy(bytes),
                    dest,
                );
            }
            Value::Array(items) => {
                dest.add_bytes("[");
                for (idx, item) in items.iter().enumerate() {
                    if idx > 0 {
                        dest.add_bytes(",");
                    }
                    self.emit_to_dest(item, dest);
                }
                dest.add_bytes("]");
            }
            Value::Object(entries) => {
                dest.add_bytes("{");
                for (idx, (k, v)) in entries.iter().enumerate() {
                    if idx > 0 {
                        dest.add_bytes(",");
                    }
                    crate::escape::write_json_escaped_string(k, dest);
                    dest.add_bytes(":");
                    self.emit_to_dest(v, dest);
                }
                dest.add_bytes("}");
            }
        }
    }
}

impl FormatEmitter for JsonEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        self.emit_to_dest(value, dest);
        Ok(())
    }
}
