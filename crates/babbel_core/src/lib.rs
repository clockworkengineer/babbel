//! # Babbel Core
//!
//! Foundational abstractions, streaming I/O, encoding detection, zero-allocation numeric formatting,
//! error diagnostic reporting, and universal data model for the Babbel data format family.
//!
//! Designed for high performance, standard library, and `no_std` / embedded targets.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod chars;
pub mod codec;
pub mod encoding;
pub mod error;
pub mod escape;
#[cfg(feature = "file-io")]
pub mod file;
pub mod io;
pub mod model;
pub mod num;

// Re-export key primitives for ergonomic downstream usage
pub use chars::{is_digit, is_hex_digit, is_newline, is_whitespace};
pub use codec::{
    BencodeEmitter, FormatCodec, FormatEmitter, FormatParser, JsonEmitter, XmlEmitter,
    YamlEmitter,
};
pub use encoding::{detect_encoding_and_strip_bom, normalize_newlines, Encoding};
pub use error::{format_error_snippet, BabbelError, ErrorCode, Location, Span};
pub use escape::{
    escape_for_json, escape_for_xml, json_needs_escaping, write_json_escaped_string,
    write_xml_escaped_string,
};
#[cfg(feature = "file-io")]
pub use file::{detect_format, read_file_to_string, write_file_from_string, Format};
pub use io::{
    Buffer, BufferDestination, BufferSource, ByteSliceSource, IByteStream, ICharStream,
    IClearable, IDestination, IIndentationAware, IPositionAware, IRewindable, ISource,
    SliceSource, StringDestination, StringSource,
};
#[cfg(feature = "file-io")]
pub use io::{FileDestination, FileSource};
pub use model::{FormatVisitor, Value};
pub use num::{format_float, format_integer};

