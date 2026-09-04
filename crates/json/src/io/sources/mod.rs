/// Module providing a buffer-based source for reading JSON data from memory
pub mod buffer;

/// Module providing a file-based source for reading JSON data from disk
#[cfg(feature = "file-io")]
pub mod file;
