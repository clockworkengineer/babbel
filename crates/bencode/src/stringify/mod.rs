/// Module for converting bencode data structures into various string formats.
/// Provides different formatting options for serializing bencode data.
pub mod default;

/// Module for converting bencode data structures into JSON format.
/// Enables interoperability with JSON-based systems and tools.
#[cfg(feature = "json")]
pub mod json;

/// Module for converting bencode data structures into YAML format.
/// Provides human-readable representation of bencode data.
#[cfg(feature = "yaml")]
pub mod yaml;

/// Module for converting bencode data structures into XML format.
/// Enables integration with XML-based systems and tools.
#[cfg(feature = "xml")]
pub mod xml;

/// Module for converting bencode data structures into TOML format.
/// Enables representation of bencode data in a human-readable configuration format.
#[cfg(feature = "toml")]
pub mod toml;

/// Visitor pattern interface for format serialization
pub mod visitor;

mod common;

