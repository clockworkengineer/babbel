//! Embedded Systems & Zero-Allocation Primitives
//!
//! Provides utilities, limits, stack-allocated buffers, and compact error reporting
//! tailored for resource-constrained microcontrollers, real-time operating systems (RTOS),
//! and bare-metal environments (`no_std` / `no_alloc`).

use core::cell::Cell;
use crate::error::ErrorCode;

// ==========================================
// 1. StackBuffer
// ==========================================

/// Fixed-size stack-allocated byte buffer using const generics.
///
/// Provides a way to store, read, and manipulate byte sequences exclusively on
/// the call stack without dynamic heap allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> StackBuffer<N> {
    /// Creates a new empty stack buffer.
    pub const fn new() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    /// Creates a stack buffer from a byte slice. Returns `None` if `slice.len() > N`.
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() > N {
            return None;
        }
        let mut buffer = Self::new();
        buffer.data[..slice.len()].copy_from_slice(slice);
        buffer.len = slice.len();
        Some(buffer)
    }

    /// Returns the data as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Returns the data as a mutable byte slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.len]
    }

    /// Returns the written content as a string slice if valid UTF-8.
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.as_slice())
    }

    /// Returns the number of bytes stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total capacity `N`.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns remaining available bytes.
    pub const fn remaining(&self) -> usize {
        N.saturating_sub(self.len)
    }

    /// Clears the buffer.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Attempts to push a single byte. Returns `false` if full.
    pub fn push(&mut self, byte: u8) -> bool {
        if self.len >= N {
            return false;
        }
        self.data[self.len] = byte;
        self.len += 1;
        true
    }

    /// Attempts to extend the buffer from a byte slice. Returns `false` if insufficient capacity.
    pub fn extend_from_slice(&mut self, slice: &[u8]) -> bool {
        if self.len + slice.len() > N {
            return false;
        }
        self.data[self.len..self.len + slice.len()].copy_from_slice(slice);
        self.len += slice.len();
        true
    }
}

impl<const N: usize> Default for StackBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

// ==========================================
// 2. MemoryTracker
// ==========================================

/// Tracks memory allocations and peak watermarks for resource-constrained environments.
#[derive(Debug, Default)]
pub struct MemoryTracker {
    current: Cell<usize>,
    peak: Cell<usize>,
    limit: usize,
}

impl MemoryTracker {
    /// Creates a new memory tracker without an upper limit.
    pub const fn new() -> Self {
        Self {
            current: Cell::new(0),
            peak: Cell::new(0),
            limit: 0,
        }
    }

    /// Creates a new memory tracker with a hard byte limit.
    pub const fn with_limit(limit: usize) -> Self {
        Self {
            current: Cell::new(0),
            peak: Cell::new(0),
            limit,
        }
    }

    /// Records an allocation of `bytes`. Returns `Err` if it would exceed `limit`.
    pub fn allocate(&self, bytes: usize) -> Result<(), &'static str> {
        let new_current = self.current.get().saturating_add(bytes);
        if self.limit > 0 && new_current > self.limit {
            return Err("Memory limit exceeded");
        }
        self.current.set(new_current);
        if new_current > self.peak.get() {
            self.peak.set(new_current);
        }
        Ok(())
    }

    /// Records a deallocation of `bytes`.
    pub fn deallocate(&self, bytes: usize) {
        let current = self.current.get();
        self.current.set(current.saturating_sub(bytes));
    }

    /// Returns the current number of allocated bytes.
    pub fn current(&self) -> usize {
        self.current.get()
    }

    /// Returns the peak watermarked bytes.
    pub fn peak(&self) -> usize {
        self.peak.get()
    }

    /// Returns the configured limit (0 = unlimited).
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Resets the tracker.
    pub fn reset(&self) {
        self.current.set(0);
        self.peak.set(0);
    }
}

// ==========================================
// 3. EmbeddedLimits
// ==========================================

/// Runtime and compile-time limits to protect against stack overflows and excessive resource consumption.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddedLimits {
    /// Maximum recursive nesting depth (default: 16).
    pub max_depth: usize,
    /// Maximum token or string length in bytes (default: 256).
    pub max_token_length: usize,
    /// Maximum container elements (default: 64).
    pub max_container_items: usize,
}

impl EmbeddedLimits {
    /// Standard conservative limits for 32-bit microcontrollers (e.g. ARM Cortex-M).
    pub const CONSERVATIVE: Self = Self {
        max_depth: 16,
        max_token_length: 256,
        max_container_items: 64,
    };

    /// Minimal limits for ultra-constrained systems (e.g. 8-bit AVR or Cortex-M0 with <= 4KB RAM).
    pub const MINIMAL: Self = Self {
        max_depth: 8,
        max_token_length: 64,
        max_container_items: 16,
    };

    /// Checks if a nesting depth exceeds configured limit.
    #[inline]
    pub fn check_depth(&self, current_depth: usize) -> Result<(), ErrorCode> {
        if current_depth > self.max_depth {
            Err(ErrorCode::SyntaxError)
        } else {
            Ok(())
        }
    }
}

impl Default for EmbeddedLimits {
    fn default() -> Self {
        Self::CONSERVATIVE
    }
}

// ==========================================
// 4. CompactError (Zero-Allocation Error)
// ==========================================

/// Zero-allocation 8-byte diagnostic error for bare-metal embedded targets (`no_alloc`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactError {
    /// Standardized error classification
    pub code: ErrorCode,
    /// 0-based byte offset in input stream
    pub offset: u32,
}

impl CompactError {
    /// Creates a new compact error.
    pub const fn new(code: ErrorCode, offset: u32) -> Self {
        Self { code, offset }
    }
}

impl core::fmt::Display for CompactError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} at byte offset {}", self.code, self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_buffer() {
        let mut buf = StackBuffer::<16>::new();
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.remaining(), 16);

        assert!(buf.push(b'H'));
        assert!(buf.push(b'i'));
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.as_slice(), b"Hi");
        assert_eq!(buf.as_str().unwrap(), "Hi");

        assert!(buf.extend_from_slice(b" embedded"));
        assert_eq!(buf.as_str().unwrap(), "Hi embedded");

        assert!(!buf.extend_from_slice(b" this will overflow the buffer capacity"));
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::with_limit(100);
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.limit(), 100);

        assert!(tracker.allocate(60).is_ok());
        assert_eq!(tracker.current(), 60);
        assert_eq!(tracker.peak(), 60);

        assert!(tracker.allocate(50).is_err()); // 60 + 50 = 110 > 100
        tracker.deallocate(20);
        assert_eq!(tracker.current(), 40);
        assert_eq!(tracker.peak(), 60); // Peak retained
    }

    #[test]
    fn test_limits() {
        let limits = EmbeddedLimits::CONSERVATIVE;
        assert!(limits.check_depth(10).is_ok());
        assert!(limits.check_depth(17).is_err());
    }

    #[test]
    fn test_compact_error() {
        let err = CompactError::new(ErrorCode::UnexpectedEof, 42);
        assert_eq!(err.code, ErrorCode::UnexpectedEof);
        assert_eq!(err.offset, 42);
    }
}
