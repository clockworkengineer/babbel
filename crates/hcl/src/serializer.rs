//! Serializer emitting universal `Value` AST into HCL v2 syntax.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

use babbel_core::Value;
use crate::error::HclError;

/// Serializer configuration for HCL emission.
#[derive(Debug, Clone, Copy)]
pub struct HclSerializerConfig {
    /// Number of spaces per indentation level.
    pub indent_step: usize,
}

impl Default for HclSerializerConfig {
    fn default() -> Self {
        Self { indent_step: 2 }
    }
}

/// Serialize a universal `Value` into an HCL formatted string.
pub fn to_string(value: &Value) -> Result<String, HclError> {
    to_string_pretty(value, 2)
}

/// Serialize a universal `Value` into an HCL formatted string with custom indentation.
pub fn to_string_pretty(value: &Value, indent: usize) -> Result<String, HclError> {
    let mut out = String::new();
    let config = HclSerializerConfig { indent_step: indent };
    emit_value_root(value, &mut out, 0, config)?;
    Ok(out)
}

fn emit_value_root(
    val: &Value,
    out: &mut String,
    depth: usize,
    config: HclSerializerConfig,
) -> Result<(), HclError> {
    match val {
        Value::Object(entries) => {
            for (k, v) in entries {
                emit_indent(out, depth, config.indent_step);
                match v {
                    Value::Object(sub_entries) => {
                        // Check if block or attribute
                        let is_block = !sub_entries.is_empty();
                        if is_block {
                            out.push_str(k);
                            out.push_str(" {\n");
                            emit_value_root(v, out, depth + 1, config)?;
                            emit_indent(out, depth, config.indent_step);
                            out.push_str("}\n");
                        } else {
                            out.push_str(k);
                            out.push_str(" = {}\n");
                        }
                    }
                    other => {
                        out.push_str(k);
                        out.push_str(" = ");
                        emit_expr(other, out, depth, config)?;
                        out.push('\n');
                    }
                }
            }
        }
        other => {
            emit_expr(other, out, depth, config)?;
            out.push('\n');
        }
    }
    Ok(())
}

fn emit_expr(
    val: &Value,
    out: &mut String,
    depth: usize,
    config: HclSerializerConfig,
) -> Result<(), HclError> {
    match val {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Integer(i) => out.push_str(&i.to_string()),
        Value::Float(f) => {
            if f.is_nan() {
                out.push_str("null");
            } else if f.is_infinite() {
                out.push_str(if *f > 0.0 { "1e999" } else { "-1e999" });
            } else {
                let mut s = format!("{:?}", f);
                if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                    s.push_str(".0");
                }
                out.push_str(&s);
            }
        }
        Value::String(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    other => out.push(other),
                }
            }
            out.push('"');
        }
        Value::Bytes(b) => {
            out.push('"');
            for byte in b {
                out.push_str(&format!("{:02x}", byte));
            }
            out.push('"');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_expr(item, out, depth, config)?;
            }
            out.push(']');
        }
        Value::Object(entries) => {
            out.push_str("{\n");
            for (k, v) in entries {
                emit_indent(out, depth + 1, config.indent_step);
                out.push_str(k);
                out.push_str(" = ");
                emit_expr(v, out, depth + 1, config)?;
                out.push('\n');
            }
            emit_indent(out, depth, config.indent_step);
            out.push('}');
        }
    }
    Ok(())
}

fn emit_indent(out: &mut String, depth: usize, step: usize) {
    for _ in 0..(depth * step) {
        out.push(' ');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_hcl() {
        let val = Value::Object(vec![
            ("instance_type".into(), Value::String("t3.micro".into())),
            ("count".into(), Value::Integer(2)),
            ("tags".into(), Value::Array(vec![Value::String("web".into()), Value::String("prod".into())])),
        ]);

        let hcl_str = to_string(&val).unwrap();
        assert!(hcl_str.contains("instance_type = \"t3.micro\""));
        assert!(hcl_str.contains("count = 2"));
    }
}
