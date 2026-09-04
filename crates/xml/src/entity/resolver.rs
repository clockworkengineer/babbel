//! # Entity Resolver Traversal Interface
//!
//! Trait definition for custom resolution of external system or public entity URIs.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};

/// Trait implemented by external entity resolvers (e.g. file system or network entity loaders).
pub trait EntityResolver {
    /// Resolves an external entity reference given its system ID URI and optional public ID.
    fn resolve_entity(&self, public_id: Option<&str>, system_id: &str) -> Result<String>;
}

/// Default no-op entity resolver rejecting external entity fetching for XXE security.
#[derive(Debug, Clone, Default)]
pub struct DefaultEntityResolver;

impl EntityResolver for DefaultEntityResolver {
    fn resolve_entity(&self, _public_id: Option<&str>, system_id: &str) -> Result<String> {
        Err(XmlError::SecurityLimitExceeded(format!(
            "External entity resolution is disabled by security policy for system_id '{system_id}'"
        )))
    }
}
