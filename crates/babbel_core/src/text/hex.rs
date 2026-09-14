//! Fast, allocation-conscious hexadecimal encoding and decoding.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// Encodes raw bytes as a lowercase hexadecimal string.
pub fn encode_hex(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0F) as usize] as char);
    }
    s
}

/// Encodes raw bytes as an uppercase hexadecimal string.
pub fn encode_hex_upper(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0F) as usize] as char);
    }
    s
}

/// Decodes a hexadecimal string (lowercase or uppercase) into raw bytes.
///
/// Strips optional leading/trailing whitespace.
pub fn decode_hex(s: &str) -> Result<Vec<u8>, &'static str> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }
    if s.len() % 2 != 0 {
        return Err("Odd hex string length");
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let chars = s.as_bytes();
    for i in (0..chars.len()).step_by(2) {
        let hi = hex_digit_value(chars[i]).ok_or("Invalid hex character")?;
        let lo = hex_digit_value(chars[i + 1]).ok_or("Invalid hex character")?;
        bytes.push((hi << 4) | lo);
    }
    Ok(bytes)
}

/// Returns the 4-bit numeric value of an ASCII hex character, if valid.
#[inline(always)]
pub fn hex_digit_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_roundtrip() {
        let original = b"Hello, World! \x00\xFF\x80";
        let encoded = encode_hex(original);
        assert_eq!(encoded, "48656c6c6f2c20576f726c64212000ff80");
        let decoded = decode_hex(&encoded).unwrap();
        assert_eq!(&decoded, original);

        let upper = encode_hex_upper(original);
        assert_eq!(upper, "48656C6C6F2C20576F726C64212000FF80");
        let decoded_upper = decode_hex(&upper).unwrap();
        assert_eq!(&decoded_upper, original);
    }

    #[test]
    fn test_decode_hex_errors() {
        assert_eq!(decode_hex("123"), Err("Odd hex string length"));
        assert_eq!(decode_hex("12zz"), Err("Invalid hex character"));
        assert_eq!(decode_hex("").unwrap(), Vec::<u8>::new());
    }
}
