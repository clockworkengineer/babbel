//! Input Sources Module
//!
//! Aggregates buffer and file-based sources for reading YAML data.
//! Provides unified interfaces for memory and disk input implementations.
//!
//! Copyright (c) 2026 YAML Library Developers

/// Module providing a buffer-based source for reading YAML data from memory
pub mod buffer;
/// Module providing a file-based source for reading YAML data from disk
#[cfg(feature = "file-io")]
pub mod file;
