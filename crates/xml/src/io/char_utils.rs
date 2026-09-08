//! # Character Classification Utilities
//!
//! Centralized helpers for validating XML character categories according to W3C XML 1.0 recommendations.

/// Returns `true` if character is valid as the first character of an XML Name (XML 1.0 5th Edition §2.3 [4]).
#[inline]
pub fn is_xml_name_start(ch: char) -> bool {
    let u = ch as u32;
    matches!(
        u,
        0x3A
            | 0x41..=0x5A
            | 0x5F
            | 0x61..=0x7A
            | 0xC0..=0xD6
            | 0xD8..=0xF6
            | 0xF8..=0x2FF
            | 0x370..=0x37D
            | 0x37F..=0x1FFF
            | 0x200C..=0x200D
            | 0x2070..=0x218F
            | 0x2C00..=0x2FEF
            | 0x3001..=0xD7FF
            | 0xF900..=0xFDCF
            | 0xFDF0..=0xFFFD
            | 0x10000..=0xEFFFF
    )
}

/// Returns `true` if character is valid within an XML Name tag (XML 1.0 5th Edition §2.3 [4a]).
#[inline]
pub fn is_xml_name_char(ch: char) -> bool {
    is_xml_name_start(ch)
        || matches!(
            ch as u32,
            0x2D | 0x2E | 0x30..=0x39 | 0xB7 | 0x0300..=0x036F | 0x203F..=0x2040
        )
}

/// Returns `true` if character is standard XML whitespace (`' '`, `'\t'`, `'\r'`, `'\n'`).
#[inline]
pub fn is_xml_whitespace(ch: char) -> bool {
    babbel_core::chars::is_whitespace(ch)
}

/// Returns `true` if character is valid according to W3C XML 1.0 (Fifth Edition) §2.2 Char production.
#[inline]
pub fn is_valid_xml_char(ch: char) -> bool {
    let u = ch as u32;
    u == 0x9
        || u == 0xA
        || u == 0xD
        || (0x20..=0xD7FF).contains(&u)
        || (0xE000..=0xFFFD).contains(&u)
        || (0x10000..=0x10FFFF).contains(&u)
}
