//! Small String Optimization (SSO) for reducing heap allocations
//!
//! This module provides a SmallString type that stores short strings inline
//! to avoid heap allocations for common small strings like keys in JSON objects.

use core::fmt;

/// Size of inline storage (23 bytes + 1 byte for length on 64-bit systems)
/// This allows strings up to 23 bytes to be stored without heap allocation
const INLINE_CAPACITY: usize = 23;

/// A string that stores small strings inline to avoid heap allocations
#[cfg(feature = "alloc")]
pub enum SmallString {
    /// Inline storage for strings up to INLINE_CAPACITY bytes
    Inline {
        len: u8,
        data: [u8; INLINE_CAPACITY],
    },
    /// Heap-allocated storage for larger strings
    Heap(alloc::string::String),
}

#[cfg(feature = "alloc")]
impl SmallString {
    /// Create a new SmallString from a string slice
    pub fn new(s: &str) -> Self {
        let bytes = s.as_bytes();
        if bytes.len() <= INLINE_CAPACITY {
            let mut data = [0u8; INLINE_CAPACITY];
            data[..bytes.len()].copy_from_slice(bytes);
            SmallString::Inline {
                len: bytes.len() as u8,
                data,
            }
        } else {
            SmallString::Heap(s.to_string())
        }
    }

    /// Get the string as a &str
    pub fn as_str(&self) -> &str {
        match self {
            SmallString::Inline { len, data } => {
                core::str::from_utf8(&data[..*len as usize]).unwrap()
            }
            SmallString::Heap(s) => s.as_str(),
        }
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        match self {
            SmallString::Inline { len, .. } => *len as usize,
            SmallString::Heap(s) => s.len(),
        }
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the string is stored inline (no heap allocation)
    pub fn is_inline(&self) -> bool {
        matches!(self, SmallString::Inline { .. })
    }

    /// Convert to a regular String
    pub fn into_string(self) -> alloc::string::String {
        match self {
            SmallString::Inline { .. } => self.as_str().to_string(),
            SmallString::Heap(s) => s,
        }
    }
}

#[cfg(feature = "alloc")]
impl From<&str> for SmallString {
    fn from(s: &str) -> Self {
        SmallString::new(s)
    }
}

#[cfg(feature = "alloc")]
impl From<alloc::string::String> for SmallString {
    fn from(s: alloc::string::String) -> Self {
        if s.len() <= INLINE_CAPACITY {
            SmallString::new(&s)
        } else {
            SmallString::Heap(s)
        }
    }
}

#[cfg(feature = "alloc")]
impl fmt::Display for SmallString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(feature = "alloc")]
impl fmt::Debug for SmallString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SmallString({:?})", self.as_str())
    }
}

#[cfg(feature = "alloc")]
impl PartialEq for SmallString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(feature = "alloc")]
impl Clone for SmallString {
    fn clone(&self) -> Self {
        match self {
            SmallString::Inline { len, data } => SmallString::Inline {
                len: *len,
                data: *data,
            },
            SmallString::Heap(s) => SmallString::Heap(s.clone()),
        }
    }
}

/// Statistics about SmallString usage
#[cfg(feature = "alloc")]
pub struct SmallStringStats {
    pub inline_count: usize,
    pub heap_count: usize,
    pub total_bytes_saved: usize,
}

#[cfg(feature = "alloc")]
impl SmallStringStats {
    pub fn new() -> Self {
        Self {
            inline_count: 0,
            heap_count: 0,
            total_bytes_saved: 0,
        }
    }

    pub fn record_string(&mut self, s: &SmallString) {
        if s.is_inline() {
            self.inline_count += 1;
            // Each inline string saves approximately 24 bytes (String header overhead)
            self.total_bytes_saved += 24;
        } else {
            self.heap_count += 1;
        }
    }

    pub fn inline_percentage(&self) -> f64 {
        let total = self.inline_count + self.heap_count;
        if total == 0 {
            0.0
        } else {
            (self.inline_count as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_inline() {
        let s = SmallString::new("hello");
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_heap() {
        let long = "this is a very long string that exceeds inline capacity";
        let s = SmallString::new(long);
        assert!(!s.is_inline());
        assert_eq!(s.as_str(), long);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_boundary() {
        // Test string exactly at inline capacity
        let s = SmallString::new("12345678901234567890123");
        assert!(s.is_inline());
        assert_eq!(s.len(), 23);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_stats() {
        let mut stats = SmallStringStats::new();

        let s1 = SmallString::new("short");
        stats.record_string(&s1);

        let s2 = SmallString::new("this is a very long string that needs heap allocation");
        stats.record_string(&s2);

        assert_eq!(stats.inline_count, 1);
        assert_eq!(stats.heap_count, 1);
        assert!(stats.inline_percentage() > 0.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_equality() {
        let s1 = SmallString::new("test");
        let s2 = SmallString::new("test");
        assert_eq!(s1, s2);
    }

    // SmallString::new — construction and classification
    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_empty() {
        let s = SmallString::new("");
        assert!(s.is_inline());
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.as_str(), "");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_single_char() {
        let s = SmallString::new("x");
        assert!(s.is_inline());
        assert_eq!(s.len(), 1);
        assert_eq!(s.as_str(), "x");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_exactly_one_over_capacity() {
        // 24 bytes → must go to heap
        let s = SmallString::new("123456789012345678901234");
        assert!(!s.is_inline());
        assert_eq!(s.len(), 24);
        assert_eq!(s.as_str(), "123456789012345678901234");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_long_roundtrip() {
        let long = "a".repeat(128);
        let s = SmallString::new(&long);
        assert!(!s.is_inline());
        assert_eq!(s.as_str(), long.as_str());
        assert_eq!(s.len(), 128);
    }

    // as_str and len consistency
    #[cfg(feature = "alloc")]
    #[test]
    fn test_as_str_matches_len_inline() {
        let s = SmallString::new("abcdef");
        assert_eq!(s.as_str().len(), s.len());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_as_str_matches_len_heap() {
        let long = "z".repeat(50);
        let s = SmallString::new(&long);
        assert_eq!(s.as_str().len(), s.len());
    }

    // Unicode and multibyte characters
    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_unicode_inline() {
        // "café" = 5 bytes in UTF-8 (é = 2 bytes) → fits inline
        let s = SmallString::new("café");
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "café");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_small_string_unicode_only_ascii_digits() {
        let s = SmallString::new("12345678901234567890123");
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "12345678901234567890123");
    }

    // is_empty
    #[cfg(feature = "alloc")]
    #[test]
    fn test_is_empty_false_for_nonempty() {
        let s = SmallString::new("a");
        assert!(!s.is_empty());
        let s2 = SmallString::new("this is longer than twenty-three bytes total");
        assert!(!s2.is_empty());
    }

    // into_string
    #[cfg(feature = "alloc")]
    #[test]
    fn test_into_string_inline() {
        let s = SmallString::new("hello");
        let owned = s.into_string();
        assert_eq!(owned, "hello");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_into_string_heap() {
        let long = "this string is definitely longer than twenty-three bytes";
        let s = SmallString::new(long);
        let owned = s.into_string();
        assert_eq!(owned.as_str(), long);
    }

    // From<&str>
    #[cfg(feature = "alloc")]
    #[test]
    fn test_from_str_inline() {
        let s = SmallString::from("key");
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "key");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_from_str_heap() {
        let long = "this is a string that will definitely not fit inline storage";
        let s = SmallString::from(long);
        assert!(!s.is_inline());
        assert_eq!(s.as_str(), long);
    }

    // From<String>
    #[cfg(feature = "alloc")]
    #[test]
    fn test_from_string_inline() {
        let owned = alloc::string::String::from("value");
        let s = SmallString::from(owned);
        assert!(s.is_inline());
        assert_eq!(s.as_str(), "value");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_from_string_heap() {
        let long = alloc::string::String::from("this_string_is_longer_than_twenty_three_bytes");
        let s = SmallString::from(long.clone());
        assert!(!s.is_inline());
        assert_eq!(s.as_str(), long.as_str());
    }

    // Clone
    #[cfg(feature = "alloc")]
    #[test]
    fn test_clone_inline() {
        let s = SmallString::new("clone me");
        let c = s.clone();
        assert_eq!(s, c);
        assert!(c.is_inline());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_clone_heap() {
        let long = "this string needs to be allocated on the heap definitely";
        let s = SmallString::new(long);
        let c = s.clone();
        assert_eq!(s, c);
        assert!(!c.is_inline());
    }

    // PartialEq — inline vs heap with same content
    #[cfg(feature = "alloc")]
    #[test]
    fn test_equality_inline_same() {
        assert_eq!(SmallString::new("abc"), SmallString::new("abc"));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_inequality_different_content() {
        assert_ne!(SmallString::new("abc"), SmallString::new("xyz"));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_inequality_different_lengths() {
        assert_ne!(SmallString::new("a"), SmallString::new("ab"));
    }

    // Display and Debug
    #[cfg(feature = "alloc")]
    #[test]
    fn test_display_inline() {
        let s = SmallString::new("hello");
        assert_eq!(alloc::format!("{}", s), "hello");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_display_heap() {
        let long = "a string that is longer than twenty-three characters for sure";
        let s = SmallString::new(long);
        assert_eq!(alloc::format!("{}", s), long);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_debug_contains_content() {
        let s = SmallString::new("test");
        let debug = alloc::format!("{:?}", s);
        assert!(debug.contains("test"));
        assert!(debug.contains("SmallString"));
    }

    // SmallStringStats
    #[cfg(feature = "alloc")]
    #[test]
    fn test_stats_new_zeros() {
        let stats = SmallStringStats::new();
        assert_eq!(stats.inline_count, 0);
        assert_eq!(stats.heap_count, 0);
        assert_eq!(stats.total_bytes_saved, 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stats_inline_percentage_empty() {
        let stats = SmallStringStats::new();
        assert_eq!(stats.inline_percentage(), 0.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stats_all_inline() {
        let mut stats = SmallStringStats::new();
        for _ in 0..4 {
            let s = SmallString::new("short");
            stats.record_string(&s);
        }
        assert_eq!(stats.inline_count, 4);
        assert_eq!(stats.heap_count, 0);
        assert_eq!(stats.inline_percentage(), 100.0);
        assert_eq!(stats.total_bytes_saved, 4 * 24);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stats_all_heap() {
        let mut stats = SmallStringStats::new();
        for _ in 0..3 {
            let long = SmallString::new("this is a string definitely longer than 23 bytes");
            stats.record_string(&long);
        }
        assert_eq!(stats.inline_count, 0);
        assert_eq!(stats.heap_count, 3);
        assert_eq!(stats.total_bytes_saved, 0);
        assert_eq!(stats.inline_percentage(), 0.0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stats_mixed_percentage() {
        let mut stats = SmallStringStats::new();
        for _ in 0..3 {
            stats.record_string(&SmallString::new("hi"));
        }
        stats.record_string(&SmallString::new(
            "this one definitely goes to the heap storage ok",
        ));
        // 3 inline out of 4 total = 75%
        assert!((stats.inline_percentage() - 75.0).abs() < 0.001);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_stats_bytes_saved_accumulate() {
        let mut stats = SmallStringStats::new();
        stats.record_string(&SmallString::new("a"));
        stats.record_string(&SmallString::new("b"));
        assert_eq!(stats.total_bytes_saved, 48); // 2 * 24
    }

    // INLINE_CAPACITY constant
    #[test]
    fn test_inline_capacity_value() {
        assert_eq!(INLINE_CAPACITY, 23);
    }
}
