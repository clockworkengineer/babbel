//! Byte Order Mark (BOM) and text encoding detection utilities.

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::borrow::Cow;

/// Detected text encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// UTF-8 encoding
    Utf8,
    /// UTF-16 Little Endian
    Utf16Le,
    /// UTF-16 Big Endian
    Utf16Be,
    /// Unknown or unsupported encoding
    Unknown,
}

/// Detects BOM markers and returns the stripped UTF-8 string or decoded content alongside the detected encoding.
pub fn detect_encoding_and_strip_bom(bytes: &[u8]) -> Result<(Cow<'_, str>, Encoding), &'static str> {
    if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
        let s = core::str::from_utf8(&bytes[3..]).map_err(|_| "Invalid UTF-8 after BOM")?;
        Ok((Cow::Borrowed(s), Encoding::Utf8))
    } else if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
        // UTF-16 LE
        let u16_slice: &[u16] = unsafe {
            core::slice::from_raw_parts(
                bytes[2..].as_ptr() as *const u16,
                (bytes.len() - 2) / 2,
            )
        };
        let s = char::decode_utf16(u16_slice.iter().cloned())
            .collect::<Result<String, _>>()
            .map_err(|_| "Invalid UTF-16 LE sequence")?;
        Ok((Cow::Owned(s), Encoding::Utf16Le))
    } else if bytes.len() >= 2 && bytes[0..2] == [0xFE, 0xFF] {
        // UTF-16 BE
        let mut u16_vec = alloc::vec::Vec::with_capacity((bytes.len() - 2) / 2);
        for chunk in bytes[2..].chunks_exact(2) {
            u16_vec.push(u16::from_be_bytes([chunk[0], chunk[1]]));
        }
        let s = char::decode_utf16(u16_vec.into_iter())
            .collect::<Result<String, _>>()
            .map_err(|_| "Invalid UTF-16 BE sequence")?;
        Ok((Cow::Owned(s), Encoding::Utf16Be))
    } else {
        let s = core::str::from_utf8(bytes).map_err(|_| "Invalid UTF-8 byte stream")?;
        Ok((Cow::Borrowed(s), Encoding::Utf8))
    }
}

/// Normalizes CRLF (`\r\n`) and standalone CR (`\r`) to LF (`\n`).
/// Avoids allocation if the input already contains only standard LF.
pub fn normalize_newlines(text: &str) -> Cow<'_, str> {
    if !text.contains('\r') {
        Cow::Borrowed(text)
    } else {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\r' {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push('\n');
            } else {
                out.push(c);
            }
        }
        Cow::Owned(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_bom() {
        let data = [0xEF, 0xBB, 0xBF, b'h', b'e', b'l', b'l', b'o'];
        let (s, enc) = detect_encoding_and_strip_bom(&data).unwrap();
        assert_eq!(enc, Encoding::Utf8);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_newline_normalization() {
        let raw = "line 1\r\nline 2\rline 3\nline 4";
        let normalized = normalize_newlines(raw);
        assert_eq!(normalized, "line 1\nline 2\nline 3\nline 4");

        let already_clean = "clean\nlines";
        match normalize_newlines(already_clean) {
            Cow::Borrowed(_) => {}
            Cow::Owned(_) => panic!("Should not allocate for already clean text"),
        }
    }
}
