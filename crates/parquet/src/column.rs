//! Columnar data encoding and decoding for Parquet PLAIN format.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use babbel_core::Value;
use crate::error::ParquetError;
use crate::metadata::Type;

/// In-memory column holding typed values for a table column.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnData {
    pub name: String,
    pub type_: Type,
    pub values: Vec<Value>,
}

impl ColumnData {
    pub fn new(name: impl Into<String>, type_: Type) -> Self {
        Self {
            name: name.into(),
            type_,
            values: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Encode column values into a PLAIN data page payload.
    /// Returns `(encoded_bytes, has_nulls)`.
    pub fn encode_plain(&self) -> Result<(Vec<u8>, bool), ParquetError> {
        let has_nulls = self.values.iter().any(|v| v.is_null());
        let mut buf = Vec::new();

        if has_nulls {
            // Encode definition levels for optional field: 1 = value present, 0 = null
            let num_values = self.values.len();
            let mut def_bits = Vec::new();
            let mut current_byte = 0u8;
            let mut bit_idx = 0;

            for val in &self.values {
                if !val.is_null() {
                    current_byte |= 1 << bit_idx;
                }
                bit_idx += 1;
                if bit_idx == 8 {
                    def_bits.push(current_byte);
                    current_byte = 0;
                    bit_idx = 0;
                }
            }
            if bit_idx > 0 {
                def_bits.push(current_byte);
            }

            // Bit-packed RLE header: ((num_groups) << 1) | 1
            let num_groups = (num_values + 7) / 8;
            let header = ((num_groups as u32) << 1) | 1;
            let mut rle_bytes = Vec::new();
            write_varint_u32(&mut rle_bytes, header);
            rle_bytes.extend_from_slice(&def_bits);

            // 4-byte length prefix of definition levels
            buf.extend_from_slice(&(rle_bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(&rle_bytes);

            // Encode non-null values
            for val in &self.values {
                if !val.is_null() {
                    self.encode_single_value(val, &mut buf)?;
                }
            }
        } else {
            // No nulls: encode all values directly
            for val in &self.values {
                self.encode_single_value(val, &mut buf)?;
            }
        }

        Ok((buf, has_nulls))
    }

    fn encode_single_value(&self, val: &Value, buf: &mut Vec<u8>) -> Result<(), ParquetError> {
        match self.type_ {
            Type::Boolean => {
                let b = val.as_bool().unwrap_or(false);
                buf.push(if b { 1 } else { 0 });
            }
            Type::Int32 => {
                let n = val.as_i64().unwrap_or(0) as i32;
                buf.extend_from_slice(&n.to_le_bytes());
            }
            Type::Int64 => {
                let n = val.as_i64().unwrap_or(0);
                buf.extend_from_slice(&n.to_le_bytes());
            }
            Type::Float => {
                let f = val.as_f64().unwrap_or(0.0) as f32;
                buf.extend_from_slice(&f.to_le_bytes());
            }
            Type::Double => {
                let f = val.as_f64().unwrap_or(0.0);
                buf.extend_from_slice(&f.to_le_bytes());
            }
            Type::ByteArray => {
                let s = match val {
                    Value::String(str_val) => str_val.as_bytes(),
                    Value::Bytes(b) => b.as_slice(),
                    _ => val.as_str().map(|s| s.as_bytes()).unwrap_or(b""),
                };
                buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
                buf.extend_from_slice(s);
            }
            _ => {
                return Err(ParquetError::UnsupportedType(alloc::format!(
                    "Cannot encode type {:?}",
                    self.type_
                )));
            }
        }
        Ok(())
    }

    /// Decode a PLAIN data page payload into column values.
    pub fn decode_plain(
        &mut self,
        data: &[u8],
        num_values: usize,
        is_optional: bool,
    ) -> Result<(), ParquetError> {
        let mut pos = 0;

        if is_optional {
            // Read 4-byte definition level length
            if data.len() < 4 {
                return Err(ParquetError::CorruptedPage("Truncated definition levels".into()));
            }
            let def_len = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
            pos += 4;

            if pos + def_len > data.len() {
                return Err(ParquetError::CorruptedPage("Definition level length exceeds page".into()));
            }

            let def_data = &data[pos..pos + def_len];
            pos += def_len;

            // Decode RLE definition levels
            let mut def_reader_pos = 0;
            let (header, h_len) = read_varint_u32(def_data, def_reader_pos)?;
            def_reader_pos += h_len;

            let mut def_bits = Vec::with_capacity(num_values);
            if (header & 1) == 1 {
                // Bit-packed run
                let num_groups = (header >> 1) as usize;
                for _ in 0..num_groups {
                    if def_reader_pos < def_data.len() {
                        let b = def_data[def_reader_pos];
                        def_reader_pos += 1;
                        for bit in 0..8 {
                            if def_bits.len() < num_values {
                                def_bits.push((b >> bit) & 1 == 1);
                            }
                        }
                    }
                }
            } else {
                // RLE run: count followed by value
                let count = (header >> 1) as usize;
                let val_byte = if def_reader_pos < def_data.len() {
                    def_data[def_reader_pos]
                } else {
                    1
                };
                for _ in 0..count {
                    def_bits.push(val_byte != 0);
                }
            }

            for is_present in def_bits {
                if is_present {
                    let val = self.decode_single_value(data, &mut pos)?;
                    self.values.push(val);
                } else {
                    self.values.push(Value::Null);
                }
            }
        } else {
            // All values present
            for _ in 0..num_values {
                let val = self.decode_single_value(data, &mut pos)?;
                self.values.push(val);
            }
        }

        Ok(())
    }

    fn decode_single_value(&self, data: &[u8], pos: &mut usize) -> Result<Value, ParquetError> {
        match self.type_ {
            Type::Boolean => {
                if *pos >= data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let b = data[*pos] != 0;
                *pos += 1;
                Ok(Value::Bool(b))
            }
            Type::Int32 => {
                if *pos + 4 > data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let n = i32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap());
                *pos += 4;
                Ok(Value::Integer(n as i128))
            }
            Type::Int64 => {
                if *pos + 8 > data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let n = i64::from_le_bytes(data[*pos..*pos + 8].try_into().unwrap());
                *pos += 8;
                Ok(Value::Integer(n as i128))
            }
            Type::Float => {
                if *pos + 4 > data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let f = f32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap());
                *pos += 4;
                Ok(Value::Float(f as f64))
            }
            Type::Double => {
                if *pos + 8 > data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let f = f64::from_le_bytes(data[*pos..*pos + 8].try_into().unwrap());
                *pos += 8;
                Ok(Value::Float(f))
            }
            Type::ByteArray => {
                if *pos + 4 > data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let len = u32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap()) as usize;
                *pos += 4;
                if *pos + len > data.len() {
                    return Err(ParquetError::UnexpectedEof);
                }
                let slice = &data[*pos..*pos + len];
                *pos += len;
                let s = core::str::from_utf8(slice)
                    .map(|str_val| str_val.to_string())
                    .map_err(|_| ParquetError::InvalidUtf8)?;
                Ok(Value::String(s))
            }
            _ => Err(ParquetError::UnsupportedType(alloc::format!(
                "Cannot decode type {:?}",
                self.type_
            ))),
        }
    }
}

fn write_varint_u32(buf: &mut Vec<u8>, mut val: u32) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            buf.push(byte | 0x80);
        } else {
            buf.push(byte);
            break;
        }
    }
}

fn read_varint_u32(data: &[u8], mut pos: usize) -> Result<(u32, usize), ParquetError> {
    let start = pos;
    let mut result: u32 = 0;
    let mut shift = 0;
    loop {
        if pos >= data.len() {
            return Err(ParquetError::UnexpectedEof);
        }
        let byte = data[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return Err(ParquetError::CorruptedPage("Varint overflow in definition levels".into()));
        }
    }
    Ok((result, pos - start))
}
