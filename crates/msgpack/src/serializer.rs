//! High-performance MessagePack serializer for universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::constants::*;
use crate::error::MsgPackError;
use babbel_core::{Value, io::IDestination};

/// Helper to write raw byte slices into any [`IDestination`].
#[inline]
pub fn write_raw_bytes(dest: &mut dyn IDestination, bytes: &[u8]) {
    for &b in bytes {
        dest.add_byte(b);
    }
}

/// Serializes a universal `Value` into MessagePack bytes, writing to any [`IDestination`].
pub fn serialize_to_dest(value: &Value, dest: &mut dyn IDestination) -> Result<(), MsgPackError> {
    match value {
        Value::Null => {
            dest.add_byte(NIL);
        }
        Value::Bool(b) => {
            dest.add_byte(if *b { TRUE } else { FALSE });
        }
        Value::Integer(i) => {
            serialize_integer(*i, dest);
        }
        Value::Float(f) => {
            dest.add_byte(FLOAT64);
            write_raw_bytes(dest, &f.to_be_bytes());
        }
        Value::String(s) => {
            serialize_str(s, dest);
        }
        Value::Bytes(b) => {
            serialize_bin(b, dest);
        }
        Value::Array(items) => {
            let len = items.len();
            if len <= 15 {
                dest.add_byte(FIXARRAY_PREFIX | (len as u8));
            } else if len <= 0xffff {
                dest.add_byte(ARRAY16);
                write_raw_bytes(dest, &(len as u16).to_be_bytes());
            } else {
                dest.add_byte(ARRAY32);
                write_raw_bytes(dest, &(len as u32).to_be_bytes());
            }
            for item in items {
                serialize_to_dest(item, dest)?;
            }
        }
        Value::Object(entries) => {
            let len = entries.len();
            if len <= 15 {
                dest.add_byte(FIXMAP_PREFIX | (len as u8));
            } else if len <= 0xffff {
                dest.add_byte(MAP16);
                write_raw_bytes(dest, &(len as u16).to_be_bytes());
            } else {
                dest.add_byte(MAP32);
                write_raw_bytes(dest, &(len as u32).to_be_bytes());
            }
            for (key, val) in entries {
                serialize_str(key, dest);
                serialize_to_dest(val, dest)?;
            }
        }
    }
    Ok(())
}

fn serialize_integer(i: i128, dest: &mut dyn IDestination) {
    if i >= 0 {
        if i <= 127 {
            dest.add_byte(i as u8);
        } else if i <= 0xff {
            dest.add_byte(UINT8);
            dest.add_byte(i as u8);
        } else if i <= 0xffff {
            dest.add_byte(UINT16);
            write_raw_bytes(dest, &(i as u16).to_be_bytes());
        } else if i <= 0xffff_ffff {
            dest.add_byte(UINT32);
            write_raw_bytes(dest, &(i as u32).to_be_bytes());
        } else {
            dest.add_byte(UINT64);
            write_raw_bytes(dest, &(i as u64).to_be_bytes());
        }
    } else if i >= -32 {
        dest.add_byte((i as i8) as u8);
    } else if i >= -128 {
        dest.add_byte(INT8);
        dest.add_byte((i as i8) as u8);
    } else if i >= -32768 {
        dest.add_byte(INT16);
        write_raw_bytes(dest, &(i as i16).to_be_bytes());
    } else if i >= -2147483648 {
        dest.add_byte(INT32);
        write_raw_bytes(dest, &(i as i32).to_be_bytes());
    } else {
        dest.add_byte(INT64);
        write_raw_bytes(dest, &(i as i64).to_be_bytes());
    }
}

fn serialize_str(s: &str, dest: &mut dyn IDestination) {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len <= 31 {
        dest.add_byte(FIXSTR_PREFIX | (len as u8));
    } else if len <= 0xff {
        dest.add_byte(STR8);
        dest.add_byte(len as u8);
    } else if len <= 0xffff {
        dest.add_byte(STR16);
        write_raw_bytes(dest, &(len as u16).to_be_bytes());
    } else {
        dest.add_byte(STR32);
        write_raw_bytes(dest, &(len as u32).to_be_bytes());
    }
    dest.add_bytes(s);
}

fn serialize_bin(b: &[u8], dest: &mut dyn IDestination) {
    let len = b.len();
    if len <= 0xff {
        dest.add_byte(BIN8);
        dest.add_byte(len as u8);
    } else if len <= 0xffff {
        dest.add_byte(BIN16);
        write_raw_bytes(dest, &(len as u16).to_be_bytes());
    } else {
        dest.add_byte(BIN32);
        write_raw_bytes(dest, &(len as u32).to_be_bytes());
    }
    write_raw_bytes(dest, b);
}

/// Serializes a universal `Value` into a `Vec<u8>` MessagePack binary payload.
pub fn to_vec(value: &Value) -> Result<Vec<u8>, MsgPackError> {
    let mut dest = babbel_core::io::BufferDestination::new();
    serialize_to_dest(value, &mut dest)?;
    Ok(dest.into_vec())
}

/// MessagePack official Timestamp extension type (`ext -1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgPackTimestamp {
    /// Seconds since the Unix epoch (`1970-01-01T00:00:00Z`).
    pub seconds: i64,
    /// Nanoseconds (`0..1_000_000_000`).
    pub nanoseconds: u32,
}

impl MsgPackTimestamp {
    pub const fn new(seconds: i64, nanoseconds: u32) -> Self {
        Self {
            seconds,
            nanoseconds,
        }
    }
}

/// Encodes a [`MsgPackTimestamp`] into the compact MessagePack Timestamp (`ext -1`) wire format
/// (`timestamp 32`, `timestamp 64`, or `timestamp 96`).
pub fn encode_timestamp(ts: MsgPackTimestamp) -> Vec<u8> {
    let mut out = Vec::with_capacity(15);
    if ts.seconds >= 0 && (ts.seconds >> 34) == 0 && ts.nanoseconds < 1_000_000_000 {
        let data64 = ((ts.nanoseconds as u64) << 34) | (ts.seconds as u64);
        if (data64 & 0xFFFF_FFFF_0000_0000) == 0 {
            // timestamp 32: fixext 4 (0xd6), type -1 (0xff), 32-bit seconds
            out.push(FIXEXT4);
            out.push(0xff);
            out.extend_from_slice(&(data64 as u32).to_be_bytes());
        } else {
            // timestamp 64: fixext 8 (0xd7), type -1 (0xff), 30-bit ns + 34-bit sec
            out.push(FIXEXT8);
            out.push(0xff);
            out.extend_from_slice(&data64.to_be_bytes());
        }
    } else {
        // timestamp 96: ext 8 (0xc7), len 12, type -1 (0xff), 32-bit ns + 64-bit signed sec
        out.push(EXT8);
        out.push(12);
        out.push(0xff);
        out.extend_from_slice(&ts.nanoseconds.to_be_bytes());
        out.extend_from_slice(&ts.seconds.to_be_bytes());
    }
    out
}

/// Decodes a [`MsgPackTimestamp`] from a MessagePack `ext -1` byte slice.
pub fn decode_timestamp(bytes: &[u8]) -> Result<MsgPackTimestamp, MsgPackError> {
    if bytes.len() < 6 {
        return Err(MsgPackError::UnexpectedEof {
            expected: 6,
            available: bytes.len(),
        });
    }
    match bytes[0] {
        FIXEXT4 if bytes[1] == 0xff => {
            let secs = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]) as i64;
            Ok(MsgPackTimestamp::new(secs, 0))
        }
        FIXEXT8 if bytes[1] == 0xff => {
            if bytes.len() < 10 {
                return Err(MsgPackError::UnexpectedEof {
                    expected: 10,
                    available: bytes.len(),
                });
            }
            let data64 = u64::from_be_bytes([
                bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9],
            ]);
            let ns = (data64 >> 34) as u32;
            let secs = (data64 & 0x0000_0003_FFFF_FFFF) as i64;
            Ok(MsgPackTimestamp::new(secs, ns))
        }
        EXT8 if bytes.len() >= 15 && bytes[1] == 12 && bytes[2] == 0xff => {
            let ns = u32::from_be_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
            let secs = i64::from_be_bytes([
                bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
            ]);
            Ok(MsgPackTimestamp::new(secs, ns))
        }
        _ => Err(MsgPackError::InvalidMarker(bytes[0])),
    }
}
