//! Shared streaming I/O traits powered by `babbel_core`.

/// Interface for sequential character reading from an input source, powered by `babbel_core`.
pub use babbel_core::io::traits::ISource as ICharStream;
/// Interface for snapshotting and restoring stream read state, powered by `babbel_core`.
pub use babbel_core::io::traits::IStatefulStream;
/// Interface for indentation-aware stream validation, powered by `babbel_core`.
pub use babbel_core::io::traits::IIndentationAware;
/// Concrete save/restore snapshot used by all ISource implementations, powered by `babbel_core`.
pub use babbel_core::io::traits::SaveState;

/// Composite source trait combining character streaming, state snapshots, and indentation tracking.
pub trait ISource: ICharStream + IStatefulStream + IIndentationAware {}

impl<T: ICharStream + IStatefulStream + IIndentationAware + ?Sized> ISource for T {}

/// Trait defining the interface for writing YAML data to a destination, powered by `babbel_core`.
pub use babbel_core::io::traits::IDestination;
