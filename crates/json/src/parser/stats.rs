//! Parser statistics and profiling for embedded systems
//!
//! Provides metrics about memory usage, allocation patterns, and performance
//! to help tune parser configuration for resource-constrained devices.

#[cfg(feature = "std")]
use std::time::Instant;

/// Statistics collected during JSON parsing
///
/// These metrics help developers understand actual resource usage
/// and tune ParserConfig for their specific use cases.
#[derive(Debug, Clone, Default)]
pub struct ParseStats {
    /// Maximum nesting depth reached during parsing
    pub peak_depth: usize,

    /// Total number of Node allocations
    pub total_nodes: usize,

    /// Number of string values parsed
    pub string_count: usize,

    /// Total bytes in all string values
    pub string_bytes: usize,

    /// Number of array elements parsed
    pub array_elements: usize,

    /// Number of object key-value pairs parsed
    pub object_pairs: usize,

    /// Largest array size encountered
    pub max_array_size: usize,

    /// Largest object size encountered
    pub max_object_size: usize,

    /// Longest string encountered (bytes)
    pub max_string_length: usize,

    /// Estimated total memory usage in bytes
    /// This is an approximation based on Node sizes and allocations
    pub estimated_memory_bytes: usize,

    /// Parse duration in microseconds (only available with std)
    #[cfg(feature = "std")]
    pub parse_time_us: Option<u64>,
}

impl ParseStats {
    /// Creates a new empty statistics tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Records entering a deeper nesting level
    pub fn enter_depth(&mut self, current_depth: usize) {
        if current_depth > self.peak_depth {
            self.peak_depth = current_depth;
        }
    }

    /// Records a string node allocation
    pub fn record_string(&mut self, len: usize) {
        self.total_nodes += 1;
        self.string_count += 1;
        self.string_bytes += len;
        if len > self.max_string_length {
            self.max_string_length = len;
        }
        // Estimate: Node enum (56 bytes) + String overhead (~24 bytes) + actual string data
        self.estimated_memory_bytes += 56 + 24 + len;
    }

    /// Records an array element
    pub fn record_array_element(&mut self, array_size: usize) {
        self.array_elements += 1;
        if array_size > self.max_array_size {
            self.max_array_size = array_size;
        }
    }

    /// Records an array node completion
    pub fn record_array(&mut self, size: usize) {
        self.total_nodes += 1;
        // Estimate: Node enum (56 bytes) + Vec overhead (~24 bytes) + element pointers
        self.estimated_memory_bytes += 56 + 24 + (size * 8);
    }

    /// Records an object key-value pair
    pub fn record_object_pair(&mut self, key_len: usize, object_size: usize) {
        self.object_pairs += 1;
        if object_size > self.max_object_size {
            self.max_object_size = object_size;
        }
        // Estimate: key string overhead + value pointer
        self.estimated_memory_bytes += 24 + key_len + 8;
    }

    /// Records an object node completion
    pub fn record_object(&mut self, size: usize) {
        self.total_nodes += 1;
        // Estimate: Node enum (56 bytes) + HashMap overhead (varies, ~96 bytes base + buckets)
        self.estimated_memory_bytes += 56 + 96 + (size * 32);
    }

    /// Records a simple node (number, boolean, null)
    pub fn record_simple_node(&mut self) {
        self.total_nodes += 1;
        // Estimate: Just the Node enum size
        self.estimated_memory_bytes += 56;
    }

    /// Returns a summary string of key statistics
    pub fn summary(&self) -> alloc::string::String {
        use alloc::format;
        format!(
            "ParseStats {{ depth: {}, nodes: {}, strings: {}, arrays: {}, objects: {}, memory: {} bytes }}",
            self.peak_depth,
            self.total_nodes,
            self.string_count,
            self.array_elements,
            self.object_pairs,
            self.estimated_memory_bytes
        )
    }

    /// Checks if parsed data fits within configuration limits
    pub fn fits_config(&self, config: &super::config::ParserConfig) -> bool {
        if let Some(max_depth) = config.max_depth {
            if self.peak_depth > max_depth {
                return false;
            }
        }
        if let Some(max_str_len) = config.max_string_length {
            if self.max_string_length > max_str_len {
                return false;
            }
        }
        if let Some(max_array) = config.max_array_size {
            if self.max_array_size > max_array {
                return false;
            }
        }
        if let Some(max_obj) = config.max_object_size {
            if self.max_object_size > max_obj {
                return false;
            }
        }
        true
    }
}

/// Timer for measuring parse duration (std only)
#[cfg(feature = "std")]
pub struct ParseTimer {
    start: Instant,
}

#[cfg(feature = "std")]
impl ParseTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::config::ParserConfig;

    #[test]
    fn test_stats_tracking() {
        let mut stats = ParseStats::new();

        stats.record_string(10);
        assert_eq!(stats.string_count, 1);
        assert_eq!(stats.string_bytes, 10);
        assert_eq!(stats.max_string_length, 10);

        stats.record_array_element(5);
        stats.record_array_element(5);
        assert_eq!(stats.array_elements, 2);
        assert_eq!(stats.max_array_size, 5);

        stats.record_object_pair(4, 2);
        assert_eq!(stats.object_pairs, 1);
        assert_eq!(stats.max_object_size, 2);

        assert!(stats.estimated_memory_bytes > 0);
    }

    #[test]
    fn test_depth_tracking() {
        let mut stats = ParseStats::new();

        stats.enter_depth(1);
        stats.enter_depth(3);
        stats.enter_depth(2);

        assert_eq!(stats.peak_depth, 3);
    }

    #[test]
    fn test_fits_config() {
        let mut stats = ParseStats::new();
        stats.peak_depth = 5;
        stats.max_string_length = 100;
        stats.max_array_size = 50;
        stats.max_object_size = 20;

        let config = ParserConfig::new();
        assert!(stats.fits_config(&config));

        let strict = ParserConfig::strict();
        assert!(stats.fits_config(&strict)); // All values fit within strict limits

        // Test exceeding limits
        stats.max_string_length = 300; // Exceeds strict's 256 byte limit
        assert!(!stats.fits_config(&strict));
    }

    #[test]
    fn test_summary() {
        let mut stats = ParseStats::new();
        stats.total_nodes = 10;
        stats.string_count = 3;

        let summary = stats.summary();
        assert!(summary.contains("nodes: 10"));
        assert!(summary.contains("strings: 3"));
    }

    // ParseStats::new / Default
    #[test]
    fn test_new_all_zeros() {
        let stats = ParseStats::new();
        assert_eq!(stats.peak_depth, 0);
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.string_count, 0);
        assert_eq!(stats.string_bytes, 0);
        assert_eq!(stats.array_elements, 0);
        assert_eq!(stats.object_pairs, 0);
        assert_eq!(stats.max_array_size, 0);
        assert_eq!(stats.max_object_size, 0);
        assert_eq!(stats.max_string_length, 0);
        assert_eq!(stats.estimated_memory_bytes, 0);
    }

    #[test]
    fn test_default_equals_new() {
        let a = ParseStats::new();
        let b = ParseStats::default();
        assert_eq!(a.peak_depth, b.peak_depth);
        assert_eq!(a.total_nodes, b.total_nodes);
        assert_eq!(a.string_count, b.string_count);
        assert_eq!(a.estimated_memory_bytes, b.estimated_memory_bytes);
    }

    // enter_depth
    #[test]
    fn test_enter_depth_only_goes_up() {
        let mut stats = ParseStats::new();
        stats.enter_depth(5);
        assert_eq!(stats.peak_depth, 5);
        stats.enter_depth(3); // lower — should not change
        assert_eq!(stats.peak_depth, 5);
        stats.enter_depth(5); // same — should not change
        assert_eq!(stats.peak_depth, 5);
    }

    #[test]
    fn test_enter_depth_zero() {
        let mut stats = ParseStats::new();
        stats.enter_depth(0);
        assert_eq!(stats.peak_depth, 0);
    }

    #[test]
    fn test_enter_depth_sequential() {
        let mut stats = ParseStats::new();
        for d in 0..=10 {
            stats.enter_depth(d);
        }
        assert_eq!(stats.peak_depth, 10);
    }

    // record_string
    #[test]
    fn test_record_string_increments_count() {
        let mut stats = ParseStats::new();
        stats.record_string(5);
        stats.record_string(10);
        assert_eq!(stats.string_count, 2);
        assert_eq!(stats.total_nodes, 2);
    }

    #[test]
    fn test_record_string_accumulates_bytes() {
        let mut stats = ParseStats::new();
        stats.record_string(4);
        stats.record_string(6);
        assert_eq!(stats.string_bytes, 10);
    }

    #[test]
    fn test_record_string_tracks_max() {
        let mut stats = ParseStats::new();
        stats.record_string(3);
        stats.record_string(100);
        stats.record_string(50);
        assert_eq!(stats.max_string_length, 100);
    }

    #[test]
    fn test_record_string_zero_length() {
        let mut stats = ParseStats::new();
        stats.record_string(0);
        assert_eq!(stats.string_count, 1);
        assert_eq!(stats.string_bytes, 0);
        assert_eq!(stats.max_string_length, 0);
        assert!(stats.estimated_memory_bytes > 0); // Node + String overhead still counted
    }

    #[test]
    fn test_record_string_memory_grows() {
        let mut stats = ParseStats::new();
        let before = stats.estimated_memory_bytes;
        stats.record_string(20);
        assert!(stats.estimated_memory_bytes > before);
    }

    // record_array_element
    #[test]
    fn test_record_array_element_count() {
        let mut stats = ParseStats::new();
        stats.record_array_element(1);
        stats.record_array_element(2);
        stats.record_array_element(3);
        assert_eq!(stats.array_elements, 3);
    }

    #[test]
    fn test_record_array_element_max_size() {
        let mut stats = ParseStats::new();
        stats.record_array_element(10);
        stats.record_array_element(5);
        stats.record_array_element(10); // same max, should remain
        assert_eq!(stats.max_array_size, 10);
    }

    #[test]
    fn test_record_array_element_does_not_add_node() {
        // record_array_element should NOT increment total_nodes (only record_array does)
        let mut stats = ParseStats::new();
        stats.record_array_element(3);
        assert_eq!(stats.total_nodes, 0);
    }

    // record_array
    #[test]
    fn test_record_array_adds_node() {
        let mut stats = ParseStats::new();
        stats.record_array(3);
        assert_eq!(stats.total_nodes, 1);
    }

    #[test]
    fn test_record_array_memory_grows() {
        let mut stats = ParseStats::new();
        let before = stats.estimated_memory_bytes;
        stats.record_array(5);
        assert!(stats.estimated_memory_bytes > before);
    }

    #[test]
    fn test_record_array_empty() {
        let mut stats = ParseStats::new();
        let before = stats.estimated_memory_bytes;
        stats.record_array(0);
        assert_eq!(stats.total_nodes, 1);
        assert!(stats.estimated_memory_bytes > before); // overhead still added
    }

    // record_object_pair
    #[test]
    fn test_record_object_pair_count() {
        let mut stats = ParseStats::new();
        stats.record_object_pair(3, 1);
        stats.record_object_pair(5, 2);
        assert_eq!(stats.object_pairs, 2);
    }

    #[test]
    fn test_record_object_pair_max_size() {
        let mut stats = ParseStats::new();
        stats.record_object_pair(4, 1);
        stats.record_object_pair(4, 8);
        stats.record_object_pair(4, 3);
        assert_eq!(stats.max_object_size, 8);
    }

    #[test]
    fn test_record_object_pair_memory_grows() {
        let mut stats = ParseStats::new();
        let before = stats.estimated_memory_bytes;
        stats.record_object_pair(10, 5);
        assert!(stats.estimated_memory_bytes > before);
    }

    // record_object
    #[test]
    fn test_record_object_adds_node() {
        let mut stats = ParseStats::new();
        stats.record_object(2);
        assert_eq!(stats.total_nodes, 1);
    }

    #[test]
    fn test_record_object_memory_grows() {
        let mut stats = ParseStats::new();
        let before = stats.estimated_memory_bytes;
        stats.record_object(3);
        assert!(stats.estimated_memory_bytes > before);
    }

    // record_simple_node
    #[test]
    fn test_record_simple_node_increments() {
        let mut stats = ParseStats::new();
        stats.record_simple_node();
        stats.record_simple_node();
        assert_eq!(stats.total_nodes, 2);
    }

    #[test]
    fn test_record_simple_node_memory() {
        let mut stats = ParseStats::new();
        let before = stats.estimated_memory_bytes;
        stats.record_simple_node();
        assert_eq!(stats.estimated_memory_bytes, before + 56);
    }

    // summary
    #[test]
    fn test_summary_contains_all_fields() {
        let mut stats = ParseStats::new();
        stats.peak_depth = 4;
        stats.total_nodes = 7;
        stats.string_count = 2;
        stats.array_elements = 5;
        stats.object_pairs = 3;
        stats.estimated_memory_bytes = 1024;
        let s = stats.summary();
        assert!(s.contains("depth: 4"));
        assert!(s.contains("nodes: 7"));
        assert!(s.contains("strings: 2"));
        assert!(s.contains("arrays: 5"));
        assert!(s.contains("objects: 3"));
        assert!(s.contains("memory: 1024"));
    }

    #[test]
    fn test_summary_empty_stats() {
        let stats = ParseStats::new();
        let s = stats.summary();
        assert!(s.contains("ParseStats"));
        assert!(s.contains("0"));
    }

    // fits_config with specific limits
    #[test]
    fn test_fits_config_unlimited() {
        let mut stats = ParseStats::new();
        stats.peak_depth = 1000;
        stats.max_string_length = 10_000;
        stats.max_array_size = 10_000;
        stats.max_object_size = 10_000;
        assert!(stats.fits_config(&ParserConfig::unlimited()));
    }

    #[test]
    fn test_fits_config_depth_exceeded() {
        let mut stats = ParseStats::new();
        stats.peak_depth = 10;
        let config = ParserConfig::new().with_max_depth(Some(5));
        assert!(!stats.fits_config(&config));
    }

    #[test]
    fn test_fits_config_depth_exactly_at_limit() {
        let mut stats = ParseStats::new();
        stats.peak_depth = 5;
        let config = ParserConfig::new().with_max_depth(Some(5));
        assert!(stats.fits_config(&config));
    }

    #[test]
    fn test_fits_config_string_exceeded() {
        let mut stats = ParseStats::new();
        stats.max_string_length = 300;
        let config = ParserConfig::new().with_max_string_length(Some(200));
        assert!(!stats.fits_config(&config));
    }

    #[test]
    fn test_fits_config_string_at_limit() {
        let mut stats = ParseStats::new();
        stats.max_string_length = 200;
        let config = ParserConfig::new().with_max_string_length(Some(200));
        assert!(stats.fits_config(&config));
    }

    #[test]
    fn test_fits_config_array_exceeded() {
        let mut stats = ParseStats::new();
        stats.max_array_size = 101;
        let config = ParserConfig::new().with_max_array_size(Some(100));
        assert!(!stats.fits_config(&config));
    }

    #[test]
    fn test_fits_config_object_exceeded() {
        let mut stats = ParseStats::new();
        stats.max_object_size = 51;
        let config = ParserConfig::new().with_max_object_size(Some(50));
        assert!(!stats.fits_config(&config));
    }

    #[test]
    fn test_fits_config_object_at_limit() {
        let mut stats = ParseStats::new();
        stats.max_object_size = 50;
        let config = ParserConfig::new().with_max_object_size(Some(50));
        assert!(stats.fits_config(&config));
    }

    // Clone and Debug
    #[test]
    fn test_clone_copies_all_fields() {
        let mut stats = ParseStats::new();
        stats.peak_depth = 7;
        stats.total_nodes = 42;
        stats.string_bytes = 512;
        let cloned = stats.clone();
        assert_eq!(cloned.peak_depth, 7);
        assert_eq!(cloned.total_nodes, 42);
        assert_eq!(cloned.string_bytes, 512);
    }

    #[test]
    fn test_debug_format_contains_struct_name() {
        let stats = ParseStats::new();
        let debug = alloc::format!("{:?}", stats);
        assert!(debug.contains("ParseStats"));
    }

    // ParseTimer (std only)
    #[cfg(feature = "std")]
    #[test]
    fn test_parse_timer_elapsed_non_negative() {
        let timer = ParseTimer::new();
        let elapsed = timer.elapsed_us();
        // elapsed_us returns u64 so it's always >= 0; just confirm it compiles and runs
        let _ = elapsed;
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_timer_elapsed_increases() {
        use std::thread;
        use std::time::Duration;
        let timer = ParseTimer::new();
        thread::sleep(Duration::from_millis(1));
        assert!(timer.elapsed_us() > 0);
    }
}
