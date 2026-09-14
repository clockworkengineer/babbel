//! MessagePack binary format markers and constants.
//!
//! Defined in accordance with the official MessagePack specification.

pub const POS_FIXINT_MASK: u8 = 0x80;
pub const POS_FIXINT_PREFIX: u8 = 0x00;

pub const FIXMAP_MASK: u8 = 0xf0;
pub const FIXMAP_PREFIX: u8 = 0x80;

pub const FIXARRAY_MASK: u8 = 0xf0;
pub const FIXARRAY_PREFIX: u8 = 0x90;

pub const FIXSTR_MASK: u8 = 0xe0;
pub const FIXSTR_PREFIX: u8 = 0xa0;

pub const NIL: u8 = 0xc0;
pub const NEVER_USED: u8 = 0xc1;
pub const FALSE: u8 = 0xc2;
pub const TRUE: u8 = 0xc3;

pub const BIN8: u8 = 0xc4;
pub const BIN16: u8 = 0xc5;
pub const BIN32: u8 = 0xc6;

pub const EXT8: u8 = 0xc7;
pub const EXT16: u8 = 0xc8;
pub const EXT32: u8 = 0xc9;

pub const FLOAT32: u8 = 0xca;
pub const FLOAT64: u8 = 0xcb;

pub const UINT8: u8 = 0xcc;
pub const UINT16: u8 = 0xcd;
pub const UINT32: u8 = 0xce;
pub const UINT64: u8 = 0xcf;

pub const INT8: u8 = 0xd0;
pub const INT16: u8 = 0xd1;
pub const INT32: u8 = 0xd2;
pub const INT64: u8 = 0xd3;

pub const FIXEXT1: u8 = 0xd4;
pub const FIXEXT2: u8 = 0xd5;
pub const FIXEXT4: u8 = 0xd6;
pub const FIXEXT8: u8 = 0xd7;
pub const FIXEXT16: u8 = 0xd8;

pub const STR8: u8 = 0xd9;
pub const STR16: u8 = 0xda;
pub const STR32: u8 = 0xdb;

pub const ARRAY16: u8 = 0xdc;
pub const ARRAY32: u8 = 0xdd;

pub const MAP16: u8 = 0xde;
pub const MAP32: u8 = 0xdf;

pub const NEG_FIXINT_MASK: u8 = 0xe0;
pub const NEG_FIXINT_PREFIX: u8 = 0xe0;

/// Default maximum recursion depth for parsing nested structures.
pub const DEFAULT_MAX_DEPTH: usize = 128;

/// Default maximum payload size (64 MB) to prevent resource exhaustion attacks.
pub const DEFAULT_MAX_SIZE: usize = 64 * 1024 * 1024;
