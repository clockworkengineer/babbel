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
pub mod csv;
pub mod embedded;
pub mod encoding;
pub mod error;
pub mod escape;
#[cfg(feature = "file-io")]
pub mod file;
pub mod ini;
pub mod io;
pub mod model;
pub mod num;
pub mod text;

// Re-export key primitives for ergonomic downstream usage
pub use chars::{is_digit, is_hex_digit, is_newline, is_whitespace};
pub use codec::{
    BencodeEmitter, FormatCodec, FormatEmitter, FormatParser, JsonEmitter, TomlEmitter, XmlEmitter,
    YamlEmitter,
};
pub use csv::{
    emit_csv, emit_csv_to, parse_csv, sniff_delimiter, CsvFieldsIter, CsvOptions, CsvPullParser,
    CsvRecord,
};
pub use embedded::{CompactError, EmbeddedLimits, MemoryTracker, StackBuffer};
pub use encoding::{detect_encoding_and_strip_bom, normalize_newlines, Encoding};
pub use error::{format_error_snippet, BabbelError, ErrorCode, Location, Span};
pub use escape::{
    escape_for_json, escape_for_toml, escape_for_xml, is_valid_toml_bare_key, json_needs_escaping,
    write_json_escaped_string, write_toml_escaped_string, write_xml_escaped_string,
};
#[cfg(feature = "file-io")]
pub use file::{
    detect_format, list_files_by_extension, read_file_to_string, write_file_from_string, Format,
};
pub use ini::{emit_ini, emit_ini_to, parse_ini, IniEvent, IniOptions, IniPullParser};
pub use io::{
    ArrayVecDestination, Buffer, BufferDestination, BufferSource, ByteSliceSource,
    ByteSourceAdapter, IByteReader, IByteStream, ICharStream, IClearable, IDestination,
    IIndentationAware, ILineReader, ILocationAware, IPeekable, IPositionAware, IRewindable,
    ISource, ITracked, LineIter, SliceDestination, SliceSource, StringDestination, StringSource,
};
#[cfg(feature = "file-io")]
pub use io::{FileDestination, FileSource};
pub use model::{FormatVisitor, Value};
pub use num::{format_float, format_integer, Numeric};
pub use text::{
    dedent, indent, line_count, split_frontmatter, trim_lines, DocumentWithFrontmatter,
    FrontmatterFormat,
};

