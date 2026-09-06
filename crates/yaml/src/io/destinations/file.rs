//! File Destination for Encoded Output
//!
//! Re-exports the unified `FileDestination` from `babbel_core::io`.
//!
//! Copyright (c) 2026 YAML Library Developers

pub use babbel_core::io::FileDestination as File;


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File as StdFile;
    use std::io::Read;

    #[test]
    fn create_file_destination_works() -> std::io::Result<()> {
        let path = "test_create.txt";
        let _file = File::new(path)?;
        assert!(fs::metadata(path).is_ok());
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn create_file_fails_with_invalid_path() {
        let result = File::new("/invalid/path/test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn write_fails_on_readonly_file() -> std::io::Result<()> {
        let path = "test_readonly.txt";
        let mut file = File::new(path)?;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(path, perms)?;

        file.add_bytes("test");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn read_fails_on_missing_file() {
        let path = "missing_file.txt";
        let file = File::new(path).unwrap();
        assert!(file.last().is_none());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn add_byte_works() -> std::io::Result<()> {
        let path = "test_byte.txt";
        let mut file = File::new(path)?;
        file.add_byte(b'A');
        file.flush()?;

        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "A");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn add_bytes_works() -> std::io::Result<()> {
        let path = "test_bytes.txt";
        let mut file = File::new(path)?;
        file.add_bytes("test");
        file.flush()?;

        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "test");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn clear_works() -> std::io::Result<()> {
        let path = "test_clear.txt";
        let mut file = File::new(path)?;
        file.add_bytes("test");
        file.clear();

        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "");

        fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn file_length_works() -> std::io::Result<()> {
        let path = "test_length.txt";
        let mut file = File::new(path)?;
        assert_eq!(file.file_length(), 0);

        file.add_byte(b'A');
        assert_eq!(file.file_length(), 1);

        file.add_bytes("test");
        assert_eq!(file.file_length(), 5);

        file.clear();
        assert_eq!(file.file_length(), 0);

        fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn file_name_works() -> std::io::Result<()> {
        let path = "test_name.txt";
        let file = File::new(path)?;
        assert_eq!(file.file_name(), path);
        fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn last_works() -> std::io::Result<()> {
        let path = "test_last.txt";
        let mut file = File::new(path)?;
        assert_eq!(file.last(), None);

        file.add_byte(b'1');
        assert_eq!(file.last(), Some(b'1'));

        file.add_byte(b'2');
        assert_eq!(file.last(), Some(b'2'));

        file.clear();
        assert_eq!(file.last(), None);

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn last_handles_empty_file() -> std::io::Result<()> {
        let path = "test_empty.txt";
        let file = File::new(path)?;
        assert_eq!(file.last(), None);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn close_works() -> std::io::Result<()> {
        let path = "test_name.txt";
        let file = File::new(path)?;
        file.close()?;
        fs::remove_file(path)?;
        Ok(())
    }

    // --- New and relevant unit tests ---
    #[test]
    fn add_bytes_empty_string_file() -> std::io::Result<()> {
        let path = "test_empty_bytes.txt";
        let mut file = File::new(path)?;
        file.add_bytes("");
        assert_eq!(file.file_length(), 0);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn clear_on_empty_file() -> std::io::Result<()> {
        let path = "test_clear_empty.txt";
        let mut file = File::new(path)?;
        file.clear();
        assert_eq!(file.file_length(), 0);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn add_byte_and_clear_and_add_again_file() -> std::io::Result<()> {
        let path = "test_add_clear_add.txt";
        let mut file = File::new(path)?;
        file.add_byte(b'a');
        file.clear();
        file.add_byte(b'b');
        assert_eq!(file.last(), Some(b'b'));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn last_after_multiple_adds_and_clear_file() -> std::io::Result<()> {
        let path = "test_last_multi.txt";
        let mut file = File::new(path)?;
        file.add_bytes("abc");
        assert_eq!(file.last(), Some(b'c'));
        file.clear();
        assert_eq!(file.last(), None);
        file.add_bytes("xyz");
        assert_eq!(file.last(), Some(b'z'));
        fs::remove_file(path)?;
        Ok(())
    }
}
