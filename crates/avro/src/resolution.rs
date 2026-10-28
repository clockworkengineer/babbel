//! Apache Avro Schema Resolution (Writer Schema -> Reader Schema Evolution).
//!
//! Implements the Avro 1.11.1 Schema Resolution specification:
//! - Record field reordering, removal (skipping unread writer fields), and addition via `default` values
//! - Field alias resolution (`aliases`)
//! - Primitive type promotion (`int -> long | float | double`, `long -> float | double`, `float -> double`, `string <-> bytes`)
//! - Union branch matching between writer and reader schemas

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::codec::AvroDecoder;
use crate::error::AvroError;
use crate::schema::AvroSchema;
use babbel_core::Value;

impl<'a> AvroDecoder<'a> {
    /// Decodes an Avro binary payload written with `writer_schema` and resolves it into `reader_schema`.
    pub fn decode_with_schemas(
        &mut self,
        writer_schema: &AvroSchema,
        reader_schema: &AvroSchema,
    ) -> Result<Value, AvroError> {
        if writer_schema == reader_schema {
            return self.decode_with_schema(writer_schema);
        }

        // If writer is a union, read the active branch index and resolve that branch against reader_schema
        if let AvroSchema::Union(w_variants) = writer_schema {
            let idx = self.read_long()?;
            if idx < 0 || (idx as usize) >= w_variants.len() {
                return Err(AvroError::Custom(
                    "union index out of bounds in Avro writer stream",
                ));
            }
            return self.decode_with_schemas(&w_variants[idx as usize], reader_schema);
        }

        // If reader is a union (and writer is not), find compatible branch in reader union
        if let AvroSchema::Union(r_variants) = reader_schema {
            for r_branch in r_variants {
                if schemas_compatible(writer_schema, r_branch) {
                    return self.decode_with_schemas(writer_schema, r_branch);
                }
            }
            return Err(AvroError::Custom(
                "writer schema does not match any branch in reader union",
            ));
        }

        match (writer_schema, reader_schema) {
            // Primitive promotions
            (AvroSchema::Int, AvroSchema::Long) => {
                self.read_long().map(|i| Value::Integer(i as i128))
            }
            (AvroSchema::Int | AvroSchema::Long, AvroSchema::Float) => {
                self.read_long().map(|i| Value::Float(i as f64))
            }
            (AvroSchema::Int | AvroSchema::Long, AvroSchema::Double) => {
                self.read_long().map(|i| Value::Float(i as f64))
            }
            (AvroSchema::Float, AvroSchema::Double) => {
                self.read_float().map(|f| Value::Float(f as f64))
            }
            (AvroSchema::String, AvroSchema::Bytes) => {
                self.read_string().map(|s| Value::Bytes(s.as_bytes().to_vec()))
            }
            (AvroSchema::Bytes, AvroSchema::String) => {
                let bytes = self.read_bytes()?;
                let s = core::str::from_utf8(bytes).map_err(|_| AvroError::InvalidUtf8)?;
                Ok(Value::String(s.to_string()))
            }

            // Array resolution
            (AvroSchema::Array { items: w_items }, AvroSchema::Array { items: r_items }) => {
                let mut result = Vec::new();
                let mut count = self.read_long()?;
                while count != 0 {
                    let actual_count = if count < 0 {
                        let _block_size = self.read_long()?;
                        -count
                    } else {
                        count
                    };
                    for _ in 0..actual_count {
                        result.push(self.decode_with_schemas(w_items, r_items)?);
                    }
                    count = self.read_long()?;
                }
                Ok(Value::Array(result))
            }

            // Map resolution
            (AvroSchema::Map { values: w_vals }, AvroSchema::Map { values: r_vals }) => {
                let mut entries = Vec::new();
                let mut count = self.read_long()?;
                while count != 0 {
                    let actual_count = if count < 0 {
                        let _block_size = self.read_long()?;
                        -count
                    } else {
                        count
                    };
                    for _ in 0..actual_count {
                        let key = self.read_string()?.to_string();
                        let val = self.decode_with_schemas(w_vals, r_vals)?;
                        entries.push((key, val));
                    }
                    count = self.read_long()?;
                }
                Ok(Value::Object(entries))
            }

            // Record evolution
            (
                AvroSchema::Record {
                    fields: w_fields, ..
                },
                AvroSchema::Record {
                    fields: r_fields, ..
                },
            ) => {
                // Decode all writer fields in writer order first, resolving matched ones directly
                let mut decoded_writer: Vec<(String, Value, &AvroSchema)> =
                    Vec::with_capacity(w_fields.len());
                for wf in w_fields {
                    // Check if any reader field matches this writer field by name or alias
                    let matched_rf = r_fields.iter().find(|rf| {
                        rf.name == wf.name || rf.aliases.iter().any(|alias| alias == &wf.name)
                    });
                    let val = if let Some(rf) = matched_rf {
                        self.decode_with_schemas(&wf.schema, &rf.schema)?
                    } else {
                        // Writer field not in reader schema: decode and discard
                        self.decode_with_schema(&wf.schema)?
                    };
                    decoded_writer.push((wf.name.clone(), val, &wf.schema));
                }

                // Assemble reader record in reader field order
                let mut out_entries = Vec::with_capacity(r_fields.len());
                for rf in r_fields {
                    if let Some((_, val, _)) = decoded_writer.iter().find(|(w_name, _, _)| {
                        w_name == &rf.name || rf.aliases.iter().any(|alias| alias == w_name)
                    }) {
                        out_entries.push((rf.name.clone(), val.clone()));
                    } else if let Some(ref def) = rf.default {
                        out_entries.push((rf.name.clone(), def.clone()));
                    } else {
                        return Err(AvroError::InvalidSchema(alloc::format!(
                            "missing required field '{}' with no default in reader schema",
                            rf.name
                        )));
                    }
                }

                Ok(Value::Object(out_entries))
            }

            _ => Err(AvroError::Custom(
                "incompatible writer and reader Avro schemas",
            )),
        }
    }
}

fn schemas_compatible(writer: &AvroSchema, reader: &AvroSchema) -> bool {
    if writer == reader {
        return true;
    }
    matches!(
        (writer, reader),
        (
            AvroSchema::Int,
            AvroSchema::Long | AvroSchema::Float | AvroSchema::Double
        ) | (AvroSchema::Long, AvroSchema::Float | AvroSchema::Double)
            | (AvroSchema::Float, AvroSchema::Double)
            | (AvroSchema::String, AvroSchema::Bytes)
            | (AvroSchema::Bytes, AvroSchema::String)
            | (AvroSchema::Record { .. }, AvroSchema::Record { .. })
            | (AvroSchema::Array { .. }, AvroSchema::Array { .. })
            | (AvroSchema::Map { .. }, AvroSchema::Map { .. })
    )
}

/// Convenience function to decode Avro binary bytes written with `writer_schema` into `reader_schema`.
pub fn from_bytes_with_schemas(
    bytes: &[u8],
    writer_schema: &AvroSchema,
    reader_schema: &AvroSchema,
) -> Result<Value, AvroError> {
    let mut decoder = AvroDecoder::new(bytes);
    decoder.decode_with_schemas(writer_schema, reader_schema)
}

