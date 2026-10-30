//! # Babbel HCL
//!
//! Fast, pure-Rust parser, serializer, and [`FormatEngine`] implementation for
//! **HashiCorp Configuration Language v2 (HCL2)** (`.hcl`, `.tf`, `.tfvars`).
//!
//! ## Quickstart
//!
//! ```rust
//! use babbel_hcl::{from_str, to_string_pretty};
//!
//! let hcl_src = r#"
//! variable "region" {
//!     default = "us-east-1"
//! }
//! "#;
//!
//! let val = from_str(hcl_src).unwrap();
//! let out = to_string_pretty(&val, 2).unwrap();
//! assert!(out.contains("variable"));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod engine;
pub mod error;
pub mod eval;
pub mod parser;
pub mod serializer;

pub use babbel_core::Value;
pub use engine::HclEngine;
pub use error::HclError;
pub use eval::HclEvalContext;
pub use parser::{HclParser, from_str, from_str_with_context};
pub use serializer::{HclSerializerConfig, to_string, to_string_pretty};

/// Deserialize a universal Babbel [`Value`] from raw HCL byte slice.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, HclError> {
    let s = core::str::from_utf8(bytes).map_err(|_| HclError::InvalidUtf8)?;
    from_str(s)
}

/// Deserialize a universal Babbel [`Value`] from raw HCL byte slice.
#[inline]
pub fn from_slice(bytes: &[u8]) -> Result<Value, HclError> {
    from_bytes(bytes)
}

/// Serialize a universal Babbel [`Value`] into an HCL byte vector (`Vec<u8>`).
#[cfg(feature = "alloc")]
pub fn to_vec(value: &Value) -> Result<alloc::vec::Vec<u8>, HclError> {
    let s = to_string(value)?;
    Ok(s.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_hcl_eval_context_functions_splats_and_comprehensions() {
        let ctx = HclEvalContext::new()
            .with_var(
                "var",
                Value::Object(vec![
                    ("env".into(), Value::String("prod".into())),
                    (
                        "tags".into(),
                        Value::Array(vec![
                            Value::String("web".into()),
                            Value::String("skip".into()),
                            Value::String("api".into()),
                        ]),
                    ),
                ]),
            )
            .with_var(
                "users",
                Value::Array(vec![
                    Value::Object(vec![("name".into(), Value::String("alice".into()))]),
                    Value::Object(vec![("name".into(), Value::String("bob".into()))]),
                ]),
            );

        let src = r#"
            banner = "deploy-${upper(var.env)}"
            joined = join(":", [for t in var.tags : upper(t) if t != "skip"])
            names  = users[*].name
            total  = max(10, 42, 7)
            b64    = base64decode(base64encode("terraform"))
        "#;

        let val = from_str_with_context(src, &ctx).unwrap();
        assert_eq!(
            val.get("banner").and_then(|v| v.as_str()),
            Some("deploy-PROD")
        );
        assert_eq!(val.get("joined").and_then(|v| v.as_str()), Some("WEB:API"));
        assert_eq!(
            val.get("names"),
            Some(&Value::Array(vec![
                Value::String("alice".into()),
                Value::String("bob".into())
            ]))
        );
        assert_eq!(val.get("total").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(val.get("b64").and_then(|v| v.as_str()), Some("terraform"));
    }
}
