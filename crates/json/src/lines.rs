//! Line-delimited JSON (JSON Lines / NDJSON / JSONL) streaming parser and emitter.
//!
//! JSON Lines is a text format where each line contains a valid single-line JSON value.
//! It is ideal for streaming large datasets, logs, and record-by-record processing.

use crate::nodes::node::Node;
use crate::parser::default::from_str;
use crate::stringify::default::stringify;
use babbel_core::io::{IDestination, ILineReader, SliceSource, StringDestination};

#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Configuration options for reading line-delimited JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonLinesConfig {
    /// Whether to skip empty lines or lines containing only whitespace (default: true).
    pub ignore_empty_lines: bool,
    /// Whether to ignore comment lines starting with `#` or `//` (default: true).
    pub ignore_comments: bool,
    /// Whether to trim whitespace before checking for comments/empty lines (default: true).
    pub trim_whitespace: bool,
}

impl Default for JsonLinesConfig {
    fn default() -> Self {
        Self {
            ignore_empty_lines: true,
            ignore_comments: true,
            trim_whitespace: true,
        }
    }
}

/// A streaming reader for Line-Delimited JSON (JSON Lines / NDJSON).
///
/// Wraps any stream implementing [`ILineReader`] and yields parsed [`Node`] instances.
pub struct JsonLinesReader<R> {
    reader: R,
    config: JsonLinesConfig,
    line_number: usize,
}

impl<R: ILineReader> JsonLinesReader<R> {
    /// Creates a new `JsonLinesReader` with default configuration.
    pub fn new(reader: R) -> Self {
        Self::with_config(reader, JsonLinesConfig::default())
    }

    /// Creates a new `JsonLinesReader` with custom configuration.
    pub fn with_config(reader: R, config: JsonLinesConfig) -> Self {
        Self {
            reader,
            config,
            line_number: 0,
        }
    }

    /// Returns the current 1-based line number in the input stream.
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Reads and parses the next JSON node from the line stream.
    ///
    /// Returns `None` when EOF is reached.
    pub fn next_node(&mut self) -> Option<Result<Node, String>> {
        while let Some(line) = self.reader.read_line() {
            self.line_number += 1;
            let check_slice = if self.config.trim_whitespace {
                line.trim()
            } else {
                &line[..]
            };

            if self.config.ignore_empty_lines && check_slice.is_empty() {
                continue;
            }

            if self.config.ignore_comments && (check_slice.starts_with('#') || check_slice.starts_with("//")) {
                continue;
            }

            return Some(from_str(check_slice));
        }

        None
    }
}

impl<R: ILineReader> Iterator for JsonLinesReader<R> {
    type Item = Result<Node, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_node()
    }
}

/// Parses an entire line-delimited JSON string into a vector of [`Node`]s.
///
/// # Examples
/// ```
/// use babbel_json::lines::parse_json_lines;
///
/// let data = "{\"id\": 1}\n{\"id\": 2}\n";
/// let records = parse_json_lines(data).unwrap();
/// assert_eq!(records.len(), 2);
/// ```
pub fn parse_json_lines(input: &str) -> Result<Vec<Node>, String> {
    let source = SliceSource::new(input);
    let reader = JsonLinesReader::new(source);
    let mut nodes = Vec::new();

    for res in reader {
        nodes.push(res?);
    }

    Ok(nodes)
}

/// Serializes a slice of [`Node`]s to a single JSON Lines string, one JSON record per line terminated by `\n`.
///
/// # Examples
/// ```
/// use babbel_json::lines::{to_json_lines, parse_json_lines};
/// use babbel_json::from_str;
///
/// let n1 = from_str("{\"a\": 1}").unwrap();
/// let n2 = from_str("{\"b\": 2}").unwrap();
/// let lines = to_json_lines(&[n1, n2]).unwrap();
/// assert_eq!(lines, "{\"a\":1}\n{\"b\":2}\n");
/// ```
pub fn to_json_lines(nodes: &[Node]) -> Result<String, String> {
    let mut dest = StringDestination::default();
    to_json_lines_stream(nodes, &mut dest)?;
    Ok(dest.into_string())
}

/// Serializes a slice of [`Node`]s to an [`IDestination`] sink as line-delimited JSON.
pub fn to_json_lines_stream(nodes: &[Node], destination: &mut dyn IDestination) -> Result<(), String> {
    for node in nodes {
        stringify(node, destination)?;
        destination.add_byte(b'\n');
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_lines_basic() {
        let input = "{\"name\": \"Alice\", \"age\": 30}\n{\"name\": \"Bob\", \"age\": 25}\n";
        let nodes = parse_json_lines(input).unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].get("name").and_then(|n| n.as_str()), Some("Alice"));
        assert_eq!(nodes[1].get("name").and_then(|n| n.as_str()), Some("Bob"));
    }

    #[test]
    fn test_parse_json_lines_with_comments_and_blanks() {
        let input = "# Header comments\n\n{\"id\": 1}\n// Inline comment\n   \n{\"id\": 2}\n";
        let nodes = parse_json_lines(input).unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_to_json_lines_roundtrip() {
        let n1 = from_str("{\"x\":10}").unwrap();
        let n2 = from_str("{\"y\":20}").unwrap();
        let serialized = to_json_lines(&[n1, n2]).unwrap();
        assert_eq!(serialized, "{\"x\":10}\n{\"y\":20}\n");

        let parsed = parse_json_lines(&serialized).unwrap();
        assert_eq!(parsed.len(), 2);
    }
}
