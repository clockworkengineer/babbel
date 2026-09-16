//! # Babbel Core
//!
//! Foundational abstractions, streaming I/O, encoding detection, zero-allocation numeric formatting,
//! error diagnostic reporting, and universal data model for the Babbel data format family.
//!
//! Designed for high performance, standard library, and `no_std` / embedded targets.
//!
//! ## Core Modules
//!
//! - **[`model::Value`]**: Universal AST supporting null, boolean, integer (i64/u64), float, string, bytes, array, and ordered map.
//! - **[`query`]**: RFC 9535 JSONPath query engine supporting root references, child property access, array indexing, slicing, filters, and recursive descent.
//! - **[`patch`]**: RFC 6902 JSON Patch (`add`, `remove`, `replace`, `move`, `copy`, `test`) and RFC 7396 JSON Merge Patch.
//! - **[`diff`]**: Structural AST diffing producing RFC 6902 and RFC 7396 patches.
//! - **[`schema`]**: JSON Schema validation (Draft 7 & 2020-12 core assertions, type checks, string formats, array & object constraints).
//! - **[`codec`]**: Format codecs, pull-parsers, and streaming emitters with format registry dispatch.
//! - **[`serde_impl`]**: Bidirectional serde interoperability via `to_value` and `from_value` (when `serde` feature enabled).

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::approx_constant)]

extern crate alloc;

pub mod chars;
pub mod codec;
pub mod csv;
pub mod diff;
pub mod embedded;
pub mod emitters;
pub mod encoding;
pub mod error;
pub mod escape;
#[cfg(feature = "file-io")]
pub mod file;
pub mod ini;
pub mod io;
pub mod model;
pub mod num;
pub mod patch;
pub mod query;
pub mod schema;
#[cfg(feature = "serde")]
pub mod serde_impl;
#[cfg(feature = "std")]
pub mod testing;
pub mod text;

// Re-export key primitives for ergonomic downstream usage
pub use chars::{is_digit, is_hex_digit, is_newline, is_whitespace};
pub use diff::{diff, diff_merge_patch};
pub use patch::{apply_merge_patch, Patch, PatchOp};
pub use query::{query as jsonpath_query, query_mut as jsonpath_query_mut, JsonPath};
pub use schema::{validate as validate_schema, CompiledSchema, SchemaValidationError};
pub use codec::{
    find_engine, find_engine_by_extension, find_engine_by_mime, BencodeEmitter, CsvEngine,
    FormatCodec, FormatEmitter, FormatEngine, FormatOptions, FormatParser, IniEngine, JsonEmitter,
    TomlEmitter, TsvEngine, XmlEmitter, YamlEmitter,
};
#[cfg(feature = "alloc")]
pub use codec::FormatRegistry;
pub use csv::{
    emit_csv, emit_csv_to, parse_csv, sniff_delimiter, CsvFieldsIter, CsvOptions, CsvPullParser,
    CsvRecord,
};
pub use embedded::{CompactError, EmbeddedLimits, MemoryTracker, StackBuffer};
pub use encoding::{detect_encoding_and_strip_bom, normalize_newlines, Encoding};
pub use error::{format_error_snippet, BabbelError, ErrorCode, Location, Span};
pub use escape::{
    escape_for_json, escape_for_toml, escape_for_xml, escape_for_yaml, is_valid_toml_bare_key,
    json_needs_escaping, write_json_escaped_string, write_toml_escaped_string,
    write_xml_escaped_string, write_yaml_escaped_string, yaml_needs_quoting,
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
    decode_hex, dedent, encode_hex, encode_hex_upper, indent, line_count, split_frontmatter,
    trim_lines, DocumentWithFrontmatter, FrontmatterFormat,
};
#[cfg(feature = "serde")]
pub use serde_impl::{from_value, to_value, SerdeError};


