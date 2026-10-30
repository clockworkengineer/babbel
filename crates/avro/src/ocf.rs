//! Apache Avro Object Container File (OCF) parser and serializer.
//!
//! Defined by the Apache Avro specification:
//! - 4-byte magic header: `Obj\x01`
//! - File metadata map (containing `avro.schema` and `avro.codec`, encoded as `map<bytes>`)
//! - 16-byte randomly generated sync marker
//! - Repeated data blocks followed by sync markers

#[cfg(not(feature = "std"))]
use alloc::{string::String, string::ToString, vec::Vec};

use crate::codec::{AvroDecoder, AvroEncoder};
use crate::error::AvroError;
use crate::schema::AvroSchema;
use babbel_core::Value;

/// 4-byte magic header identifying an Avro Object Container File.
pub const OCF_MAGIC: [u8; 4] = [b'O', b'b', b'j', 1];

/// Standard 16-byte sync marker for deterministic test runs.
pub const DEFAULT_SYNC_MARKER: [u8; 16] = [
    0x42, 0x61, 0x62, 0x62, 0x65, 0x6c, 0x41, 0x76, 0x72, 0x6f, 0x53, 0x79, 0x6e, 0x63, 0x30, 0x31,
];

/// Encodes an array of records into a complete Avro Object Container File (OCF) using `"null"` codec.
pub fn to_vec_ocf(value: &Value, schema_json: &str) -> Result<Vec<u8>, AvroError> {
    to_vec_ocf_with_codec(value, schema_json, "null")
}

/// Encodes an array of records into a complete Avro Object Container File (OCF) with the specified codec (`"null"` or `"snappy"`).
pub fn to_vec_ocf_with_codec(
    value: &Value,
    schema_json: &str,
    codec: &str,
) -> Result<Vec<u8>, AvroError> {
    if codec != "null" && codec != "snappy" {
        return Err(AvroError::Custom(
            "unsupported compression codec in Avro OCF",
        ));
    }

    let mut out = Vec::new();
    out.extend_from_slice(&OCF_MAGIC);

    // Write file metadata map: {"avro.schema": bytes(schema_json), "avro.codec": bytes(codec)}
    let mut meta_encoder = AvroEncoder::new();
    meta_encoder.write_long(2); // 2 entries
    meta_encoder.write_string("avro.schema");
    meta_encoder.write_bytes(schema_json.as_bytes());
    meta_encoder.write_string("avro.codec");
    meta_encoder.write_bytes(codec.as_bytes());
    meta_encoder.write_long(0); // end map

    out.extend_from_slice(&meta_encoder.into_vec());
    out.extend_from_slice(&DEFAULT_SYNC_MARKER);

    let parsed_schema = crate::schema::AvroSchema::parse_str(schema_json).ok();

    // Serialize payload records
    let mut data_encoder = AvroEncoder::new();
    let count = match value {
        Value::Array(items) => {
            for item in items {
                if let Some(ref s) = parsed_schema {
                    data_encoder.write_with_schema(item, s)?;
                } else {
                    data_encoder.write_value(item);
                }
            }
            items.len() as i64
        }
        single => {
            if let Some(ref s) = parsed_schema {
                data_encoder.write_with_schema(single, s)?;
            } else {
                data_encoder.write_value(single);
            }
            1i64
        }
    };

    let uncompressed_bytes = data_encoder.into_vec();
    if count > 0 {
        let block_bytes = if codec == "snappy" {
            let mut compressed = babbel_core::snappy_compress(&uncompressed_bytes);
            let crc = babbel_core::crc32_ieee(&uncompressed_bytes);
            compressed.extend_from_slice(&crc.to_be_bytes());
            compressed
        } else {
            uncompressed_bytes
        };

        let mut block_header = AvroEncoder::new();
        block_header.write_long(count);
        block_header.write_long(block_bytes.len() as i64);
        out.extend_from_slice(&block_header.into_vec());
        out.extend_from_slice(&block_bytes);
        out.extend_from_slice(&DEFAULT_SYNC_MARKER);
    }

    Ok(out)
}

/// Decodes an Avro Object Container File (OCF) into a universal `Value`.
pub fn from_bytes_ocf(bytes: &[u8]) -> Result<Value, AvroError> {
    from_bytes_ocf_with_reader_schema(bytes, None)
}

/// Decodes an Avro Object Container File (OCF) using an optional `reader_schema` for schema evolution.
pub fn from_bytes_ocf_with_reader_schema(
    bytes: &[u8],
    reader_schema: Option<&AvroSchema>,
) -> Result<Value, AvroError> {
    if bytes.len() < 24 {
        return Err(AvroError::UnexpectedEof {
            expected: 24,
            available: bytes.len(),
        });
    }

    if &bytes[0..4] != &OCF_MAGIC {
        return Err(AvroError::InvalidMagic);
    }

    let mut decoder = AvroDecoder::new(&bytes[4..]);
    // Decode metadata map (map<bytes>)
    let mut meta_count = decoder.read_long()?;
    let mut schema_str: Option<String> = None;
    let mut codec_str: Option<String> = None;

    while meta_count != 0 {
        let count = if meta_count < 0 {
            let _block_size = decoder.read_long()?;
            -meta_count
        } else {
            meta_count
        };
        for _ in 0..count {
            let key = decoder.read_string()?;
            let val = decoder.read_bytes()?;
            if key == "avro.schema" {
                schema_str = Some(
                    core::str::from_utf8(val)
                        .map_err(|_| AvroError::InvalidUtf8)?
                        .to_string(),
                );
            } else if key == "avro.codec" {
                codec_str = Some(
                    core::str::from_utf8(val)
                        .map_err(|_| AvroError::InvalidUtf8)?
                        .to_string(),
                );
            }
        }
        meta_count = decoder.read_long()?;
    }

    let active_codec = codec_str.as_deref().unwrap_or("null");
    if active_codec != "null" && active_codec != "snappy" {
        return Err(AvroError::Custom(
            "unsupported compression codec in Avro OCF",
        ));
    }

    let parsed_schema = if let Some(ref s) = schema_str {
        crate::schema::AvroSchema::parse_str(s).ok()
    } else {
        None
    };

    let cursor_after_meta = 4 + decoder.position();
    let mut current_pos = cursor_after_meta;

    if bytes.len() < current_pos + 16 {
        return Err(AvroError::UnexpectedEof {
            expected: 16,
            available: bytes.len() - current_pos,
        });
    }

    let mut sync_marker = [0u8; 16];
    sync_marker.copy_from_slice(&bytes[current_pos..current_pos + 16]);
    current_pos += 16;

    let mut records = Vec::new();
    while current_pos < bytes.len() {
        let mut block_dec = AvroDecoder::new(&bytes[current_pos..]);
        let raw_count = block_dec.read_long()?;
        let block_len = block_dec.read_long()? as usize;
        let count = raw_count.abs();
        let header_len = block_dec.position();
        current_pos += header_len;

        if bytes.len() < current_pos + block_len + 16 {
            return Err(AvroError::UnexpectedEof {
                expected: block_len + 16,
                available: bytes.len() - current_pos,
            });
        }

        let block_payload = &bytes[current_pos..current_pos + block_len];
        current_pos += block_len;

        let block_sync = &bytes[current_pos..current_pos + 16];
        if block_sync != sync_marker {
            return Err(AvroError::SyncMarkerMismatch);
        }
        current_pos += 16;

        let decompressed_buf: Vec<u8>;
        let decoded_slice: &[u8] = if active_codec == "snappy" {
            if block_payload.len() < 4 {
                return Err(AvroError::Custom(
                    "Snappy Avro OCF block missing 4-byte CRC32 trailer",
                ));
            }
            let snappy_slice = &block_payload[..block_payload.len() - 4];
            let crc_bytes = &block_payload[block_payload.len() - 4..];
            let expected_crc =
                u32::from_be_bytes([crc_bytes[0], crc_bytes[1], crc_bytes[2], crc_bytes[3]]);
            decompressed_buf = babbel_core::snappy_decompress(snappy_slice)
                .map_err(|_| AvroError::Custom("failed to decompress Snappy Avro OCF block"))?;
            let actual_crc = babbel_core::crc32_ieee(&decompressed_buf);
            if actual_crc != expected_crc {
                return Err(AvroError::Custom(
                    "Avro OCF Snappy block CRC32 checksum mismatch",
                ));
            }
            &decompressed_buf
        } else {
            block_payload
        };

        let mut payload_dec = AvroDecoder::new(decoded_slice);
        for _ in 0..count {
            match (&parsed_schema, reader_schema) {
                (Some(w_schema), Some(r_schema)) => {
                    records.push(payload_dec.decode_with_schemas(w_schema, r_schema)?);
                }
                (Some(w_schema), None) => {
                    records.push(payload_dec.decode_with_schema(w_schema)?);
                }
                _ => {
                    records.push(payload_dec.decode_value()?);
                }
            }
        }
    }

    Ok(Value::Array(records))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_ocf_roundtrip() {
        let records = Value::Array(vec![
            Value::Object(vec![
                ("id".into(), Value::Integer(1)),
                ("name".into(), Value::String("Alice".into())),
            ]),
            Value::Object(vec![
                ("id".into(), Value::Integer(2)),
                ("name".into(), Value::String("Bob".into())),
            ]),
        ]);

        let schema = r#"{"type": "record", "name": "User", "fields": [{"name": "id", "type": "long"}, {"name": "name", "type": "string"}]}"#;
        let ocf_bytes = to_vec_ocf(&records, schema).unwrap();
        assert!(ocf_bytes.starts_with(&OCF_MAGIC));

        let decoded = from_bytes_ocf(&ocf_bytes).unwrap();
        assert!(matches!(decoded, Value::Array(_)));
        let items = decoded.as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].get("name").and_then(|v| v.as_str()), Some("Alice"));
        assert_eq!(items[1].get("name").and_then(|v| v.as_str()), Some("Bob"));
    }

    #[test]
    fn test_ocf_snappy_and_schema_evolution() {
        let records = Value::Array(vec![
            Value::Object(vec![
                ("old_id".into(), Value::Integer(10)),
                ("name".into(), Value::String("Alice".into())),
                ("obsolete".into(), Value::String("drop_me".into())),
            ]),
            Value::Object(vec![
                ("old_id".into(), Value::Integer(20)),
                ("name".into(), Value::String("Bob".into())),
                ("obsolete".into(), Value::String("drop_me_too".into())),
            ]),
        ]);

        let writer_schema = r#"{
            "type": "record",
            "name": "User",
            "fields": [
                {"name": "old_id", "type": "int"},
                {"name": "name", "type": "string"},
                {"name": "obsolete", "type": "string"}
            ]
        }"#;

        let reader_schema_json = r#"{
            "type": "record",
            "name": "User",
            "fields": [
                {"name": "id", "type": "double", "aliases": ["old_id"]},
                {"name": "name", "type": "string"},
                {"name": "role", "type": "string", "default": "member"}
            ]
        }"#;
        let reader_schema = AvroSchema::parse_str(reader_schema_json).unwrap();

        let ocf_snappy = to_vec_ocf_with_codec(&records, writer_schema, "snappy").unwrap();
        let evolved = from_bytes_ocf_with_reader_schema(&ocf_snappy, Some(&reader_schema)).unwrap();
        let items = evolved.as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].get("id").and_then(|v| v.as_f64()), Some(10.0));
        assert_eq!(
            items[0].get("role").and_then(|v| v.as_str()),
            Some("member")
        );
        assert!(items[0].get("obsolete").is_none());
    }
}
