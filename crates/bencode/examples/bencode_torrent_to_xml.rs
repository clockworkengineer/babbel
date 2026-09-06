//! This module provides functionality to convert torrent files from bencode format to XML.
//! It processes files from a specified directory and creates corresponding XML outputs.

use std::path::Path;
use bencode_lib::{parse, to_xml, FileDestination, FileSource};

#[path = "common/utility.rs"]
mod bencode_utility_lib;
use bencode_utility_lib::get_torrent_file_list;

/// Converts a single torrent file from bencode format to XML format.
fn process_torrent_file(file_path: &str) -> Result<(), String> {
    let mut source = FileSource::new(file_path).map_err(|e| e.to_string())?;
    let node = parse(&mut source).map_err(|e| e.to_string())?;
    let mut destination = FileDestination::new(
        Path::new(file_path)
            .with_extension("xml")
            .to_string_lossy()
            .as_ref(),
    )
    .map_err(|e| e.to_string())?;
    to_xml(&node, &mut destination).map_err(|e| e.to_string())?;
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
