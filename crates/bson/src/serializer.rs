//! High-performance BSON serializer for universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

use crate::constants::*;
use crate::error::BsonError;
use babbel_core::{Value, io::IDestination};

/// Helper to write raw byte slices into any [`IDestination`].
#[inline]
pub fn write_raw_bytes(dest: &mut dyn IDestination, bytes: &[u8]) {
    for &b in bytes {
        dest.add_byte(b);
    }
}

/// Serializes an element key as a null-terminated CString.
fn write_cstring(key: &str, buf: &mut Vec<u8>) {
    buf.extend_from_slice(key.as_bytes());
    buf.push(0x00);
}

/// Serializes a string value as length-prefixed and null-terminated string.
fn write_string_val(s: &str, buf: &mut Vec<u8>) {
    let len = (s.len() + 1) as i32;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
    buf.push(0x00);
}

/// Serializes a single BSON element into the buffer.
fn serialize_element(key: &str, value: &Value, buf: &mut Vec<u8>) -> Result<(), BsonError> {
    match value {
        Value::Float(f) => {
            buf.push(TYPE_DOUBLE);
            write_cstring(key, buf);
            buf.extend_from_slice(&f.to_le_bytes());
        }
        Value::String(s) => {
            buf.push(TYPE_STRING);
            write_cstring(key, buf);
            write_string_val(s, buf);
        }
        Value::Bool(b) => {
            buf.push(TYPE_BOOLEAN);
            write_cstring(key, buf);
            buf.push(if *b { 0x01 } else { 0x00 });
        }
        Value::Null => {
            buf.push(TYPE_NULL);
            write_cstring(key, buf);
        }
        Value::Integer(i) => {
            if *i >= (i32::MIN as i128) && *i <= (i32::MAX as i128) {
                buf.push(TYPE_INT32);
                write_cstring(key, buf);
                buf.extend_from_slice(&(*i as i32).to_le_bytes());
            } else {
                buf.push(TYPE_INT64);
                write_cstring(key, buf);
                buf.extend_from_slice(&(*i as i64).to_le_bytes());
            }
        }
        Value::Bytes(b) => {
            buf.push(TYPE_BINARY);
            write_cstring(key, buf);
            let len = b.len() as i32;
            buf.extend_from_slice(&len.to_le_bytes());
            buf.push(BINARY_GENERIC);
            buf.extend_from_slice(b);
        }
        Value::Array(items) => {
            buf.push(TYPE_ARRAY);
            write_cstring(key, buf);
            let doc_bytes = encode_array_document(items)?;
            buf.extend_from_slice(&doc_bytes);
        }
        Value::Object(entries) => {
            buf.push(TYPE_DOCUMENT);
            write_cstring(key, buf);
            let doc_bytes = encode_document_elements(entries)?;
            buf.extend_from_slice(&doc_bytes);
        }
    }
    Ok(())
}

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

fn encode_document_elements(entries: &[(String, Value)]) -> Result<Vec<u8>, BsonError> {
    let mut elements_buf = Vec::new();
    for (k, v) in entries {
        serialize_element(k, v, &mut elements_buf)?;
    }
    let total_len = (elements_buf.len() + 4 + 1) as i32;
    let mut doc_buf = Vec::with_capacity(total_len as usize);
    doc_buf.extend_from_slice(&total_len.to_le_bytes());
    doc_buf.extend_from_slice(&elements_buf);
    doc_buf.push(0x00);
    Ok(doc_buf)
}

fn encode_array_document(items: &[Value]) -> Result<Vec<u8>, BsonError> {
    let mut elements_buf = Vec::new();
    for (i, v) in items.iter().enumerate() {
        let key = i.to_string();
        serialize_element(&key, v, &mut elements_buf)?;
    }
    let total_len = (elements_buf.len() + 4 + 1) as i32;
    let mut doc_buf = Vec::with_capacity(total_len as usize);
    doc_buf.extend_from_slice(&total_len.to_le_bytes());
    doc_buf.extend_from_slice(&elements_buf);
    doc_buf.push(0x00);
    Ok(doc_buf)
}

/// Serializes a universal `Value` into BSON bytes, writing to any [`IDestination`].
pub fn serialize_to_dest(value: &Value, dest: &mut dyn IDestination) -> Result<(), BsonError> {
    let bytes = to_vec(value)?;
    write_raw_bytes(dest, &bytes);
    Ok(())
}

/// Serializes a universal `Value` into a `Vec<u8>` BSON binary payload.
pub fn to_vec(value: &Value) -> Result<Vec<u8>, BsonError> {
    match value {
        Value::Object(entries) => encode_document_elements(entries),
        Value::Array(items) => encode_array_document(items),
        other => {
            // Primitive wrapped into root document {"value": other}
            let entries = vec![("value".into(), other.clone())];
            encode_document_elements(&entries)
        }
    }
}
