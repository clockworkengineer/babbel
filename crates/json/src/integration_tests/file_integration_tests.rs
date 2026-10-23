//! Integration tests for file operations
//! Tests reading, writing, and detecting format of actual JSON files from the files/formatted directory

#[cfg(test)]
mod tests {

    use crate::{Format, detect_format, read_file_to_string, write_file_from_string};
    use std::path::PathBuf;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    fn test_file(relative: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("files")
            .join(relative)
    }

    /// Helper function to verify file content matches expected value
    fn verify_file_content(path: impl AsRef<std::path::Path>, expected: &str) -> Result<()> {
        let content = read_file_to_string(path)?;
        assert_eq!(content, expected);
        Ok(())
    }

    #[test]
    fn test_formatted_testfile021() -> Result<()> {
        assert!(matches!(
            detect_format(test_file("formatted/testfile021.json"))?,
            Format::Utf8
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile022() -> Result<()> {
        assert!(matches!(
            detect_format(test_file("formatted/testfile022.json"))?,
            Format::Utf8bom
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile023() -> Result<()> {
        assert!(matches!(
            detect_format(test_file("formatted/testfile023.json"))?,
            Format::Utf16be
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile024() -> Result<()> {
        assert!(matches!(
            detect_format(test_file("formatted/testfile024.json"))?,
            Format::Utf16le
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile025() -> Result<()> {
        assert!(matches!(
            detect_format(test_file("formatted/testfile025.json"))?,
            Format::Utf32be
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile026() -> Result<()> {
        assert!(matches!(
            detect_format(test_file("formatted/testfile026.json"))?,
            Format::Utf32le
        ));
        Ok(())
    }

    #[test]
    fn test_read_utf8_file() -> Result<()> {
        verify_file_content(
            test_file("formatted/testfile021.json"),
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf8bom_file() -> Result<()> {
        verify_file_content(
            test_file("formatted/testfile022.json"),
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf16be_file() -> Result<()> {
        verify_file_content(
            test_file("formatted/testfile023.json"),
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf16le_file() -> Result<()> {
        verify_file_content(
            test_file("formatted/testfile024.json"),
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf32be_file() -> Result<()> {
        verify_file_content(
            test_file("formatted/testfile025.json"),
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf32le_file() -> Result<()> {
        verify_file_content(
            test_file("formatted/testfile026.json"),
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_write_formatted_testfile021() -> Result<()> {
        let path = test_file("formatted/testfile021.json");
        let test_content = "[true  , \"Out of time\",  7.89043e+18, true]";
        write_file_from_string(&path, test_content, Format::Utf8)?;
        assert_eq!(read_file_to_string(&path)?, test_content);
        Ok(())
    }

    #[test]
    fn test_write_formatted_testfile022() -> Result<()> {
        let path = test_file("formatted/testfile022.json");
        let test_content = "[true  , \"Out of time\",  7.89043e+18, true]";
        write_file_from_string(&path, test_content, Format::Utf8bom)?;
        assert_eq!(read_file_to_string(&path)?, test_content);
        Ok(())
    }
}
