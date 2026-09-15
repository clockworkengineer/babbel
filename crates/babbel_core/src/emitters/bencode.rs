use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;
use crate::codec::FormatEmitter;

/// Standard built-in Bencode format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BencodeEmitter;

impl BencodeEmitter {
    /// Serializes value to Bencode representation into `dest`.
    pub fn emit_to_dest(&self, value: &Value, dest: &mut dyn IDestination) {
        match value {
            Value::Null => {}
            Value::Bool(b) => {
                dest.add_bytes(if *b { "i1e" } else { "i0e" });
            }
            Value::Integer(i) => {
                dest.add_bytes("i");
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(*i));
                dest.add_bytes("e");
            }
            Value::Float(f) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes("i");
                let rounded = if *f >= 0.0 { (*f + 0.5) as i64 } else { (*f - 0.5) as i64 };
                dest.add_bytes(buf.format(rounded));
                dest.add_bytes("e");
            }
            Value::String(s) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(s.len()));
                dest.add_bytes(":");
                dest.add_bytes(s);
            }
            Value::Bytes(b) => {
                let mut buf = itoa::Buffer::new();
                dest.add_bytes(buf.format(b.len()));
                dest.add_bytes(":");
                if let Ok(s) = core::str::from_utf8(b) {
                    dest.add_bytes(s);
                } else {
                    for &byte in b {
                        dest.add_byte(byte);
                    }
                }
            }
            Value::Array(items) => {
                dest.add_bytes("l");
                for item in items {
                    self.emit_to_dest(item, dest);
                }
                dest.add_bytes("e");
            }
            Value::Object(entries) => {
                dest.add_bytes("d");
                for (k, v) in entries {
                    let mut buf = itoa::Buffer::new();
                    dest.add_bytes(buf.format(k.len()));
                    dest.add_bytes(":");
                    dest.add_bytes(k);
                    self.emit_to_dest(v, dest);
                }
                dest.add_bytes("e");
            }
        }
    }
}

impl FormatEmitter for BencodeEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        self.emit_to_dest(value, dest);
        Ok(())
    }
}
