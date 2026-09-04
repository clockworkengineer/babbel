//! Shared streaming I/O traits powered by `babbel_core`.

use crate::constants::{CHAR_SPACE, CHAR_TAB};

/// Interface for sequential character reading from an input source, powered by `babbel_core`.
pub use babbel_core::io::traits::ISource as ICharStream;

/// Interface for snapshotting and restoring stream read state.
pub trait IStatefulStream {
    /// Opaque, concrete snapshot of a source read position and metadata.
    fn save_state(&mut self) -> SaveState;
    /// Restore a previously-saved state.
    fn restore_state(&mut self, state: SaveState);
}

/// Interface for indentation-aware stream validation.
pub trait IIndentationAware {
    fn is_whitespace(&self, c: char) -> bool {
        c == CHAR_SPACE || c == CHAR_TAB
    }
    fn is_tab(&self, c: char) -> bool {
        c == CHAR_TAB
    }
    fn get_current_indent_level(&self) -> usize;
}

/// Composite source trait combining character streaming, state snapshots, and indentation tracking.
pub trait ISource: ICharStream + IStatefulStream + IIndentationAware {}

impl<T: ICharStream + IStatefulStream + IIndentationAware + ?Sized> ISource for T {}

/// Concrete save/restore snapshot used by all ISource implementations.
#[derive(Clone, Debug, PartialEq, Eq)]
/// SaveState
pub struct SaveState {
    /// Absolute byte position in the underlying source (file cursor or buffer index).
    pub pos: u64,
    /// The current byte at that position when the snapshot was taken (if any).
    pub current_byte: Option<u8>,
    /// Column (indent) at the snapshot.
    pub column: usize,
    /// Line number at the snapshot.
    pub line: usize,
}

/// Trait defining the interface for writing YAML data to a destination, powered by `babbel_core`.
pub use babbel_core::io::traits::IDestination;
