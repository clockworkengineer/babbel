/// Module providing a buffer-based source for reading bencode data from memory
pub mod buffer;
/// Module providing a file-based source for reading bencode data from disk
#[cfg(feature = "std")]
pub mod file;
