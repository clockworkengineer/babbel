//! Extensible format parser, emitter, and codec traits adhering to OCP and DIP.


use crate::error::BabbelError;
use crate::io::traits::IDestination;
use crate::model::Value;

/// Abstract parser converting text or byte inputs into a universal `Value` AST.
pub trait FormatParser {
    /// Parses a UTF-8 string into a universal `Value`.
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError>;

    /// Parses a raw byte slice into a universal `Value`.
    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        match core::str::from_utf8(input) {
            Ok(s) => self.parse_str(s),
            Err(_) => Err(BabbelError::encoding("input is not valid UTF-8")),
        }
    }
}

/// Abstract emitter writing a universal `Value` AST to an output destination.
pub trait FormatEmitter {
    /// Serializes a `Value` to the output destination in compact/standard format.
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError>;

    /// Serializes a `Value` to the output destination with pretty printing.
    fn emit_pretty(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        indent: usize,
    ) -> Result<(), BabbelError> {
        let _ = indent;
        self.emit(value, dest)
    }
}

/// A combined format codec providing both parsing and emission capabilities.
pub trait FormatCodec: FormatParser + FormatEmitter {
    /// Returns the name of the format (e.g., "json", "yaml", "xml", "bencode").
    fn format_name(&self) -> &'static str;
}

/// Standard built-in JSON format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct JsonEmitter;

impl FormatEmitter for JsonEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_json(dest);
        Ok(())
    }
}

/// Standard built-in YAML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct YamlEmitter;

impl FormatEmitter for YamlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_yaml(dest, 0);
        Ok(())
    }

    fn emit_pretty(
        &self,
        value: &Value,
        dest: &mut dyn IDestination,
        indent: usize,
    ) -> Result<(), BabbelError> {
        value.serialize_yaml(dest, indent);
        Ok(())
    }
}

/// Standard built-in XML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct XmlEmitter;

impl FormatEmitter for XmlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_xml(dest, None);
        Ok(())
    }
}

/// Standard built-in Bencode format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct BencodeEmitter;

impl FormatEmitter for BencodeEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_bencode(dest);
        Ok(())
    }
}

/// Standard built-in TOML format emitter delegating to universal Value serialization.
#[derive(Debug, Default, Clone, Copy)]
pub struct TomlEmitter;

impl FormatEmitter for TomlEmitter {
    fn emit(&self, value: &Value, dest: &mut dyn IDestination) -> Result<(), BabbelError> {
        value.serialize_toml(dest);
        Ok(())
    }
}

/// Universal format serialization options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    /// Whether to format output with indentation and line breaks.
    pub pretty: bool,
    /// Number of spaces for pretty indentation.
    pub indent: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            indent: 2,
        }
    }
}

impl FormatOptions {
    /// Create compact format options (no extra whitespace).
    pub const fn compact() -> Self {
        Self {
            pretty: false,
            indent: 0,
        }
    }

    /// Create pretty format options with default 2-space indentation.
    pub const fn pretty() -> Self {
        Self {
            pretty: true,
            indent: 2,
        }
    }

    /// Set custom indentation spaces (enables pretty formatting).
    pub const fn with_indent(mut self, indent: usize) -> Self {
        self.pretty = true;
        self.indent = indent;
        self
    }
}

/// Abstract format engine adhering to the Open-Closed (OCP) and Dependency Inversion (DIP) principles.
///
/// A format engine provides bidirectional transformation between raw streams/text
/// and the universal `Value` AST, without requiring callers to couple to concrete format implementations.
pub trait FormatEngine: Send + Sync {
    /// Unique format identifier (e.g., "json", "yaml", "toml", "xml", "bencode").
    fn format_id(&self) -> &'static str;

    /// Canonical MIME type (e.g., "application/json").
    fn mime_type(&self) -> &'static str;

    /// Associated file extensions without leading dots (e.g., &["json", "jsonl"]).
    fn file_extensions(&self) -> &'static [&'static str];

    /// Parse raw stream into universal Value AST.
    fn parse(&self, source: &mut dyn crate::io::traits::ISource) -> Result<Value, BabbelError>;

    /// Convenient parsing from UTF-8 text string.
    fn parse_str(&self, input: &str) -> Result<Value, BabbelError> {
        let mut src = crate::io::sources::BufferSource::new(input.as_bytes());
        self.parse(&mut src)
    }

    /// Convenient parsing from raw byte slice.
    fn parse_bytes(&self, input: &[u8]) -> Result<Value, BabbelError> {
        let mut src = crate::io::sources::BufferSource::new(input);
        self.parse(&mut src)
    }

    /// Serialize universal Value AST into output destination with format options.
    fn serialize(
        &self,
        value: &Value,
        destination: &mut dyn crate::io::traits::IDestination,
        options: &FormatOptions,
    ) -> Result<(), BabbelError>;

    /// Convenient serialization to String.
    #[cfg(feature = "alloc")]
    fn serialize_to_string(
        &self,
        value: &Value,
        options: &FormatOptions,
    ) -> Result<alloc::string::String, BabbelError> {
        let mut dest = crate::io::destinations::BufferDestination::new();
        self.serialize(value, &mut dest, options)?;
        dest.into_string()
            .map_err(|_| BabbelError::encoding("output is not valid UTF-8"))
    }

    /// Convenient serialization to bytes.
    #[cfg(feature = "alloc")]
    fn serialize_to_vec(
        &self,
        value: &Value,
        options: &FormatOptions,
    ) -> Result<alloc::vec::Vec<u8>, BabbelError> {
        let mut dest = crate::io::destinations::BufferDestination::new();
        self.serialize(value, &mut dest, options)?;
        Ok(dest.into_vec())
    }
}

/// Helper function to find an engine from a static slice by format ID.
pub fn find_engine<'a>(engines: &'a [&'a dyn FormatEngine], id: &str) -> Option<&'a dyn FormatEngine> {
    engines.iter().copied().find(|e| e.format_id().eq_ignore_ascii_case(id))
}

/// Helper function to find an engine from a static slice by file extension.
pub fn find_engine_by_extension<'a>(engines: &'a [&'a dyn FormatEngine], ext: &str) -> Option<&'a dyn FormatEngine> {
    let clean_ext = ext.strip_prefix('.').unwrap_or(ext);
    engines.iter().copied().find(|e| {
        e.file_extensions().iter().any(|fe| fe.eq_ignore_ascii_case(clean_ext))
    })
}

/// Helper function to find an engine from a static slice by MIME type.
pub fn find_engine_by_mime<'a>(engines: &'a [&'a dyn FormatEngine], mime: &str) -> Option<&'a dyn FormatEngine> {
    engines.iter().copied().find(|e| e.mime_type().eq_ignore_ascii_case(mime))
}

/// Dynamic format engine registry supporting lookup by format ID, MIME type, or file extension.
#[cfg(feature = "alloc")]
pub struct FormatRegistry {
    engines: alloc::vec::Vec<alloc::sync::Arc<dyn FormatEngine>>,
}

#[cfg(feature = "alloc")]
impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl FormatRegistry {
    /// Creates a new empty format registry.
    pub fn new() -> Self {
        Self {
            engines: alloc::vec::Vec::new(),
        }
    }

    /// Registers a new format engine.
    pub fn register(&mut self, engine: alloc::sync::Arc<dyn FormatEngine>) {
        self.engines.push(engine);
    }

    /// Finds a format engine by format identifier (e.g. "json", "yaml").
    pub fn get_by_id(&self, id: &str) -> Option<alloc::sync::Arc<dyn FormatEngine>> {
        self.engines.iter().find(|e| e.format_id().eq_ignore_ascii_case(id)).cloned()
    }

    /// Finds a format engine by MIME type (e.g. "application/json").
    pub fn get_by_mime(&self, mime: &str) -> Option<alloc::sync::Arc<dyn FormatEngine>> {
        self.engines.iter().find(|e| e.mime_type().eq_ignore_ascii_case(mime)).cloned()
    }

    /// Finds a format engine by file extension (e.g. "json", ".json", "yaml", "yml").
    pub fn get_by_extension(&self, ext: &str) -> Option<alloc::sync::Arc<dyn FormatEngine>> {
        let clean_ext = ext.strip_prefix('.').unwrap_or(ext);
        self.engines.iter().find(|e| {
            e.file_extensions().iter().any(|fe| fe.eq_ignore_ascii_case(clean_ext))
        }).cloned()
    }

    /// Returns a list of all registered format identifiers.
    pub fn available_formats(&self) -> alloc::vec::Vec<&'static str> {
        self.engines.iter().map(|e| e.format_id()).collect()
    }
}

