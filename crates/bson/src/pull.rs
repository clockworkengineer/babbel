//! Streaming zero-allocation pull parser for BSON payloads operating in O(1) memory.

use crate::constants::*;
use crate::error::BsonError;

/// Event produced by [`BsonPullParser`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BsonPullEvent<'a> {
    /// Start of a document container with total byte length
    DocumentStart(usize),
    /// End of a document container (0x00 terminator encountered)
    DocumentEnd,
    /// Field key name
    Key(&'a str),
    /// 64-bit IEEE floating-point number
    Double(f64),
    /// UTF-8 string
    String(&'a str),
    /// Embedded document start
    SubDocumentStart(usize),
    /// Embedded array start
    ArrayStart(usize),
    /// Binary data payload with subtype
    Binary {
        subtype: u8,
        data: &'a [u8],
    },
    /// 12-byte BSON ObjectId
    ObjectId(&'a [u8]),
    /// Boolean value
    Boolean(bool),
    /// UTC datetime in milliseconds since Unix epoch
    DateTime(i64),
    /// Null or undefined value
    Null,
    /// Regular expression pattern and options
    Regex {
        pattern: &'a str,
        options: &'a str,
    },
    /// 32-bit signed integer
    Int32(i32),
    /// 64-bit BSON replication timestamp
    Timestamp(u64),
    /// 64-bit signed integer
    Int64(i64),
    /// 128-bit decimal floating point raw bytes
    Decimal128(&'a [u8]),
    /// BSON MinKey sentinel
    MinKey,
    /// BSON MaxKey sentinel
    MaxKey,
    /// End of stream
    End,
}

/// A zero-allocation, streaming pull-parser over a BSON byte slice.
pub struct BsonPullParser<'a> {
    input: &'a [u8],
    cursor: usize,
    expect_value_type: Option<u8>,
}

impl<'a> BsonPullParser<'a> {
    /// Create a new BSON pull parser over the given byte slice.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            cursor: 0,
            expect_value_type: None,
        }
    }

    /// Pull the next syntactic event from the stream.
    pub fn next_event(&mut self) -> Result<BsonPullEvent<'a>, BsonError> {
        if self.cursor >= self.input.len() {
            return Ok(BsonPullEvent::End);
        }

        if let Some(elem_type) = self.expect_value_type.take() {
            return self.read_value_event(elem_type);
        }

        // At beginning of doc or next field
        if self.cursor == 0 {
            let total_len = self.read_i32()? as usize;
            return Ok(BsonPullEvent::DocumentStart(total_len));
        }

        let type_marker = self.read_u8()?;
        if type_marker == 0 {
            return Ok(BsonPullEvent::DocumentEnd);
        }

        let key = self.read_cstring()?;
        self.expect_value_type = Some(type_marker);
        Ok(BsonPullEvent::Key(key))
    }

    fn read_value_event(&mut self, elem_type: u8) -> Result<BsonPullEvent<'a>, BsonError> {
        match elem_type {
            TYPE_DOUBLE => {
                let bytes = self.read_bytes(8)?;
                let f = f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                Ok(BsonPullEvent::Double(f))
            }
            TYPE_STRING => {
                let len = self.read_i32()?;
                if len < 1 {
                    return Err(BsonError::InvalidUtf8);
                }
                let str_bytes = self.read_bytes((len - 1) as usize)?;
                let null_byte = self.read_u8()?;
                if null_byte != 0 {
                    return Err(BsonError::InvalidCString);
                }
                let text = core::str::from_utf8(str_bytes).map_err(|_| BsonError::InvalidUtf8)?;
                Ok(BsonPullEvent::String(text))
            }
            TYPE_DOCUMENT => {
                let total_len = self.read_i32()? as usize;
                Ok(BsonPullEvent::SubDocumentStart(total_len))
            }
            TYPE_ARRAY => {
                let total_len = self.read_i32()? as usize;
                Ok(BsonPullEvent::ArrayStart(total_len))
            }
            TYPE_BINARY => {
                let len = self.read_i32()?;
                if len < 0 {
                    return Err(BsonError::InvalidDocumentLength {
                        length: len,
                        available: self.remaining(),
                    });
                }
                let subtype = self.read_u8()?;
                let data = self.read_bytes(len as usize)?;
                Ok(BsonPullEvent::Binary { subtype, data })
            }
            TYPE_UNDEFINED | TYPE_NULL => Ok(BsonPullEvent::Null),
            TYPE_OBJECT_ID => {
                let data = self.read_bytes(12)?;
                Ok(BsonPullEvent::ObjectId(data))
            }
            TYPE_BOOLEAN => {
                let b = self.read_u8()?;
                Ok(BsonPullEvent::Boolean(b != 0))
            }
            TYPE_DATETIME => {
                let ms = self.read_i64()?;
                Ok(BsonPullEvent::DateTime(ms))
            }
            TYPE_REGEX => {
                let pattern = self.read_cstring()?;
                let options = self.read_cstring()?;
                Ok(BsonPullEvent::Regex { pattern, options })
            }
            TYPE_DB_POINTER => {
                let len = self.read_i32()?;
                if len > 0 {
                    let _ = self.read_bytes(len as usize)?;
                }
                let _ = self.read_bytes(12)?;
                Ok(BsonPullEvent::Null)
            }
            TYPE_JS_CODE | TYPE_SYMBOL => {
                let len = self.read_i32()?;
                if len < 1 {
                    return Err(BsonError::InvalidUtf8);
                }
                let str_bytes = self.read_bytes((len - 1) as usize)?;
                let _ = self.read_u8()?;
                let text = core::str::from_utf8(str_bytes).map_err(|_| BsonError::InvalidUtf8)?;
                Ok(BsonPullEvent::String(text))
            }
            TYPE_JS_CODE_SCOPE => {
                let _total_len = self.read_i32()?;
                let len = self.read_i32()?;
                if len < 1 {
                    return Err(BsonError::InvalidUtf8);
                }
                let str_bytes = self.read_bytes((len - 1) as usize)?;
                let _ = self.read_u8()?;
                let text = core::str::from_utf8(str_bytes).map_err(|_| BsonError::InvalidUtf8)?;
                Ok(BsonPullEvent::String(text))
            }
            TYPE_INT32 => {
                let v = self.read_i32()?;
                Ok(BsonPullEvent::Int32(v))
            }
            TYPE_TIMESTAMP => {
                let ts = self.read_u64()?;
                Ok(BsonPullEvent::Timestamp(ts))
            }
            TYPE_INT64 => {
                let v = self.read_i64()?;
                Ok(BsonPullEvent::Int64(v))
            }
            TYPE_DECIMAL128 => {
                let bytes = self.read_bytes(16)?;
                Ok(BsonPullEvent::Decimal128(bytes))
            }
            TYPE_MIN_KEY => Ok(BsonPullEvent::MinKey),
            TYPE_MAX_KEY => Ok(BsonPullEvent::MaxKey),
            other => Err(BsonError::InvalidTypeMarker(other)),
        }
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.cursor)
    }

    fn read_u8(&mut self) -> Result<u8, BsonError> {
        if self.cursor >= self.input.len() {
            return Err(BsonError::UnexpectedEof { expected: 1, available: 0 });
        }
        let b = self.input[self.cursor];
        self.cursor += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], BsonError> {
        let avail = self.remaining();
        if avail < len {
            return Err(BsonError::UnexpectedEof { expected: len, available: avail });
        }
        let slice = &self.input[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(slice)
    }

    fn read_cstring(&mut self) -> Result<&'a str, BsonError> {
        let start = self.cursor;
        while self.cursor < self.input.len() {
            if self.input[self.cursor] == 0 {
                let slice = &self.input[start..self.cursor];
                self.cursor += 1;
                return core::str::from_utf8(slice).map_err(|_| BsonError::InvalidUtf8);
            }
            self.cursor += 1;
        }
        Err(BsonError::InvalidCString)
    }

    fn read_i32(&mut self) -> Result<i32, BsonError> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i64(&mut self) -> Result<i64, BsonError> {
        let bytes = self.read_bytes(8)?;
        Ok(i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_u64(&mut self) -> Result<u64, BsonError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bson_pull_parser_simple_doc() {
        // Document with key "x" = int32(42)
        // total len (4 bytes) = 4 + 1(type) + 2(cstring "x\0") + 4(int32) + 1(terminator) = 12
        let mut data = Vec::new();
        data.extend_from_slice(&(12i32).to_le_bytes());
        data.push(TYPE_INT32);
        data.extend_from_slice(b"x\0");
        data.extend_from_slice(&(42i32).to_le_bytes());
        data.push(0x00);

        let mut parser = BsonPullParser::new(&data);
        assert_eq!(parser.next_event().unwrap(), BsonPullEvent::DocumentStart(12));
        assert_eq!(parser.next_event().unwrap(), BsonPullEvent::Key("x"));
        assert_eq!(parser.next_event().unwrap(), BsonPullEvent::Int32(42));
        assert_eq!(parser.next_event().unwrap(), BsonPullEvent::DocumentEnd);
        assert_eq!(parser.next_event().unwrap(), BsonPullEvent::End);
    }
}
