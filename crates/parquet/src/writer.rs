//! Parquet binary file writer.

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::column::ColumnData;
use crate::error::ParquetError;
use crate::metadata::*;
use crate::thrift::ThriftWriter;
use babbel_core::Value;

pub const PARQUET_MAGIC: &[u8; 4] = b"PAR1";

/// Write a tabular Babbel `Value` into standard Apache Parquet binary bytes.
pub fn write_parquet(value: &Value) -> Result<Vec<u8>, ParquetError> {
    let rows = extract_rows(value)?;
    let columns = build_columns(&rows)?;
    let num_rows = rows.len() as i64;

    let mut buf = Vec::new();
    // 1. Initial 4-byte magic
    buf.extend_from_slice(PARQUET_MAGIC);

    let mut column_chunks = Vec::new();
    let mut schema_elements = Vec::new();

    // Root schema element
    schema_elements.push(SchemaElement {
        type_: None,
        repetition_type: None,
        name: "schema".to_string(),
        num_children: Some(columns.len() as i32),
        converted_type: None,
    });

    let mut total_byte_size: i64 = 0;

    // 2. Write column chunks
    for col in &columns {
        let (page_data, has_nulls) = col.encode_plain()?;
        let num_values = col.len() as i32;

        let data_page_header = DataPageHeader {
            num_values,
            encoding: Encoding::Plain,
            definition_level_encoding: Encoding::Rle,
            repetition_level_encoding: Encoding::Rle,
        };

        let page_header = PageHeader {
            type_: 0, // DATA_PAGE
            uncompressed_page_size: page_data.len() as i32,
            compressed_page_size: page_data.len() as i32,
            data_page_header: Some(data_page_header),
        };

        let mut header_writer = ThriftWriter::new();
        page_header.encode(&mut header_writer);
        let header_bytes = header_writer.into_bytes();

        let data_page_offset = buf.len() as i64;
        let chunk_size = (header_bytes.len() + page_data.len()) as i64;
        total_byte_size += chunk_size;

        buf.extend_from_slice(&header_bytes);
        buf.extend_from_slice(&page_data);

        let repetition_type = if has_nulls {
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
            repetition_type: Some(repetition_type),
            name: col.name.clone(),
            num_children: None,
            converted_type,
        });

        let meta = ColumnMetaData {
            type_: col.type_,
            encodings: alloc::vec![Encoding::Plain],
            path_in_schema: alloc::vec![col.name.clone()],
            codec: CompressionCodec::Uncompressed,
            num_values: num_values as i64,
            total_uncompressed_size: chunk_size,
            total_compressed_size: chunk_size,
            data_page_offset,
        };

        column_chunks.push(ColumnChunk {
            file_offset: data_page_offset,
            meta_data: Some(meta),
        });
    }

    // 3. Build RowGroup
    let row_group = RowGroup {
        columns: column_chunks,
        total_byte_size,
        num_rows,
    };

    // 4. Build FileMetaData
    let file_meta = FileMetaData {
        version: 1,
        schema: schema_elements,
        num_rows,
        row_groups: alloc::vec![row_group],
        created_by: Some(concat!("babbel_parquet ", env!("CARGO_PKG_VERSION")).to_string()),
    };

    // 5. Serialize FileMetaData with Thrift
    let mut meta_writer = ThriftWriter::new();
    file_meta.encode(&mut meta_writer);
    let meta_bytes = meta_writer.into_bytes();
    let meta_len = meta_bytes.len() as u32;

    buf.extend_from_slice(&meta_bytes);
    // 6. 4-byte little-endian metadata length
    buf.extend_from_slice(&meta_len.to_le_bytes());
    // 7. Trailing 4-byte magic
    buf.extend_from_slice(PARQUET_MAGIC);

    Ok(buf)
}

fn extract_rows(value: &Value) -> Result<Vec<&Vec<(String, Value)>>, ParquetError> {
    match value {
        Value::Array(arr) => {
            let mut rows = Vec::with_capacity(arr.len());
            for v in arr {
                if let Value::Object(obj) = v {
                    rows.push(obj);
                } else {
                    return Err(ParquetError::SchemaMismatch(
                        "Expected array of row objects for Parquet serialization".to_string(),
                    ));
                }
            }
            Ok(rows)
        }
        Value::Object(obj) => {
            // Single row object
            Ok(alloc::vec![obj])
        }
        _ => Err(ParquetError::SchemaMismatch(
            "Expected Object or Array of Objects for Parquet table".to_string(),
        )),
    }
}

fn build_columns(rows: &[&Vec<(String, Value)>]) -> Result<Vec<ColumnData>, ParquetError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    // Discover column names in deterministic order
    let mut col_names = Vec::new();
    for row in rows {
        for (k, _) in *row {
            if !col_names.contains(k) {
                col_names.push(k.clone());
            }
        }
    }

    let mut columns = Vec::with_capacity(col_names.len());

    for name in col_names {
        // Infer column type from first non-null value
        let mut inferred_type = Type::ByteArray;
        for row in rows {
            if let Some((_, v)) = row.iter().find(|(k, _)| k == &name) {
                match v {
                    Value::Bool(_) => {
                        inferred_type = Type::Boolean;
                        break;
                    }
                    Value::Integer(_) => {
                        inferred_type = Type::Int64;
                        break;
                    }
                    Value::Float(_) => {
                        inferred_type = Type::Double;
                        break;
                    }
                    Value::String(_) | Value::Bytes(_) => {
                        inferred_type = Type::ByteArray;
                        break;
                    }
                    _ => {}
                }
            }
        }

        let mut col = ColumnData::new(name.clone(), inferred_type);
        for row in rows {
            let val = row
                .iter()
                .find(|(k, _)| k == &name)
                .map(|(_, v)| v.clone())
                .unwrap_or(Value::Null);
            col.values.push(val);
        }
        columns.push(col);
    }

    Ok(columns)
}
