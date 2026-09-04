//! Streaming I/O abstractions, source readers, and destination sinks.

pub mod destinations;
pub mod sources;
pub mod traits;

pub use destinations::{Buffer, BufferDestination, StringDestination};
#[cfg(feature = "file-io")]
pub use destinations::FileDestination;

pub use sources::{BufferSource, ByteSliceSource, SliceSource, StringSource};
#[cfg(feature = "file-io")]
pub use sources::FileSource;

pub use traits::{
    IByteStream, ICharStream, IClearable, IDestination, IIndentationAware, IPositionAware,
    IRewindable, ISource,
};
