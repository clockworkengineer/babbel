//! Bidirectional Serde integration for [`Value`].
//!
//! Provides [`to_value`] and [`from_value`] along with [`serde::Serialize`] and
//! [`serde::Deserialize`] implementations for [`Value`].

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, EnumAccess, IntoDeserializer, MapAccess,
    SeqAccess, VariantAccess, Visitor,
};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

use crate::error::BabbelError;
use crate::model::Value;

/// Error type for Serde conversions with [`Value`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerdeError(pub String);

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SerdeError {}

impl ser::Error for SerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerdeError(msg.to_string())
    }
}

impl de::Error for SerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        SerdeError(msg.to_string())
    }
}

impl From<SerdeError> for BabbelError {
    fn from(err: SerdeError) -> Self {
        BabbelError::custom(err.0)
    }
}

/// Converts a Rust data structure serializable by Serde into a [`Value`].
pub fn to_value<T: Serialize>(value: &T) -> Result<Value, SerdeError> {
    value.serialize(ValueSerializer)
}

/// Deserializes a [`Value`] into an instance of type `T`.
pub fn from_value<'de, T: Deserialize<'de>>(value: &'de Value) -> Result<T, SerdeError> {
    T::deserialize(ValueDeserializer { value })
}

// ============================================================================
// Serialize implementation for Value
// ============================================================================

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Integer(i) => {
                if *i <= i64::MAX as i128 && *i >= i64::MIN as i128 {
                    serializer.serialize_i64(*i as i64)
                } else {
                    serializer.serialize_i128(*i)
                }
            }
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::String(s) => serializer.serialize_str(s),
            Value::Bytes(b) => serializer.serialize_bytes(b),
            Value::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for item in arr {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Object(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

// ============================================================================
// Deserialize implementation for Value
// ============================================================================

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid Babbel Value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Integer(v as i128))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Value, E> {
        Ok(Value::Integer(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Integer(v as i128))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Value, E> {
        if v <= i128::MAX as u128 {
            Ok(Value::Integer(v as i128))
        } else {
            Ok(Value::Float(v as f64))
        }
    }

    fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
        Ok(Value::Float(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
    }

    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }
        Ok(Value::Array(vec))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut entries = Vec::new();
        while let Some((key, val)) = visitor.next_entry()? {
            entries.push((key, val));
        }
        Ok(Value::Object(entries))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Value, E> {
        Ok(Value::Bytes(v.to_vec()))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Value, E> {
        Ok(Value::Bytes(v))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

// ============================================================================
// ValueSerializer: T -> Value
// ============================================================================

struct ValueSerializer;

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = SerdeError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariantHelper;
    type SerializeMap = SerializeMapHelper;
    type SerializeStruct = SerializeStructHelper;
    type SerializeStructVariant = SerializeStructVariantHelper;

    fn serialize_bool(self, v: bool) -> Result<Value, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_i16(self, v: i16) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_i32(self, v: i32) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_i64(self, v: i64) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_i128(self, v: i128) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_u16(self, v: u16) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_u32(self, v: u32) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_u64(self, v: u64) -> Result<Value, Self::Error> {
        Ok(Value::Integer(v as i128))
    }

    fn serialize_u128(self, v: u128) -> Result<Value, Self::Error> {
        if v <= i128::MAX as u128 {
            Ok(Value::Integer(v as i128))
        } else {
            Ok(Value::Float(v as f64))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Value, Self::Error> {
        Ok(Value::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Value, Self::Error> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Value, Self::Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Value, Self::Error> {
        Ok(Value::String(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value, Self::Error> {
        Ok(Value::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Value, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, Self::Error> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, Self::Error> {
        Ok(Value::String(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, Self::Error> {
        let val = value.serialize(ValueSerializer)?;
        Ok(Value::Object(alloc::vec![(variant.to_owned(), val)]))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariantHelper {
            name: variant.to_owned(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMapHelper {
            entries: Vec::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStructHelper {
            entries: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariantHelper {
            name: variant.to_owned(),
            entries: Vec::with_capacity(len),
        })
    }
}

struct SerializeVec {
    vec: Vec<Value>,
}

impl SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Array(self.vec))
    }
}

impl SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Array(self.vec))
    }
}

impl SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Array(self.vec))
    }
}

struct SerializeTupleVariantHelper {
    name: String,
    vec: Vec<Value>,
}

impl SerializeTupleVariant for SerializeTupleVariantHelper {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.vec.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Object(alloc::vec![(
            self.name,
            Value::Array(self.vec)
        )]))
    }
}

struct SerializeMapHelper {
    entries: Vec<(String, Value)>,
    next_key: Option<String>,
}

impl SerializeMap for SerializeMapHelper {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        let key_val = key.serialize(ValueSerializer)?;
        match key_val {
            Value::String(s) => self.next_key = Some(s),
            _ => return Err(SerdeError("Map key must serialize to a String".into())),
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| SerdeError("serialize_value called before serialize_key".into()))?;
        let val = value.serialize(ValueSerializer)?;
        self.entries.push((key, val));
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Object(self.entries))
    }
}

struct SerializeStructHelper {
    entries: Vec<(String, Value)>,
}

impl SerializeStruct for SerializeStructHelper {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let val = value.serialize(ValueSerializer)?;
        self.entries.push((key.to_owned(), val));
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Object(self.entries))
    }
}

struct SerializeStructVariantHelper {
    name: String,
    entries: Vec<(String, Value)>,
}

impl SerializeStructVariant for SerializeStructVariantHelper {
    type Ok = Value;
    type Error = SerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let val = value.serialize(ValueSerializer)?;
        self.entries.push((key.to_owned(), val));
        Ok(())
    }

    fn end(self) -> Result<Value, Self::Error> {
        Ok(Value::Object(alloc::vec![(
            self.name,
            Value::Object(self.entries)
        )]))
    }
}

// ============================================================================
// ValueDeserializer: &'de Value -> T
// ============================================================================

struct ValueDeserializer<'de> {
    value: &'de Value,
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = SerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Integer(i) => {
                if let Ok(i64_val) = i64::try_from(*i) {
                    visitor.visit_i64(i64_val)
                } else {
                    visitor.visit_i128(*i)
                }
            }
            Value::Float(f) => visitor.visit_f64(*f),
            Value::String(s) => visitor.visit_str(s),
            Value::Bytes(b) => visitor.visit_bytes(b),
            Value::Array(arr) => visitor.visit_seq(SeqAccessHelper {
                iter: arr.iter(),
            }),
            Value::Object(map) => visitor.visit_map(MapAccessHelper {
                iter: map.iter(),
                next_value: None,
            }),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Bool(b) => visitor.visit_bool(*b),
            _ => Err(SerdeError("Expected boolean".into())),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Integer(i) => {
                let v: i64 = (*i).try_into().map_err(|_| SerdeError("Integer does not fit in i64".into()))?;
                visitor.visit_i64(v)
            }
            Value::Float(f) => visitor.visit_i64(*f as i64),
            _ => Err(SerdeError("Expected integer".into())),
        }
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Integer(i) => visitor.visit_i128(*i),
            Value::Float(f) => visitor.visit_i128(*f as i128),
            _ => Err(SerdeError("Expected 128-bit integer".into())),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Integer(i) if *i >= 0 => {
                let v: u64 = (*i).try_into().map_err(|_| SerdeError("Integer does not fit in u64".into()))?;
                visitor.visit_u64(v)
            }
            Value::Float(f) if *f >= 0.0 => visitor.visit_u64(*f as u64),
            _ => Err(SerdeError("Expected unsigned integer".into())),
        }
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Integer(i) if *i >= 0 => visitor.visit_u128(*i as u128),
            Value::Float(f) if *f >= 0.0 => visitor.visit_u128(*f as u128),
            _ => Err(SerdeError("Expected unsigned 128-bit integer".into())),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Float(f) => visitor.visit_f64(*f),
            Value::Integer(i) => visitor.visit_f64(*i as f64),
            _ => Err(SerdeError("Expected float".into())),
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::String(s) if s.chars().count() == 1 => {
                visitor.visit_char(s.chars().next().unwrap())
            }
            _ => Err(SerdeError("Expected single char string".into())),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::String(s) => visitor.visit_str(s),
            _ => Err(SerdeError("Expected string".into())),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Bytes(b) => visitor.visit_bytes(b),
            Value::String(s) => visitor.visit_bytes(s.as_bytes()),
            _ => Err(SerdeError("Expected bytes".into())),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            _ => Err(SerdeError("Expected null / unit".into())),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Array(arr) => visitor.visit_seq(SeqAccessHelper {
                iter: arr.iter(),
            }),
            _ => Err(SerdeError("Expected array".into())),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Object(map) => visitor.visit_map(MapAccessHelper {
                iter: map.iter(),
                next_value: None,
            }),
            _ => Err(SerdeError("Expected object / map".into())),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::String(s) => visitor.visit_enum(s.as_str().into_deserializer()),
            Value::Object(entries) if entries.len() == 1 => {
                let (variant, val) = &entries[0];
                visitor.visit_enum(EnumAccessHelper {
                    variant,
                    value: val,
                })
            }
            _ => Err(SerdeError(
                "Expected string or single-key object for enum".into(),
            )),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }
}

struct SeqAccessHelper<'de> {
    iter: core::slice::Iter<'de, Value>,
}

impl<'de> SeqAccess<'de> for SeqAccessHelper<'de> {
    type Error = SerdeError;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        match self.iter.next() {
            Some(v) => seed.deserialize(ValueDeserializer { value: v }).map(Some),
            None => Ok(None),
        }
    }
}

struct MapAccessHelper<'de> {
    iter: core::slice::Iter<'de, (String, Value)>,
    next_value: Option<&'de Value>,
}

impl<'de> MapAccess<'de> for MapAccessHelper<'de> {
    type Error = SerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.next_value = Some(v);
                seed.deserialize(k.as_str().into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
        let val = self
            .next_value
            .take()
            .ok_or_else(|| SerdeError("next_value called before next_key".into()))?;
        seed.deserialize(ValueDeserializer { value: val })
    }
}

struct EnumAccessHelper<'de> {
    variant: &'de str,
    value: &'de Value,
}

impl<'de> EnumAccess<'de> for EnumAccessHelper<'de> {
    type Error = SerdeError;
    type Variant = VariantAccessHelper<'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let variant = seed.deserialize(self.variant.into_deserializer())?;
        Ok((variant, VariantAccessHelper { value: self.value }))
    }
}

struct VariantAccessHelper<'de> {
    value: &'de Value,
}

impl<'de> VariantAccess<'de> for VariantAccessHelper<'de> {
    type Error = SerdeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Deserialize::deserialize(ValueDeserializer { value: self.value })
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Self::Error> {
        seed.deserialize(ValueDeserializer { value: self.value })
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Array(arr) => visitor.visit_seq(SeqAccessHelper {
                iter: arr.iter(),
            }),
            _ => Err(SerdeError("Expected tuple variant array".into())),
        }
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.value {
            Value::Object(entries) => visitor.visit_map(MapAccessHelper {
                iter: entries.iter(),
                next_value: None,
            }),
            _ => Err(SerdeError("Expected struct variant object".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct User {
        name: String,
        age: u32,
        active: bool,
        tags: Vec<String>,
        optional: Option<String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Role {
        Admin,
        Member(String),
        Custom { level: u8, title: String },
    }

    #[test]
    fn test_serde_struct_roundtrip() {
        let user = User {
            name: "Alice".into(),
            age: 30,
            active: true,
            tags: alloc::vec!["rust".into(), "babbel".into()],
            optional: Some("admin".into()),
        };

        let val = to_value(&user).expect("to_value");
        assert_eq!(val["name"].as_str(), Some("Alice"));
        assert_eq!(val["age"].as_i64(), Some(30));
        assert_eq!(val["active"].as_bool(), Some(true));
        assert_eq!(val["tags"][0].as_str(), Some("rust"));

        let roundtrip: User = from_value(&val).expect("from_value");
        assert_eq!(user, roundtrip);
    }

    #[test]
    fn test_serde_enum_roundtrip() {
        let role1 = Role::Admin;
        let val1 = to_value(&role1).unwrap();
        assert_eq!(val1.as_str(), Some("Admin"));
        let de1: Role = from_value(&val1).unwrap();
        assert_eq!(role1, de1);

        let role2 = Role::Member("Team Lead".into());
        let val2 = to_value(&role2).unwrap();
        let de2: Role = from_value(&val2).unwrap();
        assert_eq!(role2, de2);

        let role3 = Role::Custom {
            level: 5,
            title: "Engineer".into(),
        };
        let val3 = to_value(&role3).unwrap();
        let de3: Role = from_value(&val3).unwrap();
        assert_eq!(role3, de3);
    }
}
