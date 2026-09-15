//! Dedicated format emitter implementations adhering to SRP and OCP.
//!
//! Separates formatting and serialization logic from the universal `Value` AST model.

pub mod bencode;
pub mod json;
pub mod toml;
pub mod xml;
pub mod yaml;

pub use bencode::BencodeEmitter;
pub use json::JsonEmitter;
pub use toml::TomlEmitter;
pub use xml::XmlEmitter;
pub use yaml::YamlEmitter;
