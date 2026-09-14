//! RON (Rusty Object Notation) serializer.

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use babbel_core::{io::IDestination, Value};
use crate::error::RonError;

/// Configuration options for RON serialization.
#[derive(Debug, Clone)]
pub struct RonSerializerConfig {
    pub pretty: bool,
    pub indent_spaces: usize,
}

impl Default for RonSerializerConfig {
    fn default() -> Self {
        Self {
            pretty: false,
            indent_spaces: 2,
        }
    }
}

/// Serializes a universal `Value` AST into a RON string.
pub fn to_string(value: &Value) -> Result<String, RonError> {
    let mut dest = babbel_core::io::BufferDestination::new();
    serialize_to_dest(value, &mut dest, &RonSerializerConfig::default())?;
    dest.into_string().map_err(|_| RonError::InvalidUtf8)
}

/// Serializes a universal `Value` AST into pretty-printed RON.
pub fn to_string_pretty(value: &Value, indent: usize) -> Result<String, RonError> {
    let mut dest = babbel_core::io::BufferDestination::new();
    let config = RonSerializerConfig {
        pretty: true,
        indent_spaces: indent,
    };
    serialize_to_dest(value, &mut dest, &config)?;
    dest.into_string().map_err(|_| RonError::InvalidUtf8)
}

/// Serializes a universal `Value` AST into bytes.
pub fn to_vec(value: &Value) -> Result<Vec<u8>, RonError> {
    let mut dest = babbel_core::io::BufferDestination::new();
    serialize_to_dest(value, &mut dest, &RonSerializerConfig::default())?;
    Ok(dest.into_vec())
}

/// Serializes `value` into an [`IDestination`].
pub fn serialize_to_dest(
    value: &Value,
    dest: &mut dyn IDestination,
    config: &RonSerializerConfig,
) -> Result<(), RonError> {
    let mut serializer = RonSerializer {
        dest,
        config,
        indent_level: 0,
    };
    serializer.serialize_value(value)
}

struct RonSerializer<'a> {
    dest: &'a mut dyn IDestination,
    config: &'a RonSerializerConfig,
    indent_level: usize,
}

impl<'a> RonSerializer<'a> {
    fn write_str(&mut self, s: &str) {
        self.dest.add_bytes(s);
    }


    fn write_newline_and_indent(&mut self) {
        if self.config.pretty {
            self.dest.add_byte(b'\n');
            for _ in 0..(self.indent_level * self.config.indent_spaces) {
                self.dest.add_byte(b' ');
            }
        }
    }

    fn serialize_value(&mut self, value: &Value) -> Result<(), RonError> {
        match value {
            Value::Null => self.write_str("()"),
            Value::Bool(b) => {
                if *b {
                    self.write_str("true");
                } else {
                    self.write_str("false");
                }
            }
            Value::Integer(i) => {
                let mut buf = [0u8; 40];
                let s = format_i128(*i, &mut buf);
                self.write_str(s);
            }
            Value::Float(f) => {
                if f.is_nan() {
                    self.write_str("NaN");
                } else if f.is_infinite() {
                    if *f > 0.0 {
                        self.write_str("inf");
                    } else {
                        self.write_str("-inf");
                    }
                } else {
                    let mut s = format!("{}", f);
                    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                        s.push_str(".0");
                    }
                    self.write_str(&s);
                }
            }
            Value::String(s) => self.serialize_string(s),
            Value::Bytes(b) => self.serialize_bytes_literal(b),
            Value::Array(arr) => self.serialize_array(arr)?,
            Value::Object(entries) => self.serialize_object(entries)?,
        }
        Ok(())
    }

    fn serialize_string(&mut self, s: &str) {
        self.dest.add_byte(b'"');
        for ch in s.chars() {
            match ch {
                '"' => self.write_str("\\\""),
                '\\' => self.write_str("\\\\"),
                '\n' => self.write_str("\\n"),
                '\r' => self.write_str("\\r"),
                '\t' => self.write_str("\\t"),
                '\u{0008}' => self.write_str("\\b"),
                '\u{000C}' => self.write_str("\\f"),
                '\0' => self.write_str("\\0"),
                c if (c as u32) < 0x20 => {
                    let hex = format!("\\x{:02x}", c as u32);
                    self.write_str(&hex);
                }
                c => {
                    let mut buf = [0u8; 4];
                    let encoded = c.encode_utf8(&mut buf);
                    self.dest.add_bytes(encoded);
                }

            }
        }
        self.dest.add_byte(b'"');
    }

    fn serialize_bytes_literal(&mut self, bytes: &[u8]) {
        // Output as byte string literal: b"..."
        self.write_str("b\"");
        for &b in bytes {
            match b {
                b'"' => self.write_str("\\\""),
                b'\\' => self.write_str("\\\\"),
                b'\n' => self.write_str("\\n"),
                b'\r' => self.write_str("\\r"),
                b'\t' => self.write_str("\\t"),
                b if (0x20..0x7F).contains(&b) => self.dest.add_byte(b),
                b => {
                    let hex = format!("\\x{:02x}", b);
                    self.write_str(&hex);
                }
            }
        }
        self.dest.add_byte(b'"');
    }

    fn serialize_array(&mut self, items: &[Value]) -> Result<(), RonError> {
        if items.is_empty() {
            self.write_str("[]");
            return Ok(());
        }

        self.dest.add_byte(b'[');
        self.indent_level += 1;

        for (i, item) in items.iter().enumerate() {
            if self.config.pretty {
                self.write_newline_and_indent();
            } else if i > 0 {
                self.write_str(", ");
            }

            self.serialize_value(item)?;

            if self.config.pretty {
                self.dest.add_byte(b',');
            }
        }

        self.indent_level -= 1;
        if self.config.pretty {
            self.write_newline_and_indent();
        }
        self.dest.add_byte(b']');
        Ok(())
    }

    fn serialize_object(&mut self, entries: &[(String, Value)]) -> Result<(), RonError> {
        if entries.is_empty() {
            self.write_str("()");
            return Ok(());
        }

        let all_idents = entries.iter().all(|(k, _)| is_valid_ident(k));

        if all_idents {
            // Struct syntax: `( field: value, ... )`
            self.dest.add_byte(b'(');
            self.indent_level += 1;

            for (i, (key, val)) in entries.iter().enumerate() {
                if self.config.pretty {
                    self.write_newline_and_indent();
                } else if i > 0 {
                    self.write_str(", ");
                }

                self.write_str(key);
                self.write_str(": ");
                self.serialize_value(val)?;

                if self.config.pretty {
                    self.dest.add_byte(b',');
                }
            }

            self.indent_level -= 1;
            if self.config.pretty {
                self.write_newline_and_indent();
            }
            self.dest.add_byte(b')');
        } else {
            // Map syntax: `{ "key": value, ... }`
            self.dest.add_byte(b'{');
            self.indent_level += 1;

            for (i, (key, val)) in entries.iter().enumerate() {
                if self.config.pretty {
                    self.write_newline_and_indent();
                } else if i > 0 {
                    self.write_str(", ");
                }

                self.serialize_string(key);
                self.write_str(": ");
                self.serialize_value(val)?;

                if self.config.pretty {
                    self.dest.add_byte(b',');
                }
            }

            self.indent_level -= 1;
            if self.config.pretty {
                self.write_newline_and_indent();
            }
            self.dest.add_byte(b'}');
        }
        Ok(())
    }
}

fn format_i128<'a>(val: i128, buf: &'a mut [u8; 40]) -> &'a str {
    use core::fmt::Write;
    struct BufferWriter<'b> {
        buf: &'b mut [u8],
        pos: usize,
    }
    impl<'b> Write for BufferWriter<'b> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let end = self.pos + bytes.len();
            if end <= self.buf.len() {
                self.buf[self.pos..end].copy_from_slice(bytes);
                self.pos = end;
                Ok(())
            } else {
                Err(core::fmt::Error)
            }
        }
    }

    let mut writer = BufferWriter { buf, pos: 0 };
    let _ = write!(writer, "{}", val);
    let len = writer.pos;
    core::str::from_utf8(&buf[..len]).unwrap_or("0")
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {
            chars.all(|c| c.is_alphanumeric() || c == '_')
        }
        _ => false,
    }
}
