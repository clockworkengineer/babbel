use crate::io::traits::IDestination;
use crate::nodes::types::{Node, Numeric};
use core::marker::PhantomData;

/// Trait defining format-specific encoding strategies for JSON values and tree structures.
pub trait FormatEncoder<D: IDestination> {
    /// Encodes a boolean value
    fn encode_boolean(&mut self, val: bool, dst: &mut D) -> Result<(), String>;
    /// Encodes a numeric value
    fn encode_number(&mut self, val: &Numeric, dst: &mut D) -> Result<(), String>;
    /// Encodes a string value
    fn encode_string(&mut self, val: &str, dst: &mut D) -> Result<(), String>;
    /// Encodes a null value
    fn encode_null(&mut self, dst: &mut D) -> Result<(), String>;
    /// Called before encoding array elements
    fn begin_array(&mut self, dst: &mut D) -> Result<(), String>;
    /// Called after encoding array elements
    fn end_array(&mut self, dst: &mut D) -> Result<(), String>;
    /// Called before encoding object key-value pairs
    fn begin_object(&mut self, dst: &mut D) -> Result<(), String>;
    /// Called after encoding object key-value pairs
    fn end_object(&mut self, dst: &mut D) -> Result<(), String>;
}

/// Generic serializer that traverses a Node tree using a FormatEncoder.
pub struct Serializer<E, D> {
    encoder: E,
    _marker: PhantomData<D>,
}

impl<E: FormatEncoder<D>, D: IDestination> Serializer<E, D> {
    /// Creates a new Serializer with the given encoder
    pub fn new(encoder: E) -> Self {
        Self {
            encoder,
            _marker: PhantomData,
        }
    }

    /// Serializes a Node AST tree into the given destination
    pub fn serialize(&mut self, node: &Node, dst: &mut D) -> Result<(), String> {
        match node {
            Node::Boolean(b) => self.encoder.encode_boolean(*b, dst),
            Node::Number(n) => self.encoder.encode_number(n, dst),
            Node::Str(s) => self.encoder.encode_string(s, dst),
            Node::None => self.encoder.encode_null(dst),
            Node::Array(arr) => {
                self.encoder.begin_array(dst)?;
                for item in arr {
                    self.serialize(item, dst)?;
                }
                self.encoder.end_array(dst)
            }
            Node::Object(map) => {
                self.encoder.begin_object(dst)?;
                for (k, v) in map {
                    self.encoder.encode_string(k, dst)?;
                    self.serialize(v, dst)?;
                }
                self.encoder.end_object(dst)
            }
        }
    }
}
