//! Apache Parquet file reader and row materializer.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::column::ColumnData;
use crate::error::ParquetError;
use crate::metadata::{
    CompressionCodec, Encoding, FieldRepetitionType, FileMetaData, PageHeader, Type,
};
use crate::thrift::ThriftReader;
use babbel_core::Value;

/// Options for column projection and row slicing when reading a Parquet file.
#[derive(Debug, Clone, Default)]
pub struct ParquetReadOptions {
    /// Optional subset of column names to decode (column projection pushdown).
    /// When `None`, all columns are decoded.
    pub columns: Option<Vec<String>>,
    /// Maximum number of rows to materialize.
    pub limit: Option<usize>,
    /// Number of initial rows to skip.
    pub offset: usize,
}

impl ParquetReadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_columns<I, S>(mut self, cols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.columns = Some(cols.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

/// Read an Apache Parquet binary payload into a `Value::Array` of `Value::Object` rows.
pub fn read_parquet(data: &[u8]) -> Result<Value, ParquetError> {
    read_parquet_with_options(data, &ParquetReadOptions::default())
}

/// Read an Apache Parquet binary payload with column projection and row limit/offset options.
pub fn read_parquet_with_options(
    data: &[u8],
    options: &ParquetReadOptions,
) -> Result<Value, ParquetError> {
    // 1. Validate magic bytes ("PAR1" at head and tail) + 4-byte footer length
    if data.len() < 12 {
        return Err(ParquetError::InvalidMagic);
    }
    if &data[0..4] != b"PAR1" || &data[data.len() - 4..] != b"PAR1" {
        return Err(ParquetError::InvalidMagic);
    }

    // 2. Read footer length
    let footer_len_pos = data.len() - 8;
    let footer_len =
        u32::from_le_bytes(data[footer_len_pos..footer_len_pos + 4].try_into().unwrap()) as usize;

    if footer_len + 8 > data.len() {
        return Err(ParquetError::InvalidFooterLength(footer_len));
    }

    let footer_start = footer_len_pos - footer_len;
    let footer_bytes = &data[footer_start..footer_len_pos];

    // 3. Decode FileMetaData using Thrift Compact Protocol
    let mut thrift_reader = ThriftReader::new(footer_bytes);
    let file_meta = FileMetaData::decode(&mut thrift_reader)?;

    if file_meta.schema.is_empty() {
        return Err(ParquetError::CorruptedPage(
            "Parquet schema is missing".into(),
        ));
    }

    // Extract leaf columns from schema (skipping the root schema element at index 0)
    let leaf_schemas = &file_meta.schema[1..];
    let mut all_rows = Vec::new();

    // 4. Process each RowGroup
    for rg in &file_meta.row_groups {
        let num_rows = rg.num_rows as usize;
        if num_rows == 0 {
            continue;
        }

        let mut decoded_columns: Vec<ColumnData> = Vec::new();

        for (col_idx, col_chunk) in rg.columns.iter().enumerate() {
            let schema_elem = leaf_schemas.get(col_idx).ok_or_else(|| {
                ParquetError::SchemaMismatch(alloc::format!(
                    "Column index {} missing in schema",
                    col_idx
                ))
            })?;

            // Column projection check: skip decoding columns not requested
            if let Some(ref proj_cols) = options.columns {
                if !proj_cols.iter().any(|c| c == &schema_elem.name) {
                    continue;
                }
            }

            let meta = col_chunk.meta_data.as_ref().ok_or_else(|| {
                ParquetError::CorruptedPage("Missing ColumnMetaData in ColumnChunk".into())
            })?;

            let mut col_data = ColumnData::new(
                &schema_elem.name,
                schema_elem.type_.unwrap_or(Type::ByteArray),
            );
            let is_optional = schema_elem.repetition_type == Some(FieldRepetitionType::Optional);

            let start_offset = meta
                .dictionary_page_offset
                .filter(|&off| off > 0 && off < meta.data_page_offset)
                .unwrap_or(meta.data_page_offset) as usize;

            if start_offset >= footer_start {
                return Err(ParquetError::CorruptedPage(
                    "Page offset out of bounds".into(),
                ));
            }

            let mut cursor = start_offset;
            let mut dictionary: Option<Vec<Value>> = None;

            while col_data.len() < num_rows && cursor < footer_start {
                let mut page_reader = ThriftReader::new(&data[cursor..footer_start]);
                let page_header = PageHeader::decode(&mut page_reader)?;
                cursor += page_reader.position();

                let compressed_size = page_header.compressed_page_size as usize;
                let uncompressed_size = page_header.uncompressed_page_size as usize;

                if cursor + compressed_size > data.len() {
                    return Err(ParquetError::CorruptedPage(
                        "Page payload exceeds file bounds".into(),
                    ));
                }

                let raw_page_payload = &data[cursor..cursor + compressed_size];
                cursor += compressed_size;

                match page_header.type_ {
                    2 => {
                        // DICTIONARY_PAGE
                        let dict_hdr = page_header.dictionary_page_header.ok_or_else(|| {
                            ParquetError::CorruptedPage("Missing DictionaryPageHeader".into())
                        })?;
                        if dict_hdr.num_values < 0 {
                            return Err(ParquetError::CorruptedPage(
                                "Negative num_values in DictionaryPageHeader".into(),
                            ));
                        }
                        let decompressed =
                            decompress_page(raw_page_payload, uncompressed_size, meta.codec)?;
                        let dict_vals = col_data.decode_dictionary_values(
                            &decompressed,
                            dict_hdr.num_values as usize,
                        )?;
                        dictionary = Some(dict_vals);
                    }
                    0 => {
                        // DATA_PAGE (v1)
                        let dp_header = page_header.data_page_header.ok_or_else(|| {
                            ParquetError::CorruptedPage("Missing DataPageHeader".into())
                        })?;
                        if dp_header.num_values < 0 {
                            return Err(ParquetError::CorruptedPage(
                                "Negative num_values in DataPageHeader".into(),
                            ));
                        }
                        let decompressed =
                            decompress_page(raw_page_payload, uncompressed_size, meta.codec)?;
                        let page_num_values = dp_header.num_values as usize;
                        match dp_header.encoding {
                            Encoding::PlainDictionary | Encoding::RleDictionary => {
                                let dict_slice = dictionary.as_deref().ok_or_else(|| {
                                    ParquetError::CorruptedPage(
                                        "Dictionary page missing for RLE_DICTIONARY column".into(),
                                    )
                                })?;
                                col_data.decode_dictionary_page(
                                    &decompressed,
                                    page_num_values,
                                    is_optional,
                                    dict_slice,
                                )?;
                            }
                            _ => {
                                col_data.decode_plain(
                                    &decompressed,
                                    page_num_values,
                                    is_optional,
                                )?;
                            }
                        }
                    }
                    3 => {
                        // DATA_PAGE_V2
                        let dp2 = page_header.data_page_header_v2.ok_or_else(|| {
                            ParquetError::CorruptedPage("Missing DataPageHeaderV2".into())
                        })?;
                        if dp2.num_values < 0 {
                            return Err(ParquetError::CorruptedPage(
                                "Negative num_values in DataPageHeaderV2".into(),
                            ));
                        }
                        let rep_len = dp2.repetition_levels_byte_length.max(0) as usize;
                        let def_len = dp2.definition_levels_byte_length.max(0) as usize;
                        if rep_len + def_len > raw_page_payload.len() {
                            return Err(ParquetError::CorruptedPage(
                                "DATA_PAGE_V2 level lengths exceed page payload".into(),
                            ));
                        }
                        let def_bytes = &raw_page_payload[rep_len..rep_len + def_len];
                        let raw_values = &raw_page_payload[rep_len + def_len..];
                        let is_compressed = dp2.is_compressed.unwrap_or(true);
                        let values_uncompressed_len =
                            uncompressed_size.saturating_sub(rep_len + def_len);
                        let decompressed_values = if is_compressed {
                            decompress_page(raw_values, values_uncompressed_len, meta.codec)?
                        } else {
                            raw_values.to_vec()
                        };
                        let is_dict = matches!(
                            dp2.encoding,
                            Encoding::PlainDictionary | Encoding::RleDictionary
                        );
                        col_data.decode_data_page_v2(
                            def_bytes,
                            &decompressed_values,
                            dp2.num_values as usize,
                            is_optional,
                            is_dict,
                            dictionary.as_deref(),
                        )?;
                    }
                    _ => {
                        // Skip index or other page types
                    }
                }
            }

            decoded_columns.push(col_data);
        }

        // Transpose columnar vectors back into row objects
        for row_idx in 0..num_rows {
            let mut row_entries = Vec::with_capacity(decoded_columns.len());
            for col in &decoded_columns {
                let val = col.values.get(row_idx).cloned().unwrap_or(Value::Null);
                row_entries.push((col.name.clone(), val));
            }
            all_rows.push(Value::Object(row_entries));
        }
    }

    // Apply offset and limit if specified
    let start = options.offset.min(all_rows.len());
    let sliced = match options.limit {
        Some(limit) => {
            let end = (start + limit).min(all_rows.len());
            all_rows[start..end].to_vec()
        }
        None if start > 0 => all_rows[start..].to_vec(),
        None => all_rows,
    };

    Ok(Value::Array(sliced))
}

fn decompress_page(
    payload: &[u8],
    uncompressed_size: usize,
    codec: CompressionCodec,
) -> Result<Vec<u8>, ParquetError> {
    match codec {
        CompressionCodec::Uncompressed => Ok(payload.to_vec()),
        CompressionCodec::Snappy => babbel_core::snappy_decompress(payload)
            .map_err(|e| ParquetError::CorruptedPage(e.message)),
        CompressionCodec::Lz4Raw | CompressionCodec::Lz4 => {
            babbel_core::lz4_decompress(payload, uncompressed_size)
                .map_err(|e| ParquetError::CorruptedPage(e.message))
        }
        other => Err(ParquetError::UnsupportedCompression(alloc::format!(
            "{:?}", other
        ))),
    }
}
