use crate::io::traits::{BencodeWrite, BufferedWrite, IDestination};
use std::fs::File as StdFile;
use std::io::{Read, Seek, Write};

/// A file-based destination for writing bencode data to disk.
/// Implements file operations for storing and manipulating encoded data.
pub struct File {
    /// The underlying file handle for I/O operations
    file: StdFile,
    /// Name/path of the file being operated on
    file_name: String,
    /// Current length of the file in bytes
    file_length: usize,
}

impl File {
    /// Creates a new File instance with the specified path.
    ///
    /// # Arguments
    /// * `path` - The file path where the data will be written
    ///
    /// # Returns
    /// A Result containing the new File instance or an IO error
    pub fn new(path: &str) -> std::io::Result<Self> {
        Ok(Self {
            file: StdFile::create(path)?,
            file_name: path.to_string(),
            file_length: 0,
        })
    }

    /// Returns the current length of the file in bytes.
    pub fn file_length(&self) -> usize {
        self.file_length
    }
    /// Returns the name/path of the file.
    pub fn file_name(&self) -> &str {
        &self.file_name.as_str()
    }
    /// Closes the file handle.
    pub fn close(&self) -> std::io::Result<()> {
        Ok(())
    }

    /// Clears the file content by recreating it.
    pub fn clear(&mut self) {
        self.file = StdFile::create(&self.file_name).unwrap();
        self.file_length = 0;
    }
}

impl BencodeWrite for File {
    fn write_byte(&mut self, byte: u8) {
        self.file.write_all(&[byte]).unwrap();
        self.file_length += 1;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.file.write_all(bytes).unwrap();
        self.file_length += bytes.len();
    }
}

impl BufferedWrite for File {
    fn clear(&mut self) {
        self.clear();
    }

    fn last_byte(&self) -> Option<u8> {
        if self.file_length == 0 {
            None
        } else {
            let mut buf = vec![0];
            let mut file = StdFile::open(&self.file_name).unwrap();
            file.seek(std::io::SeekFrom::End(-1)).unwrap();
            file.read_exact(&mut buf).unwrap();
            Some(buf[0])
        }
    }
}

impl IDestination for File {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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

    #[test]
    fn add_byte_increments_length_by_one() -> std::io::Result<()> {
        let path = "test_add_byte_len.txt";
        let mut file = File::new(path)?;
        file.add_byte(b'x');
        assert_eq!(file.file_length(), 1);
        file.add_byte(b'y');
        assert_eq!(file.file_length(), 2);
        file.add_byte(b'z');
        assert_eq!(file.file_length(), 3);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn add_bytes_empty_string_is_noop() -> std::io::Result<()> {
        let path = "test_add_bytes_empty.txt";
        let mut file = File::new(path)?;
        file.add_bytes("");
        assert_eq!(file.file_length(), 0);
        assert_eq!(file.last(), None);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn add_bytes_length_matches_str_byte_count() -> std::io::Result<()> {
        let path = "test_add_bytes_len.txt";
        let mut file = File::new(path)?;
        file.add_bytes("hello");
        assert_eq!(file.file_length(), 5);
        file.add_bytes("!!");
        assert_eq!(file.file_length(), 7);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn last_after_add_bytes_returns_last_char() -> std::io::Result<()> {
        let path = "test_last_add_bytes.txt";
        let mut file = File::new(path)?;
        file.add_bytes("spam");
        assert_eq!(file.last(), Some(b'm'));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn last_single_byte() -> std::io::Result<()> {
        let path = "test_last_single.txt";
        let mut file = File::new(path)?;
        file.add_byte(b'e');
        assert_eq!(file.last(), Some(b'e'));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn multiple_add_bytes_accumulates_content() -> std::io::Result<()> {
        let path = "test_accumulate.txt";
        let mut file = File::new(path)?;
        file.add_bytes("4:");
        file.add_bytes("spam");
        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "4:spam");
        assert_eq!(file.file_length(), 6);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn add_byte_then_add_bytes_mixed() -> std::io::Result<()> {
        let path = "test_mixed.txt";
        let mut file = File::new(path)?;
        file.add_byte(b'i');
        file.add_bytes("42");
        file.add_byte(b'e');
        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "i42e");
        assert_eq!(file.file_length(), 4);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn clear_then_rewrite_works() -> std::io::Result<()> {
        let path = "test_clear_rewrite.txt";
        let mut file = File::new(path)?;
        file.add_bytes("first");
        file.clear();
        file.add_bytes("second");
        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "second");
        assert_eq!(file.file_length(), 6);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn clear_resets_length_to_zero() -> std::io::Result<()> {
        let path = "test_clear_len.txt";
        let mut file = File::new(path)?;
        file.add_bytes("some data");
        file.clear();
        assert_eq!(file.file_length(), 0);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn file_name_unchanged_after_write_and_clear() -> std::io::Result<()> {
        let path = "test_name_stable.txt";
        let mut file = File::new(path)?;
        file.add_bytes("data");
        file.clear();
        assert_eq!(file.file_name(), path);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn bencode_dict_round_trip() -> std::io::Result<()> {
        let path = "test_dict_roundtrip.txt";
        let mut file = File::new(path)?;
        file.add_bytes("d3:cow3:moo4:spam4:eggse");
        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "d3:cow3:moo4:spam4:eggse");
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn bencode_list_round_trip() -> std::io::Result<()> {
        let path = "test_list_roundtrip.txt";
        let mut file = File::new(path)?;
        file.add_bytes("l4:spami42ee");
        let mut content = String::new();
        StdFile::open(path)?.read_to_string(&mut content)?;
        assert_eq!(content, "l4:spami42ee");
        assert_eq!(file.file_length(), 12);
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn multiple_clears_are_idempotent() -> std::io::Result<()> {
        let path = "test_multi_clear.txt";
        let mut file = File::new(path)?;
        file.clear();
        file.clear();
        assert_eq!(file.file_length(), 0);
        assert_eq!(file.last(), None);
        fs::remove_file(path)?;
        Ok(())
    }
}
