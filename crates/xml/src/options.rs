//! # Parse Options & Security Policy
//!
//! Provides configurable parsing options and security thresholds to mitigate DoS and XXE vulnerabilities.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};

/// Security policy options and resource limits for the XML parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    /// Maximum XML document size in bytes (default: 100 MB).
    pub max_xml_size: usize,
    /// Maximum depth for recursive entity reference expansion (default: 512).
    pub max_entity_expansion_depth: usize,
    /// Maximum element nesting depth (default: 1000).
    pub max_nesting_depth: usize,
    /// Maximum total element count in document (default: 1,000,000).
    pub max_element_count: usize,
    /// Maximum attribute count per element (default: 10,000).
    pub max_attribute_count: usize,
    /// Maximum total attribute count across document (default: 1,000,000).
    pub max_total_attribute_count: usize,
    /// Maximum text node character size (default: 1 MB).
    pub max_text_node_size: usize,
    /// Maximum total cumulative entity expansion size in bytes (default: 10 MB, Billion Laughs guard).
    pub max_total_entity_expansion_size: usize,
    /// Allow fetching external entity URIs (default: false for XXE protection).
    pub allow_external_entities: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            max_xml_size: 100 * 1024 * 1024,
            max_entity_expansion_depth: 512,
            max_nesting_depth: 1000,
            max_element_count: 1_000_000,
            max_attribute_count: 10_000,
            max_total_attribute_count: 1_000_000,
            max_text_node_size: 1024 * 1024,
            max_total_entity_expansion_size: 10 * 1024 * 1024,
            allow_external_entities: false,
        }
    }
}

impl ParseOptions {
    /// Validates element nesting depth against `max_nesting_depth`.
    pub fn check_nesting_depth(&self, depth: usize) -> Result<()> {
        if depth > self.max_nesting_depth {
            Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum nesting depth of {} exceeded",
                self.max_nesting_depth
            )))
        } else {
            Ok(())
        }
    }

    /// Validates total element count against `max_element_count`.
    pub fn check_element_count(&self, count: usize) -> Result<()> {
        if count > self.max_element_count {
            Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum element count of {} exceeded",
                self.max_element_count
            )))
        } else {
            Ok(())
        }
    }

    /// Validates tag attribute count against `max_attribute_count`.
    pub fn check_attribute_count(&self, count: usize) -> Result<()> {
        if count > self.max_attribute_count {
            Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum element attribute count of {} exceeded",
                self.max_attribute_count
            )))
        } else {
            Ok(())
        }
    }

    /// Validates total attribute count across document against `max_total_attribute_count`.
    pub fn check_total_attribute_count(&self, count: usize) -> Result<()> {
        if count > self.max_total_attribute_count {
            Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum total attribute count of {} exceeded",
                self.max_total_attribute_count
            )))
        } else {
            Ok(())
        }
    }

    /// Validates overall XML document size against `max_xml_size`.
    pub fn check_xml_size(&self, size: usize) -> Result<()> {
        if size > self.max_xml_size {
            Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum XML document size of {} bytes exceeded (found {})",
                self.max_xml_size, size
            )))
        } else {
            Ok(())
        }
    }

    /// Validates character size of a text/CDATA node against `max_text_node_size`.
    pub fn check_text_node_size(&self, size: usize) -> Result<()> {
        if size > self.max_text_node_size {
            Err(XmlError::SecurityLimitExceeded(format!(
                "Maximum text node size of {} characters exceeded (found {})",
                self.max_text_node_size, size
            )))
        } else {
            Ok(())
        }
    }
}

