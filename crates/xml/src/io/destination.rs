//! # XML Output Destination
//!
//! Provides the [`XmlDestination`] wrapper writing serialized XML into string buffers or standard output writers.

use crate::alloc_prelude::*;
use core::fmt::Write;

/// Output destination wrapper wrapping a mutable string buffer.
#[derive(Debug, Default)]
pub struct XmlDestination {
    buffer: String,
}

impl XmlDestination {
    /// Creates a new empty [`XmlDestination`].
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Appends string slice content to destination buffer.
    pub fn write_str(&mut self, s: &str) {
        let _ = self.buffer.write_str(s);
    }

    /// Appends a single character to destination buffer.
    pub fn write_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Consumes destination and returns accumulated string buffer.
    pub fn into_string(self) -> String {
        self.buffer
    }

    /// Returns string slice reference to destination content.
    pub fn as_str(&self) -> &str {
        &self.buffer
    }
}
