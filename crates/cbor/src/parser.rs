//! High-performance CBOR (RFC 8949) decoder into universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use babbel_core::Value;
use crate::constants::*;
use crate::error::CborError;

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

/// CBOR decoder reading from a byte slice.
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
    fn read_byte(&mut self) -> Result<u8, CborError> {
        if self.cursor < self.input.len() {
            let b = self.input[self.cursor];
            self.cursor += 1;
            Ok(b)
        } else {
            Err(CborError::UnexpectedEof {
                expected: 1,
                available: 0,
            })
        }
    }

    #[inline]
    fn peek_byte(&self) -> Result<u8, CborError> {
        if self.cursor < self.input.len() {
            Ok(self.input[self.cursor])
        } else {
            Err(CborError::UnexpectedEof {
                expected: 1,
                available: 0,
            })
        }
    }

    #[inline]
    fn read_slice(&mut self, len: usize) -> Result<&'a [u8], CborError> {
        let avail = self.remaining();
        if avail < len {
            return Err(CborError::UnexpectedEof {
                expected: len,
                available: avail,
            });
        }
        let start = self.cursor;
        self.cursor += len;
        Ok(&self.input[start..self.cursor])
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, CborError> {
        let slice = self.read_slice(2)?;
        Ok(u16::from_be_bytes([slice[0], slice[1]]))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, CborError> {
        let slice = self.read_slice(4)?;
        Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, CborError> {
        let slice = self.read_slice(8)?;
        Ok(u64::from_be_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    fn read_length(&mut self, info: u8) -> Result<u64, CborError> {
        match info {
            0..=23 => Ok(info as u64),
            AI_1_BYTE => self.read_byte().map(|b| b as u64),
            AI_2_BYTES => self.read_u16().map(|u| u as u64),
            AI_4_BYTES => self.read_u32().map(|u| u as u64),
            AI_8_BYTES => self.read_u64(),
            _ => Err(CborError::InvalidInitialByte(info)),
        }
    }

    /// Decodes the next value in the stream into a universal `Value`.
    pub fn decode_value(&mut self) -> Result<Value, CborError> {
        if self.input.len() > self.config.max_size {
            return Err(CborError::SizeLimitExceeded {
                size: self.input.len(),
                limit: self.config.max_size,
            });
        }
        self.decode_value_depth(0)
    }

    fn decode_value_depth(&mut self, depth: usize) -> Result<Value, CborError> {
        if depth > self.config.max_depth {
            return Err(CborError::RecursionLimitExceeded(depth));
        }

        let initial_byte = self.read_byte()?;
        let major = initial_byte & MAJOR_TYPE_MASK;
        let info = initial_byte & ADDITIONAL_INFO_MASK;

        match major {
            MAJOR_UNSIGNED_INT => {
                let n = self.read_length(info)?;
                Ok(Value::Integer(n as i128))
            }
            MAJOR_NEGATIVE_INT => {
                let n = self.read_length(info)?;
                // RFC 8949: value is -1 - n
                let val = -1i128 - (n as i128);
                Ok(Value::Integer(val))
            }
            MAJOR_BYTE_STRING => {
                if info == AI_INDEFINITE {
                    // Indefinite-length byte string (chunks ending with BREAK)
                    let mut bytes = Vec::new();
                    while self.peek_byte()? != BYTE_BREAK {
                        let chunk = self.decode_value_depth(depth + 1)?;
                        if let Value::Bytes(b) = chunk {
                            bytes.extend_from_slice(&b);
                        } else {
                            return Err(CborError::InvalidInitialByte(initial_byte));
                        }
                    }
                    self.read_byte()?; // consume BREAK
                    Ok(Value::Bytes(bytes))
                } else {
                    let len = self.read_length(info)? as usize;
                    if len > self.config.max_size {
                        return Err(CborError::SizeLimitExceeded {
                            size: len,
                            limit: self.config.max_size,
                        });
                    }
                    let slice = self.read_slice(len)?;
                    Ok(Value::Bytes(slice.to_vec()))
                }
            }
            MAJOR_TEXT_STRING => {
                if info == AI_INDEFINITE {
                    // Indefinite-length text string (chunks ending with BREAK)
                    let mut s = String::new();
                    while self.peek_byte()? != BYTE_BREAK {
                        let chunk = self.decode_value_depth(depth + 1)?;
                        if let Value::String(chunk_str) = chunk {
                            s.push_str(&chunk_str);
                        } else {
                            return Err(CborError::InvalidInitialByte(initial_byte));
                        }
                    }
                    self.read_byte()?; // consume BREAK
                    Ok(Value::String(s))
                } else {
                    let len = self.read_length(info)? as usize;
                    if len > self.config.max_size {
                        return Err(CborError::SizeLimitExceeded {
                            size: len,
                            limit: self.config.max_size,
                        });
                    }
                    let slice = self.read_slice(len)?;
                    core::str::from_utf8(slice)
                        .map(|s| Value::String(s.to_string()))
                        .map_err(|_| CborError::InvalidUtf8)
                }
            }
            MAJOR_ARRAY => {
                if info == AI_INDEFINITE {
                    let mut items = Vec::new();
                    while self.peek_byte()? != BYTE_BREAK {
                        items.push(self.decode_value_depth(depth + 1)?);
                    }
                    self.read_byte()?; // consume BREAK
                    Ok(Value::Array(items))
                } else {
                    let len = self.read_length(info)? as usize;
                    if len > self.config.max_size {
                        return Err(CborError::SizeLimitExceeded {
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
            }
            MAJOR_MAP => {
                if info == AI_INDEFINITE {
                    let mut entries = Vec::new();
                    while self.peek_byte()? != BYTE_BREAK {
                        let key_val = self.decode_value_depth(depth + 1)?;
                        let key = match key_val {
                            Value::String(s) => s,
                            Value::Integer(i) => i.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Bytes(b) => core::str::from_utf8(&b)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|_| format!("{:?}", b)),
                            _ => return Err(CborError::InvalidMapKey),
                        };
                        let val = self.decode_value_depth(depth + 1)?;
                        entries.push((key, val));
                    }
                    self.read_byte()?; // consume BREAK
                    Ok(Value::Object(entries))
                } else {
                    let len = self.read_length(info)? as usize;
                    if len > self.config.max_size {
                        return Err(CborError::SizeLimitExceeded {
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
                            _ => return Err(CborError::InvalidMapKey),
                        };
                        let val = self.decode_value_depth(depth + 1)?;
                        entries.push((key, val));
                    }
                    Ok(Value::Object(entries))
                }
            }
            MAJOR_TAG => {
                let _tag_num = self.read_length(info)?;
                // Decode tagged inner value
                self.decode_value_depth(depth + 1)
            }
            MAJOR_SIMPLE => {
                match info {
                    SIMPLE_FALSE => Ok(Value::Bool(false)),
                    SIMPLE_TRUE => Ok(Value::Bool(true)),
                    SIMPLE_NULL | SIMPLE_UNDEFINED => Ok(Value::Null),
                    AI_1_BYTE => {
                        let val = self.read_byte()?;
                        match val {
                            SIMPLE_FALSE => Ok(Value::Bool(false)),
                            SIMPLE_TRUE => Ok(Value::Bool(true)),
                            SIMPLE_NULL | SIMPLE_UNDEFINED => Ok(Value::Null),
                            _ => Err(CborError::UnsupportedSimpleValue(val)),
                        }
                    }
                    FLOAT_16 => {
                        let bits = self.read_u16()?;
                        Ok(Value::Float(decode_f16(bits)))
                    }
                    FLOAT_32 => {
                        let bits = self.read_u32()?;
                        Ok(Value::Float(f32::from_bits(bits) as f64))
                    }
                    FLOAT_64 => {
                        let bits = self.read_u64()?;
                        Ok(Value::Float(f64::from_bits(bits)))
                    }
                    BREAK_CODE => Err(CborError::UnexpectedBreak),
                    _ => Err(CborError::InvalidInitialByte(initial_byte)),
                }
            }
            _ => Err(CborError::InvalidInitialByte(initial_byte)),
        }
    }
}

/// Decodes IEEE 754 half-precision float bits into f64.
fn decode_f16(bits: u16) -> f64 {
    let sign = (bits >> 15) & 1;
    let exp = (bits >> 10) & 0x1f;
    let mant = bits & 0x3ff;
    if exp == 0 {
        if mant == 0 {
            if sign == 1 { -0.0 } else { 0.0 }
        } else {
            let val = (mant as f64) / 1024.0 * (2.0f64).powi(-14);
            if sign == 1 { -val } else { val }
        }
    } else if exp == 31 {
        if mant == 0 {
            if sign == 1 { f64::NEG_INFINITY } else { f64::INFINITY }
        } else {
            f64::NAN
        }
    } else {
        let val = (1.0 + (mant as f64) / 1024.0) * (2.0f64).powi((exp as i32) - 15);
        if sign == 1 { -val } else { val }
    }
}

/// Convenience function parsing a CBOR byte slice into a universal `Value`.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, CborError> {
    let mut decoder = Decoder::new(bytes);
    let val = decoder.decode_value()?;
    Ok(val)
}
