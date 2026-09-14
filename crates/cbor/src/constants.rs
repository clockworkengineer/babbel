//! CBOR (RFC 8949) binary format markers, masks, and constants.

pub const MAJOR_TYPE_MASK: u8 = 0xe0;
pub const ADDITIONAL_INFO_MASK: u8 = 0x1f;

pub const MAJOR_UNSIGNED_INT: u8 = 0x00; // Major 0
pub const MAJOR_NEGATIVE_INT: u8 = 0x20; // Major 1
pub const MAJOR_BYTE_STRING: u8 = 0x40;  // Major 2
pub const MAJOR_TEXT_STRING: u8 = 0x60;  // Major 3
pub const MAJOR_ARRAY: u8 = 0x80;        // Major 4
pub const MAJOR_MAP: u8 = 0xa0;          // Major 5
pub const MAJOR_TAG: u8 = 0xc0;          // Major 6
pub const MAJOR_SIMPLE: u8 = 0xe0;       // Major 7

pub const AI_1_BYTE: u8 = 24;
pub const AI_2_BYTES: u8 = 25;
pub const AI_4_BYTES: u8 = 26;
pub const AI_8_BYTES: u8 = 27;
pub const AI_INDEFINITE: u8 = 31;

// Major 7 Special Values
pub const SIMPLE_FALSE: u8 = 20;     // 0xf4
pub const SIMPLE_TRUE: u8 = 21;      // 0xf5
pub const SIMPLE_NULL: u8 = 22;      // 0xf6
pub const SIMPLE_UNDEFINED: u8 = 23; // 0xf7
pub const FLOAT_16: u8 = 25;         // 0xf9
pub const FLOAT_32: u8 = 26;         // 0xfa
pub const FLOAT_64: u8 = 27;         // 0xfb
pub const BREAK_CODE: u8 = 31;       // 0xff

pub const BYTE_FALSE: u8 = MAJOR_SIMPLE | SIMPLE_FALSE;
pub const BYTE_TRUE: u8 = MAJOR_SIMPLE | SIMPLE_TRUE;
pub const BYTE_NULL: u8 = MAJOR_SIMPLE | SIMPLE_NULL;
pub const BYTE_UNDEFINED: u8 = MAJOR_SIMPLE | SIMPLE_UNDEFINED;
pub const BYTE_FLOAT64: u8 = MAJOR_SIMPLE | FLOAT_64;
pub const BYTE_BREAK: u8 = MAJOR_SIMPLE | BREAK_CODE;

/// Default maximum recursion depth for parsing nested structures.
pub const DEFAULT_MAX_DEPTH: usize = 128;

/// Default maximum payload size (64 MB) to prevent resource exhaustion attacks.
pub const DEFAULT_MAX_SIZE: usize = 64 * 1024 * 1024;
