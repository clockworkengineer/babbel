//! Parquet binary file reader.

#[cfg(not(feature = "std"))]
use alloc::{
    string::ToString,
    vec::Vec,
};

use babbel_core::Value;
use crate::column::ColumnData;
use crate::error::ParquetError;
use crate::metadata::*;
use crate::thrift::ThriftReader;
use crate::writer::PARQUET_MAGIC;

/// Read standard Apache Parquet binary bytes into a tabular Babbel `Value::Array`.
pub fn read_parquet(bytes: &[u8]) -> Result<Value, ParquetError> {
    if bytes.len() < 12 {
        return Err(ParquetError::InvalidMagic);
    }

    // 1. Verify leading magic 'PAR1'
    if &bytes[0..4] != PARQUET_MAGIC {
        return Err(ParquetError::InvalidMagic);
    }

    // 2. Verify trailing magic 'PAR1'
    let len = bytes.len();
    if &bytes[len - 4..len] != PARQUET_MAGIC {
        return Err(ParquetError::InvalidMagic);
    }

    // 3. Read metadata length
    let meta_len = u32::from_le_bytes(bytes[len - 8..len - 4].try_into().unwrap()) as usize;
    if meta_len > len - 8 {
        return Err(ParquetError::CorruptedPage("Metadata length exceeds file size".into()));
    }

    let meta_start = len - 8 - meta_len;
    let meta_bytes = &bytes[meta_start..len - 8];

    // 4. Decode FileMetaData
    let mut meta_reader = ThriftReader::new(meta_bytes);
    let file_meta = FileMetaData::decode(&mut meta_reader)?;

    if file_meta.schema.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }

    // Map schema elements: element 0 is root, elements 1.. are columns
    let col_schemas = &file_meta.schema[1..];
    let mut rows_count = file_meta.num_rows as usize;

    let mut decoded_columns = Vec::new();

    for schema_elem in col_schemas {
        let col_type = schema_elem.type_.unwrap_or(Type::ByteArray);
        let col_name = schema_elem.name.clone();
        let is_optional = schema_elem.repetition_type == Some(FieldRepetitionType::Optional);

        let mut col_data = ColumnData::new(col_name.clone(), col_type);

        for rg in &file_meta.row_groups {
            for chunk in &rg.columns {
                if let Some(ref meta) = chunk.meta_data {
                    if meta.path_in_schema.contains(&col_name) {
                        let offset = meta.data_page_offset as usize;
                        if offset >= bytes.len() {
                            return Err(ParquetError::UnexpectedEof);
                        }

                        // Read PageHeader
                        let mut page_reader = ThriftReader::new(&bytes[offset..]);
                        let page_header = PageHeader::decode(&mut page_reader)?;
                        let header_size = page_reader.pos;

                        let payload_offset = offset + header_size;
                        let payload_size = page_header.uncompressed_page_size as usize;
                        if payload_offset + payload_size > bytes.len() {
                            return Err(ParquetError::UnexpectedEof);
                        }

                        let payload = &bytes[payload_offset..payload_offset + payload_size];
                        let num_values = page_header
                            .data_page_header
                            .map(|d| d.num_values as usize)
                            .unwrap_or(meta.num_values as usize);

                        col_data.decode_plain(payload, num_values, is_optional)?;
                    }
                }
            }
        }

        decoded_columns.push(col_data);
    }

    if let Some(first) = decoded_columns.first() {
        rows_count = first.values.len();
    }

    // 5. Transpose columnar values into row objects
    let mut rows = Vec::with_capacity(rows_count);
    for row_idx in 0..rows_count {
        let mut row_obj = Vec::with_capacity(decoded_columns.len());
        for col in &decoded_columns {
            let val = col
                .values
                .get(row_idx)
                .cloned()
                .unwrap_or(Value::Null);
            row_obj.push((col.name.clone(), val));
        }
        rows.push(Value::Object(row_obj));
    }

    Ok(Value::Array(rows))
}
