//! Iterative (stack-based) parser implementation for bencode format.
//!
//! This parser avoids recursion by using an explicit stack, making it suitable
//! for embedded systems with limited stack space or deeply nested structures.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::HashMap;
use crate::Node::Dictionary;
use crate::constants::{DICT_START, END_MARKER, INTEGER_START, LIST_START, STRING_SEP};
use crate::error::messages::*;
use crate::io::traits::ISource;
use crate::nodes::node::Node;

/// Parser state machine states
enum ParseState {
    /// Parsing the initial/next value
    ParseValue,
    /// Inside a list, collecting elements
    InList { elements: Vec<Node> },
    /// Inside a dictionary, expecting a key
    InDictKey {
        entries: HashMap<String, Node>,
        last_key: String,
    },
    /// Inside a dictionary, expecting a value for the given key
    InDictValue {
        entries: HashMap<String, Node>,
        key: String,
        last_key: String,
    },
}

/// Parses the length prefix of a bencode string.
/// Accumulates the length directly as a `usize` without heap allocation.
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

/// Parses an integer value from the source.
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

/// Parses a string value from the source.
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

/// Iterative parser that uses an explicit stack instead of recursion.
/// This is suitable for embedded systems with limited stack space.
///
/// # Arguments
/// * `source` - The source containing bencode-encoded data
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
///
/// # Example
/// ```
/// use bencode_lib::{parse_iterative, BufferSource};
///
/// // Dictionary keys must be in lexicographical order for canonical bencode
/// let data = b"d3:agei25e4:name4:Johnee";
/// let mut source = BufferSource::new(data);
/// let node = parse_iterative(&mut source).unwrap();
/// ```
pub fn parse_iterative(source: &mut dyn ISource) -> Result<Node, String> {
    let mut stack: Vec<ParseState> = vec![ParseState::ParseValue];
    let mut value_stack: Vec<Node> = vec![];

    while let Some(state) = stack.pop() {
        match state {
            ParseState::ParseValue => {
                match source.current() {
                    Some(INTEGER_START) => {
                        value_stack.push(parse_integer(source)?);
                    }
                    Some(LIST_START) => {
                        source.next(); // skip 'l'
                        if source.current() == Some(END_MARKER) {
                            source.next();
                            value_stack.push(Node::List(vec![]));
                        } else {
                            stack.push(ParseState::InList { elements: vec![] });
                            stack.push(ParseState::ParseValue);
                        }
                    }
                    Some(DICT_START) => {
                        source.next(); // skip 'd'
                        if source.current() == Some(END_MARKER) {
                            source.next();
                            value_stack.push(Dictionary(HashMap::new()));
                        } else {
                            stack.push(ParseState::InDictKey {
                                entries: HashMap::new(),
                                last_key: String::new(),
                            });
                            stack.push(ParseState::ParseValue);
                        }
                    }
                    Some('0'..='9') => {
                        value_stack.push(parse_string(source)?);
                    }
                    Some(STRING_SEP) => {
                        return Err(ERR_INVALID_STRING_LENGTH.to_string());
                    }
                    Some(c) => {
                        return Err(unexpected_character(c));
                    }
                    None => {
                        return Err(ERR_EMPTY_INPUT.to_string());
                    }
                }
            }

            ParseState::InList { mut elements } => {
                // Pop the parsed value from value_stack
                if let Some(value) = value_stack.pop() {
                    elements.push(value);
                }

                // Check if list is complete
                match source.current() {
                    Some(END_MARKER) => {
                        source.next();
                        value_stack.push(Node::List(elements));
                    }
                    Some(_) => {
                        // Continue parsing next element
                        stack.push(ParseState::InList { elements });
                        stack.push(ParseState::ParseValue);
                    }
                    None => {
                        return Err(ERR_UNTERMINATED_LIST.to_string());
                    }
                }
            }

            ParseState::InDictKey { entries, last_key } => {
                // Pop the parsed key from value_stack
                if let Some(Node::Str(key)) = value_stack.pop() {
                    if key <= last_key {
                        return Err(ERR_DICT_KEYS_ORDER.to_string());
                    }
                    // Now parse the value for this key
                    stack.push(ParseState::InDictValue {
                        entries,
                        key: key.clone(),
                        last_key: key,
                    });
                    stack.push(ParseState::ParseValue);
                } else {
                    return Err(ERR_DICT_KEY_MUST_BE_STRING.to_string());
                }
            }

            ParseState::InDictValue {
                mut entries,
                key,
                last_key,
            } => {
                // Pop the parsed value from value_stack
                if let Some(value) = value_stack.pop() {
                    entries.insert(key, value);
                }

                // Check if dictionary is complete
                match source.current() {
                    Some(END_MARKER) => {
                        source.next();
                        value_stack.push(Dictionary(entries));
                    }
                    Some(_) => {
                        // Continue parsing next key-value pair
                        stack.push(ParseState::InDictKey { entries, last_key });
                        stack.push(ParseState::ParseValue);
                    }
                    None => {
                        return Err(ERR_UNTERMINATED_DICTIONARY.to_string());
                    }
                }
            }
        }
    }

    // Should have exactly one value left
    if value_stack.len() == 1 {
        Ok(value_stack.pop().unwrap())
    } else {
        Err("Parser error: unexpected state".to_string())
    }
}

/// Parses bencode data from a byte slice using iterative parser.
///
/// # Arguments
/// * `data` - The byte slice containing bencode-encoded data
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
pub fn parse_bytes_iterative(data: &[u8]) -> Result<Node, String> {
    use crate::io::sources::buffer::Buffer;
    let mut source = Buffer::new(data);
    parse_iterative(&mut source)
}

/// Parses bencode data from a string using iterative parser.
///
/// # Arguments
/// * `data` - The string containing bencode-encoded data
///
/// # Returns
/// * `Result<Node, String>` - Parsed Node or error message
pub fn parse_str_iterative(data: &str) -> Result<Node, String> {
    parse_bytes_iterative(data.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BufferSource;

    #[test]
    fn parse_integer_works() {
        let mut source = BufferSource::new(b"i32e");
        assert!(matches!(
            parse_iterative(&mut source),
            Ok(Node::Integer(32))
        ));
    }

    #[test]
    fn parse_string_works() {
        let mut source = BufferSource::new(b"4:test");
        assert!(matches!(parse_iterative(&mut source), Ok(Node::Str(s)) if s == "test"));
    }

    #[test]
    fn parse_list_works() {
        let mut source = BufferSource::new(b"li32ei33ee");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::Integer(32)));
                assert!(matches!(&list[1], Node::Integer(33)));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn parse_dictionary_works() {
        let mut source = BufferSource::new(b"d4:testi32ee");
        match parse_iterative(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                assert!(matches!(dict.get("test"), Some(Node::Integer(32))));
            }
            _ => panic!("Expected dictionary"),
        }
    }

    #[test]
    fn parse_nested_list_works() {
        let mut source = BufferSource::new(b"lli1ei2eeli3ei4eee");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::List(_)));
                assert!(matches!(&list[1], Node::List(_)));
            }
            _ => panic!("Expected nested list"),
        }
    }

    #[test]
    fn parse_deeply_nested_works() {
        // Create a deeply nested list: [[[[[[10]]]]]]
        let mut nested = String::from("i10e");
        for _ in 0..100 {
            nested = format!("l{}e", nested);
        }
        let result = parse_bytes_iterative(nested.as_bytes());
        assert!(result.is_ok());
    }

    #[test]
    fn parse_empty_list_works() {
        let mut source = BufferSource::new(b"le");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => assert_eq!(list.len(), 0),
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn parse_empty_dict_works() {
        let mut source = BufferSource::new(b"de");
        match parse_iterative(&mut source) {
            Ok(Dictionary(dict)) => assert_eq!(dict.len(), 0),
            _ => panic!("Expected empty dictionary"),
        }
    }

    #[test]
    fn parse_integer_with_error() {
        let mut source = BufferSource::new(b"i32");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_UNTERMINATED_INTEGER));
    }

    #[test]
    fn parse_negative_zero_fails() {
        let mut source = BufferSource::new(b"i-0e");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn parse_unordered_dict_keys_fails() {
        let mut source = BufferSource::new(b"d3:bbci32e3:abci42ee");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_DICT_KEYS_ORDER));
    }

    // --- parse_bytes_iterative / parse_str_iterative wrappers ---

    #[test]
    fn parse_bytes_iterative_integer() {
        assert!(matches!(
            parse_bytes_iterative(b"i99e"),
            Ok(Node::Integer(99))
        ));
    }

    #[test]
    fn parse_bytes_iterative_string() {
        assert!(matches!(parse_bytes_iterative(b"5:hello"), Ok(Node::Str(s)) if s == "hello"));
    }

    #[test]
    fn parse_bytes_iterative_list() {
        let result = parse_bytes_iterative(b"li1ei2ee");
        assert!(matches!(result, Ok(Node::List(_))));
    }

    #[test]
    fn parse_bytes_iterative_dict() {
        let result = parse_bytes_iterative(b"d3:keyi7ee");
        assert!(matches!(result, Ok(Dictionary(_))));
    }

    #[test]
    fn parse_str_iterative_integer() {
        assert!(matches!(parse_str_iterative("i42e"), Ok(Node::Integer(42))));
    }

    #[test]
    fn parse_str_iterative_string() {
        assert!(matches!(parse_str_iterative("4:rust"), Ok(Node::Str(s)) if s == "rust"));
    }

    #[test]
    fn parse_str_iterative_invalid_returns_error() {
        assert!(parse_str_iterative("xyz").is_err());
    }

    // --- Integer edge cases ---

    #[test]
    fn parse_integer_zero() {
        let mut source = BufferSource::new(b"i0e");
        assert!(matches!(parse_iterative(&mut source), Ok(Node::Integer(0))));
    }

    #[test]
    fn parse_integer_negative() {
        let mut source = BufferSource::new(b"i-7e");
        assert!(matches!(
            parse_iterative(&mut source),
            Ok(Node::Integer(-7))
        ));
    }

    #[test]
    fn parse_integer_max_i64() {
        let encoded = format!("i{}e", i64::MAX);
        assert!(matches!(
            parse_bytes_iterative(encoded.as_bytes()),
            Ok(Node::Integer(v)) if v == i64::MAX
        ));
    }

    #[test]
    fn parse_integer_min_i64() {
        let encoded = format!("i{}e", i64::MIN);
        assert!(matches!(
            parse_bytes_iterative(encoded.as_bytes()),
            Ok(Node::Integer(v)) if v == i64::MIN
        ));
    }

    #[test]
    fn parse_integer_invalid_chars_returns_error() {
        let mut source = BufferSource::new(b"i12xe");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_INVALID_INTEGER));
    }

    #[test]
    fn parse_empty_input_returns_error() {
        let mut source = BufferSource::new(b"");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_EMPTY_INPUT));
    }

    #[test]
    fn parse_unknown_start_byte_returns_error() {
        let mut source = BufferSource::new(b"z5:hello");
        assert!(parse_iterative(&mut source).is_err());
    }

    // --- String edge cases ---

    #[test]
    fn parse_empty_string() {
        let mut source = BufferSource::new(b"0:");
        assert!(matches!(parse_iterative(&mut source), Ok(Node::Str(s)) if s.is_empty()));
    }

    #[test]
    fn parse_single_char_string() {
        let mut source = BufferSource::new(b"1:x");
        assert!(matches!(parse_iterative(&mut source), Ok(Node::Str(s)) if s == "x"));
    }

    #[test]
    fn parse_string_length_mismatch_returns_error() {
        // claims 5 bytes but only provides 3
        let mut source = BufferSource::new(b"5:abc");
        assert!(parse_iterative(&mut source).is_err());
    }

    #[test]
    fn parse_colon_alone_returns_error() {
        let mut source = BufferSource::new(b":hello");
        assert!(parse_iterative(&mut source).is_err());
    }

    // --- List edge cases ---

    #[test]
    fn parse_list_single_integer() {
        let mut source = BufferSource::new(b"li77ee");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 1);
                assert!(matches!(&list[0], Node::Integer(77)));
            }
            _ => panic!("Expected list with one integer"),
        }
    }

    #[test]
    fn parse_list_of_strings() {
        let mut source = BufferSource::new(b"l3:foo3:bare");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::Str(s) if s == "foo"));
                assert!(matches!(&list[1], Node::Str(s) if s == "bar"));
            }
            _ => panic!("Expected list of strings"),
        }
    }

    #[test]
    fn parse_list_mixed_types() {
        let mut source = BufferSource::new(b"li1e3:twoi3ee");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 3);
                assert!(matches!(&list[0], Node::Integer(1)));
                assert!(matches!(&list[1], Node::Str(s) if s == "two"));
                assert!(matches!(&list[2], Node::Integer(3)));
            }
            _ => panic!("Expected mixed list"),
        }
    }

    #[test]
    fn parse_unterminated_list_returns_error() {
        let mut source = BufferSource::new(b"li1ei2e");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_UNTERMINATED_LIST));
    }

    #[test]
    fn parse_list_containing_empty_list() {
        // [[], []]
        let mut source = BufferSource::new(b"llelee");
        match parse_iterative(&mut source) {
            Ok(Node::List(list)) => {
                assert_eq!(list.len(), 2);
                assert!(matches!(&list[0], Node::List(inner) if inner.is_empty()));
                assert!(matches!(&list[1], Node::List(inner) if inner.is_empty()));
            }
            _ => panic!("Expected list of empty lists"),
        }
    }

    // --- Dictionary edge cases ---

    #[test]
    fn parse_dictionary_string_value() {
        let mut source = BufferSource::new(b"d3:key5:valuee");
        match parse_iterative(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 1);
                assert!(matches!(dict.get("key"), Some(Node::Str(s)) if s == "value"));
            }
            _ => panic!("Expected dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_list_value() {
        let mut source = BufferSource::new(b"d4:datali1ei2eee");
        match parse_iterative(&mut source) {
            Ok(Dictionary(dict)) => {
                assert!(matches!(dict.get("data"), Some(Node::List(_))));
            }
            _ => panic!("Expected dictionary with list value"),
        }
    }

    #[test]
    fn parse_dictionary_nested_dict() {
        let mut source = BufferSource::new(b"d5:innerd3:keyi1eee");
        match parse_iterative(&mut source) {
            Ok(Dictionary(outer)) => {
                assert!(matches!(outer.get("inner"), Some(Dictionary(_))));
            }
            _ => panic!("Expected nested dictionary"),
        }
    }

    #[test]
    fn parse_dictionary_multiple_sorted_entries() {
        let mut source = BufferSource::new(b"d1:ai1e1:bi2e1:ci3ee");
        match parse_iterative(&mut source) {
            Ok(Dictionary(dict)) => {
                assert_eq!(dict.len(), 3);
                assert!(matches!(dict.get("a"), Some(Node::Integer(1))));
                assert!(matches!(dict.get("b"), Some(Node::Integer(2))));
                assert!(matches!(dict.get("c"), Some(Node::Integer(3))));
            }
            _ => panic!("Expected dictionary with three entries"),
        }
    }

    #[test]
    fn parse_dictionary_integer_key_fails() {
        let mut source = BufferSource::new(b"di1e3:vale");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_DICT_KEY_MUST_BE_STRING));
    }

    #[test]
    fn parse_unterminated_dict_returns_error() {
        let mut source = BufferSource::new(b"d3:keyi1e");
        assert!(matches!(parse_iterative(&mut source), Err(s) if s == ERR_UNTERMINATED_DICTIONARY));
    }

    // --- Consistency with default parser ---

    #[test]
    fn parse_bytes_and_str_iterative_produce_same_result() {
        use crate::stringify::default::stringify_to_string;
        let input = "d3:agei25e4:name4:Johne";
        let from_bytes = parse_bytes_iterative(input.as_bytes()).unwrap();
        let from_str = parse_str_iterative(input).unwrap();
        // Re-encode to bencode (which sorts dict keys) for a deterministic comparison
        assert_eq!(
            stringify_to_string(&from_bytes).unwrap(),
            stringify_to_string(&from_str).unwrap()
        );
    }
}
