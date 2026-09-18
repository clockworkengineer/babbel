//! Streaming pull parser for MessagePack payloads operating in O(1) memory.

use crate::constants::*;
use crate::error::MsgPackError;

/// Event produced by [`MsgPackPullParser`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MsgPackPullEvent<'a> {
    /// Nil / null marker (0xc0)
    Nil,
    /// Boolean value (0xc2 or 0xc3)
    Bool(bool),
    /// Integer value (fixint, uint, or int)
    Integer(i128),
    /// 32-bit or 64-bit IEEE floating-point number
    Float(f64),
    /// UTF-8 string slice
    String(&'a str),
    /// Binary byte buffer slice
    Binary(&'a [u8]),
    /// Array container start with exact item count
    ArrayStart(usize),
    /// Map container start with exact key-value pair count
    MapStart(usize),
    /// Extension type with type code and payload slice
    Extension { ext_type: i8, data: &'a [u8] },
    /// End of stream
    End,
}

/// A zero-allocation, streaming pull-parser over a MessagePack byte slice.
pub struct MsgPackPullParser<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl<'a> MsgPackPullParser<'a> {
    /// Create a new pull parser over the provided byte slice.
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, cursor: 0 }
    }

    /// Pull the next token event from the stream.
    pub fn next_event(&mut self) -> Result<MsgPackPullEvent<'a>, MsgPackError> {
        if self.cursor >= self.input.len() {
            return Ok(MsgPackPullEvent::End);
        }

        let marker = self.read_u8()?;

        // Positive fixint: 0x00..=0x7f
        if marker & POS_FIXINT_MASK == POS_FIXINT_PREFIX {
            return Ok(MsgPackPullEvent::Integer(marker as i128));
        }

        // Fixmap: 0x80..=0x8f
        if (marker & FIXMAP_MASK) == FIXMAP_PREFIX {
            let len = (marker & 0x0f) as usize;
            return Ok(MsgPackPullEvent::MapStart(len));
        }

        // Fixarray: 0x90..=0x9f
        if (marker & FIXARRAY_MASK) == FIXARRAY_PREFIX {
            let len = (marker & 0x0f) as usize;
            return Ok(MsgPackPullEvent::ArrayStart(len));
        }

        // Fixstr: 0xa0..=0xbf
        if (marker & FIXSTR_MASK) == FIXSTR_PREFIX {
            let len = (marker & 0x1f) as usize;
            let bytes = self.read_bytes(len)?;
            let text = core::str::from_utf8(bytes).map_err(|_| MsgPackError::InvalidUtf8)?;
            return Ok(MsgPackPullEvent::String(text));
        }

        // Negative fixint: 0xe0..=0xff
        if (marker & NEG_FIXINT_MASK) == NEG_FIXINT_PREFIX {
            return Ok(MsgPackPullEvent::Integer((marker as i8) as i128));
        }

        match marker {
            NIL => Ok(MsgPackPullEvent::Nil),
            FALSE => Ok(MsgPackPullEvent::Bool(false)),
            TRUE => Ok(MsgPackPullEvent::Bool(true)),
            BIN8 => {
                let len = self.read_u8()? as usize;
                let data = self.read_bytes(len)?;
                Ok(MsgPackPullEvent::Binary(data))
            }
            BIN16 => {
                let len = self.read_u16()? as usize;
                let data = self.read_bytes(len)?;
                Ok(MsgPackPullEvent::Binary(data))
            }
            BIN32 => {
                let len = self.read_u32()? as usize;
                let data = self.read_bytes(len)?;
                Ok(MsgPackPullEvent::Binary(data))
            }
            EXT8 => {
                let len = self.read_u8()? as usize;
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(len)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            EXT16 => {
                let len = self.read_u16()? as usize;
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(len)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            EXT32 => {
                let len = self.read_u32()? as usize;
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(len)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            FLOAT32 => {
                let bytes = self.read_bytes(4)?;
                let f = f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64;
                Ok(MsgPackPullEvent::Float(f))
            }
            FLOAT64 => {
                let bytes = self.read_bytes(8)?;
                let f = f64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]);
                Ok(MsgPackPullEvent::Float(f))
            }
            UINT8 => {
                let val = self.read_u8()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            UINT16 => {
                let val = self.read_u16()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            UINT32 => {
                let val = self.read_u32()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            UINT64 => {
                let val = self.read_u64()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            INT8 => {
                let val = self.read_i8()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            INT16 => {
                let val = self.read_i16()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            INT32 => {
                let val = self.read_i32()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            INT64 => {
                let val = self.read_i64()?;
                Ok(MsgPackPullEvent::Integer(val as i128))
            }
            FIXEXT1 => {
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(1)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            FIXEXT2 => {
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(2)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            FIXEXT4 => {
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(4)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            FIXEXT8 => {
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(8)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            FIXEXT16 => {
                let ext_type = self.read_i8()?;
                let data = self.read_bytes(16)?;
                Ok(MsgPackPullEvent::Extension { ext_type, data })
            }
            STR8 => {
                let len = self.read_u8()? as usize;
                let bytes = self.read_bytes(len)?;
                let text = core::str::from_utf8(bytes).map_err(|_| MsgPackError::InvalidUtf8)?;
                Ok(MsgPackPullEvent::String(text))
            }
            STR16 => {
                let len = self.read_u16()? as usize;
                let bytes = self.read_bytes(len)?;
                let text = core::str::from_utf8(bytes).map_err(|_| MsgPackError::InvalidUtf8)?;
                Ok(MsgPackPullEvent::String(text))
            }
            STR32 => {
                let len = self.read_u32()? as usize;
                let bytes = self.read_bytes(len)?;
                let text = core::str::from_utf8(bytes).map_err(|_| MsgPackError::InvalidUtf8)?;
                Ok(MsgPackPullEvent::String(text))
            }
            ARRAY16 => {
                let len = self.read_u16()? as usize;
                Ok(MsgPackPullEvent::ArrayStart(len))
            }
            ARRAY32 => {
                let len = self.read_u32()? as usize;
                Ok(MsgPackPullEvent::ArrayStart(len))
            }
            MAP16 => {
                let len = self.read_u16()? as usize;
                Ok(MsgPackPullEvent::MapStart(len))
            }
            MAP32 => {
                let len = self.read_u32()? as usize;
                Ok(MsgPackPullEvent::MapStart(len))
            }
            _ => Err(MsgPackError::InvalidMarker(marker)),
        }
    }

    fn read_u8(&mut self) -> Result<u8, MsgPackError> {
        if self.cursor >= self.input.len() {
            return Err(MsgPackError::UnexpectedEof {
                expected: 1,
                available: 0,
            });
        }
        let b = self.input[self.cursor];
        self.cursor += 1;
        Ok(b)
    }

    fn read_i8(&mut self) -> Result<i8, MsgPackError> {
        self.read_u8().map(|b| b as i8)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], MsgPackError> {
        let available = self.input.len().saturating_sub(self.cursor);
        if available < len {
            return Err(MsgPackError::UnexpectedEof {
                expected: len,
                available,
            });
        }
        let slice = &self.input[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(slice)
    }

    fn read_u16(&mut self) -> Result<u16, MsgPackError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, MsgPackError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> Result<u64, MsgPackError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_i16(&mut self) -> Result<i16, MsgPackError> {
        self.read_u16().map(|v| v as i16)
    }

    fn read_i32(&mut self) -> Result<i32, MsgPackError> {
        self.read_u32().map(|v| v as i32)
    }

    fn read_i64(&mut self) -> Result<i64, MsgPackError> {
        self.read_u64().map(|v| v as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msgpack_pull_parser_scalars() {
        let data = [
            0x05, // pos fixint 5
            0xe5, // neg fixint -27
            0xc0, // nil
            0xc3, // true
            0xa4, b't', b'e', b's', b't', // fixstr "test"
            0x92, 0x01, 0x02, // fixarray [1, 2]
        ];

        let mut parser = MsgPackPullParser::new(&data);
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::Integer(5));
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::Integer(-27));
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::Nil);
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::Bool(true));
        assert_eq!(
            parser.next_event().unwrap(),
            MsgPackPullEvent::String("test")
        );
        assert_eq!(
            parser.next_event().unwrap(),
            MsgPackPullEvent::ArrayStart(2)
        );
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::Integer(1));
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::Integer(2));
        assert_eq!(parser.next_event().unwrap(), MsgPackPullEvent::End);
    }
}
