//! Example demonstrating how to read and display torrent file metadata using the bencode parser.
//! This program processes all .torrent files in the "files" directory and displays their contents
//! in a human-readable format.

#[path = "common/utility.rs"]
mod bencode_utility_lib;
use bencode_utility_lib::get_torrent_file_list;

#[path = "common/torrent.rs"]
mod torrent_file;
use torrent_file::TorrentFile;

/// Prints all metadata fields from a parsed torrent file in a human-readable format
pub fn print_details(torrent: &TorrentFile) {
    println!("Successfully parsed torrent file:");
    println!("Announce URL: {}", torrent.announce);
    println!("Announce List URLs:");
    for url in &torrent.announce_list {
        println!("  - {}", url);
    }
    println!("Encoding: {}", torrent.encoding);
    println!("Attribute: {}", torrent.attribute);
    println!("Comment: {}", torrent.comment);
    println!("Creation Date: {}", torrent.creation_date);
    println!("Created By: {}", torrent.created_by);
    println!("Length: {} bytes", torrent.length);
    println!("Name: {}", torrent.name);
    println!("Piece Length: {}", torrent.piece_length);
    println!(
        "Pieces: {}",
        torrent
            .pieces
            .as_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    );
    println!("Private Bit Mask: {}", torrent.private_flag);
    println!("Source: {}", torrent.source);
    println!("Files:");
    for file in &torrent.files {
        println!("  - {} ({} bytes)", file.path, file.length);
    }
}

/// Main entry point that:
/// 1. Retrieves a list of .torrent files from the "files" directory
/// 2. Attempts to parse each file as a torrent file
/// 3. Displays the parsed metadata or any parsing errors
fn main() {
    let torrent_files = get_torrent_file_list("files");
    for file_path in torrent_files {
        println!("\nProcessing {:?}:", file_path);
        match TorrentFile::from_file(file_path.as_ref()) {
            Ok(torrent) => {
                print_details(&torrent);
            }
            Err(e) => eprintln!("Error reading torrent file: {}", e),
        }
    }
}
