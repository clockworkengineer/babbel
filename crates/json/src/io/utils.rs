use crate::io::traits::{IDestination, ISource};

/// Strips whitespace from JSON while preserving string content.
/// Copies non-whitespace characters from source to destination, handling string literals specially.
pub fn strip(source: &mut dyn ISource, destination: &mut dyn IDestination) {
    while source.more() {
        if let Some(c) = source.current() {
            // Skip whitespace characters outside of strings
            if !(c == ' ' || c == '\t' || c == '\n' || c == '\r') {
                destination.add_byte(c as u8);

                // Handle string literals specially to preserve their whitespace
                if source.current() == Some('"') {
                    source.next();
                    // Copy characters until closing quote
                    while source.more() && source.current() != Some('"') {
                        // Handle escaped characters
                        if source.current() == Some('\\') {
                            destination.add_byte(b'\\');
                            source.next();
                        }
                        if let Some(sc) = source.current() {
                            destination.add_byte(sc as u8);
                        }
                        source.next();
                    }
                    destination.add_byte(b'"');
                }
            }
        }
        source.next();
    }
}
