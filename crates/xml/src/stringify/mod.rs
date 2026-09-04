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
use babbel_core::io::traits::IDestination;

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

/// Serializes a [`Document`] directly to an output destination implementing [`IDestination`] using default options.
pub fn stringify_to(doc: &Document, dest: &mut dyn IDestination) {
    XmlSerializer::serialize(doc, dest, &SerializeOptions::default());
}

/// Serializes a [`Document`] to an output destination implementing [`IDestination`] with custom options.
pub fn stringify_to_with_options(
    doc: &Document,
    dest: &mut dyn IDestination,
    options: &SerializeOptions,
) {
    XmlSerializer::serialize(doc, dest, options);
}

/// Serializes a [`Document`] to an output destination according to W3C C14N rules using default options.
pub fn canonicalize_to(doc: &Document, dest: &mut dyn IDestination) {
    CanonicalSerializer::serialize_canonical(doc, &CanonicalOptions::default(), dest);
}

/// Serializes a [`Document`] to an output destination according to W3C C14N rules with custom options.
pub fn canonicalize_to_with_options(
    doc: &Document,
    dest: &mut dyn IDestination,
    options: &CanonicalOptions,
) {
    CanonicalSerializer::serialize_canonical(doc, options, dest);
}
