//! Bencode protocol character and byte constants.
//!
//! All bencode format markers are defined here to ensure consistency
//! across parsers and stringifiers. Use the char variants with `ISource`-based
//! parsers and the byte variants (`BYTE_*`) with raw byte-slice parsers and
//! byte-level output destinations.

// ── Char constants (for ISource / char-based parsers) ────────────────────────

/// Integer start marker ('i'). Format: `i<digits>e`
pub(crate) const INTEGER_START: char = 'i';

/// Universal end marker ('e'). Terminates integers, lists, and dictionaries.
pub(crate) const END_MARKER: char = 'e';

/// List start marker ('l'). Format: `l<items>e`
pub(crate) const LIST_START: char = 'l';

/// Dictionary start marker ('d'). Format: `d<key><value>...e`
pub(crate) const DICT_START: char = 'd';

/// String length/content separator (':'). Format: `<length>:<bytes>`
pub(crate) const STRING_SEP: char = ':';

// ── Byte constants (for byte-slice parsers and byte-level output) ─────────────

/// Integer start marker byte (`b'i'`).
pub(crate) const BYTE_INTEGER_START: u8 = b'i';

/// Universal end marker byte (`b'e'`).
pub(crate) const BYTE_END: u8 = b'e';

/// List start marker byte (`b'l'`).
pub(crate) const BYTE_LIST_START: u8 = b'l';

/// Dictionary start marker byte (`b'd'`).
pub(crate) const BYTE_DICT_START: u8 = b'd';

/// String length/content separator byte (`b':'`).
pub(crate) const BYTE_STRING_SEP: u8 = b':';
