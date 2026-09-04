//! # Stringify Subsystem
//!
//! Handles serialization of [`Document`] DOM trees back to valid, formatted XML string output
//! and W3C Canonical XML (C14N).

pub mod canonical;
pub mod serializer;

pub use canonical::{CanonicalOptions, CanonicalSerializer};
pub use serializer::{SerializeOptions, XmlSerializer};

use crate::alloc_prelude::*;
use crate::document::Document;

/// Serializes a [`Document`] to a canonical XML string using default C14N options (without comments).
///
/// # Examples
///
/// ```
/// use xml_lib_rust::{canonicalize, parse};
///
/// let doc = parse("<root b=\"2\" a=\"1\"><empty/></root>").unwrap();
/// let c14n = canonicalize(&doc);
/// assert_eq!(c14n, "<root a=\"1\" b=\"2\"><empty></empty></root>");
/// ```
pub fn canonicalize(doc: &Document) -> String {
    CanonicalSerializer::canonicalize(doc, &CanonicalOptions::default())
}
