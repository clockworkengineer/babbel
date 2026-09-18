use crate::codec::FormatEmitter;
use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;

/// Standard built-in YAML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct YamlEmitter;

impl YamlEmitter {
    /// Serializes value to YAML representation into `dest` with indentation.
    pub fn emit_with_indent(&self, value: &Value, dest: &mut dyn IDestination, indent: usize) {
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
                if s.contains('\n') || s.contains('"') {
                    dest.add_bytes("|\n");
                    for line in s.lines() {
                        for _ in 0..(indent + 2) {
                            dest.add_byte(b' ');
                        }
                        dest.add_bytes(line);
                        dest.add_bytes("\n");
                    }
                } else {
                    dest.add_bytes(s);
                }
            }
            Value::Bytes(b) => {
                dest.add_bytes(&alloc::string::String::from_utf8_lossy(b));
            }
            Value::Array(items) => {
                if items.is_empty() {
                    dest.add_bytes("[]");
                    return;
                }
                dest.add_bytes("\n");
                for item in items {
                    for _ in 0..indent {
                        dest.add_byte(b' ');
                    }
                    dest.add_bytes("- ");
                    self.emit_with_indent(item, dest, indent + 2);
                    dest.add_bytes("\n");
                }
            }
            Value::Object(entries) => {
                if entries.is_empty() {
                    dest.add_bytes("{}");
                    return;
                }
                dest.add_bytes("\n");
                for (k, v) in entries {
                    for _ in 0..indent {
                        dest.add_byte(b' ');
                    }
                    dest.add_bytes(k);
                    dest.add_bytes(": ");
                    self.emit_with_indent(v, dest, indent + 2);
                    dest.add_bytes("\n");
                }
            }
        }
    }
}

impl FormatEmitter for YamlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        self.emit_with_indent(value, dest, 0);
        Ok(())
    }

    fn emit_pretty(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        indent: usize,
    ) -> Result<(), BabbelError> {
        self.emit_with_indent(value, dest, indent);
        Ok(())
    }
}
