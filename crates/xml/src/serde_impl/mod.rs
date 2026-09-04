//! # Serde Serialization & Deserialization
//!
//! Provides bidirectional mapping between Rust types and XML documents using `serde`.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use crate::parse;

use serde::de::{
    self, DeserializeOwned, Deserializer, MapAccess, SeqAccess, Visitor,
};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, Serializer,
};

/// Error type for XML serialization.
#[derive(Debug)]
pub struct CustomSerError(String);

impl core::fmt::Display for CustomSerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ser::Error for CustomSerError {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        CustomSerError(msg.to_string())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CustomSerError {}

/// Deserializes an XML string into a Rust data structure.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use xml_lib_rust::from_xml_str;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// struct Config {
///     port: u16,
/// }
///
/// let xml = "<config><port>8080</port></config>";
/// let cfg: Config = from_xml_str(xml).unwrap();
/// assert_eq!(cfg.port, 8080);
/// ```
pub fn from_str<T: DeserializeOwned>(xml: &str) -> Result<T> {
    let doc = parse(xml)?;
    let root_id = doc.root_element_id().ok_or_else(|| {
        XmlError::SerializationError("XML document does not contain a root element".into())
    })?;
    let deserializer = NodeDeserializer::new(&doc, root_id);
    T::deserialize(deserializer).map_err(|e| XmlError::SerializationError(e.to_string()))
}

/// Serializes a Rust data structure into an XML string with the default root tag `<root>`.
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    to_string_with_root("root", value)
}

/// Serializes a Rust data structure into an XML string with a custom root element name.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use xml_lib_rust::to_xml_string_with_root;
///
/// #[derive(Serialize)]
/// struct Message {
///     text: String,
/// }
///
/// let msg = Message { text: "hello".into() };
/// let xml = to_xml_string_with_root("msg", &msg).unwrap();
/// assert_eq!(xml, "<msg><text>hello</text></msg>");
/// ```
pub fn to_string_with_root<T: Serialize>(root_tag: &str, value: &T) -> Result<String> {
    let mut writer = XmlSerWriter::new();
    writer.output.push('<');
    writer.output.push_str(root_tag);
    writer.output.push('>');

    let serializer = ValueSerializer::new(&mut writer);
    value.serialize(serializer).map_err(|e| XmlError::SerializationError(e.to_string()))?;

    writer.output.push_str("</");
    writer.output.push_str(root_tag);
    writer.output.push('>');
    Ok(writer.output)
}

// ---------------------------------------------------------------------------
// Deserializer Implementation
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum NodeOrText<'a> {
    Element(NodeId),
    Text(&'a str),
}

struct NodeDeserializer<'a> {
    doc: &'a Document,
    target: NodeOrText<'a>,
}

impl<'a> NodeDeserializer<'a> {
    fn new(doc: &'a Document, elem_id: NodeId) -> Self {
        Self {
            doc,
            target: NodeOrText::Element(elem_id),
        }
    }

    fn from_text(doc: &'a Document, text: &'a str) -> Self {
        Self {
            doc,
            target: NodeOrText::Text(text),
        }
    }

    fn text_content(&self) -> String {
        match self.target {
            NodeOrText::Text(t) => t.to_string(),
            NodeOrText::Element(nid) => {
                let mut s = String::new();
                if let Some(node) = self.doc.get_node(nid) {
                    for &cid in &node.children {
                        if let Some(c) = self.doc.get_node(cid) {
                            if let NodeKind::Text(t) = &c.kind {
                                s.push_str(t);
                            }
                        }
                    }
                }
                s
            }
        }
    }
}

impl<'de, 'a> Deserializer<'de> for NodeDeserializer<'a> {
    type Error = de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        match self.target {
            NodeOrText::Text(t) => visitor.visit_str(t),
            NodeOrText::Element(nid) => {
                if let Some(node) = self.doc.get_node(nid) {
                    let has_attrs = match &node.kind {
                        NodeKind::Element { attributes, .. } => !attributes.is_empty(),
                        _ => false,
                    };
                    let has_child_elements = node.children.iter().any(|&c| {
                        self.doc.get_node(c).map_or(false, |n| matches!(n.kind, NodeKind::Element { .. }))
                    });
                    if has_child_elements || has_attrs {
                        self.deserialize_map(visitor)
                    } else {
                        let text = self.text_content();
                        visitor.visit_string(text)
                    }
                } else {
                    visitor.visit_string(String::new())
                }
            }
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        let trimmed = text.trim();
        if trimmed == "true" || trimmed == "1" {
            visitor.visit_bool(true)
        } else if trimmed == "false" || trimmed == "0" {
            visitor.visit_bool(false)
        } else {
            Err(de::Error::invalid_value(de::Unexpected::Str(trimmed), &"boolean ('true' or 'false')"))
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        let val = text.trim().parse::<i64>().map_err(|_| {
            de::Error::invalid_value(de::Unexpected::Str(&text), &"integer")
        })?;
        visitor.visit_i64(val)
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        let val = text.trim().parse::<u64>().map_err(|_| {
            de::Error::invalid_value(de::Unexpected::Str(&text), &"unsigned integer")
        })?;
        visitor.visit_u64(val)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        let val = text.trim().parse::<f64>().map_err(|_| {
            de::Error::invalid_value(de::Unexpected::Str(&text), &"floating point number")
        })?;
        visitor.visit_f64(val)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        let ch = text.chars().next().ok_or_else(|| de::Error::invalid_length(0, &"1 character"))?;
        visitor.visit_char(ch)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        visitor.visit_str(&text)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        visitor.visit_string(text)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        visitor.visit_bytes(text.as_bytes())
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        visitor.visit_byte_buf(text.into_bytes())
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        if let NodeOrText::Element(nid) = self.target {
            if let Some(node) = self.doc.get_node(nid) {
                let children: Vec<NodeId> = node.children.iter().copied().filter(|&c| {
                    self.doc.get_node(c).map_or(false, |n| matches!(n.kind, NodeKind::Element { .. }))
                }).collect();
                return visitor.visit_seq(ElementSeqAccess {
                    doc: self.doc,
                    children,
                    idx: 0,
                });
            }
        }
        visitor.visit_seq(ElementSeqAccess {
            doc: self.doc,
            children: Vec::new(),
            idx: 0,
        })
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        if let NodeOrText::Element(nid) = self.target {
            let access = ElementMapAccess::new(self.doc, nid);
            visitor.visit_map(access)
        } else {
            Err(de::Error::custom("Expected XML element for map/struct deserialization"))
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        let text = self.text_content();
        visitor.visit_enum(de::value::StrDeserializer::new(&text))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> core::result::Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }
}

// ---------------------------------------------------------------------------
// MapAccess for Struct / Map Elements
// ---------------------------------------------------------------------------

struct ElementMapAccess<'a> {
    doc: &'a Document,
    entries: Vec<(String, NodeOrText<'a>)>,
    idx: usize,
}

impl<'a> ElementMapAccess<'a> {
    fn new(doc: &'a Document, elem_id: NodeId) -> Self {
        let mut entries = Vec::new();
        if let Some(node) = doc.get_node(elem_id) {
            // Attributes first
            if let NodeKind::Element { attributes, .. } = &node.kind {
                for attr in attributes {
                    entries.push((attr.name.to_string(), NodeOrText::Text(attr.value.as_ref())));
                }
            }
            // Child elements
            for &c_id in &node.children {
                if let Some(c) = doc.get_node(c_id) {
                    if let NodeKind::Element { name, .. } = &c.kind {
                        entries.push((name.to_string(), NodeOrText::Element(c_id)));
                    }
                }
            }
        }
        Self {
            doc,
            entries,
            idx: 0,
        }
    }
}

impl<'de, 'a> MapAccess<'de> for ElementMapAccess<'a> {
    type Error = de::value::Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> core::result::Result<Option<K::Value>, Self::Error> {
        if self.idx < self.entries.len() {
            let key = &self.entries[self.idx].0;
            seed.deserialize(de::value::StrDeserializer::new(key)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> core::result::Result<V::Value, Self::Error> {
        let target = self.entries[self.idx].1;
        self.idx += 1;
        let deserializer = match target {
            NodeOrText::Element(nid) => NodeDeserializer::new(self.doc, nid),
            NodeOrText::Text(t) => NodeDeserializer::from_text(self.doc, t),
        };
        seed.deserialize(deserializer)
    }
}

// ---------------------------------------------------------------------------
// SeqAccess for Collections
// ---------------------------------------------------------------------------

struct ElementSeqAccess<'a> {
    doc: &'a Document,
    children: Vec<NodeId>,
    idx: usize,
}

impl<'de, 'a> SeqAccess<'de> for ElementSeqAccess<'a> {
    type Error = de::value::Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> core::result::Result<Option<T::Value>, Self::Error> {
        if self.idx < self.children.len() {
            let cid = self.children[self.idx];
            self.idx += 1;
            let deserializer = NodeDeserializer::new(self.doc, cid);
            seed.deserialize(deserializer).map(Some)
        } else {
            Ok(None)
        }
    }
}

// ---------------------------------------------------------------------------
// Serializer Implementation
// ---------------------------------------------------------------------------

struct XmlSerWriter {
    output: String,
}

impl XmlSerWriter {
    fn new() -> Self {
        Self {
            output: String::new(),
        }
    }
}

struct ValueSerializer<'a> {
    writer: &'a mut XmlSerWriter,
}

impl<'a> ValueSerializer<'a> {
    fn new(writer: &'a mut XmlSerWriter) -> Self {
        Self { writer }
    }
}

impl<'a> Serializer for ValueSerializer<'a> {
    type Ok = ();
    type Error = CustomSerError;

    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = ser::Impossible<(), Self::Error>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = ser::Impossible<(), Self::Error>;

    fn serialize_bool(self, v: bool) -> core::result::Result<(), Self::Error> {
        self.writer.output.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> core::result::Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> core::result::Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> core::result::Result<(), Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> core::result::Result<(), Self::Error> {
        self.writer.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> core::result::Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> core::result::Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> core::result::Result<(), Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> core::result::Result<(), Self::Error> {
        self.writer.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> core::result::Result<(), Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> core::result::Result<(), Self::Error> {
        self.writer.output.push_str(&v.to_string());
        Ok(())
    }

    fn serialize_char(self, v: char) -> core::result::Result<(), Self::Error> {
        self.writer.output.push(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> core::result::Result<(), Self::Error> {
        for ch in v.chars() {
            match ch {
                '&' => self.writer.output.push_str("&amp;"),
                '<' => self.writer.output.push_str("&lt;"),
                '>' => self.writer.output.push_str("&gt;"),
                '"' => self.writer.output.push_str("&quot;"),
                '\'' => self.writer.output.push_str("&apos;"),
                _ => self.writer.output.push(ch),
            }
        }
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> core::result::Result<(), Self::Error> {
        let s = core::str::from_utf8(v).map_err(ser::Error::custom)?;
        self.serialize_str(s)
    }

    fn serialize_none(self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> core::result::Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> core::result::Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> core::result::Result<(), Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> core::result::Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> core::result::Result<(), Self::Error> {
        self.writer.output.push('<');
        self.writer.output.push_str(variant);
        self.writer.output.push('>');
        value.serialize(ValueSerializer::new(self.writer))?;
        self.writer.output.push_str("</");
        self.writer.output.push_str(variant);
        self.writer.output.push('>');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> core::result::Result<Self::SerializeSeq, Self::Error> {
        Ok(SeqSerializer { writer: self.writer })
    }

    fn serialize_tuple(self, len: usize) -> core::result::Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> core::result::Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> core::result::Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ser::Error::custom("Tuple variants not supported in XML"))
    }

    fn serialize_map(self, _len: Option<usize>) -> core::result::Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer {
            writer: self.writer,
            current_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> core::result::Result<Self::SerializeStruct, Self::Error> {
        Ok(StructSerializer { writer: self.writer })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> core::result::Result<Self::SerializeStructVariant, Self::Error> {
        Err(ser::Error::custom("Struct variants not supported in XML"))
    }
}

// ---------------------------------------------------------------------------
// Composite Serializers
// ---------------------------------------------------------------------------

struct StructSerializer<'a> {
    writer: &'a mut XmlSerWriter,
}

impl<'a> SerializeStruct for StructSerializer<'a> {
    type Ok = ();
    type Error = CustomSerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> core::result::Result<(), Self::Error> {
        self.writer.output.push('<');
        self.writer.output.push_str(key);
        self.writer.output.push('>');
        value.serialize(ValueSerializer::new(self.writer))?;
        self.writer.output.push_str("</");
        self.writer.output.push_str(key);
        self.writer.output.push('>');
        Ok(())
    }

    fn end(self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}

struct SeqSerializer<'a> {
    writer: &'a mut XmlSerWriter,
}

impl<'a> SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = CustomSerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> core::result::Result<(), Self::Error> {
        self.writer.output.push_str("<item>");
        value.serialize(ValueSerializer::new(self.writer))?;
        self.writer.output.push_str("</item>");
        Ok(())
    }

    fn end(self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SeqSerializer<'a> {
    type Ok = ();
    type Error = CustomSerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> core::result::Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> core::result::Result<(), Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for SeqSerializer<'a> {
    type Ok = ();
    type Error = CustomSerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> core::result::Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> core::result::Result<(), Self::Error> {
        SerializeSeq::end(self)
    }
}

struct MapSerializer<'a> {
    writer: &'a mut XmlSerWriter,
    current_key: Option<String>,
}

impl<'a> SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = CustomSerError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> core::result::Result<(), Self::Error> {
        let mut key_writer = XmlSerWriter::new();
        key.serialize(ValueSerializer::new(&mut key_writer))?;
        self.current_key = Some(key_writer.output);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> core::result::Result<(), Self::Error> {
        let key = self.current_key.take().unwrap_or_else(|| "entry".to_string());
        self.writer.output.push('<');
        self.writer.output.push_str(&key);
        self.writer.output.push('>');
        value.serialize(ValueSerializer::new(self.writer))?;
        self.writer.output.push_str("</");
        self.writer.output.push_str(&key);
        self.writer.output.push('>');
        Ok(())
    }

    fn end(self) -> core::result::Result<(), Self::Error> {
        Ok(())
    }
}
