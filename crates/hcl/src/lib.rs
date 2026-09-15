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

pub mod error;
pub mod parser;
pub mod serializer;
pub mod engine;

pub use error::HclError;
pub use parser::{from_str, HclParser};
pub use serializer::{to_string, to_string_pretty, HclSerializerConfig};
pub use engine::HclEngine;
pub use babbel_core::Value;

/// Deserialize a universal Babbel [`Value`] from raw HCL byte slice.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, HclError> {
    let s = core::str::from_utf8(bytes).map_err(|_| HclError::InvalidUtf8)?;
    from_str(s)
}

/// Serialize a universal Babbel [`Value`] into an HCL byte vector (`Vec<u8>`).
#[cfg(feature = "alloc")]
pub fn to_vec(value: &Value) -> Result<alloc::vec::Vec<u8>, HclError> {
    let s = to_string(value)?;
    Ok(s.into_bytes())
}
