//! High-performance MessagePack serializer for universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use babbel_core::{io::IDestination, Value};
use crate::constants::*;
use crate::error::MsgPackError;

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
