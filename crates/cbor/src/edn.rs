//! RFC 8949 Section 8 Extended Diagnostic Notation (EDN) for CBOR.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use babbel_core::Value;
use crate::error::CborError;

/// Formats a universal [`Value`] into CBOR Extended Diagnostic Notation (EDN).
pub fn to_edn(val: &Value) -> String {
    let mut out = String::new();
    format_edn(val, &mut out);
    out
}

fn format_edn(val: &Value, out: &mut String) {
    match val {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Integer(i) => {
            use core::fmt::Write;
            let _ = write!(out, "{}", i);
        }
        Value::Float(f) => {
            if f.is_nan() {
                out.push_str("NaN");
            } else if *f == f64::INFINITY {
                out.push_str("Infinity");
            } else if *f == f64::NEG_INFINITY {
                out.push_str("-Infinity");
            } else {
                use core::fmt::Write;
                let _ = write!(out, "{}", f);
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
            out.push_str("h'");
            for byte in b {
                out.push_str(&format!("{:02x}", byte));
            }
            out.push('\'');
        }
        Value::Array(items) => {
            out.push('[');
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                format_edn(item, out);
            }
            out.push(']');
        }
        Value::Object(entries) => {
            out.push('{');
            for (idx, (k, v)) in entries.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                out.push_str(k);
                out.push_str("\": ");
                format_edn(v, out);
            }
            out.push('}');
        }
    }
}

/// Parses a basic CBOR Extended Diagnostic Notation (EDN) string into a universal [`Value`].
pub fn from_edn(input: &str) -> Result<Value, CborError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CborError::UnexpectedEof { expected: 1, available: 0 });
    }

    if trimmed == "null" || trimmed == "undefined" {
        return Ok(Value::Null);
    }
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }
    if trimmed == "NaN" {
        return Ok(Value::Float(f64::NAN));
    }
    if trimmed == "Infinity" {
        return Ok(Value::Float(f64::INFINITY));
    }
    if trimmed == "-Infinity" {
        return Ok(Value::Float(f64::NEG_INFINITY));
    }

    // Hex byte string: h'...'
    if trimmed.starts_with("h'") && trimmed.ends_with('\'') {
        let hex_slice = &trimmed[2..trimmed.len() - 1];
        let mut bytes = Vec::with_capacity(hex_slice.len() / 2);
        let mut chars = hex_slice.chars().filter(|c| !c.is_whitespace());
        while let Some(c1) = chars.next() {
            let c2 = chars.next().ok_or_else(|| CborError::Custom("odd hex length in h''"))?;
            let b1 = c1.to_digit(16).ok_or_else(|| CborError::Custom("invalid hex digit"))?;
            let b2 = c2.to_digit(16).ok_or_else(|| CborError::Custom("invalid hex digit"))?;
            bytes.push(((b1 << 4) | b2) as u8);
        }
        return Ok(Value::Bytes(bytes));
    }

    // Numbers
    if let Ok(i) = trimmed.parse::<i128>() {
        return Ok(Value::Integer(i));
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Quoted strings
    if (trimmed.starts_with('"') && trimmed.ends_with('"')) || (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        let unquoted = &trimmed[1..trimmed.len() - 1];
        return Ok(Value::String(unquoted.to_string()));
    }

    Err(CborError::Custom("unsupported or malformed EDN input"))
}
