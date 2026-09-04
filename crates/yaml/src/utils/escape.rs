//! String Escaping Utilities for YAML Library
//!
//! Re-exports shared escaping logic from `babbel_core::escape`.
//! Ensures consistent and correct escaping of special characters across different formats.
//!
//! Copyright (c) 2026 YAML Library Developers

pub use babbel_core::escape::{escape_for_json, escape_for_xml};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_for_json_basic() {
        assert_eq!(escape_for_json("foo"), "foo");
        assert_eq!(escape_for_json("\"\\\n\r\t"), "\\\"\\\\\\n\\r\\t");
        assert_eq!(escape_for_json("abc\u{1F}"), "abc\\u001f");
    }

    #[test]
    fn test_escape_for_json_control_chars() {
        let input = "\x01\x02\x03";
        let expected = "\\u0001\\u0002\\u0003";
        assert_eq!(escape_for_json(input), expected);
    }

    #[test]
    fn test_escape_for_xml_basic() {
        assert_eq!(escape_for_xml("foo"), "foo");
        assert_eq!(escape_for_xml("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn test_escape_for_xml_mixed() {
        let input = "a <b> & 'c' \"d\"";
        let expected = "a &lt;b&gt; &amp; &apos;c&apos; &quot;d&quot;";
        assert_eq!(escape_for_xml(input), expected);
    }
}
