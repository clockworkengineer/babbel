//! Columnar data encoding and decoding for Parquet PLAIN and RLE_DICTIONARY formats.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::error::ParquetError;
use crate::metadata::Type;
use babbel_core::Value;
use babbel_core::encoding::varint::{read_varint_u32, write_varint_u32};

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
            let num_groups = num_values.div_ceil(8);
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

    /// Decode a PLAIN dictionary page payload into a vector of dictionary values.
    pub fn decode_dictionary_values(
        &self,
        data: &[u8],
        num_values: usize,
    ) -> Result<Vec<Value>, ParquetError> {
        if num_values > data.len() {
            return Err(ParquetError::CorruptedPage(
                "Dictionary num_values exceeds available page data".into(),
            ));
        }
        let mut dict = Vec::with_capacity(num_values.min(100_000));
        let mut pos = 0;
        for _ in 0..num_values {
            dict.push(self.decode_single_value(data, &mut pos)?);
        }
        Ok(dict)
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
            let def_bits = Self::read_definition_levels_v1(data, &mut pos, num_values)?;
            for is_present in def_bits {
                if is_present {
                    let val = self.decode_single_value(data, &mut pos)?;
                    self.values.push(val);
                } else {
                    self.values.push(Value::Null);
                }
            }
        } else {
            for _ in 0..num_values {
                let val = self.decode_single_value(data, &mut pos)?;
                self.values.push(val);
            }
        }

        Ok(())
    }

    /// Decode a dictionary-encoded (`RLE_DICTIONARY` or `PLAIN_DICTIONARY`) data page v1 payload.
    pub fn decode_dictionary_page(
        &mut self,
        data: &[u8],
        num_values: usize,
        is_optional: bool,
        dict: &[Value],
    ) -> Result<(), ParquetError> {
        let mut pos = 0;
        let def_bits = if is_optional {
            Some(Self::read_definition_levels_v1(data, &mut pos, num_values)?)
        } else {
            None
        };

        let non_null_count = match &def_bits {
            Some(bits) => bits.iter().filter(|&&b| b).count(),
            None => num_values,
        };

        let indices = if non_null_count > 0 {
            if pos >= data.len() {
                return Err(ParquetError::CorruptedPage(
                    "Missing bit_width byte in dictionary page".into(),
                ));
            }
            let bit_width = data[pos];
            pos += 1;
            decode_rle_bit_packed_hybrid(&data[pos..], bit_width, non_null_count)?
        } else {
            Vec::new()
        };

        let mut idx_iter = indices.into_iter();
        if let Some(bits) = def_bits {
            for is_present in bits {
                if is_present {
                    let dict_idx = idx_iter.next().unwrap_or(0) as usize;
                    let val = dict.get(dict_idx).cloned().ok_or_else(|| {
                        ParquetError::CorruptedPage("Dictionary index out of bounds".into())
                    })?;
                    self.values.push(val);
                } else {
                    self.values.push(Value::Null);
                }
            }
        } else {
            for dict_idx in idx_iter {
                let val = dict.get(dict_idx as usize).cloned().ok_or_else(|| {
                    ParquetError::CorruptedPage("Dictionary index out of bounds".into())
                })?;
                self.values.push(val);
            }
        }

        Ok(())
    }

    /// Decode a `DATA_PAGE_V2` payload (where definition levels have an explicit byte length in the header).
    pub fn decode_data_page_v2(
        &mut self,
        def_bytes: &[u8],
        values_bytes: &[u8],
        num_values: usize,
        is_optional: bool,
        is_dictionary: bool,
        dict: Option<&[Value]>,
    ) -> Result<(), ParquetError> {
        let def_bits = if is_optional && !def_bytes.is_empty() {
            let levels = decode_rle_bit_packed_hybrid(def_bytes, 1, num_values)?;
            Some(levels.into_iter().map(|v| v != 0).collect::<Vec<bool>>())
        } else {
            None
        };

        if is_dictionary {
            let dict_slice = dict.ok_or_else(|| {
                ParquetError::CorruptedPage("Dictionary page missing for RLE_DICTIONARY v2".into())
            })?;
            let non_null_count = match &def_bits {
                Some(bits) => bits.iter().filter(|&&b| b).count(),
                None => num_values,
            };
            let indices = if non_null_count > 0 {
                if values_bytes.is_empty() {
                    return Err(ParquetError::CorruptedPage(
                        "Missing bit_width in v2 dictionary page".into(),
                    ));
                }
                let bit_width = values_bytes[0];
                decode_rle_bit_packed_hybrid(&values_bytes[1..], bit_width, non_null_count)?
            } else {
                Vec::new()
            };
            let mut idx_iter = indices.into_iter();
            if let Some(bits) = def_bits {
                for is_present in bits {
                    if is_present {
                        let dict_idx = idx_iter.next().unwrap_or(0) as usize;
                        let val = dict_slice.get(dict_idx).cloned().ok_or_else(|| {
                            ParquetError::CorruptedPage("Dictionary index out of bounds".into())
                        })?;
                        self.values.push(val);
                    } else {
                        self.values.push(Value::Null);
                    }
                }
            } else {
                for dict_idx in idx_iter {
                    let val = dict_slice.get(dict_idx as usize).cloned().ok_or_else(|| {
                        ParquetError::CorruptedPage("Dictionary index out of bounds".into())
                    })?;
                    self.values.push(val);
                }
            }
        } else {
            let mut pos = 0;
            if let Some(bits) = def_bits {
                for is_present in bits {
                    if is_present {
                        let val = self.decode_single_value(values_bytes, &mut pos)?;
                        self.values.push(val);
                    } else {
                        self.values.push(Value::Null);
                    }
                }
            } else {
                for _ in 0..num_values {
                    let val = self.decode_single_value(values_bytes, &mut pos)?;
                    self.values.push(val);
                }
            }
        }
        Ok(())
    }

    fn read_definition_levels_v1(
        data: &[u8],
        pos: &mut usize,
        num_values: usize,
    ) -> Result<Vec<bool>, ParquetError> {
        if *pos + 4 > data.len() {
            return Err(ParquetError::CorruptedPage(
                "Truncated definition levels".into(),
            ));
        }
        let def_len = u32::from_le_bytes(data[*pos..*pos + 4].try_into().unwrap()) as usize;
        *pos += 4;

        if *pos + def_len > data.len() {
            return Err(ParquetError::CorruptedPage(
                "Definition level length exceeds page".into(),
            ));
        }

        let def_data = &data[*pos..*pos + def_len];
        *pos += def_len;

        let levels = decode_rle_bit_packed_hybrid(def_data, 1, num_values)?;
        Ok(levels.into_iter().map(|v| v != 0).collect())
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

/// Decodes Parquet RLE / Bit-Packing Hybrid values of a given `bit_width`.
pub fn decode_rle_bit_packed_hybrid(
    data: &[u8],
    bit_width: u8,
    num_values: usize,
) -> Result<Vec<u32>, ParquetError> {
    if bit_width == 0 {
        return Ok(alloc::vec![0u32; num_values]);
    }
    let mut out = Vec::with_capacity(num_values.min(100_000));
    let mut pos = 0usize;
    let bytes_per_rle_val = (bit_width as usize).div_ceil(8);
    let mask = if bit_width >= 32 {
        u32::MAX
    } else {
        (1u32 << bit_width) - 1
    };

    while out.len() < num_values && pos < data.len() {
        let (header, h_len) = read_varint_u32(data, pos).map_err(|_| {
            ParquetError::CorruptedPage("Varint overflow in RLE/Bit-Packed stream".into())
        })?;
        pos += h_len;

        if (header & 1) == 1 {
            // Bit-packed run: (header >> 1) groups of 8 values
            let num_groups = (header >> 1) as usize;
            let total_vals = num_groups * 8;
            let total_bytes = num_groups * (bit_width as usize);
            let mut bit_buf = 0u64;
            let mut bits_in_buf = 0u8;
            let mut byte_idx = 0usize;

            for _ in 0..total_vals {
                while bits_in_buf < bit_width
                    && byte_idx < total_bytes
                    && pos + byte_idx < data.len()
                {
                    bit_buf |= (data[pos + byte_idx] as u64) << bits_in_buf;
                    bits_in_buf += 8;
                    byte_idx += 1;
                }
                let val = (bit_buf as u32) & mask;
                bit_buf >>= bit_width;
                bits_in_buf = bits_in_buf.saturating_sub(bit_width);
                if out.len() < num_values {
                    out.push(val);
                }
            }
            pos = (pos + total_bytes).min(data.len());
        } else {
            // RLE run: (header >> 1) repeated values
            let count = (header >> 1) as usize;
            let mut val = 0u32;
            for b in 0..bytes_per_rle_val {
                if pos < data.len() {
                    val |= (data[pos] as u32) << (b * 8);
                    pos += 1;
                }
            }
            val &= mask;
            for _ in 0..count {
                if out.len() < num_values {
                    out.push(val);
                }
            }
        }
    }

    Ok(out)
}
