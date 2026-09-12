//! TOML serialization and formatting modules.

pub mod default;
pub mod pretty;
#[cfg(feature = "format-converters")]
pub mod converters;

pub use default::emit_to;
pub use pretty::{emit_pretty_to, PrettyOptions};
#[cfg(feature = "format-converters")]
pub use converters::{
    to_bencode, to_bencode_bytes, to_json, to_json_string, to_xml, to_xml_string, to_yaml,
    to_yaml_string,
};
