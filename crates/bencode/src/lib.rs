//! A Rust library for encoding and decoding data in the Bencode format, commonly used in BitTorrent files.
//! This library provides functionality to parse, create, modify and serialize bencode data structures
//! with support for various formats including JSON, YAML and XML conversion.
//!
//! # Features
//!
//! - `std` (default): Enable standard library support
//! - Without `std`: Core bencode functionality works in `no_std` environments

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
pub(crate) use alloc::collections::BTreeMap as HashMap;

#[cfg(feature = "std")]
pub(crate) use std::collections::HashMap;

/// Centralized bencode protocol character and byte constants
pub(crate) mod constants;
/// Module defining custom error types and error handling functionality
pub mod error;
/// Module providing input/output operations for reading and writing bencode data
pub mod io;
/// Module containing utility functions and helper methods
pub mod misc;

/// Module containing configuration options for parsing and encoding
pub mod config;
/// Module providing memory management utilities for embedded systems
pub mod memory;
/// Module defining the core data structures for representing bencode nodes
pub mod nodes;
/// Module containing the parsing logic to decode bencode format into data structures
pub mod parser;
/// Module implementing serialization of data structures back to bencode format
pub mod stringify;

/// Integration tests module
mod integration_tests;

///
/// Bencode_lib API
///

/// Returns the current version of the bencode library
pub use misc::get_version as version;

/// Reads and parses a bencode-encoded file from disc (requires `std` feature)
#[cfg(feature = "std")]
pub use misc::read_bencode_file as read_file;

/// Writes bencode-encoded data to a file on disk (requires `std` feature)
#[cfg(feature = "std")]
pub use misc::write_bencode_file as write_file;

/// Destination implementation for writing bencode data to a memory buffer
pub use io::destinations::buffer::Buffer as BufferDestination;

/// Destination implementation for writing bencode data to a file (requires `std` feature)
#[cfg(feature = "std")]
pub use io::destinations::file::File as FileDestination;

/// Source implementation for reading bencode data from a memory buffer
pub use io::sources::buffer::Buffer as BufferSource;

/// Source implementation for reading bencode data from a file (requires `std` feature)
#[cfg(feature = "std")]
pub use io::sources::file::File as FileSource;

/// Core data structure representing a bencode node in the parsed tree
pub use nodes::node::Node;
pub use nodes::node::make_node;

/// Zero-copy borrowed node for embedded systems (no allocation)
pub use nodes::borrowed::BorrowedNode;

/// Type alias for fixed-size stack buffers with const generics
pub use nodes::fixed::FixedSizeBuffer;
/// Memory bounds calculator using const generics
pub use nodes::fixed::MemoryBounds;

/// Parses bencode data into a Node tree structure
pub use parser::default::parse;
/// Parses bencode data from a byte slice into a Node tree structure
pub use parser::default::parse_bytes;
/// Parses bencode data from a string into a Node tree structure
pub use parser::default::parse_str;

/// Zero-copy parser that returns borrowed nodes (no allocation)
pub use parser::borrowed::parse_borrowed;
/// Validates bencode data without building a node tree (minimal allocation)
pub use parser::borrowed::validate_bencode;

/// Parses bencode data from a byte slice using iterative parser
pub use parser::iterative::parse_bytes_iterative;
/// Iterative parser that avoids recursion (for deeply nested structures)
pub use parser::iterative::parse_iterative;
/// Parses bencode data from a string using iterative parser
pub use parser::iterative::parse_str_iterative;

/// Arena allocator for bump allocation from fixed buffers
pub use memory::Arena;
/// Memory usage tracker for embedded systems
pub use memory::MemoryTracker;
/// Stack-based fixed-size buffer
pub use memory::StackBuffer;

/// Lightweight error type for embedded systems (no heap allocation)
pub use error::embedded::BencodeError;

/// Encoder configuration options
pub use config::EncoderConfig;
/// Parser configuration options
pub use config::ParserConfig;

/// Converts a Node tree back to bencode format
pub use stringify::default::stringify;
/// Converts a Node tree to bencode format as bytes
pub use stringify::default::stringify_to_bytes;
/// Converts a Node tree to bencode format as a String
pub use stringify::default::stringify_to_string;

/// Core SOLID I/O abstractions
pub use io::traits::{BencodeRead, BencodeWrite, BufferedWrite, RewindableRead};

/// Schema validation and typed dictionary query extension trait
pub use nodes::query::NodeQueryExt;

/// Visitor pattern traits for format serialization
pub use stringify::visitor::{BencodeVisitable, BencodeVisitor};

/// Converts a Node tree to JSON format (requires "json" feature)
#[cfg(feature = "json")]
pub use stringify::json::stringify as to_json;

/// Converts a Node tree to TOML format (requires "toml" feature)
#[cfg(feature = "toml")]
pub use stringify::toml::stringify as to_toml;

/// Converts a Node tree to XML format (requires "xml" feature)
#[cfg(feature = "xml")]
pub use stringify::xml::stringify as to_xml;

/// Converts a Node tree to YAML format (requires "yaml" feature)
#[cfg(feature = "yaml")]
pub use stringify::yaml::stringify as to_yaml;

