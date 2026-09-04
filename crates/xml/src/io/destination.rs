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

    /// Clears the destination buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl babbel_core::io::traits::IDestination for XmlDestination {
    fn add_byte(&mut self, byte: u8) {
        self.buffer.push(byte as char);
    }

    fn add_bytes(&mut self, bytes: &str) {
        self.buffer.push_str(bytes);
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn last(&self) -> Option<u8> {
        self.buffer.as_bytes().last().copied()
    }
}

/// Extension trait for [`babbel_core::io::traits::IDestination`] providing string and character writing helpers.
pub trait DestinationExt {
    /// Writes a string slice to destination.
    fn write_str(&mut self, s: &str);
    /// Writes a Unicode scalar character to destination.
    fn write_char(&mut self, c: char);
}

impl<D: babbel_core::io::traits::IDestination + ?Sized> DestinationExt for D {
    #[inline]
    fn write_str(&mut self, s: &str) {
        self.add_bytes(s);
    }

    #[inline]
    fn write_char(&mut self, c: char) {
        if c.is_ascii() {
            self.add_byte(c as u8);
        } else {
            let mut buf = [0u8; 4];
            self.add_bytes(c.encode_utf8(&mut buf));
        }
    }
}
