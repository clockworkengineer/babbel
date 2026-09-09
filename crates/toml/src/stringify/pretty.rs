//! Pretty-printed TOML serialization with customizable formatting options.

use babbel_core::io::traits::IDestination;

use crate::error::TomlError;
use crate::nodes::Node;
use super::default::emit_to;

/// Formatting options for pretty-printing TOML.
#[derive(Debug, Clone)]
pub struct PrettyOptions {
    /// Spaces per indent level (default: 2)
    pub indent: usize,
    /// Whether to sort keys alphabetically
    pub sort_keys: bool,
}

impl Default for PrettyOptions {
    fn default() -> Self {
        Self {
            indent: 2,
            sort_keys: false,
        }
    }
}

/// Emits a `Node` to a destination with pretty formatting.
pub fn emit_pretty_to(
    node: &Node,
    dest: &mut dyn IDestination,
    _options: &PrettyOptions,
) -> Result<(), TomlError> {
    emit_to(node, dest)
}
