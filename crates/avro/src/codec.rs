//! Apache Avro binary serialization and deserialization engine.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use crate::error::AvroError;
use crate::schema::AvroSchema;
use babbel_core::Value;

/// Binary decoder for raw Avro byte payloads.
pub struct AvroDecoder<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl<'a> AvroDecoder<'a> {
    /// Create a new Avro decoder over the provided slice.
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, cursor: 0 }
    }

    /// Returns current cursor offset.
    pub fn position(&self) -> usize {
        self.cursor
    }

    /// Read raw boolean.
    pub fn read_bool(&mut self) -> Result<bool, AvroError> {
        let b = self.read_u8()?;
        match b {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(AvroError::Custom("invalid boolean byte in Avro")),
        }
    }

    /// Read zigzag-encoded variable-length 64-bit integer.
    pub fn read_long(&mut self) -> Result<i64, AvroError> {
        let mut n: u64 = 0;
        let mut shift = 0;

        loop {
            if shift >= 64 {
                return Err(AvroError::InvalidVarint);
            }
            let b = self.read_u8()?;
            n |= ((b & 0x7F) as u64) << shift;
            if (b & 0x80) == 0 {
                break;
            }
            shift += 7;
        }

        // Zigzag decode: (n >> 1) ^ (-(n & 1))
        let val = ((n >> 1) as i64) ^ (-((n & 1) as i64));
        Ok(val)
    }

    /// Read 32-bit float in little-endian.
    pub fn read_float(&mut self) -> Result<f32, AvroError> {
        let bytes = self.read_slice(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read 64-bit double in little-endian.
    pub fn read_double(&mut self) -> Result<f64, AvroError> {
        let bytes = self.read_slice(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read length-prefixed bytes.
    pub fn read_bytes(&mut self) -> Result<&'a [u8], AvroError> {
        let len = self.read_long()?;
        if len < 0 {
            return Err(AvroError::Custom("negative byte array length in Avro"));
        }
        self.read_slice(len as usize)
    }

    /// Read length-prefixed UTF-8 string.
    pub fn read_string(&mut self) -> Result<&'a str, AvroError> {
        let bytes = self.read_bytes()?;
        core::str::from_utf8(bytes).map_err(|_| AvroError::InvalidUtf8)
    }

    /// Decode generic Avro value based on standard union tagging.
    pub fn decode_value(&mut self) -> Result<Value, AvroError> {
        if self.cursor >= self.input.len() {
            return Ok(Value::Null);
        }

        let tag = self.read_long()?;
        match tag {
            0 => Ok(Value::Null),
            1 => self.read_bool().map(Value::Bool),
            2 => self.read_long().map(|i| Value::Integer(i as i128)),
            3 => self.read_double().map(Value::Float),
            4 => self.read_string().map(|s| Value::String(s.to_string())),
            5 => self.read_bytes().map(|b| Value::Bytes(b.to_vec())),
            6 => self.decode_array(),
            7 => self.decode_map(),
            _ => Err(AvroError::Custom("unrecognized Avro union type tag")),
        }
    }

    /// Decode an Avro value according to an explicit [`AvroSchema`].
    pub fn decode_with_schema(&mut self, schema: &AvroSchema) -> Result<Value, AvroError> {
        match schema {
            AvroSchema::Null => Ok(Value::Null),
            AvroSchema::Boolean => self.read_bool().map(Value::Bool),
            AvroSchema::Int => self.read_long().map(|i| Value::Integer(i as i128)),
            AvroSchema::Long => self.read_long().map(|i| Value::Integer(i as i128)),
            AvroSchema::Float => self.read_float().map(|f| Value::Float(f as f64)),
            AvroSchema::Double => self.read_double().map(Value::Float),
            AvroSchema::Bytes => self.read_bytes().map(|b| Value::Bytes(b.to_vec())),
            AvroSchema::String => self.read_string().map(|s| Value::String(s.to_string())),
            AvroSchema::Record { fields, .. } => {
                let mut entries = Vec::with_capacity(fields.len());
                for f in fields {
                    let val = self.decode_with_schema(&f.schema)?;
                    entries.push((f.name.clone(), val));
                }
                Ok(Value::Object(entries))
            }
            AvroSchema::Union(variants) => {
                let idx = self.read_long()?;
                if idx < 0 || (idx as usize) >= variants.len() {
                    return Err(AvroError::Custom(
                        "union index out of bounds in Avro stream",
                    ));
                }
                self.decode_with_schema(&variants[idx as usize])
            }
            AvroSchema::Enum { symbols, .. } => {
                let idx = self.read_long()?;
                if idx < 0 || (idx as usize) >= symbols.len() {
                    return Err(AvroError::Custom(
                        "enum symbol index out of bounds in Avro stream",
                    ));
                }
                Ok(Value::String(symbols[idx as usize].clone()))
            }
            AvroSchema::Array { items } => {
                let mut result = Vec::new();
                let mut count = self.read_long()?;
                while count != 0 {
                    let actual_count = if count < 0 {
                        let _block_size = self.read_long()?;
                        -count
                    } else {
                        count
                    };
                    for _ in 0..actual_count {
                        result.push(self.decode_with_schema(items)?);
                    }
                    count = self.read_long()?;
                }
                Ok(Value::Array(result))
            }
            AvroSchema::Map { values } => {
                let mut entries = Vec::new();
                let mut count = self.read_long()?;
                while count != 0 {
                    let actual_count = if count < 0 {
                        let _block_size = self.read_long()?;
                        -count
                    } else {
                        count
                    };
                    for _ in 0..actual_count {
                        let key = self.read_string()?.to_string();
                        let val = self.decode_with_schema(values)?;
                        entries.push((key, val));
                    }
                    count = self.read_long()?;
                }
                Ok(Value::Object(entries))
            }
            AvroSchema::Fixed { size, .. } => {
                let slice = self.read_slice(*size)?;
                Ok(Value::Bytes(slice.to_vec()))
            }
            AvroSchema::Named(_) => self.decode_value(),
        }
    }

    fn decode_array(&mut self) -> Result<Value, AvroError> {
        let mut items = Vec::new();
        let mut count = self.read_long()?;

        while count > 0 {
            for _ in 0..count {
                items.push(self.decode_value()?);
            }
            let next_count = self.read_long()?;
            if next_count <= 0 {
                break;
            }
            count = next_count;
        }

        Ok(Value::Array(items))
    }

    fn decode_map(&mut self) -> Result<Value, AvroError> {
        let mut entries = Vec::new();
        let mut count = self.read_long()?;

        while count > 0 {
            for _ in 0..count {
                let key = self.read_string()?.to_string();
                let val = self.decode_value()?;
                entries.push((key, val));
            }
            let next_count = self.read_long()?;
            if next_count <= 0 {
                break;
            }
            count = next_count;
        }

        Ok(Value::Object(entries))
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, AvroError> {
        if self.cursor >= self.input.len() {
            return Err(AvroError::UnexpectedEof {
                expected: 1,
                available: 0,
            });
        }
        let b = self.input[self.cursor];
        self.cursor += 1;
        Ok(b)
    }

    #[inline]
    fn read_slice(&mut self, len: usize) -> Result<&'a [u8], AvroError> {
        let avail = self.input.len().saturating_sub(self.cursor);
        if avail < len {
            return Err(AvroError::UnexpectedEof {
                expected: len,
                available: avail,
            });
        }
        let slice = &self.input[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(slice)
    }
}

/// Binary encoder for raw Avro byte payloads.
pub struct AvroEncoder {
    buf: Vec<u8>,
}

impl AvroEncoder {
    /// Create a new Avro encoder with default capacity.
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(128),
        }
    }

    /// Write a boolean.
    pub fn write_bool(&mut self, b: bool) {
        self.buf.push(if b { 1 } else { 0 });
    }

    /// Write zigzag-encoded variable-length 64-bit integer.
    pub fn write_long(&mut self, val: i64) {
        let mut n = ((val << 1) ^ (val >> 63)) as u64;
        while (n & !0x7F) != 0 {
            self.buf.push(((n & 0x7F) as u8) | 0x80);
            n >>= 7;
        }
        self.buf.push((n & 0x7F) as u8);
    }

    /// Write 32-bit float in little-endian.
    pub fn write_float(&mut self, f: f32) {
        self.buf.extend_from_slice(&f.to_le_bytes());
    }

    /// Write 64-bit double in little-endian.
    pub fn write_double(&mut self, d: f64) {
        self.buf.extend_from_slice(&d.to_le_bytes());
    }

    /// Write length-prefixed bytes.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_long(bytes.len() as i64);
        self.buf.extend_from_slice(bytes);
    }

    /// Write length-prefixed UTF-8 string.
    pub fn write_string(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    /// Write a universal `Value` AST in Avro binary union encoding.
    pub fn write_value(&mut self, val: &Value) {
        match val {
            Value::Null => {
                self.write_long(0);
            }
            Value::Bool(b) => {
                self.write_long(1);
                self.write_bool(*b);
            }
            Value::Integer(i) => {
                self.write_long(2);
                self.write_long(*i as i64);
            }
            Value::Float(f) => {
                self.write_long(3);
                self.write_double(*f);
            }
            Value::String(s) => {
                self.write_long(4);
                self.write_string(s);
            }
            Value::Bytes(b) => {
                self.write_long(5);
                self.write_bytes(b);
            }
            Value::Array(items) => {
                self.write_long(6);
                if items.is_empty() {
                    self.write_long(0);
                } else {
                    self.write_long(items.len() as i64);
                    for item in items {
                        self.write_value(item);
                    }
                    self.write_long(0);
                }
            }
            Value::Object(entries) => {
                self.write_long(7);
                if entries.is_empty() {
                    self.write_long(0);
                } else {
                    self.write_long(entries.len() as i64);
                    for (k, v) in entries {
                        self.write_string(k);
                        self.write_value(v);
                    }
                    self.write_long(0);
                }
            }
        }
    }

    /// Write a universal `Value` according to an explicit [`AvroSchema`].
    pub fn write_with_schema(&mut self, val: &Value, schema: &AvroSchema) -> Result<(), AvroError> {
        match schema {
            AvroSchema::Null => Ok(()),
            AvroSchema::Boolean => {
                let b = match val {
                    Value::Bool(b) => *b,
                    _ => false,
                };
                self.write_bool(b);
                Ok(())
            }
            AvroSchema::Int | AvroSchema::Long => {
                let n = match val {
                    Value::Integer(i) => *i as i64,
                    Value::Float(f) => *f as i64,
                    _ => 0,
                };
                self.write_long(n);
                Ok(())
            }
            AvroSchema::Float => {
                let f = match val {
                    Value::Float(f) => *f as f32,
                    Value::Integer(i) => *i as f32,
                    _ => 0.0,
                };
                self.write_float(f);
                Ok(())
            }
            AvroSchema::Double => {
                let d = match val {
                    Value::Float(f) => *f,
                    Value::Integer(i) => *i as f64,
                    _ => 0.0,
                };
                self.write_double(d);
                Ok(())
            }
            AvroSchema::Bytes => {
                let bytes = match val {
                    Value::Bytes(b) => b.as_slice(),
                    Value::String(s) => s.as_bytes(),
                    _ => &[],
                };
                self.write_bytes(bytes);
                Ok(())
            }
            AvroSchema::String => {
                let s = match val {
                    Value::String(s) => s.as_str(),
                    _ => "",
                };
                self.write_string(s);
                Ok(())
            }
            AvroSchema::Record { fields, .. } => {
                for f in fields {
                    let field_val = val
                        .get(&f.name)
                        .or(f.default.as_ref())
                        .unwrap_or(&Value::Null);
                    self.write_with_schema(field_val, &f.schema)?;
                }
                Ok(())
            }
            AvroSchema::Union(variants) => {
                let (idx, chosen) = variants
                    .iter()
                    .enumerate()
                    .find(|(_, s)| matches_schema(val, s))
                    .or_else(|| variants.first().map(|s| (0, s)))
                    .ok_or_else(|| AvroError::Custom("empty union in Avro schema"))?;
                self.write_long(idx as i64);
                self.write_with_schema(val, chosen)
            }
            AvroSchema::Enum { symbols, .. } => {
                let s = match val {
                    Value::String(s) => s.as_str(),
                    _ => "",
                };
                let idx = symbols.iter().position(|sym| sym == s).unwrap_or(0);
                self.write_long(idx as i64);
                Ok(())
            }
            AvroSchema::Array { items } => {
                let list = match val {
                    Value::Array(arr) => arr.as_slice(),
                    _ => &[],
                };
                if !list.is_empty() {
                    self.write_long(list.len() as i64);
                    for item in list {
                        self.write_with_schema(item, items)?;
                    }
                }
                self.write_long(0);
                Ok(())
            }
            AvroSchema::Map { values } => {
                let entries = match val {
                    Value::Object(obj) => obj.as_slice(),
                    _ => &[],
                };
                if !entries.is_empty() {
                    self.write_long(entries.len() as i64);
                    for (k, v) in entries {
                        self.write_string(k);
                        self.write_with_schema(v, values)?;
                    }
                }
                self.write_long(0);
                Ok(())
            }
            AvroSchema::Fixed { size, .. } => {
                let bytes = match val {
                    Value::Bytes(b) => b.as_slice(),
                    _ => &[],
                };
                let mut out = bytes.to_vec();
                out.resize(*size, 0);
                self.buf.extend_from_slice(&out);
                Ok(())
            }
            AvroSchema::Named(_) => {
                self.write_value(val);
                Ok(())
            }
        }
    }

    /// Consume the encoder and return the raw byte vector.
    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }
}

fn matches_schema(val: &Value, schema: &AvroSchema) -> bool {
    match (val, schema) {
        (Value::Null, AvroSchema::Null) => true,
        (Value::Bool(_), AvroSchema::Boolean) => true,
        (Value::Integer(_), AvroSchema::Int | AvroSchema::Long) => true,
        (Value::Float(_), AvroSchema::Float | AvroSchema::Double) => true,
        (Value::String(_), AvroSchema::String | AvroSchema::Enum { .. }) => true,
        (Value::Bytes(_), AvroSchema::Bytes | AvroSchema::Fixed { .. }) => true,
        (Value::Array(_), AvroSchema::Array { .. }) => true,
        (Value::Object(_), AvroSchema::Record { .. } | AvroSchema::Map { .. }) => true,
        _ => false,
    }
}

/// Convenience function to encode a `Value` into Avro binary format.
pub fn to_vec(value: &Value) -> Result<Vec<u8>, AvroError> {
    let mut encoder = AvroEncoder::new();
    encoder.write_value(value);
    Ok(encoder.into_vec())
}

/// Convenience function to encode a `Value` according to an explicit `AvroSchema`.
pub fn to_vec_with_schema(value: &Value, schema: &AvroSchema) -> Result<Vec<u8>, AvroError> {
    let mut encoder = AvroEncoder::new();
    encoder.write_with_schema(value, schema)?;
    Ok(encoder.into_vec())
}

/// Convenience function to decode Avro binary bytes into a `Value`.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, AvroError> {
    let mut decoder = AvroDecoder::new(bytes);
    decoder.decode_value()
}

/// Convenience function to decode Avro binary bytes according to an explicit `AvroSchema`.
pub fn from_bytes_with_schema(bytes: &[u8], schema: &AvroSchema) -> Result<Value, AvroError> {
    let mut decoder = AvroDecoder::new(bytes);
    decoder.decode_with_schema(schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_roundtrip() {
        let mut encoder = AvroEncoder::new();
        encoder.write_long(0);
        encoder.write_long(-1);
        encoder.write_long(1);
        encoder.write_long(1234567890);
        encoder.write_long(-9876543210);

        let bytes = encoder.into_vec();
        let mut decoder = AvroDecoder::new(&bytes);
        assert_eq!(decoder.read_long().unwrap(), 0);
        assert_eq!(decoder.read_long().unwrap(), -1);
        assert_eq!(decoder.read_long().unwrap(), 1);
        assert_eq!(decoder.read_long().unwrap(), 1234567890);
        assert_eq!(decoder.read_long().unwrap(), -9876543210);
    }
}
