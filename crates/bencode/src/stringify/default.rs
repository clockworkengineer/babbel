//! Module providing functionality to convert bencode nodes into their string representation.
//! Implements the bencode encoding rules for different node types.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::constants::{BYTE_DICT_START, BYTE_END, BYTE_INTEGER_START, BYTE_LIST_START, BYTE_STRING_SEP};
use crate::io::traits::{BencodeWrite, IDestination};
use crate::nodes::node::*;
use crate::stringify::visitor::{BencodeVisitable, BencodeVisitor};

/// Writes a `u64` as decimal ASCII bytes directly to `writer` (no allocation).
#[inline]
fn write_u64_write(writer: &mut (impl BencodeWrite + ?Sized), mut value: u64) {
    let mut buf = [0u8; 20];
    let mut pos = 20usize;
    loop {
        pos -= 1;
        buf[pos] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    writer.write_bytes(&buf[pos..]);
}

/// Writes a `usize` as decimal ASCII bytes directly to `writer` (no allocation).
#[inline]
fn write_usize_write(writer: &mut (impl BencodeWrite + ?Sized), value: usize) {
    write_u64_write(writer, value as u64);
}

/// Writes an `i64` as decimal ASCII bytes directly to `writer` (no allocation).
#[inline]
fn write_i64_write(writer: &mut (impl BencodeWrite + ?Sized), value: i64) {
    if value < 0 {
        writer.write_byte(b'-');
        if value == i64::MIN {
            writer.write_bytes(b"9223372036854775808");
            return;
        }
        write_u64_write(writer, (-value) as u64);
    } else {
        write_u64_write(writer, value as u64);
    }
}

/// Bencode format serializer implementing the `BencodeVisitor` pattern.
pub struct BencodeEncoder<'a, W: BencodeWrite + ?Sized> {
    writer: &'a mut W,
}

impl<'a, W: BencodeWrite + ?Sized> BencodeEncoder<'a, W> {
    /// Creates a new BencodeEncoder writing to the provided destination.
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W: BencodeWrite + ?Sized> BencodeVisitor for BencodeEncoder<'a, W> {
    type Error = String;

    fn visit_integer(&mut self, value: i64) -> Result<(), Self::Error> {
        self.writer.write_byte(BYTE_INTEGER_START);
        write_i64_write(self.writer, value);
        self.writer.write_byte(BYTE_END);
        Ok(())
    }

    fn visit_string(&mut self, value: &str) -> Result<(), Self::Error> {
        write_usize_write(self.writer, value.len());
        self.writer.write_byte(BYTE_STRING_SEP);
        self.writer.write_bytes(value.as_bytes());
        Ok(())
    }

    fn visit_list_start(&mut self) -> Result<(), Self::Error> {
        self.writer.write_byte(BYTE_LIST_START);
        Ok(())
    }

    fn visit_list_end(&mut self) -> Result<(), Self::Error> {
        self.writer.write_byte(BYTE_END);
        Ok(())
    }

    fn visit_dict_start(&mut self) -> Result<(), Self::Error> {
        self.writer.write_byte(BYTE_DICT_START);
        Ok(())
    }

    fn visit_dict_key(&mut self, key: &str) -> Result<(), Self::Error> {
        write_usize_write(self.writer, key.len());
        self.writer.write_byte(BYTE_STRING_SEP);
        self.writer.write_bytes(key.as_bytes());
        Ok(())
    }

    fn visit_dict_end(&mut self) -> Result<(), Self::Error> {
        self.writer.write_byte(BYTE_END);
        Ok(())
    }

    fn visit_none(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Converts a bencode Node into its string representation and writes it to the destination.
///
/// # Arguments
/// * `node` - The bencode node to stringify
/// * `destination` - The destination to write the string representation to
pub fn stringify(node: &Node, destination: &mut dyn IDestination) -> Result<(), String> {
    let mut encoder = BencodeEncoder::new(destination);
    node.accept(&mut encoder)
}

/// Converts a bencode Node into its string representation and returns it as a String.
/// This is a convenience function that creates a BufferDestination internally.
///
/// # Arguments
/// * `node` - The bencode node to stringify
///
/// # Returns
/// * `Result<String, String>` - The bencode string representation or error message
pub fn stringify_to_string(node: &Node) -> Result<String, String> {
    use crate::io::destinations::buffer::Buffer;
    let mut destination = Buffer::new();
    stringify(node, &mut destination)?;
    Ok(destination.to_string())
}

/// Converts a bencode Node into its byte representation and returns it as a Vec<u8>.
/// This is a convenience function that creates a BufferDestination internally.
///
/// # Arguments
/// * `node` - The bencode node to stringify
///
/// # Returns
/// * `Result<Vec<u8>, String>` - The bencode byte representation or error message
pub fn stringify_to_bytes(node: &Node) -> Result<Vec<u8>, String> {
    let s = stringify_to_string(node)?;
    Ok(s.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BufferDestination;
    use std::collections::HashMap;

    #[test]
    fn stringify_integer_works() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(32), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "i32e");
    }

    #[test]
    fn stringify_string_works() {
        let mut destination = BufferDestination::new();
        stringify(&make_node("test"), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "4:test");
    }

    #[test]
    fn stringify_empty_list_works() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(vec![] as Vec<Node>), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "le");
    }

    #[test]
    fn stringify_list_works() {
        let mut destination = BufferDestination::new();
        let list = vec![make_node(32), make_node("test")];
        stringify(&make_node(list), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "li32e4:teste");
    }

    #[test]
    fn stringify_empty_dictionary_works() {
        let mut destination = BufferDestination::new();
        stringify(&make_node(HashMap::new()), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "de");
    }

    #[test]
    fn stringify_dictionary_works() {
        let mut destination = BufferDestination::new();
        let mut dict = HashMap::new();
        dict.insert(String::from("key"), make_node(32));
        stringify(&make_node(dict), &mut destination).unwrap();
        assert_eq!(destination.to_string(), "d3:keyi32ee");
    }

    #[test]
    fn stringify_none_works() {
        let mut destination = BufferDestination::new();
        stringify(&Node::None, &mut destination).unwrap();
        assert_eq!(destination.to_string(), "");
    }

    // --- Integer edge cases ---

    #[test]
    fn stringify_integer_zero() {
        assert_eq!(stringify_to_string(&make_node(0i64)).unwrap(), "i0e");
    }

    #[test]
    fn stringify_integer_negative() {
        assert_eq!(stringify_to_string(&make_node(-42i64)).unwrap(), "i-42e");
    }

    #[test]
    fn stringify_integer_max_i64() {
        let expected = format!("i{}e", i64::MAX);
        assert_eq!(
            stringify_to_string(&Node::Integer(i64::MAX)).unwrap(),
            expected
        );
    }

    #[test]
    fn stringify_integer_min_i64() {
        let expected = format!("i{}e", i64::MIN);
        assert_eq!(
            stringify_to_string(&Node::Integer(i64::MIN)).unwrap(),
            expected
        );
    }

    // --- String edge cases ---

    #[test]
    fn stringify_empty_string() {
        assert_eq!(stringify_to_string(&make_node("")).unwrap(), "0:");
    }

    #[test]
    fn stringify_single_char_string() {
        assert_eq!(stringify_to_string(&make_node("x")).unwrap(), "1:x");
    }

    #[test]
    fn stringify_string_with_spaces() {
        assert_eq!(
            stringify_to_string(&make_node("hello world")).unwrap(),
            "11:hello world"
        );
    }

    #[test]
    fn stringify_string_length_matches_byte_len() {
        // "café" is 4 Unicode chars but 5 UTF-8 bytes
        let s = "café";
        let node = Node::Str(s.to_string());
        let encoded = stringify_to_string(&node).unwrap();
        let expected_len = s.len(); // Rust String::len() is byte length
        assert_eq!(encoded, format!("{}:{}", expected_len, s));
    }

    // --- List edge cases ---

    #[test]
    fn stringify_list_of_integers() {
        let list = vec![make_node(1i64), make_node(2i64), make_node(3i64)];
        assert_eq!(
            stringify_to_string(&make_node(list)).unwrap(),
            "li1ei2ei3ee"
        );
    }

    #[test]
    fn stringify_list_of_strings() {
        let list = vec![make_node("foo"), make_node("bar")];
        assert_eq!(
            stringify_to_string(&make_node(list)).unwrap(),
            "l3:foo3:bare"
        );
    }

    #[test]
    fn stringify_nested_list() {
        let inner = vec![make_node(1i64), make_node(2i64)];
        let outer = vec![make_node(inner), make_node(3i64)];
        assert_eq!(
            stringify_to_string(&make_node(outer)).unwrap(),
            "lli1ei2eei3ee"
        );
    }

    #[test]
    fn stringify_list_with_none_skipped() {
        let list = vec![make_node(1i64), Node::None, make_node(2i64)];
        // None produces no output, so it's skipped between the two integers
        assert_eq!(stringify_to_string(&make_node(list)).unwrap(), "li1ei2ee");
    }

    // --- Dictionary edge cases ---

    #[test]
    fn stringify_dictionary_sorts_keys() {
        let mut dict = HashMap::new();
        dict.insert(String::from("z"), make_node(1i64));
        dict.insert(String::from("a"), make_node(2i64));
        dict.insert(String::from("m"), make_node(3i64));
        let result = stringify_to_string(&make_node(dict)).unwrap();
        assert_eq!(result, "d1:ai2e1:mi3e1:zi1ee");
    }

    #[test]
    fn stringify_dictionary_string_value() {
        let mut dict = HashMap::new();
        dict.insert(String::from("key"), make_node("value"));
        assert_eq!(
            stringify_to_string(&make_node(dict)).unwrap(),
            "d3:key5:valuee"
        );
    }

    #[test]
    fn stringify_dictionary_list_value() {
        let mut dict = HashMap::new();
        dict.insert(
            String::from("nums"),
            make_node(vec![make_node(1i64), make_node(2i64)]),
        );
        assert_eq!(
            stringify_to_string(&make_node(dict)).unwrap(),
            "d4:numsli1ei2eee"
        );
    }

    #[test]
    fn stringify_nested_dictionary() {
        let mut inner = HashMap::new();
        inner.insert(String::from("x"), make_node(9i64));
        let mut outer = HashMap::new();
        outer.insert(String::from("inner"), make_node(inner));
        assert_eq!(
            stringify_to_string(&make_node(outer)).unwrap(),
            "d5:innerd1:xi9eee"
        );
    }

    #[test]
    fn stringify_dictionary_multiple_sorted_keys() {
        let mut dict = HashMap::new();
        dict.insert(String::from("age"), make_node(25i64));
        dict.insert(String::from("name"), make_node("John"));
        let result = stringify_to_string(&make_node(dict)).unwrap();
        assert_eq!(result, "d3:agei25e4:name4:Johne");
    }

    // --- stringify_to_bytes ---

    #[test]
    fn stringify_to_bytes_integer() {
        let bytes = stringify_to_bytes(&make_node(42i64)).unwrap();
        assert_eq!(bytes, b"i42e");
    }

    #[test]
    fn stringify_to_bytes_string() {
        let bytes = stringify_to_bytes(&make_node("hi")).unwrap();
        assert_eq!(bytes, b"2:hi");
    }

    #[test]
    fn stringify_to_bytes_list() {
        let bytes = stringify_to_bytes(&make_node(vec![make_node(1i64)])).unwrap();
        assert_eq!(bytes, b"li1ee");
    }

    // --- Round-trip consistency ---

    #[test]
    fn stringify_to_string_and_bytes_consistent() {
        let node = make_node(vec![make_node("hello"), make_node(7i64)]);
        let s = stringify_to_string(&node).unwrap();
        let b = stringify_to_bytes(&node).unwrap();
        assert_eq!(s.as_bytes(), b.as_slice());
    }

    #[test]
    fn stringify_roundtrip_with_default_parser() {
        use crate::parse_bytes;
        let original = {
            let mut dict = HashMap::new();
            dict.insert(String::from("age"), make_node(30i64));
            dict.insert(String::from("name"), make_node("Alice"));
            make_node(dict)
        };
        let encoded = stringify_to_string(&original).unwrap();
        let parsed = parse_bytes(encoded.as_bytes()).unwrap();
        let re_encoded = stringify_to_string(&parsed).unwrap();
        assert_eq!(encoded, re_encoded);
    }
}
