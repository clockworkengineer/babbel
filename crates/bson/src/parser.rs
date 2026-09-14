//! High-performance BSON decoder into universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use babbel_core::Value;
use crate::constants::*;
use crate::error::BsonError;

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

/// BSON decoder reading from a byte slice.
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
    fn read_byte(&mut self) -> Result<u8, BsonError> {
        if self.cursor < self.input.len() {
            let b = self.input[self.cursor];
            self.cursor += 1;
            Ok(b)
        } else {
            Err(BsonError::UnexpectedEof {
                expected: 1,
                available: 0,
            })
        }
    }

    #[inline]
    fn read_slice(&mut self, len: usize) -> Result<&'a [u8], BsonError> {
        let avail = self.remaining();
        if avail < len {
            return Err(BsonError::UnexpectedEof {
                expected: len,
                available: avail,
            });
        }
        let start = self.cursor;
        self.cursor += len;
        Ok(&self.input[start..self.cursor])
    }

    #[inline]
    fn read_i32(&mut self) -> Result<i32, BsonError> {
        let slice = self.read_slice(4)?;
        Ok(i32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    #[inline]
    fn read_i64(&mut self) -> Result<i64, BsonError> {
        let slice = self.read_slice(8)?;
        Ok(i64::from_le_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, BsonError> {
        let slice = self.read_slice(8)?;
        Ok(u64::from_le_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    #[inline]
    fn read_f64(&mut self) -> Result<f64, BsonError> {
        let slice = self.read_slice(8)?;
        Ok(f64::from_le_bytes([
            slice[0], slice[1], slice[2], slice[3],
            slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    fn read_cstring(&mut self) -> Result<String, BsonError> {
        let start = self.cursor;
        while self.cursor < self.input.len() {
            if self.input[self.cursor] == 0 {
                let slice = &self.input[start..self.cursor];
                self.cursor += 1; // skip null terminator
                return core::str::from_utf8(slice)
                    .map(|s| s.to_string())
                    .map_err(|_| BsonError::InvalidUtf8);
            }
            self.cursor += 1;
        }
        Err(BsonError::InvalidCString)
    }

    fn read_string(&mut self) -> Result<String, BsonError> {
        let len = self.read_i32()?;
        if len < 1 {
            return Err(BsonError::InvalidUtf8);
        }
        let str_bytes_len = (len - 1) as usize;
        if str_bytes_len > self.config.max_size {
            return Err(BsonError::SizeLimitExceeded {
                size: str_bytes_len,
                limit: self.config.max_size,
            });
        }
        let slice = self.read_slice(str_bytes_len)?;
        let null_byte = self.read_byte()?;
        if null_byte != 0 {
            return Err(BsonError::InvalidCString);
        }
        core::str::from_utf8(slice)
            .map(|s| s.to_string())
            .map_err(|_| BsonError::InvalidUtf8)
    }

    /// Decodes the top-level document into a universal `Value`.
    pub fn decode_document(&mut self) -> Result<Value, BsonError> {
        if self.input.len() > self.config.max_size {
            return Err(BsonError::SizeLimitExceeded {
                size: self.input.len(),
                limit: self.config.max_size,
            });
        }
        self.decode_document_depth(0)
    }

    fn decode_document_depth(&mut self, depth: usize) -> Result<Value, BsonError> {
        if depth > self.config.max_depth {
            return Err(BsonError::RecursionLimitExceeded(depth));
        }

        let doc_start = self.cursor;
        let total_len = self.read_i32()?;
        if total_len < 5 {
            return Err(BsonError::InvalidDocumentLength {
                length: total_len,
                available: self.remaining(),
            });
        }
        let total_usize = total_len as usize;
        if total_usize > self.config.max_size {
            return Err(BsonError::SizeLimitExceeded {
                size: total_usize,
                limit: self.config.max_size,
            });
        }
        let expected_end = doc_start + total_usize;
        if expected_end > self.input.len() {
            return Err(BsonError::InvalidDocumentLength {
                length: total_len,
                available: self.input.len() - doc_start,
            });
        }

        let mut entries = Vec::new();
        while self.cursor < expected_end - 1 {
            let elem_type = self.read_byte()?;
            if elem_type == 0 {
                // Unexpected early terminator
                break;
            }
            let key = self.read_cstring()?;
            let value = self.decode_element_value(elem_type, depth + 1)?;
            entries.push((key, value));
        }

        let terminator = self.read_byte()?;
        if terminator != 0 {
            return Err(BsonError::InvalidCString);
        }

        if depth == 0 && !entries.is_empty() && entries.iter().enumerate().all(|(i, (k, _))| k == &i.to_string()) {
            return Ok(Value::Array(entries.into_iter().map(|(_, v)| v).collect()));
        }

        Ok(Value::Object(entries))
    }

    fn decode_array_depth(&mut self, depth: usize) -> Result<Value, BsonError> {
        let doc_val = self.decode_document_depth(depth)?;
        if let Value::Object(entries) = doc_val {
            let items: Vec<Value> = entries.into_iter().map(|(_, v)| v).collect();
            Ok(Value::Array(items))
        } else {
            Ok(doc_val)
        }
    }

    fn decode_element_value(&mut self, elem_type: u8, depth: usize) -> Result<Value, BsonError> {
        match elem_type {
            TYPE_DOUBLE => self.read_f64().map(Value::Float),
            TYPE_STRING => self.read_string().map(Value::String),
            TYPE_DOCUMENT => self.decode_document_depth(depth),
            TYPE_ARRAY => self.decode_array_depth(depth),
            TYPE_BINARY => {
                let len = self.read_i32()?;
                if len < 0 {
                    return Err(BsonError::InvalidDocumentLength {
                        length: len,
                        available: self.remaining(),
                    });
                }
                let len_usize = len as usize;
                if len_usize > self.config.max_size {
                    return Err(BsonError::SizeLimitExceeded {
                        size: len_usize,
                        limit: self.config.max_size,
                    });
                }
                let _subtype = self.read_byte()?;
                let bytes = self.read_slice(len_usize)?.to_vec();
                Ok(Value::Bytes(bytes))
            }
            TYPE_UNDEFINED | TYPE_NULL => Ok(Value::Null),
            TYPE_OBJECT_ID => {
                let bytes = self.read_slice(12)?.to_vec();
                Ok(Value::Bytes(bytes))
            }
            TYPE_BOOLEAN => {
                let b = self.read_byte()?;
                Ok(Value::Bool(b != 0))
            }
            TYPE_DATETIME => {
                let ms = self.read_i64()?;
                Ok(Value::Integer(ms as i128))
            }
            TYPE_REGEX => {
                let pattern = self.read_cstring()?;
                let options = self.read_cstring()?;
                Ok(Value::String(format!("/{}/{}", pattern, options)))
            }
            TYPE_DB_POINTER => {
                let _str = self.read_string()?;
                let _id = self.read_slice(12)?;
                Ok(Value::Null)
            }
            TYPE_JS_CODE | TYPE_SYMBOL => self.read_string().map(Value::String),
            TYPE_JS_CODE_SCOPE => {
                let _total_len = self.read_i32()?;
                let code = self.read_string()?;
                let scope = self.decode_document_depth(depth)?;
                Ok(Value::Object(vec![
                    ("code".into(), Value::String(code)),
                    ("scope".into(), scope),
                ]))
            }
            TYPE_INT32 => {
                let i = self.read_i32()?;
                Ok(Value::Integer(i as i128))
            }
            TYPE_TIMESTAMP => {
                let ts = self.read_u64()?;
                Ok(Value::Integer(ts as i128))
            }
            TYPE_INT64 => {
                let i = self.read_i64()?;
                Ok(Value::Integer(i as i128))
            }
            TYPE_DECIMAL128 => {
                let bytes = self.read_slice(16)?.to_vec();
                Ok(Value::Bytes(bytes))
            }
            TYPE_MIN_KEY => Ok(Value::String("$minKey".into())),
            TYPE_MAX_KEY => Ok(Value::String("$maxKey".into())),
            _ => Err(BsonError::InvalidTypeMarker(elem_type)),
        }
    }
}

/// Convenience function parsing a BSON byte slice into a universal `Value`.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, BsonError> {
    let mut decoder = Decoder::new(bytes);
    decoder.decode_document()
}
