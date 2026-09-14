//! High-performance, streaming-capable MessagePack decoder into universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use babbel_core::Value;
use crate::constants::*;
use crate::error::MsgPackError;

/// Decoder configuration options.
#[derive(Debug, Clone, Copy)]
pub struct DecoderConfig {
    /// Maximum allowed recursion depth.
    pub max_depth: usize,
    /// Maximum allowed input payload size in bytes.
    pub max_size: usize,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            max_size: DEFAULT_MAX_SIZE,
        }
    }
}

/// MessagePack decoder reading from a byte slice.
pub struct Decoder<'a> {
    input: &'a [u8],
    cursor: usize,
    config: DecoderConfig,
}

impl<'a> Decoder<'a> {
    /// Creates a new decoder for the given byte slice with default configuration.
    pub fn new(input: &'a [u8]) -> Self {
        Self::with_config(input, DecoderConfig::default())
    }

    /// Creates a new decoder for the given byte slice with custom configuration.
    pub fn with_config(input: &'a [u8], config: DecoderConfig) -> Self {
        Self {
            input,
            cursor: 0,
            config,
        }
    }

    /// Returns the number of unconsumed bytes.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.cursor)
    }

    /// Returns the current cursor position.
    #[inline]
    pub fn position(&self) -> usize {
        self.cursor
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, MsgPackError> {
        if self.cursor < self.input.len() {
            let b = self.input[self.cursor];
            self.cursor += 1;
            Ok(b)
        } else {
            Err(MsgPackError::UnexpectedEof {
                expected: 1,
                available: 0,
            })
        }
    }

    #[inline]
    fn read_slice(&mut self, len: usize) -> Result<&'a [u8], MsgPackError> {
        let avail = self.remaining();
        if avail < len {
            return Err(MsgPackError::UnexpectedEof {
                expected: len,
                available: avail,
            });
        }
        let start = self.cursor;
        self.cursor += len;
        Ok(&self.input[start..self.cursor])
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, MsgPackError> {
        self.read_byte()
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, MsgPackError> {
        let slice = self.read_slice(2)?;
        Ok(u16::from_be_bytes([slice[0], slice[1]]))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, MsgPackError> {
        let slice = self.read_slice(4)?;
        Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, MsgPackError> {
        let slice = self.read_slice(8)?;
        Ok(u64::from_be_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    #[inline]
    fn read_i8(&mut self) -> Result<i8, MsgPackError> {
        self.read_byte().map(|b| b as i8)
    }

    #[inline]
    fn read_i16(&mut self) -> Result<i16, MsgPackError> {
        let slice = self.read_slice(2)?;
        Ok(i16::from_be_bytes([slice[0], slice[1]]))
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, MsgPackError> {
        let slice = self.read_slice(4)?;
        Ok(i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, MsgPackError> {
        let slice = self.read_slice(8)?;
        Ok(i64::from_be_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    #[inline]
    fn read_f32(&mut self) -> Result<f32, MsgPackError> {
        let slice = self.read_slice(4)?;
        Ok(f32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, MsgPackError> {
        let slice = self.read_slice(8)?;
        Ok(f64::from_be_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    #[inline]
    fn read_str_bytes(&mut self, len: usize) -> Result<String, MsgPackError> {
        if len > self.config.max_size {
            return Err(MsgPackError::SizeLimitExceeded {
                size: len,
                limit: self.config.max_size,
            });
        }
        let slice = self.read_slice(len)?;
        core::str::from_utf8(slice)
            .map(|s| s.to_string())
            .map_err(|_| MsgPackError::InvalidUtf8)
    }

    #[inline]
    fn read_bin_bytes(&mut self, len: usize) -> Result<Vec<u8>, MsgPackError> {
        if len > self.config.max_size {
            return Err(MsgPackError::SizeLimitExceeded {
                size: len,
                limit: self.config.max_size,
            });
        }
        let slice = self.read_slice(len)?;
        Ok(slice.to_vec())
    }

    /// Decodes the next value in the stream into a universal `Value`.
    pub fn decode_value(&mut self) -> Result<Value, MsgPackError> {
        if self.input.len() > self.config.max_size {
            return Err(MsgPackError::SizeLimitExceeded {
                size: self.input.len(),
                limit: self.config.max_size,
            });
        }
        self.decode_value_depth(0)
    }

    fn decode_value_depth(&mut self, depth: usize) -> Result<Value, MsgPackError> {
        if depth > self.config.max_depth {
            return Err(MsgPackError::RecursionLimitExceeded(depth));
        }

        let marker = self.read_byte()?;

        // Positive fixint: 0x00 - 0x7f
        if marker <= 0x7f {
            return Ok(Value::Integer(marker as i128));
        }

        // FixMap: 0x80 - 0x8f
        if (marker & FIXMAP_MASK) == FIXMAP_PREFIX {
            let len = (marker & 0x0f) as usize;
            return self.decode_map(len, depth);
        }

        // FixArray: 0x90 - 0x9f
        if (marker & FIXARRAY_MASK) == FIXARRAY_PREFIX {
            let len = (marker & 0x0f) as usize;
            return self.decode_array(len, depth);
        }

        // FixStr: 0xa0 - 0xbf
        if (marker & FIXSTR_MASK) == FIXSTR_PREFIX {
            let len = (marker & 0x1f) as usize;
            return self.read_str_bytes(len).map(Value::String);
        }

        // Negative fixint: 0xe0 - 0xff
        if marker >= 0xe0 {
            return Ok(Value::Integer((marker as i8) as i128));
        }

        match marker {
            NIL => Ok(Value::Null),
            NEVER_USED => Err(MsgPackError::InvalidMarker(marker)),
            FALSE => Ok(Value::Bool(false)),
            TRUE => Ok(Value::Bool(true)),

            BIN8 => {
                let len = self.read_u8()? as usize;
                self.read_bin_bytes(len).map(Value::Bytes)
            }
            BIN16 => {
                let len = self.read_u16()? as usize;
                self.read_bin_bytes(len).map(Value::Bytes)
            }
            BIN32 => {
                let len = self.read_u32()? as usize;
                self.read_bin_bytes(len).map(Value::Bytes)
            }

            EXT8 => {
                let len = self.read_u8()? as usize;
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(len).map(Value::Bytes)
            }
            EXT16 => {
                let len = self.read_u16()? as usize;
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(len).map(Value::Bytes)
            }
            EXT32 => {
                let len = self.read_u32()? as usize;
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(len).map(Value::Bytes)
            }

            FLOAT32 => self.read_f32().map(|f| Value::Float(f as f64)),
            FLOAT64 => self.read_f64().map(Value::Float),

            UINT8 => self.read_u8().map(|u| Value::Integer(u as i128)),
            UINT16 => self.read_u16().map(|u| Value::Integer(u as i128)),
            UINT32 => self.read_u32().map(|u| Value::Integer(u as i128)),
            UINT64 => self.read_u64().map(|u| Value::Integer(u as i128)),

            INT8 => self.read_i8().map(|i| Value::Integer(i as i128)),
            INT16 => self.read_i16().map(|i| Value::Integer(i as i128)),
            INT32 => self.read_i32().map(|i| Value::Integer(i as i128)),
            INT64 => self.read_i64().map(|i| Value::Integer(i as i128)),

            FIXEXT1 => {
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(1).map(Value::Bytes)
            }
            FIXEXT2 => {
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(2).map(Value::Bytes)
            }
            FIXEXT4 => {
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(4).map(Value::Bytes)
            }
            FIXEXT8 => {
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(8).map(Value::Bytes)
            }
            FIXEXT16 => {
                let _ext_type = self.read_i8()?;
                self.read_bin_bytes(16).map(Value::Bytes)
            }

            STR8 => {
                let len = self.read_u8()? as usize;
                self.read_str_bytes(len).map(Value::String)
            }
            STR16 => {
                let len = self.read_u16()? as usize;
                self.read_str_bytes(len).map(Value::String)
            }
            STR32 => {
                let len = self.read_u32()? as usize;
                self.read_str_bytes(len).map(Value::String)
            }

            ARRAY16 => {
                let len = self.read_u16()? as usize;
                self.decode_array(len, depth)
            }
            ARRAY32 => {
                let len = self.read_u32()? as usize;
                self.decode_array(len, depth)
            }

            MAP16 => {
                let len = self.read_u16()? as usize;
                self.decode_map(len, depth)
            }
            MAP32 => {
                let len = self.read_u32()? as usize;
                self.decode_map(len, depth)
            }

            _ => Err(MsgPackError::InvalidMarker(marker)),
        }
    }

    fn decode_array(&mut self, len: usize, depth: usize) -> Result<Value, MsgPackError> {
        if len > self.config.max_size {
            return Err(MsgPackError::SizeLimitExceeded {
                size: len,
                limit: self.config.max_size,
            });
        }
        let mut items = Vec::with_capacity(len.min(1024));
        for _ in 0..len {
            items.push(self.decode_value_depth(depth + 1)?);
        }
        Ok(Value::Array(items))
    }

    fn decode_map(&mut self, len: usize, depth: usize) -> Result<Value, MsgPackError> {
        if len > self.config.max_size {
            return Err(MsgPackError::SizeLimitExceeded {
                size: len,
                limit: self.config.max_size,
            });
        }
        let mut entries = Vec::with_capacity(len.min(1024));
        for _ in 0..len {
            let key_val = self.decode_value_depth(depth + 1)?;
            let key = match key_val {
                Value::String(s) => s,
                Value::Integer(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Bytes(b) => core::str::from_utf8(&b)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| format!("{:?}", b)),
                _ => return Err(MsgPackError::InvalidMapKey),
            };
            let val = self.decode_value_depth(depth + 1)?;
            entries.push((key, val));
        }
        Ok(Value::Object(entries))
    }
}

/// Convenience function parsing a MessagePack byte slice into a universal `Value`.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, MsgPackError> {
    let mut decoder = Decoder::new(bytes);
    let val = decoder.decode_value()?;
    Ok(val)
}
