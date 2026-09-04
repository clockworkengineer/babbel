//! Zero-copy parser for bencode data.
//! This parser creates BorrowedNode structures that reference the input buffer
//! without allocating or copying data, making it suitable for embedded systems.

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::error::messages::*;
use crate::constants::{BYTE_DICT_START, BYTE_END, BYTE_INTEGER_START, BYTE_LIST_START, BYTE_STRING_SEP};
use crate::nodes::borrowed::BorrowedNode;


/// Parses bencode data from a byte slice without allocation, returning borrowed nodes.
///
/// This is a zero-copy parser suitable for embedded systems with limited memory.
/// The returned node tree contains references to the input buffer rather than
/// owned copies of the data.
///
/// # Arguments
/// * `input` - The byte slice containing bencode-encoded data
///
/// # Returns
/// * `Ok(BorrowedNode)` - The parsed node tree with borrowed data
/// * `Err(String)` - Description of the parsing error
///
/// # Example
/// ```
/// use bencode_lib::parse_borrowed;
///
/// let data = b"i42e";
/// let node = parse_borrowed(data).unwrap();
/// assert_eq!(node.as_integer(), Some(42));
/// ```
pub fn parse_borrowed(input: &[u8]) -> Result<BorrowedNode, String> {
    let mut position = 0;
    parse_node(input, &mut position)
}

/// Internal function to parse a single bencode node from the input
fn parse_node<'a>(input: &'a [u8], position: &mut usize) -> Result<BorrowedNode<'a>, String> {
    if *position >= input.len() {
        return Err(ERR_EMPTY_INPUT.to_string());
    }

    match input[*position] {
        BYTE_INTEGER_START => parse_integer(input, position),
        BYTE_LIST_START => parse_list(input, position),
        BYTE_DICT_START => parse_dictionary(input, position),
        b'0'..=b'9' => parse_bytes(input, position),
        c => Err(unexpected_character(c as char)),
    }
}

/// Parses a bencode integer (i<number>e)
fn parse_integer<'a>(input: &'a [u8], position: &mut usize) -> Result<BorrowedNode<'a>, String> {
    *position += 1; // Skip 'i'

    let start = *position;
    let mut end = start;

    // Find the end marker
    while end < input.len() && input[end] != BYTE_END {
        end += 1;
    }

    if end >= input.len() {
        return Err(ERR_UNTERMINATED_INTEGER.to_string());
    }

    // Parse the integer from the slice
    let int_slice = &input[start..end];
    let int_str = core::str::from_utf8(int_slice).map_err(|_| ERR_INVALID_INTEGER.to_string())?;

    let value = int_str
        .parse::<i64>()
        .map_err(|_| ERR_INVALID_INTEGER.to_string())?;

    *position = end + 1; // Skip 'e'
    Ok(BorrowedNode::Integer(value))
}

/// Parses a bencode byte string (<length>:<bytes>)
fn parse_bytes<'a>(input: &'a [u8], position: &mut usize) -> Result<BorrowedNode<'a>, String> {
    let start = *position;
    let mut end = start;

    // Find the colon delimiter
    while end < input.len() && input[end] != BYTE_STRING_SEP {
        end += 1;
    }

    if end >= input.len() {
        return Err(ERR_INVALID_STRING_LENGTH.to_string());
    }

    // Parse the length
    let length_slice = &input[start..end];
    let length_str =
        core::str::from_utf8(length_slice).map_err(|_| ERR_INVALID_STRING_LENGTH.to_string())?;

    let length = length_str
        .parse::<usize>()
        .map_err(|_| ERR_INVALID_STRING_LENGTH.to_string())?;

    *position = end + 1; // Skip ':'

    // Check if we have enough bytes
    if *position + length > input.len() {
        return Err(ERR_STRING_TOO_SHORT.to_string());
    }

    // Return a borrowed slice without copying
    let bytes = &input[*position..*position + length];
    *position += length;

    Ok(BorrowedNode::Bytes(bytes))
}

/// Parses a bencode list (l<items>e)
fn parse_list<'a>(input: &'a [u8], position: &mut usize) -> Result<BorrowedNode<'a>, String> {
    *position += 1; // Skip 'l'

    let mut list = Vec::new();

    while *position < input.len() && input[*position] != BYTE_END {
        let node = parse_node(input, position)?;
        list.push(node);
    }

    if *position >= input.len() {
        return Err(ERR_UNTERMINATED_LIST.to_string());
    }

    *position += 1; // Skip 'e'
    Ok(BorrowedNode::List(list))
}

/// Parses a bencode dictionary (d<key-value pairs>e)
fn parse_dictionary<'a>(input: &'a [u8], position: &mut usize) -> Result<BorrowedNode<'a>, String> {
    *position += 1; // Skip 'd'

    let mut dict = HashMap::new();
    let mut last_key: Option<&[u8]> = None;

    while *position < input.len() && input[*position] != BYTE_END {
        // Parse key (must be a byte string)
        let key_node = parse_node(input, position)?;
        let key = match key_node {
            BorrowedNode::Bytes(b) => b,
            _ => return Err(ERR_DICT_KEY_MUST_BE_STRING.to_string()),
        };

        // Check key ordering (bencode requires sorted keys)
        if let Some(prev_key) = last_key {
            if key <= prev_key {
                return Err(ERR_DICT_KEYS_ORDER.to_string());
            }
        }
        last_key = Some(key);

        // Parse value
        let value = parse_node(input, position)?;
        dict.insert(key, value);
    }

    if *position >= input.len() {
        return Err(ERR_UNTERMINATED_DICTIONARY.to_string());
    }

    *position += 1; // Skip 'e'
    Ok(BorrowedNode::Dictionary(dict))
}

/// Validates bencode data without building a node tree.
///
/// This is useful for quickly checking if data is valid bencode without
/// allocating memory for the parsed structure. Ideal for embedded systems
/// that need to validate data before committing memory resources.
///
/// # Arguments
/// * `input` - The byte slice to validate
///
/// # Returns
/// * `Ok(())` - The input is valid bencode
/// * `Err(String)` - Description of the validation error
///
/// # Example
/// ```
/// use bencode_lib::validate_bencode;
///
/// assert!(validate_bencode(b"i42e").is_ok());
/// assert!(validate_bencode(b"invalid").is_err());
/// ```
pub fn validate_bencode(input: &[u8]) -> Result<(), String> {
    let mut position = 0;
    validate_node(input, &mut position)?;

    // Ensure we consumed all input
    if position != input.len() {
        return Err("Trailing data after bencode structure".to_string());
    }

    Ok(())
}

/// Internal validation function that doesn't allocate nodes
fn validate_node(input: &[u8], position: &mut usize) -> Result<(), String> {
    if *position >= input.len() {
        return Err(ERR_EMPTY_INPUT.to_string());
    }

    match input[*position] {
        BYTE_INTEGER_START => validate_integer(input, position),
        BYTE_LIST_START => validate_list(input, position),
        BYTE_DICT_START => validate_dictionary(input, position),
        b'0'..=b'9' => validate_bytes(input, position),
        c => Err(unexpected_character(c as char)),
    }
}

fn validate_integer(input: &[u8], position: &mut usize) -> Result<(), String> {
    *position += 1; // Skip 'i'

    let start = *position;
    let mut end = start;

    while end < input.len() && input[end] != BYTE_END {
        end += 1;
    }

    if end >= input.len() {
        return Err(ERR_UNTERMINATED_INTEGER.to_string());
    }

    let int_slice = &input[start..end];
    let int_str = core::str::from_utf8(int_slice).map_err(|_| ERR_INVALID_INTEGER.to_string())?;

    int_str
        .parse::<i64>()
        .map_err(|_| ERR_INVALID_INTEGER.to_string())?;

    *position = end + 1;
    Ok(())
}

fn validate_bytes(input: &[u8], position: &mut usize) -> Result<(), String> {
    let start = *position;
    let mut end = start;

    while end < input.len() && input[end] != BYTE_STRING_SEP {
        end += 1;
    }

    if end >= input.len() {
        return Err(ERR_INVALID_STRING_LENGTH.to_string());
    }

    let length_slice = &input[start..end];
    let length_str =
        core::str::from_utf8(length_slice).map_err(|_| ERR_INVALID_STRING_LENGTH.to_string())?;

    let length = length_str
        .parse::<usize>()
        .map_err(|_| ERR_INVALID_STRING_LENGTH.to_string())?;

    *position = end + 1;

    if *position + length > input.len() {
        return Err(ERR_STRING_TOO_SHORT.to_string());
    }

    *position += length;
    Ok(())
}

fn validate_list(input: &[u8], position: &mut usize) -> Result<(), String> {
    *position += 1; // Skip 'l'

    while *position < input.len() && input[*position] != BYTE_END {
        validate_node(input, position)?;
    }

    if *position >= input.len() {
        return Err(ERR_UNTERMINATED_LIST.to_string());
    }

    *position += 1;
    Ok(())
}

fn validate_dictionary(input: &[u8], position: &mut usize) -> Result<(), String> {
    *position += 1; // Skip 'd'

    let mut last_key_start = 0;
    let mut last_key_len = 0;
    let mut first_key = true;

    while *position < input.len() && input[*position] != BYTE_END {
        // Validate key is a byte string
        if !matches!(input[*position], b'0'..=b'9') {
            return Err(ERR_DICT_KEY_MUST_BE_STRING.to_string());
        }

        // Parse the key to extract the actual bytes
        let len_start = *position;
        let mut len_end = len_start;
        while len_end < input.len() && input[len_end] != BYTE_STRING_SEP {
            len_end += 1;
        }

        if len_end >= input.len() {
            return Err(ERR_INVALID_STRING_LENGTH.to_string());
        }

        let length_str = core::str::from_utf8(&input[len_start..len_end])
            .map_err(|_| ERR_INVALID_STRING_LENGTH.to_string())?;
        let length = length_str
            .parse::<usize>()
            .map_err(|_| ERR_INVALID_STRING_LENGTH.to_string())?;

        let key_bytes_start = len_end + 1; // Skip ':'
        let key_bytes_end = key_bytes_start + length;

        if key_bytes_end > input.len() {
            return Err(ERR_STRING_TOO_SHORT.to_string());
        }

        // Check key ordering
        if !first_key {
            let prev_key = &input[last_key_start..last_key_start + last_key_len];
            let curr_key = &input[key_bytes_start..key_bytes_end];
            if curr_key <= prev_key {
                return Err(ERR_DICT_KEYS_ORDER.to_string());
            }
        }

        last_key_start = key_bytes_start;
        last_key_len = length;
        first_key = false;

        *position = key_bytes_end;

        // Validate value
        validate_node(input, position)?;
    }

    if *position >= input.len() {
        return Err(ERR_UNTERMINATED_DICTIONARY.to_string());
    }

    *position += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_borrowed_integer() {
        let result = parse_borrowed(b"i42e").unwrap();
        assert_eq!(result.as_integer(), Some(42));
    }

    #[test]
    fn parse_borrowed_bytes() {
        let result = parse_borrowed(b"5:hello").unwrap();
        assert_eq!(result.as_bytes(), Some(&b"hello"[..]));
    }

    #[test]
    fn parse_borrowed_list() {
        let result = parse_borrowed(b"li1ei2ei3ee").unwrap();
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_integer(), Some(1));
        assert_eq!(list[1].as_integer(), Some(2));
        assert_eq!(list[2].as_integer(), Some(3));
    }

    #[test]
    fn parse_borrowed_dictionary() {
        let result = parse_borrowed(b"d3:agei42e4:name4:Johne").unwrap();
        let dict = result.as_dictionary().unwrap();
        assert_eq!(dict.len(), 2);
        assert_eq!(dict[&b"age"[..]].as_integer(), Some(42));
        assert_eq!(dict[&b"name"[..]].as_bytes(), Some(&b"John"[..]));
    }

    #[test]
    fn parse_borrowed_complex() {
        // List with mixed types
        let data = b"li42e5:helloli1ei2eee";
        let result = parse_borrowed(data).unwrap();
        let list = result.as_list().unwrap();
        assert_eq!(list[0].as_integer(), Some(42));
        assert_eq!(list[1].as_bytes(), Some(&b"hello"[..]));
        assert!(list[2].is_list());
    }

    #[test]
    fn validate_bencode_valid() {
        assert!(validate_bencode(b"i42e").is_ok());
        assert!(validate_bencode(b"5:hello").is_ok());
        assert!(validate_bencode(b"li1ei2ee").is_ok());
        assert!(validate_bencode(b"d3:key5:valuee").is_ok());
    }

    #[test]
    fn validate_bencode_invalid() {
        assert!(validate_bencode(b"").is_err());
        assert!(validate_bencode(b"i42").is_err()); // Unterminated
        assert!(validate_bencode(b"invalid").is_err());
        assert!(validate_bencode(b"i42e garbage").is_err()); // Trailing data
    }

    #[test]
    fn validate_bencode_no_allocation() {
        // This test ensures validation works without building nodes
        let large_data = b"li1ei2ei3ei4ei5ei6ei7ei8ei9ei10ee";
        assert!(validate_bencode(large_data).is_ok());
    }

    #[test]
    fn borrowed_node_lifetime() {
        let data = b"5:hello";
        let node = parse_borrowed(data).unwrap();
        // Node borrows from data, which is still in scope
        assert_eq!(node.as_bytes(), Some(&b"hello"[..]));
    }

    // ── integers ──────────────────────────────────────────────────────────────

    #[test]
    fn parse_integer_zero() {
        let n = parse_borrowed(b"i0e").unwrap();
        assert_eq!(n.as_integer(), Some(0));
    }

    #[test]
    fn parse_integer_negative() {
        let n = parse_borrowed(b"i-7e").unwrap();
        assert_eq!(n.as_integer(), Some(-7));
    }

    #[test]
    fn parse_integer_large_positive() {
        let n = parse_borrowed(b"i9223372036854775807e").unwrap(); // i64::MAX
        assert_eq!(n.as_integer(), Some(i64::MAX));
    }

    #[test]
    fn parse_integer_large_negative() {
        let n = parse_borrowed(b"i-9223372036854775808e").unwrap(); // i64::MIN
        assert_eq!(n.as_integer(), Some(i64::MIN));
    }

    #[test]
    fn parse_integer_unterminated_is_error() {
        assert!(parse_borrowed(b"i42").is_err());
    }

    #[test]
    fn parse_integer_invalid_chars_is_error() {
        assert!(parse_borrowed(b"iabc e").is_err());
    }

    // ── byte strings ──────────────────────────────────────────────────────────

    #[test]
    fn parse_bytes_zero_length() {
        let n = parse_borrowed(b"0:").unwrap();
        assert_eq!(n.as_bytes(), Some(&b""[..]));
    }

    #[test]
    fn parse_bytes_binary_content() {
        let n = parse_borrowed(b"3:\xFF\x00\xFE").unwrap();
        assert_eq!(n.as_bytes(), Some(&b"\xFF\x00\xFE"[..]));
    }

    #[test]
    fn parse_bytes_missing_colon_is_error() {
        assert!(parse_borrowed(b"5hello").is_err());
    }

    #[test]
    fn parse_bytes_length_exceeds_input_is_error() {
        assert!(parse_borrowed(b"10:hi").is_err());
    }

    #[test]
    fn parse_bytes_exact_length_matches() {
        let n = parse_borrowed(b"4:spam").unwrap();
        assert_eq!(n.as_bytes(), Some(&b"spam"[..]));
        assert_eq!(n.as_bytes().unwrap().len(), 4);
    }

    // ── lists ─────────────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_list() {
        let n = parse_borrowed(b"le").unwrap();
        assert!(n.is_list());
        assert_eq!(n.as_list().unwrap().len(), 0);
    }

    #[test]
    fn parse_list_single_item() {
        let n = parse_borrowed(b"li1ee").unwrap();
        let list = n.as_list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].as_integer(), Some(1));
    }

    #[test]
    fn parse_list_of_strings() {
        let n = parse_borrowed(b"l4:spam4:eggse").unwrap();
        let list = n.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_bytes(), Some(&b"spam"[..]));
        assert_eq!(list[1].as_bytes(), Some(&b"eggs"[..]));
    }

    #[test]
    fn parse_nested_list() {
        // l [l i1e i2e e] [l i3e e] e  →  lli1ei2eeli3eee
        let n = parse_borrowed(b"lli1ei2eeli3eee").unwrap();
        let outer = n.as_list().unwrap();
        assert_eq!(outer.len(), 2);
        assert_eq!(outer[0].as_list().unwrap().len(), 2);
        assert_eq!(outer[1].as_list().unwrap().len(), 1);
    }

    #[test]
    fn parse_list_unterminated_is_error() {
        assert!(parse_borrowed(b"li1e").is_err());
    }

    // ── dictionaries ──────────────────────────────────────────────────────────

    #[test]
    fn parse_empty_dictionary() {
        let n = parse_borrowed(b"de").unwrap();
        assert!(n.is_dictionary());
        assert_eq!(n.as_dictionary().unwrap().len(), 0);
    }

    #[test]
    fn parse_dictionary_single_entry() {
        let n = parse_borrowed(b"d3:fooi1ee").unwrap();
        let d = n.as_dictionary().unwrap();
        assert_eq!(d.len(), 1);
        assert_eq!(d[&b"foo"[..]].as_integer(), Some(1));
    }

    #[test]
    fn parse_dictionary_value_is_list() {
        let n = parse_borrowed(b"d4:listli1ei2eee").unwrap();
        let d = n.as_dictionary().unwrap();
        let list = d[&b"list"[..]].as_list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn parse_dictionary_value_is_dict() {
        let n = parse_borrowed(b"d5:innerd3:keyi9eee").unwrap();
        let outer = n.as_dictionary().unwrap();
        let inner = outer[&b"inner"[..]].as_dictionary().unwrap();
        assert_eq!(inner[&b"key"[..]].as_integer(), Some(9));
    }

    #[test]
    fn parse_dictionary_keys_out_of_order_is_error() {
        // "b" comes before "a" lexicographically → invalid
        assert!(parse_borrowed(b"d1:bi1e1:ai2ee").is_err());
    }

    #[test]
    fn parse_dictionary_integer_key_is_error() {
        assert!(parse_borrowed(b"di1ei2ee").is_err());
    }

    #[test]
    fn parse_dictionary_unterminated_is_error() {
        assert!(parse_borrowed(b"d3:fooi1e").is_err());
    }

    // ── validate_bencode ──────────────────────────────────────────────────────

    #[test]
    fn validate_empty_input_is_error() {
        assert!(validate_bencode(b"").is_err());
    }

    #[test]
    fn validate_integer_zero() {
        assert!(validate_bencode(b"i0e").is_ok());
    }

    #[test]
    fn validate_negative_integer() {
        assert!(validate_bencode(b"i-42e").is_ok());
    }

    #[test]
    fn validate_empty_string() {
        assert!(validate_bencode(b"0:").is_ok());
    }

    #[test]
    fn validate_empty_list() {
        assert!(validate_bencode(b"le").is_ok());
    }

    #[test]
    fn validate_empty_dict() {
        assert!(validate_bencode(b"de").is_ok());
    }

    #[test]
    fn validate_trailing_data_is_error() {
        assert!(validate_bencode(b"i1e garbage").is_err());
    }

    #[test]
    fn validate_unterminated_list_is_error() {
        assert!(validate_bencode(b"li1e").is_err());
    }

    #[test]
    fn validate_unterminated_dict_is_error() {
        assert!(validate_bencode(b"d3:foo").is_err());
    }

    #[test]
    fn validate_dict_keys_out_of_order_is_error() {
        assert!(validate_bencode(b"d1:bi0e1:ai0ee").is_err());
    }

    #[test]
    fn validate_dict_integer_key_is_error() {
        assert!(validate_bencode(b"di0ei0ee").is_err());
    }

    #[test]
    fn validate_nested_structures_ok() {
        assert!(validate_bencode(b"d4:listli1ei2eee").is_ok());
    }

    #[test]
    fn validate_string_too_short_is_error() {
        assert!(validate_bencode(b"10:hi").is_err());
    }

    // ── zero-copy property ────────────────────────────────────────────────────

    #[test]
    fn parse_bytes_borrows_from_input() {
        let input = b"5:world";
        let node = parse_borrowed(input).unwrap();
        let bytes = node.as_bytes().unwrap();
        // The slice must point into the original input
        let input_start = input.as_ptr() as usize;
        let bytes_start = bytes.as_ptr() as usize;
        assert!(bytes_start >= input_start && bytes_start < input_start + input.len());
    }

    #[test]
    fn parse_unknown_start_byte_is_error() {
        assert!(parse_borrowed(b"x").is_err());
        assert!(parse_borrowed(b"!").is_err());
    }
}
