//! Streaming I/O abstractions, source readers, and destination sinks.

pub mod destinations;
pub mod sources;
pub mod traits;

#[cfg(feature = "file-io")]
pub use destinations::FileDestination;
pub use destinations::{
    ArrayVecDestination, Buffer, BufferDestination, SliceDestination, StringDestination,
};

#[cfg(feature = "file-io")]
pub use sources::FileSource;
#[cfg(feature = "std")]
pub use sources::ReaderSource;
pub use sources::{
    BufferSource, ByteSliceSource, ByteSourceAdapter, SliceSource, StringSource, read_all_bytes,
    read_all_string,
};

pub use traits::*;
