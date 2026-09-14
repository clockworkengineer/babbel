//! BSON (Binary JSON) element types, subtypes, and constants.

pub const TYPE_DOUBLE: u8 = 0x01;
pub const TYPE_STRING: u8 = 0x02;
pub const TYPE_DOCUMENT: u8 = 0x03;
pub const TYPE_ARRAY: u8 = 0x04;
pub const TYPE_BINARY: u8 = 0x05;
pub const TYPE_UNDEFINED: u8 = 0x06;
pub const TYPE_OBJECT_ID: u8 = 0x07;
pub const TYPE_BOOLEAN: u8 = 0x08;
pub const TYPE_DATETIME: u8 = 0x09;
pub const TYPE_NULL: u8 = 0x0A;
pub const TYPE_REGEX: u8 = 0x0B;
pub const TYPE_DB_POINTER: u8 = 0x0C;
pub const TYPE_JS_CODE: u8 = 0x0D;
pub const TYPE_SYMBOL: u8 = 0x0E;
pub const TYPE_JS_CODE_SCOPE: u8 = 0x0F;
pub const TYPE_INT32: u8 = 0x10;
pub const TYPE_TIMESTAMP: u8 = 0x11;
pub const TYPE_INT64: u8 = 0x12;
pub const TYPE_DECIMAL128: u8 = 0x13;
pub const TYPE_MIN_KEY: u8 = 0xFF;
pub const TYPE_MAX_KEY: u8 = 0x7F;

// Binary Subtypes
pub const BINARY_GENERIC: u8 = 0x00;
pub const BINARY_FUNCTION: u8 = 0x01;
pub const BINARY_OLD: u8 = 0x02;
pub const BINARY_UUID_OLD: u8 = 0x03;
pub const BINARY_UUID: u8 = 0x04;
pub const BINARY_MD5: u8 = 0x05;
pub const BINARY_USER_DEFINED: u8 = 0x80;

/// Default maximum recursion depth for parsing nested documents.
pub const DEFAULT_MAX_DEPTH: usize = 128;

/// Default maximum document payload size (64 MB) to prevent resource exhaustion attacks.
pub const DEFAULT_MAX_SIZE: usize = 64 * 1024 * 1024;
