//! Fast, allocation-conscious Base64 encoding and decoding.
//!
//! Compliant with RFC 4648 (standard and URL-safe variants).

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::error::ErrorCode;

const STANDARD_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encodes raw bytes into a standard RFC 4648 Base64 string with `=` padding.
pub fn encode_base64(bytes: &[u8]) -> String {
    encode_with_table(bytes, STANDARD_TABLE, true)
}

/// Encodes raw bytes into a URL-safe RFC 4648 Base64 string without padding.
pub fn encode_base64_url(bytes: &[u8]) -> String {
    encode_with_table(bytes, URL_SAFE_TABLE, false)
}

fn encode_with_table(bytes: &[u8], table: &[u8; 64], pad: bool) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let out_len = if pad {
        (bytes.len() + 2) / 3 * 4
    } else {
        match bytes.len() % 3 {
            0 => bytes.len() / 3 * 4,
            1 => bytes.len() / 3 * 4 + 2,
            _ => bytes.len() / 3 * 4 + 3,
        }
    };

    let mut out = String::with_capacity(out_len);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;

        out.push(table[((n >> 18) & 0x3F) as usize] as char);
        out.push(table[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            out.push(table[((n >> 6) & 0x3F) as usize] as char);
        } else if pad {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(table[(n & 0x3F) as usize] as char);
        } else if pad {
            out.push('=');
        }
    }

    out
}

/// Decodes an RFC 4648 Base64 string (standard or URL-safe, with or without padding).
///
/// Ignores ASCII whitespace characters (` `, `\t`, `\r`, `\n`).
pub fn decode_base64(s: &str) -> Result<Vec<u8>, ErrorCode> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u8;

    for b in s.bytes() {
        if b.is_ascii_whitespace() || b == b'=' {
            continue;
        }
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return Err(ErrorCode::InvalidEncoding),
        };

        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }

    Ok(out)
}

/// Validates whether a string consists of valid RFC 4648 Base64 characters and optional padding.
pub fn is_valid_base64(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return true;
    }
    let mut saw_pad = false;
    let mut count = 0usize;

    for b in trimmed.bytes() {
        if b.is_ascii_whitespace() {
            continue;
        }
        if b == b'=' {
            saw_pad = true;
            count += 1;
            continue;
        }
        if saw_pad {
            // No characters allowed after padding
            return false;
        }
        let valid = matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'-' | b'_');
        if !valid {
            return false;
        }
        count += 1;
    }

    // Base64 groups must be a multiple of 4 if padded, or have remainder != 1 if unpadded
    count % 4 != 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let test_cases: &[&[u8]] = &[
            b"",
            b"f",
            b"fo",
            b"foo",
            b"foob",
            b"fooba",
            b"foobar",
            b"The quick brown fox jumps over the lazy dog.",
            &[0x00, 0xFF, 0xFE, 0x80, 0x7F, 0x42],
        ];

        for &case in test_cases {
            let encoded = encode_base64(case);
            let decoded = decode_base64(&encoded).expect("decode failed");
            assert_eq!(decoded, case);

            let encoded_url = encode_base64_url(case);
            let decoded_url = decode_base64(&encoded_url).expect("decode url failed");
            assert_eq!(decoded_url, case);

            assert!(is_valid_base64(&encoded));
        }
    }

    #[test]
    fn test_rfc4648_test_vectors() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(b"foob"), "Zm9vYg==");
        assert_eq!(encode_base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");

        assert_eq!(decode_base64("Zm9vYmFy").unwrap(), b"foobar");
        assert_eq!(decode_base64("Zm9vYg==").unwrap(), b"foob");
    }

    #[test]
    fn test_invalid_base64() {
        assert!(!is_valid_base64("Zm9v!@#"));
        assert!(!is_valid_base64("Zg==extra"));
        assert!(decode_base64("Zm9v!@#").is_err());
    }
}

