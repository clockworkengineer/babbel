//! json_lib - A lightweight, modular JSON toolkit for Rust
//!
//! This library provides a flexible JSON implementation with:
//! - Core Node type for representing JSON structures
//! - Parser to build Node trees from streams
//! - Multiple format serializers (JSON, YAML, XML, Bencode)
//! - File and buffer I/O abstractions (with `file-io` feature)
//! - Pretty-printing utilities
//! - Unicode-aware file handling (with `file-io` feature)
//!
//! ## no_std Support
//!
//! This library supports `no_std` environments with the `alloc` crate.
//! Disable default features and enable only what you need:
//!
//! ```toml
//! [dependencies]
//! json_lib = { version = "0.1", default-features = false, features = ["alloc"] }
//! ```
//!
//! Minimum supported Rust version: 1.88.0

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Module defining error types and handling for JSON operations.
pub mod error;

// Re-export ParseError for convenience
pub use error::parse_error::ParseError;

/// Module handling JSON file reading and writing operations
#[cfg(feature = "file-io")]
pub mod file;

/// Module providing input/output operations for reading and writing JSON data
pub mod io;

/// Module containing utility functions and helpers for JSON processing
pub mod misc;

/// Module containing JSON data structure definitions and node types
pub mod nodes;

/// Module implementing JSON parsing and value extraction
pub mod parser;

/// Module for converting JSON structures to formatted strings
pub mod stringify;

// Re-export pretty printing for convenience
pub use stringify::pretty::stringify_pretty;

/// Macros for convenient JSON construction
#[cfg(feature = "alloc")]
#[macro_use]
pub mod macros;

// Re-export macro helper functions so macros can access them via $crate
#[cfg(feature = "alloc")]
#[doc(hidden)]
pub use macros::{__json_map_new, __json_vec_new};

/// Embedded systems utilities and helpers
#[cfg(feature = "alloc")]
pub mod embedded;

/// Integration tests module
#[cfg(test)]
mod integration_tests;

/// This enum represents different Unicode text file formats with their corresponding byte order marks (BOM)
#[cfg(feature = "file-io")]
pub use file::file::Format;

/// This function detects the Unicode format of a text file by examining its byte order mark (BOM)
#[cfg(feature = "file-io")]
pub use file::file::detect_format;

/// This function reads a text file and returns its content as a String, handling different Unicode formats
#[cfg(feature = "file-io")]
pub use file::file::read_file_to_string;

/// This function writes a string to a file in the specified Unicode format
#[cfg(feature = "file-io")]
pub use file::file::write_file_from_string;
///
/// JSON_lib API
///

/// Returns the current version of the JSON library
pub use misc::get_version as version;
/// Prints a formatted string to the destination.
pub use misc::print;
/// Strip whitespace from a string.
pub use misc::strip as strip_whitespace;

/// Destination implementation for writing JSON data to a memory buffer
pub use io::destinations::buffer::Buffer as BufferDestination;
/// Destination implementation for writing JSON data to a file
#[cfg(feature = "file-io")]
pub use io::destinations::file::File as FileDestination;

/// Source implementation for reading JSON data from a memory buffer
pub use io::sources::buffer::Buffer as BufferSource;

/// Source implementation for reading JSON data from a file
#[cfg(feature = "file-io")]
pub use io::sources::file::File as FileSource;
/// Core data structure representing a JSON node and numerical node in the parsed tree
pub use nodes::node::Node;
/// Core data structure representing a numeric value node in the parsed tree
pub use nodes::node::Numeric;
/// Parser configuration for controlling resource limits
pub use parser::config::ParserConfig;
/// Convenience function to parse JSON from bytes
pub use parser::default::from_bytes;
/// Convenience function to parse JSON from a string
pub use parser::default::from_str;
/// Parses json data into a Node tree structure
pub use parser::default::parse;
/// Parses json data with custom configuration for resource limits
pub use parser::default::parse_with_config;
/// Parser statistics for profiling and memory tracking
#[cfg(feature = "alloc")]
pub use parser::stats::ParseStats;
/// Validates JSON syntax without allocating memory
pub use parser::validate::validate_json;

// Performance-optimized parsing and stringification
/// Arena allocator for reduced allocation overhead
#[cfg(feature = "alloc")]
pub use parser::arena;
/// Fast parsing utilities with SIMD optimizations
#[cfg(feature = "alloc")]
pub use parser::fast;
/// Small String Optimization for reduced heap allocations
#[cfg(feature = "alloc")]
pub use parser::sso;
/// Optimized stringify with lazy string escaping
#[cfg(feature = "alloc")]
pub use stringify::optimized::stringify_optimized;

/// Converts a Node tree to Bencode format
#[cfg(feature = "format-bencode")]
pub use stringify::bencode::stringify as to_bencode;

/// Converts a Node tree back to JSON format
pub use stringify::default::stringify;

/// Converts a Node tree to TOML format
#[cfg(feature = "format-toml")]
pub use stringify::toml::stringify as to_toml;

/// Converts a Node tree to XML format
#[cfg(feature = "format-xml")]
pub use stringify::xml::stringify as to_xml;

/// Converts a Node tree to YAML format
#[cfg(feature = "format-yaml")]
pub use stringify::yaml::stringify as to_yaml;

// JSON Pointer support (RFC 6901)
/// Gets a value from a Node using a JSON Pointer string
#[cfg(feature = "json-pointer")]
pub use nodes::json_pointer::get as pointer_get;

/// Gets a mutable reference to a value using a JSON Pointer string
#[cfg(feature = "json-pointer")]
pub use nodes::json_pointer::get_mut as pointer_get_mut;

/// Removes a value from a Node using a JSON Pointer string
#[cfg(feature = "json-pointer")]
pub use nodes::json_pointer::remove as pointer_remove;

/// Sets a value in a Node using a JSON Pointer string
#[cfg(feature = "json-pointer")]
pub use nodes::json_pointer::set as pointer_set;

// Advanced features (Phase 13)
/// JSON Schema validation
#[cfg(feature = "alloc")]
pub use nodes::schema;

/// JSON Patch (RFC 6902)
#[cfg(feature = "json-pointer")]
pub use nodes::patch;

/// JSON Merge Patch (RFC 7386)
#[cfg(feature = "alloc")]
pub use nodes::merge_patch;

/// JSON5 comment stripping
#[cfg(feature = "alloc")]
pub use parser::json5;
