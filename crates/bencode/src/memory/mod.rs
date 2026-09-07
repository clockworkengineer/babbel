//! Memory pool utilities for embedded systems.
//!
//! This module provides tools for tracking and managing memory allocation
//! in resource-constrained environments. It includes:
//! - Memory usage tracking
//! - Bounded allocation helpers
//! - Stack-based buffer management

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::cell::Cell;

pub use babbel_core::embedded::{MemoryTracker, StackBuffer};

/// A simple arena allocator that allocates from a fixed buffer.
///
/// This provides bump allocation from a pre-allocated buffer, useful for
/// embedded systems that want to avoid heap fragmentation or have no heap.
pub struct Arena {
    buffer: Vec<u8>,
    position: Cell<usize>,
}

impl Arena {
    /// Creates a new arena with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: Cell::new(0),
        }
    }

    /// Allocates a byte slice from the arena.
    /// Returns None if there's insufficient space.
    pub fn alloc_bytes(&self, size: usize) -> Option<&mut [u8]> {
        let pos = self.position.get();
        let new_pos = pos + size;

        if new_pos > self.buffer.capacity() {
            return None;
        }

        self.position.set(new_pos);

        // Safety: We've checked the bounds and we own the buffer
        // This is safe because we're returning a unique mutable reference
        unsafe {
            let ptr = self.buffer.as_ptr().add(pos) as *mut u8;
            Some(core::slice::from_raw_parts_mut(ptr, size))
        }
    }

    /// Returns the number of bytes allocated.
    pub fn used(&self) -> usize {
        self.position.get()
    }

    /// Returns the total capacity of the arena.
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Returns the number of bytes remaining.
    pub fn remaining(&self) -> usize {
        self.capacity() - self.used()
    }

    /// Resets the arena, allowing all memory to be reused.
    ///
    /// # Safety
    /// This invalidates all previously allocated slices from this arena.
    /// The caller must ensure no references to allocated data remain.
    pub unsafe fn reset(&self) {
        self.position.set(0);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn memory_tracker_overflow_and_limit() {
        let tracker = MemoryTracker::with_limit(10);
        assert!(tracker.allocate(5).is_ok());
        assert!(tracker.allocate(6).is_err());
        tracker.deallocate(5);
        assert_eq!(tracker.current(), 0);
        tracker.reset();
        assert_eq!(tracker.current(), 0);
    }

    #[test]
    fn arena_allocation_errors() {
        let arena = Arena::with_capacity(8);
        assert!(arena.alloc_bytes(4).is_some());
        assert!(arena.alloc_bytes(5).is_none()); // Not enough space
        let arena = Arena::with_capacity(0);
        assert!(arena.alloc_bytes(1).is_none());
    }

    #[test]
    fn stack_buffer_error_cases() {
        let mut buffer = StackBuffer::<2>::new();
        assert!(buffer.push(b'a'));
        assert!(buffer.push(b'b'));
        assert!(!buffer.push(b'c'));
        buffer.clear();
        assert!(buffer.is_empty());
        assert!(buffer.extend_from_slice(b"ab"));
        assert!(!buffer.extend_from_slice(b"cd"));
    }
    use super::*;

    #[test]
    fn memory_tracker_basic() {
        let tracker = MemoryTracker::new();
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);

        tracker.allocate(100).unwrap();
        assert_eq!(tracker.current(), 100);
        assert_eq!(tracker.peak(), 100);

        tracker.allocate(50).unwrap();
        assert_eq!(tracker.current(), 150);
        assert_eq!(tracker.peak(), 150);

        tracker.deallocate(50);
        assert_eq!(tracker.current(), 100);
        assert_eq!(tracker.peak(), 150); // Peak doesn't decrease
    }

    #[test]
    fn memory_tracker_with_limit() {
        let tracker = MemoryTracker::with_limit(200);

        assert!(tracker.allocate(100).is_ok());
        assert!(tracker.allocate(50).is_ok());
        assert!(tracker.allocate(51).is_err()); // Would exceed limit
        assert_eq!(tracker.current(), 150);
    }

    #[test]
    fn memory_tracker_reset() {
        let tracker = MemoryTracker::new();
        tracker.allocate(100).unwrap();
        assert_eq!(tracker.current(), 100);

        tracker.reset();
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);
    }

    #[test]
    fn arena_basic() {
        let arena = Arena::with_capacity(1024);
        assert_eq!(arena.capacity(), 1024);
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 1024);

        let slice1 = arena.alloc_bytes(100).unwrap();
        assert_eq!(slice1.len(), 100);
        assert_eq!(arena.used(), 100);
        assert_eq!(arena.remaining(), 924);

        let slice2 = arena.alloc_bytes(200).unwrap();
        assert_eq!(slice2.len(), 200);
        assert_eq!(arena.used(), 300);
    }

    #[test]
    fn arena_out_of_space() {
        let arena = Arena::with_capacity(100);

        assert!(arena.alloc_bytes(50).is_some());
        assert!(arena.alloc_bytes(50).is_some());
        assert!(arena.alloc_bytes(1).is_none()); // Out of space
    }

    #[test]
    fn stack_buffer_basic() {
        let mut buffer = StackBuffer::<256>::new();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 256);
        assert!(buffer.is_empty());

        assert!(buffer.push(b'a'));
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.as_slice(), b"a");

        assert!(buffer.extend_from_slice(b"bcdef"));
        assert_eq!(buffer.as_slice(), b"abcdef");
    }

    #[test]
    fn stack_buffer_from_slice() {
        let buffer = StackBuffer::<10>::from_slice(b"hello").unwrap();
        assert_eq!(buffer.as_slice(), b"hello");

        let too_large = StackBuffer::<5>::from_slice(b"hello world");
        assert!(too_large.is_none());
    }

    #[test]
    fn stack_buffer_overflow() {
        let mut buffer = StackBuffer::<3>::new();
        assert!(buffer.push(b'a'));
        assert!(buffer.push(b'b'));
        assert!(buffer.push(b'c'));
        assert!(!buffer.push(b'd')); // Buffer full

        assert_eq!(buffer.as_slice(), b"abc");
    }

    #[test]
    fn stack_buffer_clear() {
        let mut buffer = StackBuffer::<10>::new();
        buffer.extend_from_slice(b"test");
        assert_eq!(buffer.len(), 4);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    // ── MemoryTracker additional tests ────────────────────────────────────────

    #[test]
    fn memory_tracker_limit_returns_configured_value() {
        let tracker = MemoryTracker::with_limit(512);
        assert_eq!(tracker.limit(), 512);
    }

    #[test]
    fn memory_tracker_no_limit_returns_zero() {
        let tracker = MemoryTracker::new();
        assert_eq!(tracker.limit(), 0);
    }

    #[test]
    fn memory_tracker_no_limit_allows_large_allocation() {
        let tracker = MemoryTracker::new();
        assert!(tracker.allocate(usize::MAX / 2).is_ok());
    }

    #[test]
    fn memory_tracker_allocate_exactly_at_limit_succeeds() {
        let tracker = MemoryTracker::with_limit(100);
        assert!(tracker.allocate(100).is_ok());
        assert_eq!(tracker.current(), 100);
    }

    #[test]
    fn memory_tracker_allocate_one_over_limit_fails() {
        let tracker = MemoryTracker::with_limit(100);
        assert!(tracker.allocate(101).is_err());
        // current must be unchanged after a rejected allocation
        assert_eq!(tracker.current(), 0);
    }

    #[test]
    fn memory_tracker_peak_not_reduced_by_deallocate() {
        let tracker = MemoryTracker::new();
        tracker.allocate(200).unwrap();
        tracker.deallocate(200);
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 200);
    }

    #[test]
    fn memory_tracker_peak_tracks_highest_watermark() {
        let tracker = MemoryTracker::new();
        tracker.allocate(50).unwrap();
        tracker.allocate(50).unwrap(); // peak = 100
        tracker.deallocate(80);
        tracker.allocate(10).unwrap(); // current = 30, peak stays 100
        assert_eq!(tracker.peak(), 100);
        assert_eq!(tracker.current(), 30);
    }

    #[test]
    fn memory_tracker_deallocate_saturates_at_zero() {
        let tracker = MemoryTracker::new();
        tracker.allocate(10).unwrap();
        tracker.deallocate(50); // more than current – should saturate at 0
        assert_eq!(tracker.current(), 0);
    }

    #[test]
    fn memory_tracker_multiple_resets_are_idempotent() {
        let tracker = MemoryTracker::new();
        tracker.allocate(100).unwrap();
        tracker.reset();
        tracker.reset();
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.peak(), 0);
    }

    #[test]
    fn memory_tracker_allocate_after_reset() {
        let tracker = MemoryTracker::with_limit(100);
        tracker.allocate(100).unwrap();
        tracker.reset();
        // After reset the full limit should be available again
        assert!(tracker.allocate(100).is_ok());
    }

    #[test]
    fn memory_tracker_error_message() {
        let tracker = MemoryTracker::with_limit(10);
        let err = tracker.allocate(11).unwrap_err();
        assert_eq!(err, "Memory limit exceeded");
    }

    // ── Arena additional tests ────────────────────────────────────────────────

    #[test]
    fn arena_zero_capacity_rejects_all_allocs() {
        let arena = Arena::with_capacity(0);
        // A zero-size alloc succeeds (0 <= 0), but any non-zero alloc is rejected
        assert!(arena.alloc_bytes(1).is_none());
        assert!(arena.alloc_bytes(usize::MAX).is_none());
    }

    #[test]
    fn arena_remaining_starts_equal_to_capacity() {
        let arena = Arena::with_capacity(256);
        assert_eq!(arena.remaining(), 256);
    }

    #[test]
    fn arena_remaining_decrements_by_allocated_size() {
        let arena = Arena::with_capacity(100);
        arena.alloc_bytes(30).unwrap();
        assert_eq!(arena.remaining(), 70);
        arena.alloc_bytes(20).unwrap();
        assert_eq!(arena.remaining(), 50);
    }

    #[test]
    fn arena_write_to_allocated_slice() {
        let arena = Arena::with_capacity(64);
        let slice = arena.alloc_bytes(5).unwrap();
        slice.copy_from_slice(b"hello");
        assert_eq!(slice, b"hello");
    }

    #[test]
    fn arena_allocate_exact_capacity() {
        let arena = Arena::with_capacity(16);
        let slice = arena.alloc_bytes(16).unwrap();
        assert_eq!(slice.len(), 16);
        assert_eq!(arena.used(), 16);
        assert_eq!(arena.remaining(), 0);
        assert!(arena.alloc_bytes(1).is_none());
    }

    #[test]
    fn arena_reset_allows_reuse() {
        let arena = Arena::with_capacity(32);
        arena.alloc_bytes(32).unwrap();
        assert_eq!(arena.remaining(), 0);
        unsafe { arena.reset() };
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 32);
        assert!(arena.alloc_bytes(16).is_some());
    }

    #[test]
    fn arena_multiple_small_allocations() {
        let arena = Arena::with_capacity(10);
        for _ in 0..10 {
            assert!(arena.alloc_bytes(1).is_some());
        }
        assert!(arena.alloc_bytes(1).is_none());
    }

    // ── StackBuffer additional tests ──────────────────────────────────────────

    #[test]
    fn stack_buffer_default_is_empty() {
        let buffer = StackBuffer::<16>::default();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 16);
    }

    #[test]
    fn stack_buffer_from_slice_exact_capacity() {
        let buffer = StackBuffer::<5>::from_slice(b"hello").unwrap();
        assert_eq!(buffer.as_slice(), b"hello");
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn stack_buffer_from_slice_empty() {
        let buffer = StackBuffer::<8>::from_slice(b"").unwrap();
        assert!(buffer.is_empty());
        assert_eq!(buffer.as_slice(), b"");
    }

    #[test]
    fn stack_buffer_from_slice_too_large_returns_none() {
        assert!(StackBuffer::<3>::from_slice(b"abcd").is_none());
    }

    #[test]
    fn stack_buffer_as_mut_slice_allows_writes() {
        let mut buffer = StackBuffer::<5>::from_slice(b"aaaaa").unwrap();
        buffer.as_mut_slice()[0] = b'z';
        assert_eq!(buffer.as_slice()[0], b'z');
    }

    #[test]
    fn stack_buffer_extend_from_slice_partial_fill() {
        let mut buffer = StackBuffer::<10>::new();
        assert!(buffer.extend_from_slice(b"abc"));
        assert_eq!(buffer.len(), 3);
        assert!(buffer.extend_from_slice(b"de"));
        assert_eq!(buffer.as_slice(), b"abcde");
    }

    #[test]
    fn stack_buffer_extend_from_slice_too_large_leaves_buffer_unchanged() {
        let mut buffer = StackBuffer::<4>::new();
        assert!(buffer.extend_from_slice(b"ab"));
        assert!(!buffer.extend_from_slice(b"cde")); // would overflow
        // Content must be unchanged
        assert_eq!(buffer.as_slice(), b"ab");
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn stack_buffer_clear_then_reuse() {
        let mut buffer = StackBuffer::<8>::new();
        buffer.extend_from_slice(b"first");
        buffer.clear();
        assert!(buffer.is_empty());
        buffer.extend_from_slice(b"second");
        assert_eq!(buffer.as_slice(), b"second");
    }

    #[test]
    fn stack_buffer_push_until_full_clear_push_again() {
        let mut buffer = StackBuffer::<2>::new();
        assert!(buffer.push(b'x'));
        assert!(buffer.push(b'y'));
        assert!(!buffer.push(b'z')); // full
        buffer.clear();
        assert!(buffer.push(b'a'));
        assert_eq!(buffer.as_slice(), b"a");
    }

    #[test]
    fn stack_buffer_capacity_matches_const_generic() {
        assert_eq!(StackBuffer::<1>::new().capacity(), 1);
        assert_eq!(StackBuffer::<64>::new().capacity(), 64);
        assert_eq!(StackBuffer::<1024>::new().capacity(), 1024);
    }
}
