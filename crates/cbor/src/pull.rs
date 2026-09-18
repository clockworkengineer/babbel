//! Streaming pull parser for CBOR (RFC 8949) payloads operating in O(1) memory.

use crate::constants::*;
use crate::error::CborError;

/// Event produced by [`CborPullParser`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CborPullEvent<'a> {
    /// Positive unsigned integer
    Unsigned(u64),
    /// Negative integer: -1 - val
    Negative(u64),
    /// Byte string slice (chunked or definite length)
    ByteString { len: Option<usize>, data: &'a [u8] },
    /// UTF-8 Text string slice (chunked or definite length)
    TextString { len: Option<usize>, text: &'a str },
    /// Array container start with definite or indefinite length
    ArrayStart(Option<usize>),
    /// Map container start with definite or indefinite length
    MapStart(Option<usize>),
    /// Semantic tag number
    Tag(u64),
    /// Simple value (null=22, undefined=23, bool=20/21, unassigned)
    Simple(u8),
    /// Floating point number
    Float(f64),
    /// Indefinite-length container delimiter (0xFF)
    Break,
    /// End of stream
    End,
}

/// A zero-allocation, streaming pull-parser over a CBOR byte slice.
pub struct CborPullParser<'a> {
    input: &'a [u8],
    cursor: usize,
}

impl<'a> CborPullParser<'a> {
    /// Create a new pull parser over the provided byte slice.
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, cursor: 0 }
    }

    /// Pull the next token event from the stream.
    pub fn next_event(&mut self) -> Result<CborPullEvent<'a>, CborError> {
        if self.cursor >= self.input.len() {
            return Ok(CborPullEvent::End);
        }

        let initial_byte = self.read_u8()?;
        let major = initial_byte & MAJOR_TYPE_MASK;
        let info = initial_byte & ADDITIONAL_INFO_MASK;

        match major {
            MAJOR_UNSIGNED_INT => {
                let val = self.read_argument(info)?;
                Ok(CborPullEvent::Unsigned(val))
            }
            MAJOR_NEGATIVE_INT => {
                let val = self.read_argument(info)?;
                Ok(CborPullEvent::Negative(val))
            }
            MAJOR_BYTE_STRING => {
                if info == AI_INDEFINITE {
                    Ok(CborPullEvent::ByteString {
                        len: None,
                        data: &[],
                    })
                } else {
                    let len = self.read_argument(info)? as usize;
                    let data = self.read_bytes(len)?;
                    Ok(CborPullEvent::ByteString {
                        len: Some(len),
                        data,
                    })
                }
            }
            MAJOR_TEXT_STRING => {
                if info == AI_INDEFINITE {
                    Ok(CborPullEvent::TextString {
                        len: None,
                        text: "",
                    })
                } else {
                    let len = self.read_argument(info)? as usize;
                    let bytes = self.read_bytes(len)?;
                    let text = core::str::from_utf8(bytes).map_err(|_| CborError::InvalidUtf8)?;
                    Ok(CborPullEvent::TextString {
                        len: Some(len),
                        text,
                    })
                }
            }
            MAJOR_ARRAY => {
                if info == AI_INDEFINITE {
                    Ok(CborPullEvent::ArrayStart(None))
                } else {
                    let len = self.read_argument(info)? as usize;
                    Ok(CborPullEvent::ArrayStart(Some(len)))
                }
            }
            MAJOR_MAP => {
                if info == AI_INDEFINITE {
                    Ok(CborPullEvent::MapStart(None))
                } else {
                    let len = self.read_argument(info)? as usize;
                    Ok(CborPullEvent::MapStart(Some(len)))
                }
            }
            MAJOR_TAG => {
                let tag_val = self.read_argument(info)?;
                Ok(CborPullEvent::Tag(tag_val))
            }
            MAJOR_SIMPLE => match info {
                SIMPLE_FALSE => Ok(CborPullEvent::Simple(20)),
                SIMPLE_TRUE => Ok(CborPullEvent::Simple(21)),
                SIMPLE_NULL => Ok(CborPullEvent::Simple(22)),
                SIMPLE_UNDEFINED => Ok(CborPullEvent::Simple(23)),
                AI_1_BYTE => {
                    let val = self.read_u8()?;
                    Ok(CborPullEvent::Simple(val))
                }
                FLOAT_16 => {
                    let bytes = self.read_bytes(2)?;
                    let bits = u16::from_be_bytes([bytes[0], bytes[1]]);
                    let f = f16_to_f64(bits);
                    Ok(CborPullEvent::Float(f))
                }
                FLOAT_32 => {
                    let bytes = self.read_bytes(4)?;
                    let f = f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64;
                    Ok(CborPullEvent::Float(f))
                }
                FLOAT_64 => {
                    let bytes = self.read_bytes(8)?;
                    let f = f64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ]);
                    Ok(CborPullEvent::Float(f))
                }
                BREAK_CODE => Ok(CborPullEvent::Break),
                simple => Ok(CborPullEvent::Simple(simple)),
            },
            _ => Err(CborError::InvalidInitialByte(initial_byte)),
        }
    }

    fn read_u8(&mut self) -> Result<u8, CborError> {
        if self.cursor >= self.input.len() {
            return Err(CborError::UnexpectedEof {
                expected: 1,
                available: 0,
            });
        }
        let b = self.input[self.cursor];
        self.cursor += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], CborError> {
        let available = self.input.len().saturating_sub(self.cursor);
        if available < len {
            return Err(CborError::UnexpectedEof {
                expected: len,
                available,
            });
        }
        let slice = &self.input[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(slice)
    }

    fn read_argument(&mut self, info: u8) -> Result<u64, CborError> {
        match info {
            val if val < 24 => Ok(val as u64),
            AI_1_BYTE => self.read_u8().map(|b| b as u64),
            AI_2_BYTES => {
                let bytes = self.read_bytes(2)?;
                Ok(u16::from_be_bytes([bytes[0], bytes[1]]) as u64)
            }
            AI_4_BYTES => {
                let bytes = self.read_bytes(4)?;
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64)
            }
            AI_8_BYTES => {
                let bytes = self.read_bytes(8)?;
                Ok(u64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]))
            }
            _ => Err(CborError::Custom("invalid argument info")),
        }
    }
}

fn f16_to_f64(bits: u16) -> f64 {
    let sign = (bits & 0x8000) != 0;
    let exp = (bits & 0x7c00) >> 10;
    let mant = bits & 0x03ff;

    let s = if sign { -1.0 } else { 1.0 };

    if exp == 0 {
        if mant == 0 {
            if sign { -0.0 } else { 0.0 }
        } else {
            s * (mant as f64) * (2.0f64.powi(-24))
        }
    } else if exp == 0x1f {
        if mant == 0 {
            if sign {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        } else {
            f64::NAN
        }
    } else {
        s * (1.0 + (mant as f64) / 1024.0) * (2.0f64.powi((exp as i32) - 15))
    }
}
