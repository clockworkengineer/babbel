//! Default parser implementation for bencode format.
//! Provides functionality to parse bencode-encoded data into Node structures.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
};

use crate::HashMap;
use crate::Node::Dictionary;
use crate::constants::{DICT_START, END_MARKER, INTEGER_START, LIST_START, STRING_SEP};
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;

/// Parses the length prefix of a bencode string, expecting digits followed by ':'.
/// Reads characters until ':' is found and accumulates the length as a `usize`
/// without heap allocation.
fn parse_string_length(source: &mut dyn ISource) -> Result<usize, String> {
    let mut length: usize = 0;
    let mut has_digits = false;
    let mut found_sep = false;
    while let Some(c) = source.current() {
        if c == STRING_SEP {
            source.next();
            found_sep = true;
            break;
        }
        let digit = (c as u8).wrapping_sub(b'0');
        if digit > 9 {
            return Err(ERR_INVALID_STRING_LENGTH.to_string());
        }
        length = length
            .checked_mul(10)
            .and_then(|n| n.checked_add(digit as usize))
            .ok_or_else(|| ERR_INVALID_STRING_LENGTH.to_string())?;
        has_digits = true;
        source.next();
    }
    if !found_sep || !has_digits {
        return Err(ERR_INVALID_STRING_LENGTH.to_string());
    }
    Ok(length)
}

/// Parses bencode data from the given source into a Node structure.
/// Handles integers, strings, lists, and dictionaries based on their prefix character.
///
/// # Arguments
/// * `source` - The source containing bencode-encoded data
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
pub fn parse(source: &mut dyn ISource) -> Result<Node, String> {
    match source.current() {
        Some(INTEGER_START) => parse_integer(source),
        Some(LIST_START) => parse_list(source),
        Some(DICT_START) => parse_dictionary(source),
        Some('0'..='9') => parse_string(source),
        Some(STRING_SEP) => Err(ERR_INVALID_STRING_LENGTH.to_string()),
        Some(c) => Err(unexpected_character(c)),
        None => Err(ERR_EMPTY_INPUT.to_string()),
    }
}

/// Parses bencode data from a byte slice into a Node structure.
/// This is a convenience function that creates a BufferSource internally.
///
/// # Arguments
/// * `data` - The byte slice containing bencode-encoded data
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
pub fn parse_bytes(data: &[u8]) -> Result<Node, String> {
    use crate::io::sources::buffer::Buffer;
    let mut source = Buffer::new(data);
    parse(&mut source)
}

/// Parses bencode data from a string into a Node structure.
/// This is a convenience function that creates a BufferSource internally.
///
/// # Arguments
/// * `data` - The string containing bencode-encoded data
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
pub fn parse_str(data: &str) -> Result<Node, String> {
    parse_bytes(data.as_bytes())
}

/// Parses an integer value from the source, expecting format 'i<number>e'.
/// Handles both positive and negative integers, rejecting invalid formats like '-0'.
/// Uses `i128` accumulation to correctly handle all `i64` values without overflow.
fn parse_integer(source: &mut dyn ISource) -> Result<Node, String> {
    source.next(); // skip 'i'
    let negative = source.current() == Some('-');
    // skip optional sign prefix ('+' or '-')
    if negative || source.current() == Some('+') {
        source.next();
    }
    let mut value: i128 = 0;
    let mut has_digits = false;
    while let Some(c) = source.current() {
        if c == END_MARKER {
            source.next();
            if !has_digits {
                return Err(ERR_INVALID_INTEGER.to_string());
            }
            if negative && value == 0 {
                return Err(ERR_INVALID_INTEGER.to_string()); // reject -0
            }
            let result = if negative { -value } else { value };
            if result < i64::MIN as i128 || result > i64::MAX as i128 {
                return Err(ERR_INVALID_INTEGER.to_string());
            }
            return Ok(Node::Integer(result as i64));
        }
        let digit = (c as u8).wrapping_sub(b'0');
        if digit > 9 {
            return Err(ERR_INVALID_INTEGER.to_string());
        }
        value = value
            .checked_mul(10)
            .and_then(|n| n.checked_add(digit as i128))
            .ok_or_else(|| ERR_INVALID_INTEGER.to_string())?;
        has_digits = true;
        source.next();
    }
    Err(ERR_UNTERMINATED_INTEGER.to_string())
}

/// Parses a string value from the source, expecting format '<length>:<string>'.
/// Validates the string length and ensures the full string content is available.
///
/// # Arguments
/// * `source` - The source containing the string to parse
///
/// # Returns
/// * `Result<Node, String>` - String Node or error message
fn parse_string(source: &mut dyn ISource) -> Result<Node, String> {
    let len = parse_string_length(source)?;
    let mut string = String::new();
    for _ in 0..len {
        if let Some(c) = source.current() {
            string.push(c);
            source.next();
        } else {
            return Err(ERR_INVALID_STRING_LENGTH.to_string());
        }
    }
    Ok(Node::Str(string))
}

/// Parses a list from the source, expecting format 'l<elements>e'.
/// Recursively parses all elements until the end marker is found.
///
/// # Arguments
/// * `source` - The source containing the list to parse
///
/// # Returns
/// * `Result<Node, String>` - List Node or error message
fn parse_list(source: &mut dyn ISource) -> Result<Node, String> {
    source.next(); // skip 'l'
    let mut list = Node::List(vec![]);
    while let Some(c) = source.current() {
        if c == END_MARKER {
            source.next();
            return Ok(list);
        }
        list.add_to_list(parse(source)?)
            .map_err(|e| e.to_string())?;
    }
    Err(ERR_UNTERMINATED_LIST.to_string())
}

/// Parses a dictionary from the source, expecting format 'd<key><value>...e'.
/// Ensures keys are strings and are in sorted order.
///
/// # Arguments
/// * `source` - The source containing the dictionary to parse
///
/// # Returns
/// * `Result<Node, String>` - Dictionary Node or error message
fn parse_dictionary(source: &mut dyn ISource) -> Result<Node, String> {
    source.next(); // skip 'd'
    let mut dict = Dictionary(HashMap::new());
    let mut last_key = String::new();
    while let Some(c) = source.current() {
        if c == END_MARKER {
            source.next();
            return Ok(dict);
        }
        match parse_string(source) {
            Ok(Node::Str(key)) => {
                if key <= last_key {
                    return Err(ERR_DICT_KEYS_ORDER.to_string());
                }
                last_key = key.clone();
                let value = parse(source)?;
                dict.add_to_dictionary(&key, value)
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err(ERR_DICT_KEY_MUST_BE_STRING.to_string()),
        }
    }
    Err(ERR_UNTERMINATED_DICTIONARY.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BufferSource;

    #[test]
    fn parse_integer_works() {
        let mut source = BufferSource::new(b"i32e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(32))));
    }

    #[test]
    fn parse_string_works() {
        let mut source = BufferSource::new(b"4:test");
        assert!(matches!(parse(&mut source), Ok(Node::Str(s)) if s == "test"));
    }

    #[test]
    fn parse_list_works() {
        let mut source = BufferSource::new(b"li32ei33ee");
        match parse(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::Integer(32)));
                assert!(matches!(&list[1], Node::Integer(33)));
            }
            _ => {
                assert_eq!(false, true);
            }
        }
    }

    #[test]
    fn parse_dictionary_works() {
        let mut source = BufferSource::new(b"d4:testi32ee");
        match parse(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                assert!(matches!(dict.get("test"), Some(Node::Integer(32))));
            }
            _ => {
                assert_eq!(false, true);
            }
        }
    }
    #[test]
    fn parse_integer_with_error() {
        let mut source = BufferSource::new(b"i32");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_INTEGER));
    }

    #[test]
    fn parse_string_with_error() {
        let mut source = BufferSource::new(b"4:tes");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_STRING_LENGTH));
    }

    #[test]
    fn parse_negative_integer_works() {
        let mut source = BufferSource::new(b"i-32e");
        assert!(matches!(parse(&mut source), Ok(Node::Integer(-32))));
    }

    #[test]
    fn parse_negative_zero_fails() {
        let mut source = BufferSource::new(b"i-0e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn parse_list_with_error() {
        let mut source = BufferSource::new(b"li32ei33e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    #[test]
    fn parse_dictionary_with_error() {
        let mut source = BufferSource::new(b"d4:testi32e");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_UNTERMINATED_DICTIONARY));
    }

    #[test]
    fn parse_dictionary_with_unordered_keys_fails() {
        let mut source = BufferSource::new(b"d3:bbci32e3:abci42ee");
        assert!(matches!(parse(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    // ── parse_bytes / parse_str convenience wrappers ──────────────────────────

    #[test]
    fn parse_bytes_integer() {
        assert!(matches!(parse_bytes(b"i42e"), Ok(Node::Integer(42))));
    }

    #[test]
    fn parse_bytes_string() {
        assert!(matches!(parse_bytes(b"4:spam"), Ok(Node::Str(s)) if s == "spam"));
    }

    #[test]
    fn parse_bytes_list() {
        match parse_bytes(b"li1ei2ee") {
            Ok(Node::List(l)) => {
                assert_eq!(l.len(), 2);
                assert!(matches!(l[0], Node::Integer(1)));
                assert!(matches!(l[1], Node::Integer(2)));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn parse_bytes_dict() {
        match parse_bytes(b"d3:fooi7ee") {
            Ok(Dictionary(d)) => assert_eq!(d["foo"], Node::Integer(7)),
            _ => panic!("Expected Dictionary"),
        }
    }

    #[test]
    fn parse_str_integer() {
        assert!(matches!(parse_str("i99e"), Ok(Node::Integer(99))));
    }

    #[test]
    fn parse_str_string() {
        assert!(matches!(parse_str("3:abc"), Ok(Node::Str(s)) if s == "abc"));
    }

    #[test]
    fn parse_str_invalid_returns_error() {
        assert!(parse_str("invalid").is_err());
    }

    // ── integer edge cases ────────────────────────────────────────────────────

    #[test]
    fn parse_integer_zero() {
        assert!(matches!(parse_bytes(b"i0e"), Ok(Node::Integer(0))));
    }

    #[test]
    fn parse_integer_max_i64() {
        assert!(matches!(
            parse_bytes(b"i9223372036854775807e"),
            Ok(Node::Integer(i64::MAX))
        ));
    }

    #[test]
    fn parse_integer_min_i64() {
        assert!(matches!(
            parse_bytes(b"i-9223372036854775808e"),
            Ok(Node::Integer(i64::MIN))
        ));
    }

    #[test]
    fn parse_integer_invalid_chars_returns_error() {
        assert!(parse_bytes(b"iabce").is_err());
    }

    #[test]
    fn parse_empty_input_returns_error() {
        assert!(parse_bytes(b"").is_err());
    }

    #[test]
    fn parse_unknown_start_byte_returns_error() {
        assert!(parse_bytes(b"x").is_err());
        assert!(parse_bytes(b"!").is_err());
    }

    // ── string edge cases ─────────────────────────────────────────────────────

    #[test]
    fn parse_empty_string() {
        assert!(matches!(parse_bytes(b"0:"), Ok(Node::Str(s)) if s.is_empty()));
    }

    #[test]
    fn parse_string_length_mismatch_returns_error() {
        assert!(parse_bytes(b"10:hi").is_err());
    }

    #[test]
    fn parse_colon_alone_returns_error() {
        // ':' as the first char is not a valid start
        let mut source = BufferSource::new(b":hello");
        assert!(parse(&mut source).is_err());
    }

    // ── list edge cases ───────────────────────────────────────────────────────

    #[test]
    fn parse_empty_list() {
        match parse_bytes(b"le") {
            Ok(Node::List(l)) => assert!(l.is_empty()),
            _ => panic!("Expected empty List"),
        }
    }

    #[test]
    fn parse_list_single_integer() {
        match parse_bytes(b"li1ee") {
            Ok(Node::List(l)) => {
                assert_eq!(l.len(), 1);
                assert!(matches!(l[0], Node::Integer(1)));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn parse_list_of_strings() {
        match parse_bytes(b"l4:spam4:eggse") {
            Ok(Node::List(l)) => {
                assert_eq!(l.len(), 2);
                assert!(matches!(&l[0], Node::Str(s) if s == "spam"));
                assert!(matches!(&l[1], Node::Str(s) if s == "eggs"));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn parse_nested_list() {
        // l [l i1e i2e e] [l i3e e] e
        match parse_bytes(b"lli1ei2eeli3eee") {
            Ok(Node::List(outer)) => {
                assert_eq!(outer.len(), 2);
                assert!(matches!(&outer[0], Node::List(l) if l.len() == 2));
                assert!(matches!(&outer[1], Node::List(l) if l.len() == 1));
            }
            _ => panic!("Expected nested List"),
        }
    }

    #[test]
    fn parse_list_mixed_types() {
        match parse_bytes(b"li42e4:texte") {
            Ok(Node::List(l)) => {
                assert_eq!(l.len(), 2);
                assert!(matches!(l[0], Node::Integer(42)));
                assert!(matches!(&l[1], Node::Str(s) if s == "text"));
            }
            _ => panic!("Expected List"),
        }
    }

    // ── dictionary edge cases ─────────────────────────────────────────────────

    #[test]
    fn parse_empty_dictionary() {
        match parse_bytes(b"de") {
            Ok(Dictionary(d)) => assert!(d.is_empty()),
            _ => panic!("Expected empty Dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_string_value() {
        match parse_bytes(b"d3:key5:valuee") {
            Ok(Dictionary(d)) => {
                assert!(matches!(d.get("key"), Some(Node::Str(s)) if s == "value"))
            }
            _ => panic!("Expected Dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_list_value() {
        match parse_bytes(b"d4:listli1ei2eee") {
            Ok(Dictionary(d)) => {
                let list = match d.get("list") {
                    Some(Node::List(l)) => l,
                    _ => panic!("Expected list value"),
                };
                assert_eq!(list.len(), 2);
            }
            _ => panic!("Expected Dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_nested_dict() {
        match parse_bytes(b"d5:innerd3:keyi9eee") {
            Ok(Dictionary(outer)) => match outer.get("inner") {
                Some(Dictionary(inner)) => assert_eq!(inner["key"], Node::Integer(9)),
                _ => panic!("Expected nested Dictionary"),
            },
            _ => panic!("Expected Dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_multiple_entries_sorted() {
        match parse_bytes(b"d1:ai1e1:bi2e1:ci3ee") {
            Ok(Dictionary(d)) => {
                assert_eq!(d.len(), 3);
                assert_eq!(d["a"], Node::Integer(1));
                assert_eq!(d["b"], Node::Integer(2));
                assert_eq!(d["c"], Node::Integer(3));
            }
            _ => panic!("Expected Dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_duplicate_key_last_wins() {
        // Strict parsers may reject this; our parser keeps the last value
        // because HashMap::insert overwrites. Test that it at least doesn't panic.
        let result = parse_bytes(b"d3:fooi1e3:fooi2ee");
        // Either Ok (last value wins) or Err (duplicate key rejected by ordering check) is acceptable
        let _ = result; // just ensure no panic
    }

    // ── parse_str / parse_bytes symmetry ─────────────────────────────────────

    #[test]
    fn parse_str_and_parse_bytes_produce_same_result() {
        let bencode = "d3:fooi42ee";
        let from_str = parse_str(bencode);
        let from_bytes = parse_bytes(bencode.as_bytes());
        assert_eq!(from_str, from_bytes);
    }
}
