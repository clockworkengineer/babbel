//! Integration tests for file operations
//! Tests reading, writing, and detecting format of actual JSON files from the files/formatted directory

#[cfg(test)]
mod tests {

    use crate::{Format, detect_format, read_file_to_string, write_file_from_string};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// Helper function to verify file content matches expected value
    fn verify_file_content(filename: &str, expected: &str) -> Result<()> {
        let content = read_file_to_string(filename)?;
        assert_eq!(content, expected);
        Ok(())
    }

    #[test]
    fn test_formatted_testfile021() -> Result<()> {
        assert!(matches!(
            detect_format("../files/formatted/testfile021.json")?,
            Format::Utf8
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile022() -> Result<()> {
        assert!(matches!(
            detect_format("../files/formatted/testfile022.json")?,
            Format::Utf8bom
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile023() -> Result<()> {
        assert!(matches!(
            detect_format("../files/formatted/testfile023.json")?,
            Format::Utf16be
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile024() -> Result<()> {
        assert!(matches!(
            detect_format("../files/formatted/testfile024.json")?,
            Format::Utf16le
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile025() -> Result<()> {
        assert!(matches!(
            detect_format("../files/formatted/testfile025.json")?,
            Format::Utf32be
        ));
        Ok(())
    }

    #[test]
    fn test_formatted_testfile026() -> Result<()> {
        assert!(matches!(
            detect_format("../files/formatted/testfile026.json")?,
            Format::Utf32le
        ));
        Ok(())
    }

    #[test]
    fn test_read_utf8_file() -> Result<()> {
        verify_file_content(
            "../files/formatted/testfile021.json",
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf8bom_file() -> Result<()> {
        verify_file_content(
            "../files/formatted/testfile022.json",
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf16be_file() -> Result<()> {
        verify_file_content(
            "../files/formatted/testfile023.json",
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf16le_file() -> Result<()> {
        verify_file_content(
            "../files/formatted/testfile024.json",
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf32be_file() -> Result<()> {
        verify_file_content(
            "../files/formatted/testfile025.json",
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_read_utf32le_file() -> Result<()> {
        verify_file_content(
            "../files/formatted/testfile026.json",
            "[true  , \"Out of time\",  7.89043e+18, true]",
        )
    }

    #[test]
    fn test_write_formatted_testfile021() -> Result<()> {
        let test_content = "[true  , \"Out of time\",  7.89043e+18, true]";
        write_file_from_string(
            "../files/formatted/testfile021.json",
            test_content,
            Format::Utf8,
        )?;
        assert_eq!(
            read_file_to_string("../files/formatted/testfile021.json")?,
            test_content
        );
        Ok(())
    }

    #[test]
    fn test_write_formatted_testfile022() -> Result<()> {
        let test_content = "[true  , \"Out of time\",  7.89043e+18, true]";
        write_file_from_string(
            "../files/formatted/testfile022.json",
            test_content,
            Format::Utf8bom,
        )?;
        assert_eq!(
            read_file_to_string("../files/formatted/testfile022.json")?,
            test_content
        );
        Ok(())
    }
}
