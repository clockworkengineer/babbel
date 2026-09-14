//! Pure-Rust LEB128 unsigned varint and ZigZag signed integer encoding.
//!
//! Provides fast, allocation-free serialization and deserialization for variable-length integers
//! across binary formats (Parquet, Protocol Buffers, Thrift, WebAssembly).

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::ErrorCode;

/// Encodes an unsigned 32-bit integer as LEB128 variable-length bytes into `buf`.
pub fn write_varint_u32(buf: &mut Vec<u8>, mut val: u32) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            buf.push(byte | 0x80);
        } else {
            buf.push(byte);
            break;
        }
    }
}

/// Decodes an unsigned 32-bit integer from LEB128 bytes starting at `pos`.
///
/// Returns `(value, bytes_consumed)`.
pub fn read_varint_u32(data: &[u8], mut pos: usize) -> Result<(u32, usize), ErrorCode> {
    let start = pos;
    let mut result: u32 = 0;
    let mut shift = 0;
    loop {
        if pos >= data.len() {
            return Err(ErrorCode::UnexpectedEof);
        }
        let byte = data[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 35 {
            return Err(ErrorCode::InvalidEncoding);
        }
    }
    Ok((result, pos - start))
}

/// Encodes an unsigned 64-bit integer as LEB128 variable-length bytes into `buf`.
pub fn write_varint_u64(buf: &mut Vec<u8>, mut val: u64) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            buf.push(byte | 0x80);
        } else {
            buf.push(byte);
            break;
        }
    }
}

/// Decodes an unsigned 64-bit integer from LEB128 bytes starting at `pos`.
///
/// Returns `(value, bytes_consumed)`.
pub fn read_varint_u64(data: &[u8], mut pos: usize) -> Result<(u64, usize), ErrorCode> {
    let start = pos;
    let mut result: u64 = 0;
    let mut shift = 0;
    loop {
        if pos >= data.len() {
            return Err(ErrorCode::UnexpectedEof);
        }
        let byte = data[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 70 {
            return Err(ErrorCode::InvalidEncoding);
        }
    }
    Ok((result, pos - start))
}

/// ZigZag encodes a signed 32-bit integer into an unsigned 32-bit integer.
#[inline(always)]
pub fn zigzag_encode_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

/// ZigZag decodes an unsigned 32-bit integer into a signed 32-bit integer.
#[inline(always)]
pub fn zigzag_decode_i32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

/// ZigZag encodes a signed 64-bit integer into an unsigned 64-bit integer.
#[inline(always)]
pub fn zigzag_encode_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// ZigZag decodes an unsigned 64-bit integer into a signed 64-bit integer.
#[inline(always)]
pub fn zigzag_decode_i64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_u32_roundtrip() {
        let cases = [0, 1, 127, 128, 255, 300, 16384, u32::MAX / 2, u32::MAX];
        for &val in &cases {
            let mut buf = Vec::new();
            write_varint_u32(&mut buf, val);
            let (decoded, consumed) = read_varint_u32(&buf, 0).expect("read failed");
            assert_eq!(decoded, val);
            assert_eq!(consumed, buf.len());
        }
    }

    #[test]
    fn test_varint_u64_roundtrip() {
        let cases = [0, 1, 127, 128, 16384, u32::MAX as u64 + 1, u64::MAX];
        for &val in &cases {
            let mut buf = Vec::new();
            write_varint_u64(&mut buf, val);
            let (decoded, consumed) = read_varint_u64(&buf, 0).expect("read failed");
            assert_eq!(decoded, val);
            assert_eq!(consumed, buf.len());
        }
    }

    #[test]
    fn test_zigzag_roundtrip() {
        let cases_32 = [0, -1, 1, -2, 2, i32::MIN, i32::MAX];
        for &val in &cases_32 {
            let encoded = zigzag_encode_i32(val);
            let decoded = zigzag_decode_i32(encoded);
            assert_eq!(decoded, val);
        }

        let cases_64 = [0, -1, 1, -2, 2, i64::MIN, i64::MAX];
        for &val in &cases_64 {
            let encoded = zigzag_encode_i64(val);
            let decoded = zigzag_decode_i64(encoded);
            assert_eq!(decoded, val);
        }
    }
}
