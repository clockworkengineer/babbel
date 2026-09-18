//! MongoDB Extended JSON v2 (Canonical & Relaxed) support.
//!
//! Conforms to the official MongoDB Extended JSON specification:
//! - Canonical Extended JSON preserves exact BSON type fidelity (e.g. `{"$numberLong": "123"}`, `{"$oid": "..."}`, `{"$binary": ...}`).
//! - Relaxed Extended JSON outputs standard JSON primitives where lossless (e.g. regular numbers instead of string wrappers).

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use crate::error::BsonError;
use babbel_core::Value;

/// Extended JSON output mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EJsonMode {
    /// Canonical Extended JSON format (strict BSON type representations).
    Canonical,
    /// Relaxed Extended JSON format (standard JSON types where lossless).
    Relaxed,
}

/// Convert a universal Babbel [`Value`] into an Extended JSON [`Value`].
pub fn to_extended_json(val: &Value, mode: EJsonMode) -> Value {
    match val {
        Value::Null => Value::Null,
        Value::Bool(b) => Value::Bool(*b),
        Value::Integer(i) => match mode {
            EJsonMode::Relaxed => {
                // If it fits safely in standard 32-bit integer range, leave as Integer
                if *i >= i32::MIN as i128 && *i <= i32::MAX as i128 {
                    Value::Integer(*i)
                } else {
                    Value::Object(vec![(
                        "$numberLong".to_string(),
                        Value::String(i.to_string()),
                    )])
                }
            }
            EJsonMode::Canonical => Value::Object(vec![(
                "$numberLong".to_string(),
                Value::String(i.to_string()),
            )]),
        },
        Value::Float(f) => match mode {
            EJsonMode::Relaxed => {
                if f.is_nan() {
                    Value::Object(vec![(
                        "$numberDouble".to_string(),
                        Value::String("NaN".to_string()),
                    )])
                } else if f.is_infinite() {
                    let s = if *f > 0.0 { "Infinity" } else { "-Infinity" };
                    Value::Object(vec![(
                        "$numberDouble".to_string(),
                        Value::String(s.to_string()),
                    )])
                } else {
                    Value::Float(*f)
                }
            }
            EJsonMode::Canonical => {
                let s = if f.is_nan() {
                    "NaN".to_string()
                } else if f.is_infinite() {
                    if *f > 0.0 {
                        "Infinity".to_string()
                    } else {
                        "-Infinity".to_string()
                    }
                } else {
                    format!("{:?}", f)
                };
                Value::Object(vec![("$numberDouble".to_string(), Value::String(s))])
            }
        },
        Value::String(s) => Value::String(s.clone()),
        Value::Bytes(b) => {
            // Hex/Base64 binary wrapper: {"$binary": {"base64": "...", "subType": "00"}}
            let hex_chars: String = b.iter().map(|byte| format!("{:02x}", byte)).collect();
            Value::Object(vec![(
                "$binary".to_string(),
                Value::Object(vec![
                    ("hex".to_string(), Value::String(hex_chars)),
                    ("subType".to_string(), Value::String("00".to_string())),
                ]),
            )])
        }
        Value::Array(items) => {
            let transformed: Vec<Value> = items
                .iter()
                .map(|item| to_extended_json(item, mode))
                .collect();
            Value::Array(transformed)
        }
        Value::Object(entries) => {
            let transformed: Vec<(String, Value)> = entries
                .iter()
                .map(|(k, v)| (k.clone(), to_extended_json(v, mode)))
                .collect();
            Value::Object(transformed)
        }
    }
}

/// Convert an Extended JSON AST back into a canonical Babbel [`Value`].
pub fn from_extended_json(val: &Value) -> Result<Value, BsonError> {
    match val {
        Value::Object(entries) if entries.len() == 1 => {
            let (k, v) = &entries[0];
            match k.as_str() {
                "$oid" => {
                    if let Value::String(s) = v {
                        let bytes = hex_to_bytes(s)?;
                        return Ok(Value::Bytes(bytes));
                    }
                }
                "$numberLong" | "$numberInt" => {
                    if let Value::String(s) = v {
                        if let Ok(i) = s.parse::<i128>() {
                            return Ok(Value::Integer(i));
                        }
                    } else if let Value::Integer(i) = v {
                        return Ok(Value::Integer(*i));
                    }
                }
                "$numberDouble" => {
                    if let Value::String(s) = v {
                        match s.as_str() {
                            "NaN" => return Ok(Value::Float(f64::NAN)),
                            "Infinity" => return Ok(Value::Float(f64::INFINITY)),
                            "-Infinity" => return Ok(Value::Float(f64::NEG_INFINITY)),
                            other => {
                                if let Ok(f) = other.parse::<f64>() {
                                    return Ok(Value::Float(f));
                                }
                            }
                        }
                    } else if let Value::Float(f) = v {
                        return Ok(Value::Float(*f));
                    }
                }
                "$date" => {
                    if let Value::Object(date_entries) = v {
                        for (dk, dv) in date_entries {
                            if dk == "$numberLong" {
                                if let Value::String(s) = dv {
                                    if let Ok(ms) = s.parse::<i128>() {
                                        return Ok(Value::Integer(ms));
                                    }
                                }
                            }
                        }
                    } else if let Value::Integer(ms) = v {
                        return Ok(Value::Integer(*ms));
                    }
                }
                "$binary" => {
                    if let Value::Object(bin_entries) = v {
                        for (bk, bv) in bin_entries {
                            if bk == "hex" {
                                if let Value::String(hex_str) = bv {
                                    let bytes = hex_to_bytes(hex_str)?;
                                    return Ok(Value::Bytes(bytes));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            // Fallback object recursion if not a recognized single-key extended json wrapper
            let mut res = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                res.push((k.clone(), from_extended_json(v)?));
            }
            Ok(Value::Object(res))
        }
        Value::Object(entries) => {
            let mut res = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                res.push((k.clone(), from_extended_json(v)?));
            }
            Ok(Value::Object(res))
        }
        Value::Array(items) => {
            let mut res = Vec::with_capacity(items.len());
            for item in items {
                res.push(from_extended_json(item)?);
            }
            Ok(Value::Array(res))
        }
        other => Ok(other.clone()),
    }
}

fn hex_to_bytes(s: &str) -> Result<Vec<u8>, BsonError> {
    if s.len() % 2 != 0 {
        return Err(BsonError::Custom(
            "invalid hex string length for extended json",
        ));
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let chars: Vec<char> = s.chars().collect();
    for chunk in chars.chunks(2) {
        let hex_str: String = chunk.iter().collect();
        let b = u8::from_str_radix(&hex_str, 16)
            .map_err(|_| BsonError::Custom("invalid hex character in extended json"))?;
        bytes.push(b);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ejson_roundtrip() {
        let val = Value::Integer(9876543210);
        let ejson = to_extended_json(&val, EJsonMode::Canonical);
        assert_eq!(
            ejson,
            Value::Object(vec![(
                "$numberLong".to_string(),
                Value::String("9876543210".to_string())
            )])
        );
        let back = from_extended_json(&ejson).unwrap();
        assert_eq!(back, val);
    }
}
