use std::collections::HashMap;
use bencode_lib::{parse, FileSource, Node};
use std::path::Path;

/// Represents details about a single file within a torrent
///
/// # Fields
/// * `path` - The file path relative to the torrent root directory
/// * `length` - Size of the file in bytes
#[derive(Debug, PartialEq)]
pub struct FileDetails {
    pub path: String,
    pub length: u64,
}

/// Represents the metadata contained in a .torrent file
///
/// # Fields
/// * `announce` - The tracker URL
/// * `announce_list` - List of backup tracker URLs
/// * `encoding` - Character encoding used for strings
/// * `attribute` - Optional attribute field
/// * `comment` - Optional comment about the torrent
/// * `creation_date` - Unix timestamp of when the torrent was created
/// * `created_by` - Name/version of the program that created the torrent
/// * `length` - Total size of all files in bytes
/// * `name` - Name of the torrent (usually the directory name)
/// * `piece_length` - Size of each piece in bytes
/// * `pieces` - String of concatenated 20-byte SHA1 piece hashes
/// * `private_flag` - Whether the torrent is private (1) or public (0)
/// * `source` - Source of the torrent
/// * `files` - List of files included in the torrent
#[derive(Debug, PartialEq)]
pub struct TorrentFile {
    pub announce: String,
    pub announce_list: Vec<String>,
    pub  encoding: String,
    pub attribute: u64,
    pub comment: String,
    pub creation_date: u64,
    pub created_by: String,
    pub length: u64,
    pub name: String,
    pub piece_length: u64,
    pub pieces: String,
    pub private_flag: u64,
    pub source: String,
    pub files: Vec<FileDetails>,
}

impl TorrentFile {
    /// Helper method to get an integer value from a dictionary
    ///
    /// # Arguments
    /// * `dict` - Dictionary to search in
    /// * `key` - Key to look up
    /// * `default` - Default value if key is not found
    ///
    /// # Returns
    /// Integer value or default if not found
    fn get_integer(dict: &HashMap<String, Node>, key: &str, default: u64) -> u64 {
        if let Some(Node::Integer(n)) = dict.get(key) {
            return *n as u64;
        }
        default
    }

    /// Helper method to get a string value from a dictionary
    ///
    /// # Arguments
    /// * `dict` - Dictionary to search in 
    /// * `key` - Key to look up
    /// * `default` - Default value if key is not found
    ///
    /// # Returns
    /// String value or default if not found
    fn get_string(dict: &HashMap<String, Node>, key: &str, default: &str) -> String {
        if let Some(Node::Str(s)) = dict.get(key) {
            return s.clone();
        }
        default.to_string()
    }

    /// Helper method to get an integer from the info dictionary
    ///
    /// # Arguments
    /// * `dict` - Root dictionary containing info dict
    /// * `key` - Key to look up in info dict
    /// * `default` - Default value if key is not found
    ///
    /// # Returns
    /// Integer value or default if not found
    fn get_info_integer(dict: &HashMap<String, Node>, key: &str, default: u64) -> u64 {
        if let Some(Node::Dictionary(info_dict)) = dict.get("info") {
            if let Some(Node::Integer(n)) = info_dict.get(key) {
                return *n as u64;
            }
        }
        default
    }

    /// Helper method to get a string from the info dictionary
    ///
    /// # Arguments 
    /// * `dict` - Root dictionary containing info dict
    /// * `key` - Key to look up in info dict
    /// * `default` - Default value if key is not found
    ///
    /// # Returns
    /// String value or default if not found
    fn get_info_string(dict: &HashMap<String, Node>, key: &str, default: &str) -> String {
        if let Some(Node::Dictionary(info_dict)) = dict.get("info") {
            if let Some(Node::Str(s)) = info_dict.get(key) {
                return s.clone();
            }
        }
        default.to_string()
    }

    /// Helper method to extract the announce-list from the torrent
    ///
    /// # Arguments
    /// * `dict` - Root dictionary containing the announce-list
    ///
    /// # Returns
    /// Vector of tracker URLs from the announce-list or empty vector if not found
    fn get_announce_list(dict: &HashMap<String, Node>) -> Vec<String> {
        match dict.get("announce-list") {
            Some(Node::List(list)) => list
                .iter()
                .filter_map(|item| match item {
                    Node::List(sublist) => sublist.first().and_then(|url| match url {
                        Node::Str(s) => Some(s.clone()),
                        _ => None,
                    }),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Helper method to extract the file list from the info dictionary
    ///
    /// # Arguments
    /// * `dict` - Root dictionary containing info dict with files
    ///
    /// # Returns
    /// Vector of FileDetails or empty vector if not found
    fn get_file_list(dict: &HashMap<String, Node>) -> Vec<FileDetails> {
        if let Some(Node::Dictionary(info_dict)) = dict.get("info") {
            if let Some(Node::List(files_list)) = info_dict.get("files") {
                files_list
                    .iter()
                    .filter_map(|file| {
                        if let Node::Dictionary(file_dict) = file {
                            let length = Self::get_integer(file_dict, "length", 0);
                            let path = match file_dict.get("path") {
                                Some(Node::List(path_list)) => path_list
                                    .iter()
                                    .filter_map(|p| match p {
                                        Node::Str(s) => Some(s.clone()),
                                        _ => None,
                                    })
                                    .collect::<Vec<String>>()
                                    .join("/"),
                                _ => return None,
                            };
                            Some(FileDetails { path, length })
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Creates a new TorrentFile instance by parsing a .torrent file
    ///
    /// # Arguments
    /// * `path` - Path to the .torrent file to parse
    ///
    /// # Returns
    /// Result containing either the parsed TorrentFile or an error message
    pub fn from_file(path: &Path) -> Result<TorrentFile, String> {
        match FileSource::new(path.to_str().unwrap()) {
            Ok(mut file) => match parse(&mut file) {
                Ok(Node::Dictionary(dict)) => {
                    Self::validate_required_keys(&dict)?;
                    Ok(TorrentFile {
                        announce: Self::get_string(&dict, "announce", ""),
                        announce_list: Self::get_announce_list(&dict),
                        encoding: Self::get_string(&dict, "encoding", "UTF-8"),
                        attribute: Self::get_info_integer(&dict, "attribute", 0),
                        comment: Self::get_string(&dict, "comment", ""),
                        creation_date: Self::get_integer(&dict, "creation date", 0),
                        created_by: Self::get_string(&dict, "created by", ""),
                        length: Self::get_info_integer(&dict, "length", 0),
                        name: Self::get_info_string(&dict, "name", ""),
                        piece_length: Self::get_info_integer(&dict, "piece length", 0),
                        pieces: Self::get_info_string(&dict, "pieces", ""),
                        private_flag: Self::get_info_integer(&dict, "private", 0),
                        source: Self::get_info_string(&dict, "source", ""),
                        files: Self::get_file_list(&dict),
                    })
                }
                Err(s) => Err(s),
                _ => Err("Invalid torrent file format".to_string()),
            },
            Err(e) => Err(format!("Failed to open file: {}", e)),
        }
    }

    /// Validates that all required keys are present in the torrent dictionary
    ///
    /// # Arguments
    /// * `dict` - Dictionary to validate
    ///
    /// # Returns
    /// Ok if all required keys are present, Err with the message if any are missing
    pub fn validate_required_keys(dict: &HashMap<String, Node>) -> Result<(), String> {
        let required_keys = ["announce", "info"];
        for key in required_keys {
            if !dict.contains_key(key) {
                return Err(format!("Missing required key: {}", key));
            }
        }

        if let Some(Node::Dictionary(info)) = dict.get("info") {
            let required_info_keys = ["name", "piece length", "pieces"];
            for key in required_info_keys {
                if !info.contains_key(key) {
                    return Err(format!("Missing required info key: {}", key));
                }
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_get_integer() {
        let mut dict = HashMap::new();
        dict.insert("test".to_string(), Node::Integer(42));

        assert_eq!(TorrentFile::get_integer(&dict, "test", 0), 42);
        assert_eq!(TorrentFile::get_integer(&dict, "nonexistent", 100), 100);
    }

    #[test]
    fn test_get_string() {
        let mut dict = HashMap::new();
        dict.insert("test".to_string(), Node::Str("value".to_string()));

        assert_eq!(TorrentFile::get_string(&dict, "test", "default"), "value");
        assert_eq!(TorrentFile::get_string(&dict, "nonexistent", "default"), "default");
    }

    #[test]
    fn test_get_info_integer() {
        let mut info_dict = HashMap::new();
        info_dict.insert("attr".to_string(), Node::Integer(42));

        let mut dict = HashMap::new();
        dict.insert("info".to_string(), Node::Dictionary(info_dict));

        assert_eq!(TorrentFile::get_info_integer(&dict, "attr", 0), 42);
        assert_eq!(TorrentFile::get_info_integer(&dict, "nonexistent", 100), 100);
    }

    #[test]
    fn test_validate_required_keys() {
        let mut info_dict = HashMap::new();
        info_dict.insert("name".to_string(), Node::Str("test".to_string()));
        info_dict.insert("piece length".to_string(), Node::Integer(1));
        info_dict.insert("pieces".to_string(), Node::Str("test".to_string()));

        let mut dict = HashMap::new();
        dict.insert("announce".to_string(), Node::Str("test".to_string()));
        dict.insert("info".to_string(), Node::Dictionary(info_dict));

        assert!(TorrentFile::validate_required_keys(&dict).is_ok());
    }

    #[test]
    fn test_validate_required_keys_missing() {
        let dict = HashMap::new();
        assert!(TorrentFile::validate_required_keys(&dict).is_err());
    }

    #[test]
    fn test_get_announce_list() {
        let mut announce_list = Vec::new();
        announce_list.push(Node::List(vec![Node::Str("test1".to_string())]));
        announce_list.push(Node::List(vec![Node::Str("test2".to_string())]));

        let mut dict = HashMap::new();
        dict.insert("announce-list".to_string(), Node::List(announce_list));

        let result = TorrentFile::get_announce_list(&dict);
        assert_eq!(result, vec!["test1".to_string(), "test2".to_string()]);
    }
}
