/// Shared string constants and escape utilities for JSON serialization
pub mod escape;

/// FormatEncoder and Serializer trait abstractions
pub mod traits;

/// Module for default JSON string formatting and serialization
pub mod default;
pub mod pretty;

/// Optimized stringify with lazy escaping
#[cfg(feature = "alloc")]
pub mod optimized;

/// Module for Bencode format serialization and string conversion
#[cfg(feature = "format-bencode")]
pub mod bencode;

/// Module for YAML format serialization and string conversion
#[cfg(feature = "format-yaml")]
pub mod yaml;

/// Module for XML format serialization and string conversion
#[cfg(feature = "format-xml")]
pub mod xml;

/// Module for TOML format serialization and string conversion
#[cfg(feature = "format-toml")]
pub mod toml;
