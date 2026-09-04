/// Module providing a buffer-based destination for writing bencode data into memory
pub mod buffer;
/// Module providing a file-based destination for writing bencode data to disk
#[cfg(feature = "std")]
pub mod file;