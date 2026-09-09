//! TOML serialization and formatting modules.

pub mod default;
pub mod pretty;

pub use default::emit_to;
pub use pretty::{emit_pretty_to, PrettyOptions};
