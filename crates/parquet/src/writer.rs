//! Apache Parquet file writer and schema inference engine.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::column::ColumnData;
use crate::error::ParquetError;
use crate::metadata::{
    ColumnChunk, ColumnMetaData, CompressionCodec, DataPageHeader, Encoding, FieldRepetitionType,
    FileMetaData, PageHeader, RowGroup, SchemaElement, Type,
};
use crate::thrift::ThriftWriter;
use babbel_core::Value;

/// Options for writing Apache Parquet files.
#[derive(Debug, Clone)]
pub struct ParquetWriteOptions {
    /// Column page compression codec (`Uncompressed`, `Snappy`, `Lz4Raw`).
    pub compression: CompressionCodec,
}

impl Default for ParquetWriteOptions {
    fn default() -> Self {
        Self {
            compression: CompressionCodec::Uncompressed,
        }
    }
}

impl ParquetWriteOptions {
    /// Use Snappy page compression.
    pub fn snappy() -> Self {
        Self {
            compression: CompressionCodec::Snappy,
        }
    }

    /// Use LZ4_RAW page compression.
    pub fn lz4_raw() -> Self {
        Self {
            compression: CompressionCodec::Lz4Raw,
        }
    }

    /// Set compression codec explicitly.
    pub fn with_compression(mut self, codec: CompressionCodec) -> Self {
        self.compression = codec;
        self
    }
}

/// Write a `Value` (either `Value::Array` of `Value::Object` rows, or a single `Value::Object`)
/// into a valid Apache Parquet binary byte vector.
pub fn write_parquet(value: &Value) -> Result<Vec<u8>, ParquetError> {
    write_parquet_with_options(value, &ParquetWriteOptions::default())
}

/// Write a `Value` into a valid Apache Parquet binary byte vector with custom write options.
pub fn write_parquet_with_options(
    value: &Value,
    options: &ParquetWriteOptions,
) -> Result<Vec<u8>, ParquetError> {
    let rows = match value {
        Value::Array(arr) => arr.as_slice(),
        single @ Value::Object(_) => core::slice::from_ref(single),
        _ => {
            return Err(ParquetError::SchemaMismatch(alloc::format!(
                "Expected Array of Objects or single Object, found {:?}",
                value
            )));
        }
    };

    // 1. Infer schema columns and types across all rows
    let mut col_names: Vec<String> = Vec::new();
    let mut col_types: Vec<Type> = Vec::new();
    let mut col_has_nulls: Vec<bool> = Vec::new();

    for row in rows {
        if let Value::Object(entries) = row {
            for (k, v) in entries {
                if let Some(pos) = col_names.iter().position(|n| n == k) {
                    if v.is_null() {
                        col_has_nulls[pos] = true;
                    } else if col_types[pos] == Type::ByteArray {
                        // Already promoted or set
                    } else {
                        col_types[pos] = infer_parquet_type(v);
                    }
                } else {
                    col_names.push(k.clone());
                    col_types.push(if v.is_null() {
                        Type::ByteArray
                    } else {
                        infer_parquet_type(v)
                    });
                    col_has_nulls.push(v.is_null());
                }
            }
        }
    }

    // Refine types if first rows had nulls
    for (col_idx, col_name) in col_names.iter().enumerate() {
        for row in rows {
            if let Value::Object(entries) = row {
                if let Some((_, val)) = entries.iter().find(|(k, _)| k == col_name) {
                    if !val.is_null() {
                        col_types[col_idx] = infer_parquet_type(val);
                        break;
                    }
                } else {
                    col_has_nulls[col_idx] = true;
                }
            }
        }
    }

    // 2. Build ColumnData vectors
    let mut columns: Vec<ColumnData> = col_names
        .iter()
        .zip(col_types.iter())
        .map(|(name, &t)| ColumnData::new(name.clone(), t))
        .collect();

    for row in rows {
        if let Value::Object(entries) = row {
            for (col_idx, col_name) in col_names.iter().enumerate() {
                let val = entries
                    .iter()
                    .find(|(k, _)| k == col_name)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null);
                if val.is_null() {
                    col_has_nulls[col_idx] = true;
                }
                columns[col_idx].values.push(val);
            }
        }
    }

    // 3. Assemble file buffer starting with "PAR1" magic
    let mut out = Vec::new();
    out.extend_from_slice(b"PAR1");

    let mut schema_elements = Vec::with_capacity(columns.len() + 1);
    schema_elements.push(SchemaElement {
        type_: None,
        repetition_type: None,
        name: "schema".to_string(),
        num_children: Some(columns.len() as i32),
        converted_type: None,
    });

    let mut column_chunks = Vec::with_capacity(columns.len());
    let mut total_rg_bytes = 0i64;

    for (col_idx, col) in columns.iter().enumerate() {
        let is_optional = col_has_nulls[col_idx];
        let rep_type = if is_optional {
            FieldRepetitionType::Optional
        } else {
            FieldRepetitionType::Required
        };
        let converted_type = if col.type_ == Type::ByteArray {
            Some(0) // UTF8
        } else {
            None
        };

        schema_elements.push(SchemaElement {
            type_: Some(col.type_),
            repetition_type: Some(rep_type),
            name: col.name.clone(),
            num_children: None,
            converted_type,
        });

        let data_page_offset = out.len() as i64;
        let (uncompressed_payload, _) = col.encode_plain()?;
        let uncompressed_size = uncompressed_payload.len() as i32;

        let compressed_payload = match options.compression {
            CompressionCodec::Uncompressed => uncompressed_payload,
            CompressionCodec::Snappy => babbel_core::snappy_compress(&uncompressed_payload),
            CompressionCodec::Lz4Raw | CompressionCodec::Lz4 => {
                babbel_core::lz4_compress(&uncompressed_payload)
            }
            other => {
                return Err(ParquetError::UnsupportedCompression(alloc::format!(
                    "{:?}",
                    other
                )));
            }
        };
        let compressed_size = compressed_payload.len() as i32;

        // Write PageHeader
        let page_header = PageHeader {
            type_: 0, // DATA_PAGE
            uncompressed_page_size: uncompressed_size,
            compressed_page_size: compressed_size,
            data_page_header: Some(DataPageHeader {
                num_values: col.len() as i32,
                encoding: Encoding::Plain,
                definition_level_encoding: Encoding::Rle,
                repetition_level_encoding: Encoding::Rle,
            }),
            dictionary_page_header: None,
            data_page_header_v2: None,
        };

        let mut page_thrift = ThriftWriter::new();
        page_header.encode(&mut page_thrift);
        let page_header_bytes = page_thrift.into_vec();

        out.extend_from_slice(&page_header_bytes);
        out.extend_from_slice(&compressed_payload);

        let chunk_total_uncompressed = (page_header_bytes.len() as i64) + (uncompressed_size as i64);
        let chunk_total_compressed = (page_header_bytes.len() as i64) + (compressed_size as i64);
        total_rg_bytes += chunk_total_uncompressed;

        column_chunks.push(ColumnChunk {
            file_offset: data_page_offset,
            meta_data: Some(ColumnMetaData {
                type_: col.type_,
                encodings: vec![Encoding::Plain, Encoding::Rle],
                path_in_schema: vec![col.name.clone()],
                codec: options.compression,
                num_values: col.len() as i64,
                total_uncompressed_size: chunk_total_uncompressed,
                total_compressed_size: chunk_total_compressed,
                data_page_offset,
                dictionary_page_offset: None,
            }),
        });
    }

    let row_groups = if rows.is_empty() || columns.is_empty() {
        Vec::new()
    } else {
        vec![RowGroup {
            columns: column_chunks,
            total_byte_size: total_rg_bytes,
            num_rows: rows.len() as i64,
        }]
    };

    let file_meta = FileMetaData {
        version: 1,
        schema: schema_elements,
        num_rows: rows.len() as i64,
        row_groups,
        created_by: Some("babbel_parquet 0.3.0".to_string()),
    };

    let mut footer_thrift = ThriftWriter::new();
    file_meta.encode(&mut footer_thrift);
    let footer_bytes = footer_thrift.into_vec();
    let footer_len = footer_bytes.len() as u32;

    out.extend_from_slice(&footer_bytes);
    out.extend_from_slice(&footer_len.to_le_bytes());
    out.extend_from_slice(b"PAR1");

    Ok(out)
}

fn infer_parquet_type(val: &Value) -> Type {
    match val {
        Value::Bool(_) => Type::Boolean,
        Value::Integer(_) => Type::Int64,
        Value::Float(_) => Type::Double,
        Value::String(_) | Value::Bytes(_) => Type::ByteArray,
        _ => Type::ByteArray,
    }
}
