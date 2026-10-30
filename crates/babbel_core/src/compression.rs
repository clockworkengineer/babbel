//! Zero-dependency, `no_std` + `alloc` compression codecs (Snappy, LZ4 Raw) and CRC32.
//!
//! Used by `babbel_parquet` (`SNAPPY`, `LZ4_RAW` column pages) and `babbel_avro` (`snappy` OCF blocks).

use crate::error::BabbelError;
use alloc::vec::Vec;

/// Computes the IEEE 802.3 CRC-32 checksum of `data`.
pub fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Decompresses a raw Snappy-compressed byte slice.
pub fn snappy_decompress(input: &[u8]) -> Result<Vec<u8>, BabbelError> {
    let mut pos = 0usize;
    let uncompressed_len = read_varint32(input, &mut pos)?;
    if uncompressed_len > 256 * 1024 * 1024 {
        return Err(BabbelError::syntax(
            "Snappy uncompressed length exceeds safety limit",
        ));
    }
    let mut out = Vec::with_capacity((uncompressed_len as usize).min(65536));

    while pos < input.len() {
        let tag = input[pos];
        pos += 1;
        let elem_type = tag & 0x03;

        if elem_type == 0 {
            // Literal
            let len_code = (tag >> 2) as usize;
            let lit_len = if len_code < 60 {
                len_code + 1
            } else {
                let extra_bytes = len_code - 59;
                if pos + extra_bytes > input.len() {
                    return Err(BabbelError::syntax(
                        "Snappy truncated literal length header",
                    ));
                }
                let mut len_minus_1 = 0usize;
                for i in 0..extra_bytes {
                    len_minus_1 |= (input[pos + i] as usize) << (i * 8);
                }
                pos += extra_bytes;
                len_minus_1 + 1
            };

            if pos + lit_len > input.len() {
                return Err(BabbelError::syntax("Snappy literal extends past input"));
            }
            out.extend_from_slice(&input[pos..pos + lit_len]);
            pos += lit_len;
        } else {
            // Copy back-reference
            let (length, offset) = match elem_type {
                1 => {
                    if pos >= input.len() {
                        return Err(BabbelError::syntax("Snappy truncated 1-byte copy offset"));
                    }
                    let next = input[pos] as usize;
                    pos += 1;
                    let len = 4 + (((tag >> 2) & 0x07) as usize);
                    let off = (((tag >> 5) as usize) << 8) | next;
                    (len, off)
                }
                2 => {
                    if pos + 2 > input.len() {
                        return Err(BabbelError::syntax("Snappy truncated 2-byte copy offset"));
                    }
                    let off = u16::from_le_bytes([input[pos], input[pos + 1]]) as usize;
                    pos += 2;
                    let len = 1 + ((tag >> 2) as usize);
                    (len, off)
                }
                3 => {
                    if pos + 4 > input.len() {
                        return Err(BabbelError::syntax("Snappy truncated 4-byte copy offset"));
                    }
                    let off = u32::from_le_bytes([
                        input[pos],
                        input[pos + 1],
                        input[pos + 2],
                        input[pos + 3],
                    ]) as usize;
                    pos += 4;
                    let len = 1 + ((tag >> 2) as usize);
                    (len, off)
                }
                _ => unreachable!(),
            };

            if offset == 0 || offset > out.len() {
                return Err(BabbelError::syntax(
                    "Snappy invalid copy back-reference offset",
                ));
            }

            copy_backref(&mut out, offset, length);
        }
    }

    if out.len() != uncompressed_len as usize {
        return Err(BabbelError::syntax(
            "Snappy decompressed length does not match header varint",
        ));
    }

    Ok(out)
}

/// Compresses `input` into a Snappy-compressed byte vector.
pub fn snappy_compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + 32);
    write_varint32(&mut out, input.len() as u32);

    if input.len() < 15 {
        emit_snappy_literal(&mut out, input);
        return out;
    }

    // 4096-entry hash table mapping 4-byte prefixes to positions
    let mut table = [0usize; 4096];
    let mut next_emit = 0usize;
    let mut ip = 1usize;
    let ip_limit = input.len() - 4;

    table[hash4(&input[0..4])] = 0;

    while ip <= ip_limit {
        let h = hash4(&input[ip..ip + 4]);
        let candidate = table[h];
        table[h] = ip;

        let offset = ip.saturating_sub(candidate);
        if offset > 0 && offset <= 65535 && input[candidate..candidate + 4] == input[ip..ip + 4] {
            // Emit pending literals before match
            if next_emit < ip {
                emit_snappy_literal(&mut out, &input[next_emit..ip]);
            }

            // Extend match
            let mut match_len = 4usize;
            while ip + match_len < input.len()
                && input[candidate + match_len] == input[ip + match_len]
            {
                match_len += 1;
            }

            emit_snappy_copy(&mut out, offset, match_len);
            ip += match_len;
            next_emit = ip;
            if ip <= ip_limit {
                table[hash4(&input[ip - 1..ip + 3])] = ip - 1;
            }
        } else {
            ip += 1;
        }
    }

    if next_emit < input.len() {
        emit_snappy_literal(&mut out, &input[next_emit..]);
    }

    out
}

fn emit_snappy_literal(out: &mut Vec<u8>, lit: &[u8]) {
    if lit.is_empty() {
        return;
    }
    let n = lit.len() - 1;
    if n < 60 {
        out.push((n as u8) << 2);
    } else if n < 256 {
        out.push(60 << 2);
        out.push(n as u8);
    } else if n < 65536 {
        out.push(61 << 2);
        out.extend_from_slice(&(n as u16).to_le_bytes());
    } else if n < (1 << 24) {
        out.push(62 << 2);
        out.push((n & 0xFF) as u8);
        out.push(((n >> 8) & 0xFF) as u8);
        out.push(((n >> 16) & 0xFF) as u8);
    } else {
        out.push(63 << 2);
        out.extend_from_slice(&(n as u32).to_le_bytes());
    }
    out.extend_from_slice(lit);
}

fn emit_snappy_copy(out: &mut Vec<u8>, offset: usize, mut len: usize) {
    while len >= 68 {
        out.push((63 << 2) | 2);
        out.extend_from_slice(&(offset as u16).to_le_bytes());
        len -= 64;
    }
    if len > 64 {
        out.push((59 << 2) | 2);
        out.extend_from_slice(&(offset as u16).to_le_bytes());
        len -= 60;
    }
    if (4..=11).contains(&len) && offset < 2048 {
        let len_bits = ((len - 4) as u8) << 2;
        let off_hi = ((offset >> 8) as u8) << 5;
        out.push(off_hi | len_bits | 1);
        out.push((offset & 0xFF) as u8);
    } else {
        out.push((((len - 1) as u8) << 2) | 2);
        out.extend_from_slice(&(offset as u16).to_le_bytes());
    }
}

/// Decompresses a raw LZ4 block (`LZ4_RAW`) into a byte vector.
pub fn lz4_decompress(input: &[u8], max_uncompressed_len: usize) -> Result<Vec<u8>, BabbelError> {
    let mut out = Vec::with_capacity(max_uncompressed_len.min(65536));
    let mut pos = 0usize;

    while pos < input.len() {
        let token = input[pos];
        pos += 1;

        // Literal length
        let mut lit_len = (token >> 4) as usize;
        if lit_len == 15 {
            loop {
                if pos >= input.len() {
                    return Err(BabbelError::syntax("LZ4 truncated literal length"));
                }
                let extra = input[pos] as usize;
                pos += 1;
                lit_len += extra;
                if extra != 255 {
                    break;
                }
            }
        }

        if pos + lit_len > input.len() {
            return Err(BabbelError::syntax("LZ4 literal extends past input block"));
        }
        out.extend_from_slice(&input[pos..pos + lit_len]);
        pos += lit_len;

        // End of block reached after last literal sequence
        if pos >= input.len() {
            break;
        }

        // 2-byte little-endian offset
        if pos + 2 > input.len() {
            return Err(BabbelError::syntax("LZ4 truncated match offset"));
        }
        let offset = u16::from_le_bytes([input[pos], input[pos + 1]]) as usize;
        pos += 2;
        if offset == 0 || offset > out.len() {
            return Err(BabbelError::syntax("LZ4 invalid match offset"));
        }

        // Match length
        let mut match_len = ((token & 0x0F) as usize) + 4;
        if (token & 0x0F) == 15 {
            loop {
                if pos >= input.len() {
                    return Err(BabbelError::syntax("LZ4 truncated match length"));
                }
                let extra = input[pos] as usize;
                pos += 1;
                match_len += extra;
                if extra != 255 {
                    break;
                }
            }
        }

        copy_backref(&mut out, offset, match_len);
    }

    Ok(out)
}

/// Compresses `input` into a raw LZ4 block (`LZ4_RAW`).
pub fn lz4_compress(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(input.len() + 16);

    if input.len() < 13 {
        emit_lz4_last_literals(&mut out, input);
        return out;
    }

    let mut table = [0usize; 4096];
    let mut anchor = 0usize;
    let mut ip = 1usize;
    let match_limit = input.len() - 5;
    let mflimit = input.len() - 12;

    table[hash4(&input[0..4])] = 0;

    while ip <= mflimit {
        let h = hash4(&input[ip..ip + 4]);
        let candidate = table[h];
        table[h] = ip;

        let offset = ip.saturating_sub(candidate);
        if offset > 0 && offset <= 65535 && input[candidate..candidate + 4] == input[ip..ip + 4] {
            let lit_len = ip - anchor;
            let mut match_len = 4usize;
            while ip + match_len <= match_limit
                && input[candidate + match_len] == input[ip + match_len]
            {
                match_len += 1;
            }

            let lit_token = lit_len.min(15) as u8;
            let match_minus_4 = match_len - 4;
            let match_token = match_minus_4.min(15) as u8;
            out.push((lit_token << 4) | match_token);

            if lit_len >= 15 {
                write_lz4_length(&mut out, lit_len - 15);
            }
            out.extend_from_slice(&input[anchor..ip]);

            out.extend_from_slice(&(offset as u16).to_le_bytes());

            if match_minus_4 >= 15 {
                write_lz4_length(&mut out, match_minus_4 - 15);
            }

            ip += match_len;
            anchor = ip;
            if ip <= mflimit {
                table[hash4(&input[ip - 1..ip + 3])] = ip - 1;
            }
        } else {
            ip += 1;
        }
    }

    emit_lz4_last_literals(&mut out, &input[anchor..]);
    out
}

fn emit_lz4_last_literals(out: &mut Vec<u8>, literals: &[u8]) {
    let lit_len = literals.len();
    let lit_token = lit_len.min(15) as u8;
    out.push(lit_token << 4);
    if lit_len >= 15 {
        write_lz4_length(out, lit_len - 15);
    }
    out.extend_from_slice(literals);
}

fn write_lz4_length(out: &mut Vec<u8>, mut remaining: usize) {
    while remaining >= 255 {
        out.push(255);
        remaining -= 255;
    }
    out.push(remaining as u8);
}

#[inline]
fn hash4(bytes: &[u8]) -> usize {
    let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    ((v.wrapping_mul(0x1E35_A7BD)) >> 20) as usize
}

#[inline]
fn copy_backref(out: &mut Vec<u8>, offset: usize, length: usize) {
    if offset >= length {
        let start = out.len() - offset;
        out.extend_from_within(start..start + length);
    } else {
        for _ in 0..length {
            let b = out[out.len() - offset];
            out.push(b);
        }
    }
}

fn read_varint32(input: &[u8], pos: &mut usize) -> Result<u32, BabbelError> {
    let mut result = 0u32;
    let mut shift = 0u32;
    while *pos < input.len() {
        let byte = input[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 35 {
            return Err(BabbelError::syntax("Snappy varint32 overflow"));
        }
    }
    Err(BabbelError::syntax("Unexpected EOF reading Snappy varint"))
}

fn write_varint32(out: &mut Vec<u8>, mut val: u32) {
    while val >= 0x80 {
        out.push(((val as u8) & 0x7F) | 0x80);
        val >>= 7;
    }
    out.push(val as u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32_ieee_standard_vector() {
        assert_eq!(crc32_ieee(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32_ieee(b""), 0x0000_0000);
    }

    #[test]
    fn test_snappy_roundtrip() {
        let inputs: &[&[u8]] = &[
            b"",
            b"hello",
            b"abcabcabcabcabcabcabcabcabcabcabcabc",
            b"The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog!",
        ];
        for &inp in inputs {
            let compressed = snappy_compress(inp);
            let decompressed = snappy_decompress(&compressed).unwrap();
            assert_eq!(decompressed.as_slice(), inp);
        }
    }

    #[test]
    fn test_lz4_roundtrip() {
        let inputs: &[&[u8]] = &[
            b"",
            b"short",
            b"1234567890123456789012345678901234567890",
            b"Parquet columnar compression with LZ4_RAW block format repeating Parquet columnar compression!",
        ];
        for &inp in inputs {
            let compressed = lz4_compress(inp);
            let decompressed = lz4_decompress(&compressed, inp.len()).unwrap();
            assert_eq!(decompressed.as_slice(), inp);
        }
    }
}
