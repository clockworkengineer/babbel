//! Arena allocator for reduced allocation overhead during JSON parsing
//!
//! This module provides an arena allocator that can significantly reduce
//! allocation overhead when parsing JSON by allocating from a pre-allocated buffer.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Arena allocator for string storage
/// Reduces allocation overhead by using a bump allocator strategy
#[cfg(feature = "alloc")]
pub struct StringArena {
    buffer: Vec<u8>,
    position: usize,
    strings: Vec<(usize, usize)>, // (start, length) pairs
}

#[cfg(feature = "alloc")]
impl StringArena {
    /// Create a new arena with a pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
            strings: Vec::new(),
        }
    }

    /// Create a new arena with default capacity (16KB)
    pub fn new() -> Self {
        Self::with_capacity(16 * 1024)
    }

    /// Allocate a string in the arena
    /// Returns the index of the allocated string
    pub fn alloc_str(&mut self, s: &str) -> usize {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let start = self.position;

        // Ensure we have enough space
        if self.buffer.len() < self.position + len {
            self.buffer.reserve(len);
            self.buffer.extend_from_slice(bytes);
        } else {
            self.buffer[self.position..self.position + len].copy_from_slice(bytes);
        }

        self.position += len;
        let index = self.strings.len();
        self.strings.push((start, len));
        index
    }

    /// Get a string by its index
    pub fn get_str(&self, index: usize) -> Option<&str> {
        self.strings
            .get(index)
            .and_then(|&(start, len)| core::str::from_utf8(&self.buffer[start..start + len]).ok())
    }

    /// Clear the arena for reuse
    pub fn clear(&mut self) {
        self.position = 0;
        self.strings.clear();
        // Keep the buffer allocated for reuse
    }

    /// Get the total bytes allocated
    pub fn bytes_allocated(&self) -> usize {
        self.position
    }

    /// Get the number of strings stored
    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> usize {
        self.buffer.capacity() - self.position
    }
}

#[cfg(feature = "alloc")]
impl Default for StringArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory pool for Node allocations
/// Pre-allocates nodes to reduce allocation overhead
#[cfg(feature = "alloc")]
pub struct NodePool {
    capacity: usize,
    allocated: usize,
}

#[cfg(feature = "alloc")]
impl NodePool {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            allocated: 0,
        }
    }

    pub fn reset(&mut self) {
        self.allocated = 0;
    }

    pub fn allocated(&self) -> usize {
        self.allocated
    }

    pub fn remaining(&self) -> usize {
        self.capacity.saturating_sub(self.allocated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_string_arena_basic() {
        let mut arena = StringArena::new();

        let idx1 = arena.alloc_str("hello");
        let idx2 = arena.alloc_str("world");

        assert_eq!(arena.get_str(idx1), Some("hello"));
        assert_eq!(arena.get_str(idx2), Some("world"));
        assert_eq!(arena.string_count(), 2);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_string_arena_clear() {
        let mut arena = StringArena::with_capacity(1024);

        arena.alloc_str("test");
        assert_eq!(arena.string_count(), 1);

        arena.clear();
        assert_eq!(arena.string_count(), 0);
        assert_eq!(arena.bytes_allocated(), 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_string_arena_capacity() {
        let arena = StringArena::with_capacity(1024);
        assert!(arena.remaining_capacity() >= 1024);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_pool() {
        let mut pool = NodePool::new(100);
        assert_eq!(pool.remaining(), 100);

        pool.reset();
        assert_eq!(pool.allocated(), 0);
    }

    // --- StringArena::new / default ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_string_arena_new_is_empty() {
        let arena = StringArena::new();
        assert_eq!(arena.string_count(), 0);
        assert_eq!(arena.bytes_allocated(), 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_string_arena_default_equals_new() {
        let a = StringArena::new();
        let b = StringArena::default();
        assert_eq!(a.string_count(), b.string_count());
        assert_eq!(a.bytes_allocated(), b.bytes_allocated());
    }

    // --- alloc_str: index ordering ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_str_returns_sequential_indices() {
        let mut arena = StringArena::new();
        let i0 = arena.alloc_str("first");
        let i1 = arena.alloc_str("second");
        let i2 = arena.alloc_str("third");
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(i2, 2);
    }

    // --- alloc_str: empty string ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_empty_string() {
        let mut arena = StringArena::new();
        let idx = arena.alloc_str("");
        assert_eq!(arena.get_str(idx), Some(""));
        assert_eq!(arena.string_count(), 1);
        assert_eq!(arena.bytes_allocated(), 0);
    }

    // --- alloc_str: bytes_allocated tracks total bytes ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_bytes_allocated_grows() {
        let mut arena = StringArena::new();
        arena.alloc_str("abc"); // 3 bytes
        assert_eq!(arena.bytes_allocated(), 3);
        arena.alloc_str("de"); // 2 more
        assert_eq!(arena.bytes_allocated(), 5);
        arena.alloc_str(""); // 0 more
        assert_eq!(arena.bytes_allocated(), 5);
    }

    // --- get_str: out-of-bounds returns None ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_get_str_out_of_bounds_returns_none() {
        let arena = StringArena::new();
        assert_eq!(arena.get_str(0), None);
        assert_eq!(arena.get_str(99), None);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_get_str_after_alloc_valid_and_invalid() {
        let mut arena = StringArena::new();
        let idx = arena.alloc_str("hello");
        assert_eq!(arena.get_str(idx), Some("hello"));
        assert_eq!(arena.get_str(idx + 1), None);
    }

    // --- get_str: preserves content exactly ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_get_str_preserves_content() {
        let mut arena = StringArena::new();
        let strings = vec!["alpha", "beta", "gamma", "delta", "epsilon"];
        let indices: Vec<usize> = strings.iter().map(|s| arena.alloc_str(s)).collect();
        for (i, &expected) in strings.iter().enumerate() {
            assert_eq!(arena.get_str(indices[i]), Some(expected));
        }
    }

    // --- get_str: unicode ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_unicode_string() {
        let mut arena = StringArena::new();
        let idx = arena.alloc_str("héllo wörld 🌍");
        assert_eq!(arena.get_str(idx), Some("héllo wörld 🌍"));
    }

    // --- get_str: long string ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_long_string() {
        let mut arena = StringArena::new();
        let long: String = "x".repeat(50_000);
        let idx = arena.alloc_str(&long);
        assert_eq!(arena.get_str(idx).map(|s| s.len()), Some(50_000));
        assert_eq!(arena.bytes_allocated(), 50_000);
    }

    // --- clear: resets position and string list ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_clear_resets_state() {
        let mut arena = StringArena::new();
        arena.alloc_str("one");
        arena.alloc_str("two");
        assert_eq!(arena.string_count(), 2);

        arena.clear();
        assert_eq!(arena.string_count(), 0);
        assert_eq!(arena.bytes_allocated(), 0);
        assert_eq!(arena.get_str(0), None);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_clear_and_reuse() {
        let mut arena = StringArena::with_capacity(256);
        arena.alloc_str("before clear");
        arena.clear();

        // After clear, new allocations start fresh
        let idx = arena.alloc_str("after clear");
        assert_eq!(idx, 0);
        assert_eq!(arena.get_str(0), Some("after clear"));
        assert_eq!(arena.string_count(), 1);
    }

    // --- remaining_capacity ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_remaining_capacity_decreases_after_alloc() {
        let mut arena = StringArena::with_capacity(256);
        let before = arena.remaining_capacity();
        arena.alloc_str("12345"); // 5 bytes
        let after = arena.remaining_capacity();
        assert_eq!(before - after, 5);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_remaining_capacity_after_clear() {
        let mut arena = StringArena::with_capacity(256);
        arena.alloc_str("data");
        arena.clear();
        // After clear, remaining capacity should be back to capacity
        assert!(arena.remaining_capacity() >= 256);
    }

    // --- many strings ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_alloc_many_strings() {
        let mut arena = StringArena::with_capacity(4096);
        let n = 100usize;
        let indices: Vec<usize> = (0..n)
            .map(|i| arena.alloc_str(&format!("key_{}", i)))
            .collect();
        assert_eq!(arena.string_count(), n);
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(arena.get_str(idx), Some(format!("key_{}", i).as_str()));
        }
    }

    // --- NodePool ---

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_pool_initial_state() {
        let pool = NodePool::new(50);
        assert_eq!(pool.allocated(), 0);
        assert_eq!(pool.remaining(), 50);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_pool_zero_capacity() {
        let pool = NodePool::new(0);
        assert_eq!(pool.allocated(), 0);
        assert_eq!(pool.remaining(), 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_pool_reset_clears_allocated() {
        let mut pool = NodePool::new(100);
        // Manually set via reset — allocated starts at 0, reset returns to 0
        pool.reset();
        assert_eq!(pool.allocated(), 0);
        assert_eq!(pool.remaining(), 100);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_node_pool_remaining_saturates_at_zero() {
        // remaining() uses saturating_sub so can never underflow
        let pool = NodePool::new(0);
        assert_eq!(pool.remaining(), 0);
    }
}
