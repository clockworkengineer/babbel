//! Apache Avro Schema parser, data model, and type representations.
//!
//! Defined by the Apache Avro 1.x specification:
//! - Primitive types: `null`, `boolean`, `int`, `long`, `float`, `double`, `bytes`, `string`
//! - Complex types: `record`, `enum`, `array`, `map`, `union`, `fixed`
//! - Named type references and namespaces

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::boxed::Box;

use crate::error::AvroError;
use babbel_core::Value;

/// An Avro record field definition.
#[derive(Debug, Clone, PartialEq)]
pub struct AvroField {
    /// Field name.
    pub name: String,
    /// Avro schema for this field's value.
    pub schema: AvroSchema,
    /// Optional default value.
    pub default: Option<Value>,
}

/// Avro Schema AST representing either primitive or complex Avro data types.
#[derive(Debug, Clone, PartialEq)]
pub enum AvroSchema {
    /// No value (0 bytes).
    Null,
    /// A binary boolean (1 byte: 0 or 1).
    Boolean,
    /// 32-bit signed integer encoded with variable-length zigzag coding.
    Int,
    /// 64-bit signed integer encoded with variable-length zigzag coding.
    Long,
    /// A 32-bit IEEE 754 floating-point number.
    Float,
    /// A 64-bit IEEE 754 floating-point number.
    Double,
    /// A sequence of 8-bit unsigned bytes.
    Bytes,
    /// A unicode character sequence encoded in UTF-8.
    String,
    /// An Avro record with named, typed fields.
    Record {
        /// Short or qualified record name.
        name: String,
        /// Namespace of the record.
        namespace: Option<String>,
        /// Record fields in serialization order.
        fields: Vec<AvroField>,
    },
    /// An enumeration of symbolic string names.
    Enum {
        /// Enum name.
        name: String,
        /// List of symbolic names.
        symbols: Vec<String>,
    },
    /// A homogenous list of items.
    Array {
        /// Item schema.
        items: Box<AvroSchema>,
    },
    /// A key-value map where keys are UTF-8 strings and values follow a schema.
    Map {
        /// Value schema.
        values: Box<AvroSchema>,
    },
    /// A tagged union of schemas.
    Union(Vec<AvroSchema>),
    /// A fixed-size sequence of bytes.
    Fixed {
        /// Fixed type name.
        name: String,
        /// Size in bytes.
        size: usize,
    },
    /// A reference to a previously defined named schema.
    Named(String),
}

impl AvroSchema {
    /// Parse an Avro schema JSON string into an [`AvroSchema`].
    pub fn parse_str(json: &str) -> Result<Self, AvroError> {
        let node =
            babbel_json::from_str(json).map_err(|e| AvroError::InvalidSchema(format!("{}", e)))?;
        let val = Value::from(node);
        let mut env = Vec::new();
        let schema = Self::from_value_with_env(&val, None, &mut env)?;
        Ok(Self::resolve_named(schema, &env))
    }

    /// Parse an Avro schema from a universal [`Value`] AST.
    pub fn from_value(val: &Value) -> Result<Self, AvroError> {
        let mut env = Vec::new();
        let schema = Self::from_value_with_env(val, None, &mut env)?;
        Ok(Self::resolve_named(schema, &env))
    }

    fn from_value_with_env(
        val: &Value,
        enclosing_ns: Option<&str>,
        env: &mut Vec<(String, AvroSchema)>,
    ) -> Result<Self, AvroError> {
        match val {
            Value::String(s) => match s.as_str() {
                "null" => Ok(AvroSchema::Null),
                "boolean" => Ok(AvroSchema::Boolean),
                "int" => Ok(AvroSchema::Int),
                "long" => Ok(AvroSchema::Long),
                "float" => Ok(AvroSchema::Float),
                "double" => Ok(AvroSchema::Double),
                "bytes" => Ok(AvroSchema::Bytes),
                "string" => Ok(AvroSchema::String),
                named => Ok(AvroSchema::Named(named.to_string())),
            },
            Value::Array(variants) => {
                let mut schemas = Vec::with_capacity(variants.len());
                for v in variants {
                    schemas.push(Self::from_value_with_env(v, enclosing_ns, env)?);
                }
                Ok(AvroSchema::Union(schemas))
            }
            Value::Object(map) => {
                let type_val = map
                    .iter()
                    .find(|(k, _)| k == "type")
                    .map(|(_, v)| v)
                    .ok_or_else(|| {
                        AvroError::InvalidSchema("missing 'type' in schema object".into())
                    })?;

                match type_val {
                    Value::String(type_str) => match type_str.as_str() {
                        "record" => {
                            let name = map
                                .iter()
                                .find(|(k, _)| k == "name")
                                .and_then(|(_, v)| v.as_str())
                                .unwrap_or("Record")
                                .to_string();

                            let ns = map
                                .iter()
                                .find(|(k, _)| k == "namespace")
                                .and_then(|(_, v)| v.as_str())
                                .or(enclosing_ns);

                            let full_name = match ns {
                                Some(n) if !n.is_empty() && !name.contains('.') => {
                                    format!("{}.{}", n, name)
                                }
                                _ => name.clone(),
                            };

                            let fields_val = map
                                .iter()
                                .find(|(k, _)| k == "fields")
                                .and_then(|(_, v)| v.as_array())
                                .ok_or_else(|| {
                                    AvroError::InvalidSchema("record missing 'fields' array".into())
                                })?;

                            let mut fields = Vec::with_capacity(fields_val.len());
                            for f_val in fields_val {
                                let f_name = f_val
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| {
                                        AvroError::InvalidSchema(
                                            "record field missing 'name'".into(),
                                        )
                                    })?
                                    .to_string();
                                let f_type_val = f_val.get("type").ok_or_else(|| {
                                    AvroError::InvalidSchema("record field missing 'type'".into())
                                })?;
                                let f_schema = Self::from_value_with_env(f_type_val, ns, env)?;
                                let f_default = f_val.get("default").cloned();
                                fields.push(AvroField {
                                    name: f_name,
                                    schema: f_schema,
                                    default: f_default,
                                });
                            }

                            let schema = AvroSchema::Record {
                                name: full_name.clone(),
                                namespace: ns.map(|s| s.to_string()),
                                fields,
                            };
                            env.push((full_name, schema.clone()));
                            env.push((name, schema.clone()));
                            Ok(schema)
                        }
                        "enum" => {
                            let name = map
                                .iter()
                                .find(|(k, _)| k == "name")
                                .and_then(|(_, v)| v.as_str())
                                .unwrap_or("Enum")
                                .to_string();
                            let symbols_val = map
                                .iter()
                                .find(|(k, _)| k == "symbols")
                                .and_then(|(_, v)| v.as_array())
                                .ok_or_else(|| {
                                    AvroError::InvalidSchema("enum missing 'symbols' array".into())
                                })?;
                            let mut symbols = Vec::with_capacity(symbols_val.len());
                            for s in symbols_val {
                                if let Some(str_val) = s.as_str() {
                                    symbols.push(str_val.to_string());
                                }
                            }
                            let schema = AvroSchema::Enum {
                                name: name.clone(),
                                symbols,
                            };
                            env.push((name, schema.clone()));
                            Ok(schema)
                        }
                        "array" => {
                            let items_val = map
                                .iter()
                                .find(|(k, _)| k == "items")
                                .map(|(_, v)| v)
                                .ok_or_else(|| {
                                    AvroError::InvalidSchema("array missing 'items'".into())
                                })?;
                            let items =
                                Box::new(Self::from_value_with_env(items_val, enclosing_ns, env)?);
                            Ok(AvroSchema::Array { items })
                        }
                        "map" => {
                            let values_val = map
                                .iter()
                                .find(|(k, _)| k == "values")
                                .map(|(_, v)| v)
                                .ok_or_else(|| {
                                    AvroError::InvalidSchema("map missing 'values'".into())
                                })?;
                            let values =
                                Box::new(Self::from_value_with_env(values_val, enclosing_ns, env)?);
                            Ok(AvroSchema::Map { values })
                        }
                        "fixed" => {
                            let name = map
                                .iter()
                                .find(|(k, _)| k == "name")
                                .and_then(|(_, v)| v.as_str())
                                .unwrap_or("Fixed")
                                .to_string();
                            let size = map
                                .iter()
                                .find(|(k, _)| k == "size")
                                .and_then(|(_, v)| v.as_i64())
                                .unwrap_or(0) as usize;
                            let schema = AvroSchema::Fixed {
                                name: name.clone(),
                                size,
                            };
                            env.push((name, schema.clone()));
                            Ok(schema)
                        }
                        "null" => Ok(AvroSchema::Null),
                        "boolean" => Ok(AvroSchema::Boolean),
                        "int" => Ok(AvroSchema::Int),
                        "long" => Ok(AvroSchema::Long),
                        "float" => Ok(AvroSchema::Float),
                        "double" => Ok(AvroSchema::Double),
                        "bytes" => Ok(AvroSchema::Bytes),
                        "string" => Ok(AvroSchema::String),
                        named => Ok(AvroSchema::Named(named.to_string())),
                    },
                    Value::Array(_) | Value::Object(_) => {
                        Self::from_value_with_env(type_val, enclosing_ns, env)
                    }
                    _ => Err(AvroError::InvalidSchema(
                        "invalid 'type' format in schema".into(),
                    )),
                }
            }
            _ => Err(AvroError::InvalidSchema(
                "schema must be string, array, or object".into(),
            )),
        }
    }

    fn resolve_named(schema: AvroSchema, env: &[(String, AvroSchema)]) -> AvroSchema {
        match schema {
            AvroSchema::Named(ref name) => {
                if let Some((_, resolved)) = env.iter().find(|(n, _)| n == name) {
                    resolved.clone()
                } else {
                    schema
                }
            }
            AvroSchema::Array { items } => AvroSchema::Array {
                items: Box::new(Self::resolve_named(*items, env)),
            },
            AvroSchema::Map { values } => AvroSchema::Map {
                values: Box::new(Self::resolve_named(*values, env)),
            },
            AvroSchema::Union(variants) => AvroSchema::Union(
                variants
                    .into_iter()
                    .map(|s| Self::resolve_named(s, env))
                    .collect(),
            ),
            AvroSchema::Record {
                name,
                namespace,
                fields,
            } => {
                let resolved_fields = fields
                    .into_iter()
                    .map(|f| AvroField {
                        name: f.name,
                        schema: Self::resolve_named(f.schema, env),
                        default: f.default,
                    })
                    .collect();
                AvroSchema::Record {
                    name,
                    namespace,
                    fields: resolved_fields,
                }
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_schema() {
        let json = r#"{
            "type": "record",
            "name": "User",
            "namespace": "example.avro",
            "fields": [
                {"name": "name", "type": "string"},
                {"name": "favorite_number", "type": ["int", "null"]},
                {"name": "favorite_color", "type": ["string", "null"]}
            ]
        }"#;

        let schema = AvroSchema::parse_str(json).unwrap();
        match schema {
            AvroSchema::Record {
                name,
                namespace,
                fields,
            } => {
                assert_eq!(name, "example.avro.User");
                assert_eq!(namespace.as_deref(), Some("example.avro"));
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0].name, "name");
                assert_eq!(fields[0].schema, AvroSchema::String);

                assert_eq!(fields[1].name, "favorite_number");
                assert_eq!(
                    fields[1].schema,
                    AvroSchema::Union(vec![AvroSchema::Int, AvroSchema::Null])
                );

                assert_eq!(fields[2].name, "favorite_color");
                assert_eq!(
                    fields[2].schema,
                    AvroSchema::Union(vec![AvroSchema::String, AvroSchema::Null])
                );
            }
            _ => panic!("expected record schema"),
        }
    }
}
