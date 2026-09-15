//! Apache Avro Object Container File (OCF) parser and serializer.
//!
//! Defined by the Apache Avro specification:
//! - 4-byte magic header: `Obj\x01`
//! - File metadata map (containing `avro.schema` and `avro.codec`, encoded as `map<bytes>`)
//! - 16-byte randomly generated sync marker
//! - Repeated data blocks followed by sync markers

#[cfg(not(feature = "std"))]
use alloc::{string::String, string::ToString, vec::Vec};

use babbel_core::Value;
use crate::codec::{AvroDecoder, AvroEncoder};
use crate::error::AvroError;

/// 4-byte magic header identifying an Avro Object Container File.
pub const OCF_MAGIC: [u8; 4] = [b'O', b'b', b'j', 1];

/// Standard 16-byte sync marker for deterministic test runs.
pub const DEFAULT_SYNC_MARKER: [u8; 16] = [
    0x42, 0x61, 0x62, 0x62, 0x65, 0x6c, 0x41, 0x76,
    0x72, 0x6f, 0x53, 0x79, 0x6e, 0x63, 0x30, 0x31,
];

/// Encodes an array of records into a complete Avro Object Container File (OCF).
pub fn to_vec_ocf(value: &Value, schema_json: &str) -> Result<Vec<u8>, AvroError> {
    let mut out = Vec::new();
    out.extend_from_slice(&OCF_MAGIC);

    // Write file metadata map: {"avro.schema": bytes(schema_json), "avro.codec": bytes("null")}
    let mut meta_encoder = AvroEncoder::new();
    meta_encoder.write_long(2); // 2 entries
    meta_encoder.write_string("avro.schema");
    meta_encoder.write_bytes(schema_json.as_bytes());
    meta_encoder.write_string("avro.codec");
    meta_encoder.write_bytes(b"null");
    meta_encoder.write_long(0); // end map

    out.extend_from_slice(&meta_encoder.into_vec());
    out.extend_from_slice(&DEFAULT_SYNC_MARKER);

    // Serialize payload records
    let mut data_encoder = AvroEncoder::new();
    let count = match value {
        Value::Array(items) => {
            for item in items {
                data_encoder.write_value(item);
            }
            items.len() as i64
        }
        single => {
            data_encoder.write_value(single);
            1i64
        }
    };

    let block_bytes = data_encoder.into_vec();
    if count > 0 {
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
    if bytes.len() < 24 {
        return Err(AvroError::UnexpectedEof { expected: 24, available: bytes.len() });
    }

    if &bytes[0..4] != &OCF_MAGIC {
        return Err(AvroError::InvalidMagic);
    }

    let mut decoder = AvroDecoder::new(&bytes[4..]);
    // Decode metadata map (map<bytes>)
    let mut meta_count = decoder.read_long()?;
    while meta_count > 0 {
        for _ in 0..meta_count {
            let _key = decoder.read_string()?;
            let _val = decoder.read_bytes()?;
        }
        meta_count = decoder.read_long()?;
    }

    let cursor_after_meta = 4 + decoder.position();
    let mut current_pos = cursor_after_meta;

    if bytes.len() < current_pos + 16 {
        return Err(AvroError::UnexpectedEof { expected: 16, available: bytes.len() - current_pos });
    }

    let mut sync_marker = [0u8; 16];
    sync_marker.copy_from_slice(&bytes[current_pos..current_pos + 16]);
    current_pos += 16;

    let mut records = Vec::new();
    while current_pos < bytes.len() {
        let mut block_dec = AvroDecoder::new(&bytes[current_pos..]);
        let count = block_dec.read_long()?;
        let block_len = block_dec.read_long()? as usize;
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

        let mut payload_dec = AvroDecoder::new(block_payload);
        for _ in 0..count {
            records.push(payload_dec.decode_value()?);
        }
    }

    Ok(Value::Array(records))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
