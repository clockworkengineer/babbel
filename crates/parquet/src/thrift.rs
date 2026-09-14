//! Pure-Rust Thrift Compact Protocol encoder and decoder for Parquet metadata.

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::error::ParquetError;

pub const TYPE_STOP: u8 = 0;
pub const TYPE_BOOLEAN_TRUE: u8 = 1;
pub const TYPE_BOOLEAN_FALSE: u8 = 2;
pub const TYPE_BYTE: u8 = 3;
pub const TYPE_I16: u8 = 4;
pub const TYPE_I32: u8 = 5;
pub const TYPE_I64: u8 = 6;
pub const TYPE_DOUBLE: u8 = 7;
pub const TYPE_BINARY: u8 = 8;
pub const TYPE_LIST: u8 = 9;
pub const TYPE_SET: u8 = 10;
pub const TYPE_MAP: u8 = 11;
pub const TYPE_STRUCT: u8 = 12;

/// Writer for Thrift Compact Protocol.
#[derive(Debug, Default, Clone)]
pub struct ThriftWriter {
    pub buf: Vec<u8>,
    last_field_id: Vec<i16>,
}

impl ThriftWriter {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            last_field_id: alloc::vec![0],
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn write_struct_begin(&mut self) {
        self.last_field_id.push(0);
    }

    pub fn write_struct_end(&mut self) {
        self.write_byte(TYPE_STOP);
        self.last_field_id.pop();
    }

    pub fn write_field_header(&mut self, field_id: i16, field_type: u8) {
        let last = *self.last_field_id.last().expect("struct stack");
        let delta = field_id - last;
        if delta > 0 && delta <= 15 {
            let b = ((delta as u8) << 4) | (field_type & 0x0F);
            self.write_byte(b);
        } else {
            self.write_byte(field_type & 0x0F);
            self.write_i16(field_id);
        }
        if let Some(l) = self.last_field_id.last_mut() {
            *l = field_id;
        }
    }

    pub fn write_bool_field(&mut self, field_id: i16, val: bool) {
        let field_type = if val {
            TYPE_BOOLEAN_TRUE
        } else {
            TYPE_BOOLEAN_FALSE
        };
        self.write_field_header(field_id, field_type);
    }

    pub fn write_byte(&mut self, b: u8) {
        self.buf.push(b);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_i32(&mut self, val: i32) {
        let zigzag = ((val << 1) ^ (val >> 31)) as u32;
        self.write_varint_u32(zigzag);
    }

    pub fn write_i64(&mut self, val: i64) {
        let zigzag = ((val << 1) ^ (val >> 63)) as u64;
        self.write_varint_u64(zigzag);
    }

    pub fn write_i16(&mut self, val: i16) {
        self.write_i32(val as i32);
    }

    pub fn write_double(&mut self, val: f64) {
        let bits = val.to_bits();
        self.buf.extend_from_slice(&bits.to_le_bytes());
    }

    pub fn write_binary(&mut self, bytes: &[u8]) {
        self.write_varint_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }

    pub fn write_string(&mut self, s: &str) {
        self.write_binary(s.as_bytes());
    }

    pub fn write_list_header(&mut self, elem_type: u8, size: usize) {
        if size < 15 {
            let b = ((size as u8) << 4) | (elem_type & 0x0F);
            self.write_byte(b);
        } else {
            let b = 0xF0 | (elem_type & 0x0F);
            self.write_byte(b);
            self.write_varint_u32(size as u32);
        }
    }

    fn write_varint_u32(&mut self, mut val: u32) {
        loop {
            let byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                self.buf.push(byte | 0x80);
            } else {
                self.buf.push(byte);
                break;
            }
        }
    }

    fn write_varint_u64(&mut self, mut val: u64) {
        loop {
            let byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                self.buf.push(byte | 0x80);
            } else {
                self.buf.push(byte);
                break;
            }
        }
    }
}

/// Reader for Thrift Compact Protocol.
#[derive(Debug, Clone)]
pub struct ThriftReader<'a> {
    pub buf: &'a [u8],
    pub pos: usize,
    last_field_id: Vec<i16>,
}

impl<'a> ThriftReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            last_field_id: alloc::vec![0],
        }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }

    pub fn read_byte(&mut self) -> Result<u8, ParquetError> {
        if self.pos < self.buf.len() {
            let b = self.buf[self.pos];
            self.pos += 1;
            Ok(b)
        } else {
            Err(ParquetError::UnexpectedEof)
        }
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ParquetError> {
        if self.pos + len <= self.buf.len() {
            let slice = &self.buf[self.pos..self.pos + len];
            self.pos += len;
            Ok(slice)
        } else {
            Err(ParquetError::UnexpectedEof)
        }
    }

    pub fn read_struct_begin(&mut self) {
        self.last_field_id.push(0);
    }

    pub fn read_struct_end(&mut self) {
        self.last_field_id.pop();
    }

    /// Read field header: returns `(field_id, field_type)`.
    /// If field_type == `TYPE_STOP` (0), the struct has ended.
    pub fn read_field_header(&mut self) -> Result<(i16, u8), ParquetError> {
        let b = self.read_byte()?;
        if b == TYPE_STOP {
            return Ok((0, TYPE_STOP));
        }

        let delta = (b >> 4) as i16;
        let field_type = b & 0x0F;

        let last = *self.last_field_id.last().expect("struct stack");
        let field_id = if delta == 0 {
            self.read_i16()?
        } else {
            last + delta
        };

        if let Some(l) = self.last_field_id.last_mut() {
            *l = field_id;
        }
        Ok((field_id, field_type))
    }

    pub fn read_i32(&mut self) -> Result<i32, ParquetError> {
        let zigzag = self.read_varint_u32()?;
        let val = ((zigzag >> 1) as i32) ^ (-((zigzag & 1) as i32));
        Ok(val)
    }

    pub fn read_i64(&mut self) -> Result<i64, ParquetError> {
        let zigzag = self.read_varint_u64()?;
        let val = ((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64));
        Ok(val)
    }

    pub fn read_i16(&mut self) -> Result<i16, ParquetError> {
        let val = self.read_i32()?;
        Ok(val as i16)
    }

    pub fn read_double(&mut self) -> Result<f64, ParquetError> {
        let bytes = self.read_bytes(8)?;
        let arr: [u8; 8] = bytes.try_into().unwrap();
        Ok(f64::from_le_bytes(arr))
    }

    pub fn read_binary(&mut self) -> Result<&'a [u8], ParquetError> {
        let len = self.read_varint_u32()? as usize;
        self.read_bytes(len)
    }

    pub fn read_string(&mut self) -> Result<String, ParquetError> {
        let bytes = self.read_binary()?;
        core::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|_| ParquetError::InvalidUtf8)
    }

    pub fn read_list_header(&mut self) -> Result<(u8, usize), ParquetError> {
        let b = self.read_byte()?;
        let size_low = (b >> 4) as usize;
        let elem_type = b & 0x0F;
        let size = if size_low == 0x0F {
            self.read_varint_u32()? as usize
        } else {
            size_low
        };
        Ok((elem_type, size))
    }

    /// Skip a field value if we encounter an unknown field_type
    pub fn skip(&mut self, field_type: u8) -> Result<(), ParquetError> {
        match field_type {
            TYPE_BOOLEAN_TRUE | TYPE_BOOLEAN_FALSE => Ok(()),
            TYPE_BYTE => {
                self.read_byte()?;
                Ok(())
            }
            TYPE_I16 | TYPE_I32 => {
                self.read_i32()?;
                Ok(())
            }
            TYPE_I64 => {
                self.read_i64()?;
                Ok(())
            }
            TYPE_DOUBLE => {
                self.read_double()?;
                Ok(())
            }
            TYPE_BINARY => {
                self.read_binary()?;
                Ok(())
            }
            TYPE_LIST | TYPE_SET => {
                let (elem_ty, len) = self.read_list_header()?;
                for _ in 0..len {
                    self.skip(elem_ty)?;
                }
                Ok(())
            }
            TYPE_STRUCT => {
                self.read_struct_begin();
                loop {
                    let (_, ty) = self.read_field_header()?;
                    if ty == TYPE_STOP {
                        break;
                    }
                    self.skip(ty)?;
                }
                self.read_struct_end();
                Ok(())
            }
            _ => Err(ParquetError::ThriftError(format!(
                "Cannot skip unsupported Thrift type {}",
                field_type
            ))),
        }
    }

    fn read_varint_u32(&mut self) -> Result<u32, ParquetError> {
        let mut result: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0x7F) as u32) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
            if shift >= 35 {
                return Err(ParquetError::ThriftError("Varint overflow".to_string()));
            }
        }
        Ok(result)
    }

    fn read_varint_u64(&mut self) -> Result<u64, ParquetError> {
        let mut result: u64 = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0x7F) as u64) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
            if shift >= 70 {
                return Err(ParquetError::ThriftError("Varint64 overflow".to_string()));
            }
        }
        Ok(result)
    }
}
