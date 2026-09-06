//! This program converts BitTorrent files from bencode format to YAML format.
//! It processes all .torrent files in the "files" directory and creates corresponding .yaml files.

use std::path::Path;
use bencode_lib::{parse, to_yaml, FileDestination, FileSource};

#[path = "common/utility.rs"]
mod bencode_utility_lib;
use bencode_utility_lib::get_torrent_file_list;

/// Converts a single torrent file from bencode to YAML format
fn process_torrent_file(file_path: &str) -> Result<(), String> {
    let mut source = FileSource::new(file_path).map_err(|e| e.to_string())?;
    let node = parse(&mut source).map_err(|e| e.to_string())?;
    let mut destination = FileDestination::new(Path::new(file_path).with_extension("yaml").to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
    to_yaml(&node, &mut destination)?;
    Ok(())
}

fn main() {
    let torrent_files = get_torrent_file_list("files");
    for file_path in torrent_files {
        match process_torrent_file(&file_path) {
            Ok(()) => println!("Successfully converted {}", file_path),
            Err(e) => eprintln!("Failed to convert {}: {}", file_path, e),
        }
    }
}
