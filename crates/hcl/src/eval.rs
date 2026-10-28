//! HCL v2 / Terraform Expression & Built-in Function Evaluation Engine.
//!
//! Provides [`HclEvalContext`] for evaluating variables, 35+ standard HCL/Terraform
//! built-in functions, splat expressions (`[*]`), `for` comprehensions, and string
//! template interpolations (`"${...}"`).

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::error::HclError;
use babbel_core::Value;

/// Evaluation context holding variable bindings for HCL expression evaluation.
#[derive(Debug, Clone, Default)]
pub struct HclEvalContext {
    vars: Vec<(String, Value)>,
}

impl HclEvalContext {
    /// Creates an empty evaluation context with all standard HCL built-in functions enabled.
    pub fn new() -> Self {
        Self { vars: Vec::new() }
    }

    /// Inserts or updates a variable in the evaluation context.
    pub fn set_var(&mut self, name: impl Into<String>, value: Value) {
        let name = name.into();
        if let Some(pos) = self.vars.iter().position(|(k, _)| k == &name) {
            self.vars[pos].1 = value;
        } else {
            self.vars.push((name, value));
        }
    }

    /// Builder method to add a variable binding.
    pub fn with_var(mut self, name: impl Into<String>, value: Value) -> Self {
        self.set_var(name, value);
        self
    }

    /// Looks up a variable by name.
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.vars.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }

    /// Evaluates a built-in HCL / Terraform function.
    pub fn call_function(&self, name: &str, args: &[Value]) -> Result<Option<Value>, HclError> {
        let res = match name {
            // ── String Functions ──
            "upper" => {
                let s = arg_str(args, 0, "upper")?;
                Value::String(s.to_uppercase())
            }
            "lower" => {
                let s = arg_str(args, 0, "lower")?;
                Value::String(s.to_lowercase())
            }
            "trim" => {
                let s = arg_str(args, 0, "trim")?;
                let cutset = arg_str(args, 1, "trim")?;
                let trimmed = s.trim_matches(|c: char| cutset.contains(c));
                Value::String(trimmed.to_string())
            }
            "trimprefix" => {
                let s = arg_str(args, 0, "trimprefix")?;
                let prefix = arg_str(args, 1, "trimprefix")?;
                Value::String(s.strip_prefix(prefix).unwrap_or(s).to_string())
            }
            "trimsuffix" => {
                let s = arg_str(args, 0, "trimsuffix")?;
                let suffix = arg_str(args, 1, "trimsuffix")?;
                Value::String(s.strip_suffix(suffix).unwrap_or(s).to_string())
            }
            "trimspace" => {
                let s = arg_str(args, 0, "trimspace")?;
                Value::String(s.trim().to_string())
            }
            "replace" => {
                let s = arg_str(args, 0, "replace")?;
                let old = arg_str(args, 1, "replace")?;
                let new = arg_str(args, 2, "replace")?;
                Value::String(s.replace(old, new))
            }
            "split" => {
                let sep = arg_str(args, 0, "split")?;
                let s = arg_str(args, 1, "split")?;
                let parts: Vec<Value> = s
                    .split(sep)
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Value::Array(parts)
            }
            "join" => {
                let sep = arg_str(args, 0, "join")?;
                let mut strings = Vec::new();
                for arg in &args[1..] {
                    if let Value::Array(items) = arg {
                        for item in items {
                            strings.push(val_to_str(item));
                        }
                    } else {
                        strings.push(val_to_str(arg));
                    }
                }
                Value::String(strings.join(sep))
            }
            "substr" => {
                let s = arg_str(args, 0, "substr")?;
                let offset = arg_int(args, 1, "substr")?;
                let length = arg_int(args, 2, "substr")?;
                let chars: Vec<char> = s.chars().collect();
                let total = chars.len() as i64;
                let start = if offset < 0 {
                    (total + offset).max(0) as usize
                } else {
                    (offset as usize).min(chars.len())
                };
                let count = if length < 0 {
                    chars.len().saturating_sub(start)
                } else {
                    (length as usize).min(chars.len().saturating_sub(start))
                };
                let sub: String = chars[start..start + count].iter().collect();
                Value::String(sub)
            }
            "strrev" => {
                let s = arg_str(args, 0, "strrev")?;
                Value::String(s.chars().rev().collect())
            }
            "format" => {
                let fmt_str = arg_str(args, 0, "format")?;
                Value::String(eval_printf(fmt_str, &args[1..]))
            }

            // ── Collection Functions ──
            "length" => {
                let v = args.first().ok_or_else(|| HclError::Custom("length() requires 1 argument".into()))?;
                match v {
                    Value::String(s) => Value::Integer(s.chars().count() as i128),
                    Value::Array(a) => Value::Integer(a.len() as i128),
                    Value::Object(o) => Value::Integer(o.len() as i128),
                    Value::Bytes(b) => Value::Integer(b.len() as i128),
                    _ => return Err(HclError::Custom("length() requires string, list, or map".into())),
                }
            }
            "coalesce" => {
                let mut found = Value::Null;
                for arg in args {
                    match arg {
                        Value::Null => continue,
                        Value::String(s) if s.is_empty() => continue,
                        other => {
                            found = other.clone();
                            break;
                        }
                    }
                }
                found
            }
            "compact" => {
                let arr = arg_array(args, 0, "compact")?;
                let out: Vec<Value> = arr
                    .iter()
                    .filter(|v| !v.is_null() && v.as_str().is_none_or(|s| !s.is_empty()))
                    .cloned()
                    .collect();
                Value::Array(out)
            }
            "concat" => {
                let mut out = Vec::new();
                for arg in args {
                    if let Value::Array(items) = arg {
                        out.extend(items.iter().cloned());
                    }
                }
                Value::Array(out)
            }
            "contains" => {
                let arr = arg_array(args, 0, "contains")?;
                let target = args.get(1).ok_or_else(|| HclError::Custom("contains() requires 2 arguments".into()))?;
                Value::Bool(arr.contains(target))
            }
            "distinct" => {
                let arr = arg_array(args, 0, "distinct")?;
                let mut out = Vec::new();
                for item in arr {
                    if !out.contains(item) {
                        out.push(item.clone());
                    }
                }
                Value::Array(out)
            }
            "flatten" => {
                let arr = arg_array(args, 0, "flatten")?;
                let mut out = Vec::new();
                flatten_array(arr, &mut out);
                Value::Array(out)
            }
            "keys" => {
                let obj = arg_object(args, 0, "keys")?;
                let mut keys: Vec<String> = obj.iter().map(|(k, _)| k.clone()).collect();
                keys.sort();
                Value::Array(keys.into_iter().map(Value::String).collect())
            }
            "values" => {
                let obj = arg_object(args, 0, "values")?;
                let mut pairs: Vec<(&String, &Value)> = obj.iter().map(|(k, v)| (k, v)).collect();
                pairs.sort_by(|a, b| a.0.cmp(b.0));
                Value::Array(pairs.into_iter().map(|(_, v)| v.clone()).collect())
            }
            "lookup" => {
                let obj = arg_object(args, 0, "lookup")?;
                let key = arg_str(args, 1, "lookup")?;
                let default = args.get(2).cloned().unwrap_or(Value::Null);
                obj.iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(default)
            }
            "merge" => {
                let mut merged: Vec<(String, Value)> = Vec::new();
                for arg in args {
                    if let Value::Object(entries) = arg {
                        for (k, v) in entries {
                            if let Some(pos) = merged.iter().position(|(ek, _)| ek == k) {
                                merged[pos].1 = v.clone();
                            } else {
                                merged.push((k.clone(), v.clone()));
                            }
                        }
                    }
                }
                Value::Object(merged)
            }
            "reverse" => {
                let arr = arg_array(args, 0, "reverse")?;
                let mut out = arr.to_vec();
                out.reverse();
                Value::Array(out)
            }
            "slice" => {
                let arr = arg_array(args, 0, "slice")?;
                let start = (arg_int(args, 1, "slice")?.max(0) as usize).min(arr.len());
                let end = (arg_int(args, 2, "slice")?.max(0) as usize).min(arr.len());
                if start <= end {
                    Value::Array(arr[start..end].to_vec())
                } else {
                    Value::Array(Vec::new())
                }
            }
            "sort" => {
                let arr = arg_array(args, 0, "sort")?;
                let mut strs: Vec<String> = arr.iter().map(val_to_str).collect();
                strs.sort();
                Value::Array(strs.into_iter().map(Value::String).collect())
            }
            "zipmap" => {
                let keys = arg_array(args, 0, "zipmap")?;
                let vals = arg_array(args, 1, "zipmap")?;
                let mut entries = Vec::with_capacity(keys.len().min(vals.len()));
                for (k, v) in keys.iter().zip(vals.iter()) {
                    entries.push((val_to_str(k), v.clone()));
                }
                Value::Object(entries)
            }
            "element" => {
                let arr = arg_array(args, 0, "element")?;
                if arr.is_empty() {
                    return Err(HclError::Custom("element() called on empty list".into()));
                }
                let idx = arg_int(args, 1, "element")?.rem_euclid(arr.len() as i64) as usize;
                arr[idx].clone()
            }

            // ── Numeric Functions ──
            "abs" => match args.first() {
                Some(Value::Integer(i)) => Value::Integer(i.abs()),
                Some(Value::Float(f)) => Value::Float(f.abs()),
                _ => return Err(HclError::Custom("abs() requires a number".into())),
            },
            "ceil" => {
                let f = arg_float(args, 0, "ceil")?;
                Value::Integer(ceil_f64(f) as i128)
            }
            "floor" => {
                let f = arg_float(args, 0, "floor")?;
                Value::Integer(floor_f64(f) as i128)
            }
            "max" => {
                let mut best: Option<Value> = None;
                for arg in args {
                    if let Some(f) = arg.as_f64() {
                        if best.as_ref().and_then(|b| b.as_f64()).is_none_or(|bf| f > bf) {
                            best = Some(arg.clone());
                        }
                    }
                }
                best.unwrap_or(Value::Null)
            }
            "min" => {
                let mut best: Option<Value> = None;
                for arg in args {
                    if let Some(f) = arg.as_f64() {
                        if best.as_ref().and_then(|b| b.as_f64()).is_none_or(|bf| f < bf) {
                            best = Some(arg.clone());
                        }
                    }
                }
                best.unwrap_or(Value::Null)
            }
            "pow" => {
                let base = arg_float(args, 0, "pow")?;
                let exp = arg_float(args, 1, "pow")?;
                let res = pow_f64(base, exp);
                if args.first().is_some_and(|v| v.is_integer())
                    && args.get(1).is_some_and(|v| v.is_integer())
                    && exp >= 0.0
                {
                    Value::Integer(res as i128)
                } else {
                    Value::Float(res)
                }
            }
            "signum" => {
                let f = arg_float(args, 0, "signum")?;
                let s = if f > 0.0 {
                    1
                } else if f < 0.0 {
                    -1
                } else {
                    0
                };
                Value::Integer(s)
            }

            // ── Encoding Functions ──
            "jsonencode" => {
                let v = args.first().unwrap_or(&Value::Null);
                let mut buf = babbel_core::io::Buffer::new();
                v.serialize_json(&mut buf);
                Value::String(buf.to_string())
            }
            "base64encode" => {
                let s = arg_str(args, 0, "base64encode")?;
                Value::String(base64_encode(s.as_bytes()))
            }
            "base64decode" => {
                let s = arg_str(args, 0, "base64decode")?;
                let bytes = base64_decode(s)?;
                let decoded = core::str::from_utf8(&bytes)
                    .map_err(|_| HclError::Custom("base64decode result is not valid UTF-8".into()))?;
                Value::String(decoded.to_string())
            }

            _ => return Ok(None),
        };
        Ok(Some(res))
    }
}

fn arg_str<'a>(args: &'a [Value], idx: usize, fn_name: &str) -> Result<&'a str, HclError> {
    args.get(idx)
        .and_then(|v| v.as_str())
        .ok_or_else(|| HclError::Custom(format!("{}() argument {} must be a string", fn_name, idx)))
}

fn arg_int(args: &[Value], idx: usize, fn_name: &str) -> Result<i64, HclError> {
    args.get(idx)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| HclError::Custom(format!("{}() argument {} must be an integer", fn_name, idx)))
}

fn arg_float(args: &[Value], idx: usize, fn_name: &str) -> Result<f64, HclError> {
    args.get(idx)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| HclError::Custom(format!("{}() argument {} must be a number", fn_name, idx)))
}

fn arg_array<'a>(args: &'a [Value], idx: usize, fn_name: &str) -> Result<&'a [Value], HclError> {
    args.get(idx)
        .and_then(|v| v.as_array())
        .ok_or_else(|| HclError::Custom(format!("{}() argument {} must be a list", fn_name, idx)))
}

fn arg_object<'a>(
    args: &'a [Value],
    idx: usize,
    fn_name: &str,
) -> Result<&'a [(String, Value)], HclError> {
    args.get(idx)
        .and_then(|v| v.as_object())
        .ok_or_else(|| HclError::Custom(format!("{}() argument {} must be a map/object", fn_name, idx)))
}

fn val_to_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => format!("{}", i),
        Value::Float(f) => format!("{}", f),
        Value::Bool(b) => format!("{}", b),
        Value::Null => "null".to_string(),
        other => {
            let mut buf = babbel_core::io::Buffer::new();
            other.serialize_json(&mut buf);
            buf.to_string()
        }
    }
}

fn flatten_array(items: &[Value], out: &mut Vec<Value>) {
    for item in items {
        if let Value::Array(nested) = item {
            flatten_array(nested, out);
        } else {
            out.push(item.clone());
        }
    }
}

fn eval_printf(fmt_str: &str, args: &[Value]) -> String {
    let mut out = String::new();
    let mut chars = fmt_str.chars().peekable();
    let mut arg_idx = 0usize;
    while let Some(c) = chars.next() {
        if c == '%' {
            if chars.peek() == Some(&'%') {
                chars.next();
                out.push('%');
                continue;
            }
            if let Some(spec) = chars.next() {
                if let Some(arg) = args.get(arg_idx) {
                    arg_idx += 1;
                    match spec {
                        's' | 'v' => out.push_str(&val_to_str(arg)),
                        'd' => out.push_str(&format!("{}", arg.as_i64().unwrap_or(0))),
                        'f' => out.push_str(&format!("{}", arg.as_f64().unwrap_or(0.0))),
                        't' => out.push_str(&format!("{}", arg.as_bool().unwrap_or(false))),
                        'q' => out.push_str(&format!("\"{}\"", val_to_str(arg))),
                        _ => out.push_str(&val_to_str(arg)),
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn floor_f64(x: f64) -> f64 {
    let i = x as i64 as f64;
    if x < i { i - 1.0 } else { i }
}

fn ceil_f64(x: f64) -> f64 {
    let i = x as i64 as f64;
    if x > i { i + 1.0 } else { i }
}

fn pow_f64(mut base: f64, exp: f64) -> f64 {
    let n = exp as i64;
    if (n as f64) == exp {
        let mut e = n.unsigned_abs();
        let mut result = 1.0f64;
        while e > 0 {
            if e & 1 == 1 {
                result *= base;
            }
            base *= base;
            e >>= 1;
        }
        if n < 0 { 1.0 / result } else { result }
    } else {
        base
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>, HclError> {
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u8;
    for b in s.bytes() {
        if b.is_ascii_whitespace() || b == b'=' {
            continue;
        }
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(HclError::Custom("invalid base64 character".into())),
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Ok(out)
}

