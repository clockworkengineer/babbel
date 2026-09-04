//! Universal Unicode text file formats, Byte Order Mark (BOM) detection, and file I/O.
//!
//! Provides utilities for reading and writing Unicode text files across UTF-8, UTF-16, and UTF-32
//! in both little-endian and big-endian variants with automatic BOM handling.

use std::fs::File;
use std::io::{Read, Result, Write};

/// Represents different Unicode text file formats with their corresponding byte order marks (BOM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// UTF-8 without BOM
    Utf8,
    /// UTF-8 with BOM (EF BB BF)
    Utf8bom,
    /// UTF-16 Little Endian (FF FE)
    Utf16le,
    /// UTF-16 Big Endian (FE FF)
    Utf16be,
    /// UTF-32 Little Endian (FF FE 00 00)
    Utf32le,
    /// UTF-32 Big Endian (00 00 FE FF)
    Utf32be,
}

impl Format {
    /// Returns the byte order mark (BOM) bytes for each format.
    pub fn get_bom(&self) -> &'static [u8] {
        match self {
            Format::Utf8 => &[],
            Format::Utf8bom => &[0xEF, 0xBB, 0xBF],
            Format::Utf16le => &[0xFF, 0xFE],
            Format::Utf16be => &[0xFE, 0xFF],
            Format::Utf32le => &[0xFF, 0xFE, 0x00, 0x00],
            Format::Utf32be => &[0x00, 0x00, 0xFE, 0xFF],
        }
    }
}

/// Detects the Unicode format of a text file by examining its byte order mark (BOM).
pub fn detect_format(filename: &str) -> Result<Format> {
    let mut file = File::open(filename)?;
    let mut bom_buffer = [0u8; 4];
    let bytes_read = file.read(&mut bom_buffer)?;

    let format = match &bom_buffer[..bytes_read] {
        [0xEF, 0xBB, 0xBF, ..] => Format::Utf8bom,
        [0xFE, 0xFF, ..] => Format::Utf16be,
        [0xFF, 0xFE, 0x00, 0x00] => Format::Utf32le,
        [0x00, 0x00, 0xFE, 0xFF] => Format::Utf32be,
        [0xFF, 0xFE, ..] => Format::Utf16le,
        _ => Format::Utf8,
    };

    Ok(format)
}

/// Writes a string to a file in the specified Unicode format, prepending the corresponding BOM.
pub fn write_file_from_string(filename: &str, content: &str, format: Format) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(format.get_bom())?;

    match format {
        Format::Utf8 | Format::Utf8bom => {
            file.write_all(content.as_bytes())?;
        }
        Format::Utf16le => {
            for c in content.encode_utf16() {
                file.write_all(&c.to_le_bytes())?;
            }
        }
        Format::Utf16be => {
            for c in content.encode_utf16() {
                file.write_all(&c.to_be_bytes())?;
            }
        }
        Format::Utf32le => {
            for c in content.chars() {
                file.write_all(&(c as u32).to_le_bytes())?;
            }
        }
        Format::Utf32be => {
            for c in content.chars() {
                file.write_all(&(c as u32).to_be_bytes())?;
            }
        }
    }
    Ok(())
}

/// Reads a text file and returns its content as a normalized UTF-8 String, stripping BOM if present.
pub fn read_file_to_string(filename: &str) -> Result<String> {
    let mut content = String::new();
    let format = detect_format(filename)?;
    let mut file = File::open(filename)?;

    /// Helper function to read and skip over the BOM bytes
    fn read_and_skip_bom(file: &mut File, size: usize) -> Result<()> {
        let mut buf = vec![0u8; size];
        file.read_exact(&mut buf)
    }

    /// Helper function to process UTF-16 encoded files
    fn process_utf16(file: &mut File, is_be: bool) -> Result<String> {
        read_and_skip_bom(file, 2)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let content = String::from_utf16(
            &bytes
                .chunks(2)
                .map(|chunk| {
                    if is_be {
                        u16::from_be_bytes([chunk[0], chunk[1]])
                    } else {
                        u16::from_le_bytes([chunk[0], chunk[1]])
                    }
                })
                .collect::<Vec<u16>>(),
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(content.replace("\r\n", "\n"))
    }

    /// Helper function to process UTF-32 encoded files
    fn process_utf32(file: &mut File, is_be: bool) -> Result<String> {
        read_and_skip_bom(file, 4)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let content = bytes
            .chunks(4)
            .map(|chunk| {
                if is_be {
                    u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                } else {
                    u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                }
            })
            .map(|cp| char::from_u32(cp).unwrap_or('\u{FFFD}'))
            .collect::<String>();

        Ok(content.replace("\r\n", "\n"))
    }

    match format {
        Format::Utf8bom => {
            read_and_skip_bom(&mut file, 3)?;
            file.read_to_string(&mut content)?;
        }
        Format::Utf16be => return process_utf16(&mut file, true),
        Format::Utf16le => return process_utf16(&mut file, false),
        Format::Utf32be => return process_utf32(&mut file, true),
        Format::Utf32le => return process_utf32(&mut file, false),
        Format::Utf8 => {
            file.read_to_string(&mut content)?;
        }
    }

    Ok(content.replace("\r\n", "\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_file(filename: &str, bom: &[u8]) -> Result<()> {
        let mut file = File::create(filename)?;
        file.write_all(bom)?;
        file.write_all(b"test content")?;
        Ok(())
    }

    #[test]
    fn test_utf8() -> Result<()> {
        let fname = "test_core_utf8.txt";
        create_test_file(fname, &[])?;
        assert!(matches!(detect_format(fname)?, Format::Utf8));
        fs::remove_file(fname)?;
        Ok(())
    }

    #[test]
    fn test_utf8_bom() -> Result<()> {
        let fname = "test_core_utf8_bom.txt";
        create_test_file(fname, &[0xEF, 0xBB, 0xBF])?;
        assert!(matches!(detect_format(fname)?, Format::Utf8bom));
        fs::remove_file(fname)?;
        Ok(())
    }

    #[test]
    fn test_utf16_le() -> Result<()> {
        let fname = "test_core_utf16le.txt";
        create_test_file(fname, &[0xFF, 0xFE])?;
        assert!(matches!(detect_format(fname)?, Format::Utf16le));
        fs::remove_file(fname)?;
        Ok(())
    }

    #[test]
    fn test_utf16_be() -> Result<()> {
        let fname = "test_core_utf16be.txt";
        create_test_file(fname, &[0xFE, 0xFF])?;
        assert!(matches!(detect_format(fname)?, Format::Utf16be));
        fs::remove_file(fname)?;
        Ok(())
    }

    #[test]
    fn test_utf32_le() -> Result<()> {
        let fname = "test_core_utf32le.txt";
        create_test_file(fname, &[0xFF, 0xFE, 0x00, 0x00])?;
        assert!(matches!(detect_format(fname)?, Format::Utf32le));
        fs::remove_file(fname)?;
        Ok(())
    }

    #[test]
    fn test_utf32_be() -> Result<()> {
        let fname = "test_core_utf32be.txt";
        create_test_file(fname, &[0x00, 0x00, 0xFE, 0xFF])?;
        assert!(matches!(detect_format(fname)?, Format::Utf32be));
        fs::remove_file(fname)?;
        Ok(())
    }

    #[test]
    fn test_roundtrip_formats() -> Result<()> {
        let test_content = "Hello, Babbel Universal Unicode!\nLine 2";
        let formats = [
            ("test_rt_utf8.txt", Format::Utf8),
            ("test_rt_utf8bom.txt", Format::Utf8bom),
            ("test_rt_utf16le.txt", Format::Utf16le),
            ("test_rt_utf16be.txt", Format::Utf16be),
            ("test_rt_utf32le.txt", Format::Utf32le),
            ("test_rt_utf32be.txt", Format::Utf32be),
        ];

        for (fname, fmt) in formats {
            write_file_from_string(fname, test_content, fmt)?;
            assert_eq!(detect_format(fname)?, fmt);
            assert_eq!(read_file_to_string(fname)?, test_content);
            fs::remove_file(fname)?;
        }
        Ok(())
    }
}
