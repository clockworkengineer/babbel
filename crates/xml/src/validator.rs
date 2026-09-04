//! # XML Validator Trait
//!
//! Provides a unified abstract validation trait ([`XmlValidator`]) adhering to SOLID design principles.

use crate::document::Document;
use crate::error::Result;

/// Abstract interface for XML schema and rule set validators.
///
/// Implemented by [`DtdValidator`](crate::DtdValidator) and [`XsdValidator`](crate::XsdValidator),
/// as well as custom user-defined validation logic.
pub trait XmlValidator {
    /// Validates an in-memory DOM [`Document`] against the schema or ruleset.
    ///
    /// # Errors
    /// Returns [`XmlError`](crate::XmlError) if validation fails.
    fn validate(&self, doc: &Document) -> Result<()>;
}
