//! # Character Classification Utilities
//!
//! Centralized helpers for validating XML character categories according to W3C XML 1.0 recommendations.

/// Returns `true` if character is valid as the first character of an XML Name.
#[inline]
pub fn is_xml_name_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == ':'
}

/// Returns `true` if character is valid within an XML Name tag.
#[inline]
pub fn is_xml_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ':' || ch == '.'
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
