//! High-performance CBOR (RFC 8949) serializer for universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use babbel_core::{io::IDestination, Value};
use crate::constants::*;
use crate::error::CborError;

/// Helper to write raw byte slices into any [`IDestination`].
#[inline]
pub fn write_raw_bytes(dest: &mut dyn IDestination, bytes: &[u8]) {
    for &b in bytes {
        dest.add_byte(b);
    }
}

/// Writes major type and integer/length header according to RFC 8949 canonical encoding rules.
#[inline]
pub fn write_header(major: u8, num: u64, dest: &mut dyn IDestination) {
    if num <= 23 {
        dest.add_byte(major | (num as u8));
    } else if num <= 0xff {
        dest.add_byte(major | AI_1_BYTE);
        dest.add_byte(num as u8);
    } else if num <= 0xffff {
        dest.add_byte(major | AI_2_BYTES);
        write_raw_bytes(dest, &(num as u16).to_be_bytes());
    } else if num <= 0xffff_ffff {
        dest.add_byte(major | AI_4_BYTES);
        write_raw_bytes(dest, &(num as u32).to_be_bytes());
    } else {
        dest.add_byte(major | AI_8_BYTES);
        write_raw_bytes(dest, &num.to_be_bytes());
    }
}

/// Serializes a universal `Value` into CBOR bytes, writing to any [`IDestination`].
pub fn serialize_to_dest(value: &Value, dest: &mut dyn IDestination) -> Result<(), CborError> {
    match value {
        Value::Null => {
            dest.add_byte(BYTE_NULL);
        }
        Value::Bool(b) => {
            dest.add_byte(if *b { BYTE_TRUE } else { BYTE_FALSE });
        }
        Value::Integer(i) => {
            if *i >= 0 {
                write_header(MAJOR_UNSIGNED_INT, *i as u64, dest);
            } else {
                let n = (-1 - *i) as u64;
                write_header(MAJOR_NEGATIVE_INT, n, dest);
            }
        }
        Value::Float(f) => {
            dest.add_byte(BYTE_FLOAT64);
            write_raw_bytes(dest, &f.to_be_bytes());
        }
        Value::String(s) => {
            write_header(MAJOR_TEXT_STRING, s.len() as u64, dest);
            dest.add_bytes(s);
        }
        Value::Bytes(b) => {
            write_header(MAJOR_BYTE_STRING, b.len() as u64, dest);
            write_raw_bytes(dest, b);
        }
        Value::Array(items) => {
            write_header(MAJOR_ARRAY, items.len() as u64, dest);
            for item in items {
                serialize_to_dest(item, dest)?;
            }
        }
        Value::Object(entries) => {
            write_header(MAJOR_MAP, entries.len() as u64, dest);
            for (key, val) in entries {
                write_header(MAJOR_TEXT_STRING, key.len() as u64, dest);
                dest.add_bytes(key);
                serialize_to_dest(val, dest)?;
            }
        }
    }
    Ok(())
}

/// Serializes a universal `Value` into a `Vec<u8>` CBOR binary payload.
pub fn to_vec(value: &Value) -> Result<Vec<u8>, CborError> {
    let mut dest = babbel_core::io::BufferDestination::new();
    serialize_to_dest(value, &mut dest)?;
    Ok(dest.into_vec())
}
