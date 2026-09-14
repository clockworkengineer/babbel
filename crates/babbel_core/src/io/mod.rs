//! Streaming I/O abstractions, source readers, and destination sinks.

pub mod destinations;
pub mod sources;
pub mod traits;

pub use destinations::{
    ArrayVecDestination, Buffer, BufferDestination, SliceDestination, StringDestination,
};
#[cfg(feature = "file-io")]
pub use destinations::FileDestination;

pub use sources::{
    read_all_bytes, read_all_string, BufferSource, ByteSliceSource, ByteSourceAdapter, SliceSource,
    StringSource,
};
#[cfg(feature = "std")]
pub use sources::ReaderSource;
#[cfg(feature = "file-io")]
pub use sources::FileSource;

pub use traits::*;
