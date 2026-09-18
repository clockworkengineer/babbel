//! Direct Serde serialization and deserialization APIs across all supported formats.
//!
//! Allows any Rust type implementing [`serde::Serialize`] or [`serde::Deserialize`]
//! to be serialized to and deserialized from any format engine in the Babbel ecosystem.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

use babbel_core::{BabbelError, FormatEmitter, from_value, to_value};

/// Deserialize an instance of type `T` from a string in the specified format name
/// (e.g. `"json"`, `"yaml"`, `"toml"`, `"ron"`, `"kdl"`, `"xml"`, `"csv"`, `"ini"`).
pub fn from_str<T: serde::de::DeserializeOwned>(
    input: &str,
    format: &str,
) -> Result<T, BabbelError> {
    let registry = crate::default_registry();
    let engine = registry
        .get(format)
        .ok_or_else(|| BabbelError::custom(format!("unsupported format: {}", format)))?;
    let value = engine.parse_str(input)?;
    from_value(&value).map_err(|e| BabbelError::custom(e.0))
}

/// Deserialize an instance of type `T` from a byte slice in the specified format name
/// (e.g. `"cbor"`, `"msgpack"`, `"bson"`, `"bencode"`, `"json"`, `"parquet"`).
pub fn from_slice<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    format: &str,
) -> Result<T, BabbelError> {
    let registry = crate::default_registry();
    let engine = registry
        .get(format)
        .ok_or_else(|| BabbelError::custom(format!("unsupported format: {}", format)))?;
    let value = engine.parse_bytes(bytes)?;
    from_value(&value).map_err(|e| BabbelError::custom(e.0))
}

/// Serialize a data structure `T` into a string in the specified format.
pub fn to_string<T: serde::Serialize>(value: &T, format: &str) -> Result<String, BabbelError> {
    let registry = crate::default_registry();
    let engine = registry
        .get(format)
        .ok_or_else(|| BabbelError::custom(format!("unsupported format: {}", format)))?;
    let ast = to_value(value).map_err(|e| BabbelError::custom(e.0))?;
    let mut dest = babbel_core::io::Buffer::new();
    engine.emit(&ast, &mut dest)?;
    Ok(dest.to_string())
}

/// Serialize a data structure `T` into a formatted, pretty-printed string in the specified format.
pub fn to_string_pretty<T: serde::Serialize>(
    value: &T,
    format: &str,
    indent: usize,
) -> Result<String, BabbelError> {
    let registry = crate::default_registry();
    let engine = registry
        .get(format)
        .ok_or_else(|| BabbelError::custom(format!("unsupported format: {}", format)))?;
    let ast = to_value(value).map_err(|e| BabbelError::custom(e.0))?;
    let mut dest = babbel_core::io::Buffer::new();
    engine.emit_pretty(&ast, &mut dest, indent)?;
    Ok(dest.to_string())
}

/// Serialize a data structure `T` into a byte vector in the specified format.
pub fn to_vec<T: serde::Serialize>(value: &T, format: &str) -> Result<Vec<u8>, BabbelError> {
    let registry = crate::default_registry();
    let engine = registry
        .get(format)
        .ok_or_else(|| BabbelError::custom(format!("unsupported format: {}", format)))?;
    let ast = to_value(value).map_err(|e| BabbelError::custom(e.0))?;
    let mut dest = babbel_core::io::Buffer::new();
    engine.emit(&ast, &mut dest)?;
    Ok(dest.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct UserConfig {
        username: String,
        retries: u32,
        active: bool,
    }

    #[test]
    fn test_cross_format_serde() {
        let config = UserConfig {
            username: "alice".into(),
            retries: 3,
            active: true,
        };

        // JSON
        let json_str = to_string(&config, "json").unwrap();
        let decoded_json: UserConfig = from_str(&json_str, "json").unwrap();
        assert_eq!(decoded_json, config);

        // YAML
        let yaml_str = to_string(&config, "yaml").unwrap();
        let decoded_yaml: UserConfig = from_str(&yaml_str, "yaml").unwrap();
        assert_eq!(decoded_yaml, config);

        // CBOR
        let cbor_bytes = to_vec(&config, "cbor").unwrap();
        let decoded_cbor: UserConfig = from_slice(&cbor_bytes, "cbor").unwrap();
        assert_eq!(decoded_cbor, config);

        // MessagePack
        let msgpack_bytes = to_vec(&config, "msgpack").unwrap();
        let decoded_msgpack: UserConfig = from_slice(&msgpack_bytes, "msgpack").unwrap();
        assert_eq!(decoded_msgpack, config);
    }
}
