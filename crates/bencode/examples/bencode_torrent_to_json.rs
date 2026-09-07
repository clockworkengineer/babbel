//! Example demonstrating conversion of torrent files from bencode format to JSON format.
//! Takes torrent files from the "files" directory and creates corresponding JSON files.

use std::path::Path;
use babbel_bencode::{parse, to_json, FileDestination, FileSource};

#[path = "common/utility.rs"]
mod bencode_utility_lib;
use bencode_utility_lib::get_torrent_file_list;

/// Converts a single torrent file from bencode format to JSON format
fn process_torrent_file(file_path: &str) -> Result<(), String> {
    let mut source = FileSource::new(file_path).map_err(|e| e.to_string())?;
    let node = parse(&mut source).map_err(|e| e.to_string())?;
    let mut destination = FileDestination::new(Path::new(file_path).with_extension("json").to_string_lossy().as_ref()).map_err(|e| e.to_string())?;
    to_json(&node, &mut destination).map_err(|e| e.to_string())?;
    Ok(())
}

/// Main function that processes all torrent files in the "files" directory
fn main() {
    let torrent_files = get_torrent_file_list("files");
    for file_path in torrent_files {
        match process_torrent_file(&file_path) {
            Ok(()) => println!("Successfully converted {}", file_path),
            Err(e) => eprintln!("Failed to convert {}: {}", file_path, e),
        }
    }
}
