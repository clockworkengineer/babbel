//! Memory bounds utilities using const generics for embedded systems.
//!
//! This module provides compile-time memory calculations to help developers
//! understand and control memory usage when parsing bencode data.

/// Memory requirements calculator for bencode structures using const generics.
///
/// Since Rust's type system requires heap allocation for recursive types like
/// nested lists/dictionaries, this module helps you calculate memory bounds
/// when using the various parsing strategies.
pub struct MemoryBounds;

impl MemoryBounds {
    /// Calculates stack space needed for a StackBuffer with const generic size.
    ///
    /// # Example
    /// ```
    /// use bencode_lib::MemoryBounds;
    ///
    /// const BUFFER_SIZE: usize = 256;
    /// const STACK_BYTES: usize = MemoryBounds::stack_buffer_size(BUFFER_SIZE);
    /// // STACK_BYTES is known at compile time
    /// ```
    pub const fn stack_buffer_size(capacity: usize) -> usize {
        capacity + core::mem::size_of::<usize>() // data array + length field
    }

    /// Estimates heap bytes for parse_borrowed() based on structure complexity.
    ///
    /// This provides a conservative upper bound for memory planning.
    ///
    /// # Arguments
    /// * `num_nodes` - Total number of nodes (integers + strings + containers)
    /// * `num_containers` - Number of lists and dictionaries
    /// * `avg_container_size` - Average items per container
    ///
    /// # Returns
    /// Estimated heap bytes needed
    pub const fn borrowed_parse_estimate(
        num_nodes: usize,
        num_containers: usize,
        avg_container_size: usize,
    ) -> usize {
        // Base node size (enum discriminant + data)
        let node_overhead = num_nodes * 24;

        // Vec/HashMap capacity overhead
        let container_overhead = num_containers * avg_container_size * 16;

        node_overhead + container_overhead
    }

    /// Calculates maximum nesting depth safe for a given stack size.
    ///
    /// Each level of nesting adds stack frames during parsing.
    /// Use this to ensure stack safety on embedded systems.
    ///
    /// # Arguments
    /// * `stack_bytes` - Available stack space in bytes
    /// * `bytes_per_frame` - Stack frame size (typically 64-256 bytes)
    ///
    /// # Returns
    /// Maximum safe nesting depth
    pub const fn max_safe_depth(stack_bytes: usize, bytes_per_frame: usize) -> usize {
        if bytes_per_frame == 0 {
            return 0;
        }
        // Reserve 50% for safety margin
        (stack_bytes / 2) / bytes_per_frame
    }
}

/// Type alias demonstrating const generic buffer sizing.
///
/// # Example
/// ```
/// use bencode_lib::FixedSizeBuffer;
///
/// // Create a 512-byte buffer at compile time
/// type My512ByteBuffer = FixedSizeBuffer<512>;
/// let buf = My512ByteBuffer::new();
/// ```
pub type FixedSizeBuffer<const N: usize> = crate::StackBuffer<N>;

/// Compile-time assertion that a buffer size is sufficient.
///
/// Use this macro to ensure your buffer sizes are adequate at compile time.
///
/// # Example
/// ```
/// use bencode_lib::assert_buffer_size;
///
/// const MIN_SIZE: usize = 256;
/// const ACTUAL_SIZE: usize = 512;
///
/// // Compile-time assertion
/// assert_buffer_size!(ACTUAL_SIZE, MIN_SIZE);
/// ```
#[macro_export]
macro_rules! assert_buffer_size {
    ($size:expr, $min:expr) => {
        const _: () = assert!($size >= $min, "Buffer size too small");
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_bounds_stack_buffer() {
        const SIZE: usize = MemoryBounds::stack_buffer_size(256);
        assert_eq!(SIZE, 256 + core::mem::size_of::<usize>());
    }

    #[test]
    fn memory_bounds_estimate() {
        // Structure with 10 nodes, 2 containers, avg 5 items each
        let estimate = MemoryBounds::borrowed_parse_estimate(10, 2, 5);

        // 10 nodes * 24 + 2 containers * 5 items * 16
        assert_eq!(estimate, 10 * 24 + 2 * 5 * 16);
    }

    #[test]
    fn max_safe_depth_calculation() {
        const STACK_SIZE: usize = 8192; // 8KB stack
        const FRAME_SIZE: usize = 128;

        let depth = MemoryBounds::max_safe_depth(STACK_SIZE, FRAME_SIZE);

        // Should allow reasonable nesting
        assert!(depth >= 16);
        assert!(depth <= 64);
    }

    #[test]
    fn fixed_size_buffer_type() {
        let _buf: FixedSizeBuffer<256> = FixedSizeBuffer::new();
        assert_eq!(FixedSizeBuffer::<256>::new().capacity(), 256);
    }

    // ── stack_buffer_size ─────────────────────────────────────────────────────

    #[test]
    fn stack_buffer_size_zero_capacity() {
        const SIZE: usize = MemoryBounds::stack_buffer_size(0);
        assert_eq!(SIZE, core::mem::size_of::<usize>());
    }

    #[test]
    fn stack_buffer_size_one_byte() {
        const SIZE: usize = MemoryBounds::stack_buffer_size(1);
        assert_eq!(SIZE, 1 + core::mem::size_of::<usize>());
    }

    #[test]
    fn stack_buffer_size_large_capacity() {
        const SIZE: usize = MemoryBounds::stack_buffer_size(65536);
        assert_eq!(SIZE, 65536 + core::mem::size_of::<usize>());
    }

    #[test]
    fn stack_buffer_size_is_const_evaluable() {
        // Verifies that the function can be used in a const context
        const _: usize = MemoryBounds::stack_buffer_size(128);
    }

    #[test]
    fn stack_buffer_size_grows_linearly_with_capacity() {
        const A: usize = MemoryBounds::stack_buffer_size(100);
        const B: usize = MemoryBounds::stack_buffer_size(200);
        assert_eq!(B - A, 100);
    }

    // ── borrowed_parse_estimate ───────────────────────────────────────────────

    #[test]
    fn borrowed_parse_estimate_all_zeros() {
        assert_eq!(MemoryBounds::borrowed_parse_estimate(0, 0, 0), 0);
    }

    #[test]
    fn borrowed_parse_estimate_only_nodes_no_containers() {
        let est = MemoryBounds::borrowed_parse_estimate(5, 0, 0);
        assert_eq!(est, 5 * 24);
    }

    #[test]
    fn borrowed_parse_estimate_only_containers_no_nodes() {
        let est = MemoryBounds::borrowed_parse_estimate(0, 3, 10);
        assert_eq!(est, 3 * 10 * 16);
    }

    #[test]
    fn borrowed_parse_estimate_single_node() {
        let est = MemoryBounds::borrowed_parse_estimate(1, 0, 0);
        assert_eq!(est, 24);
    }

    #[test]
    fn borrowed_parse_estimate_scales_with_nodes() {
        let est_a = MemoryBounds::borrowed_parse_estimate(10, 0, 0);
        let est_b = MemoryBounds::borrowed_parse_estimate(20, 0, 0);
        assert_eq!(est_b, est_a * 2);
    }

    #[test]
    fn borrowed_parse_estimate_scales_with_containers() {
        let est_a = MemoryBounds::borrowed_parse_estimate(0, 1, 10);
        let est_b = MemoryBounds::borrowed_parse_estimate(0, 2, 10);
        assert_eq!(est_b, est_a * 2);
    }

    #[test]
    fn borrowed_parse_estimate_is_const_evaluable() {
        const _: usize = MemoryBounds::borrowed_parse_estimate(5, 2, 3);
    }

    // ── max_safe_depth ────────────────────────────────────────────────────────

    #[test]
    fn max_safe_depth_zero_frame_size_returns_zero() {
        assert_eq!(MemoryBounds::max_safe_depth(8192, 0), 0);
    }

    #[test]
    fn max_safe_depth_zero_stack_returns_zero() {
        assert_eq!(MemoryBounds::max_safe_depth(0, 128), 0);
    }

    #[test]
    fn max_safe_depth_respects_fifty_percent_safety_margin() {
        // With 200 bytes stack and 10-byte frames: (200 / 2) / 10 = 10
        assert_eq!(MemoryBounds::max_safe_depth(200, 10), 10);
    }

    #[test]
    fn max_safe_depth_large_frame_limits_depth() {
        // 8 KB stack, huge 4 KB frames → very shallow
        let depth = MemoryBounds::max_safe_depth(8192, 4096);
        assert_eq!(depth, 1);
    }

    #[test]
    fn max_safe_depth_exact_fit() {
        // (100 / 2) / 10 = 5
        assert_eq!(MemoryBounds::max_safe_depth(100, 10), 5);
    }

    #[test]
    fn max_safe_depth_is_const_evaluable() {
        const _: usize = MemoryBounds::max_safe_depth(8192, 128);
    }

    // ── FixedSizeBuffer type alias ────────────────────────────────────────────

    #[test]
    fn fixed_size_buffer_is_empty_on_creation() {
        let buf = FixedSizeBuffer::<64>::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn fixed_size_buffer_capacity_matches_const_param() {
        assert_eq!(FixedSizeBuffer::<1>::new().capacity(), 1);
        assert_eq!(FixedSizeBuffer::<512>::new().capacity(), 512);
        assert_eq!(FixedSizeBuffer::<1024>::new().capacity(), 1024);
    }

    #[test]
    fn fixed_size_buffer_push_and_read() {
        let mut buf = FixedSizeBuffer::<4>::new();
        assert!(buf.push(b'i'));
        assert!(buf.push(b'4'));
        assert!(buf.push(b'2'));
        assert!(buf.push(b'e'));
        assert_eq!(buf.as_slice(), b"i42e");
    }

    #[test]
    fn fixed_size_buffer_push_when_full_returns_false() {
        let mut buf = FixedSizeBuffer::<2>::new();
        assert!(buf.push(b'a'));
        assert!(buf.push(b'b'));
        assert!(!buf.push(b'c'));
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn fixed_size_buffer_clear_resets_length() {
        let mut buf = FixedSizeBuffer::<8>::new();
        buf.push(b'x');
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn fixed_size_buffer_from_slice_works() {
        let buf = FixedSizeBuffer::<5>::from_slice(b"hello").unwrap();
        assert_eq!(buf.as_slice(), b"hello");
    }

    #[test]
    fn fixed_size_buffer_from_slice_too_large_returns_none() {
        assert!(FixedSizeBuffer::<3>::from_slice(b"abcd").is_none());
    }
}
