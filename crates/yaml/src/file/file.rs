//! YAML File Format Utilities
//!
//! Re-exports universal Unicode text file formats, BOM detection, and file I/O from `babbel_core`.
//! Supports UTF-8, UTF-16, and UTF-32 in both little and big endian variants.
//!
//! Copyright (c) 2026 YAML Library Developers

pub use babbel_core::file::{
    detect_format, read_file_to_string, write_file_from_string, Format,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{Result, Write};

    fn test_path(filename: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(filename)
    }

    /// Creates a test file with the specified BOM and content
    fn create_test_file(path: &std::path::Path, bom: &[u8]) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(bom)?;
        file.write_all(b"test content")?;
        Ok(())
    }

    #[test]
    fn test_utf8() -> Result<()> {
        let p = test_path("test_yaml_utf8.txt");
        create_test_file(&p, &[])?;
        assert!(matches!(
            detect_format(&p)?,
            Format::Utf8
        ));
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_utf8_bom() -> Result<()> {
        let p = test_path("test_yaml_utf8_bom.txt");
        create_test_file(&p, &[0xEF, 0xBB, 0xBF])?;
        assert!(matches!(
            detect_format(&p)?,
            Format::Utf8bom
        ));
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_utf16_le() -> Result<()> {
        let p = test_path("test_yaml_utf16le.txt");
        create_test_file(&p, &[0xFF, 0xFE])?;
        assert!(matches!(
            detect_format(&p)?,
            Format::Utf16le
        ));
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_utf16_be() -> Result<()> {
        let p = test_path("test_yaml_utf16be.txt");
        create_test_file(&p, &[0xFE, 0xFF])?;
        assert!(matches!(
            detect_format(&p)?,
            Format::Utf16be
        ));
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_utf32_le() -> Result<()> {
        let p = test_path("test_yaml_utf32le.txt");
        create_test_file(&p, &[0xFF, 0xFE, 0x00, 0x00])?;
        assert!(matches!(
            detect_format(&p)?,
            Format::Utf32le
        ));
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_utf32_be() -> Result<()> {
        let p = test_path("test_yaml_utf32be.txt");
        create_test_file(&p, &[0x00, 0x00, 0xFE, 0xFF])?;
        assert!(matches!(
            detect_format(&p)?,
            Format::Utf32be
        ));
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_write_utf8() -> Result<()> {
        let p = test_path("test_yaml_write_utf8.txt");
        let test_content = "Test UTF-8 content";
        write_file_from_string(&p, test_content, Format::Utf8)?;
        assert_eq!(
            read_file_to_string(&p)?,
            test_content
        );
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_write_utf8bom() -> Result<()> {
        let p = test_path("test_yaml_write_utf8bom.txt");
        let test_content = "Test UTF-8 BOM content";
        write_file_from_string(
            &p,
            test_content,
            Format::Utf8bom,
        )?;
        assert_eq!(
            read_file_to_string(&p)?,
            test_content
        );
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_write_utf16le() -> Result<()> {
        let p = test_path("test_yaml_write_utf16le.txt");
        let test_content = "Test UTF-16LE content";
        write_file_from_string(
            &p,
            test_content,
            Format::Utf16le,
        )?;
        assert_eq!(
            read_file_to_string(&p)?,
            test_content
        );
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_write_utf16be() -> Result<()> {
        let p = test_path("test_yaml_write_utf16be.txt");
        let test_content = "Test UTF-16BE content";
        write_file_from_string(
            &p,
            test_content,
            Format::Utf16be,
        )?;
        assert_eq!(
            read_file_to_string(&p)?,
            test_content
        );
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_write_utf32le() -> Result<()> {
        let p = test_path("test_yaml_write_utf32le.txt");
        let test_content = "Test UTF-32LE content";
        write_file_from_string(
            &p,
            test_content,
            Format::Utf32le,
        )?;
        assert_eq!(
            read_file_to_string(&p)?,
            test_content
        );
        fs::remove_file(&p)?;
        Ok(())
    }

    #[test]
    fn test_write_utf32be() -> Result<()> {
        let p = test_path("test_yaml_write_utf32be.txt");
        let test_content = "Test UTF-32BE content";
        write_file_from_string(
            &p,
            test_content,
            Format::Utf32be,
        )?;
        assert_eq!(
            read_file_to_string(&p)?,
            test_content
        );
        fs::remove_file(&p)?;
        Ok(())
    }
}
