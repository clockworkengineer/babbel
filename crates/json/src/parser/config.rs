//! Parser configuration for controlling memory usage and preventing resource exhaustion
//! Provides limits for parsing depth, string sizes, and collection sizes

/// Configuration for JSON parser to control resource usage
///
/// Default values are chosen for embedded systems with limited resources:
/// - Max depth: 32 levels (prevents stack overflow)
/// - Max string length: 4096 bytes (prevents excessive memory allocation)
/// - Max array size: 1024 elements
/// - Max object size: 256 key-value pairs
#[derive(Debug, Clone, Copy)]
pub struct ParserConfig {
    /// Maximum nesting depth for objects and arrays
    /// Set to None for unlimited depth (not recommended for embedded systems)
    pub max_depth: Option<usize>,

    /// Maximum length of a string value in bytes
    /// Set to None for unlimited length (not recommended for embedded systems)
    pub max_string_length: Option<usize>,

    /// Maximum number of elements in an array
    /// Set to None for unlimited size (not recommended for embedded systems)
    pub max_array_size: Option<usize>,

    /// Maximum number of key-value pairs in an object
    /// Set to None for unlimited size (not recommended for embedded systems)
    pub max_object_size: Option<usize>,
}

impl ParserConfig {
    /// Creates a new parser configuration with default limits suitable for embedded systems
    ///
    /// Default limits:
    /// - Max depth: 32 levels
    /// - Max string length: 4096 bytes
    /// - Max array size: 1024 elements
    /// - Max object size: 256 key-value pairs
    pub fn new() -> Self {
        Self {
            max_depth: Some(32),
            max_string_length: Some(4096),
            max_array_size: Some(1024),
            max_object_size: Some(256),
        }
    }

    /// Creates a configuration with no limits (uses default behavior)
    ///
    /// Warning: This is not recommended for embedded systems as it can lead to
    /// stack overflow or excessive memory consumption
    pub fn unlimited() -> Self {
        Self {
            max_depth: None,
            max_string_length: None,
            max_array_size: None,
            max_object_size: None,
        }
    }

    /// Creates a very strict configuration for highly constrained embedded systems
    ///
    /// Strict limits:
    /// - Max depth: 16 levels
    /// - Max string length: 256 bytes
    /// - Max array size: 64 elements
    /// - Max object size: 32 key-value pairs
    pub fn strict() -> Self {
        Self {
            max_depth: Some(16),
            max_string_length: Some(256),
            max_array_size: Some(64),
            max_object_size: Some(32),
        }
    }

    /// Sets the maximum nesting depth
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets the maximum string length in bytes
    pub fn with_max_string_length(mut self, max_string_length: Option<usize>) -> Self {
        self.max_string_length = max_string_length;
        self
    }

    /// Sets the maximum array size
    pub fn with_max_array_size(mut self, max_array_size: Option<usize>) -> Self {
        self.max_array_size = max_array_size;
        self
    }

    /// Sets the maximum object size
    pub fn with_max_object_size(mut self, max_object_size: Option<usize>) -> Self {
        self.max_object_size = max_object_size;
        self
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ParserConfig::default();
        assert_eq!(config.max_depth, Some(32));
        assert_eq!(config.max_string_length, Some(4096));
        assert_eq!(config.max_array_size, Some(1024));
        assert_eq!(config.max_object_size, Some(256));
    }

    #[test]
    fn test_unlimited_config() {
        let config = ParserConfig::unlimited();
        assert_eq!(config.max_depth, None);
        assert_eq!(config.max_string_length, None);
        assert_eq!(config.max_array_size, None);
        assert_eq!(config.max_object_size, None);
    }

    #[test]
    fn test_strict_config() {
        let config = ParserConfig::strict();
        assert_eq!(config.max_depth, Some(16));
        assert_eq!(config.max_string_length, Some(256));
        assert_eq!(config.max_array_size, Some(64));
        assert_eq!(config.max_object_size, Some(32));
    }

    #[test]
    fn test_builder_pattern() {
        let config = ParserConfig::new()
            .with_max_depth(Some(10))
            .with_max_string_length(Some(100))
            .with_max_array_size(Some(50))
            .with_max_object_size(Some(25));

        assert_eq!(config.max_depth, Some(10));
        assert_eq!(config.max_string_length, Some(100));
        assert_eq!(config.max_array_size, Some(50));
        assert_eq!(config.max_object_size, Some(25));
    }

    // --- new() matches default() ---

    #[test]
    fn test_new_equals_default() {
        let a = ParserConfig::new();
        let b = ParserConfig::default();
        assert_eq!(a.max_depth, b.max_depth);
        assert_eq!(a.max_string_length, b.max_string_length);
        assert_eq!(a.max_array_size, b.max_array_size);
        assert_eq!(a.max_object_size, b.max_object_size);
    }

    // --- clone / copy ---

    #[test]
    fn test_clone_preserves_values() {
        let original = ParserConfig::new()
            .with_max_depth(Some(8))
            .with_max_string_length(Some(512));
        let cloned = original; // Copy
        assert_eq!(cloned.max_depth, Some(8));
        assert_eq!(cloned.max_string_length, Some(512));
        assert_eq!(cloned.max_array_size, original.max_array_size);
        assert_eq!(cloned.max_object_size, original.max_object_size);
    }

    // --- builder: setting each field individually from new() ---

    #[test]
    fn test_with_max_depth_only() {
        let config = ParserConfig::new().with_max_depth(Some(5));
        assert_eq!(config.max_depth, Some(5));
        // Other fields unchanged from new()
        assert_eq!(config.max_string_length, Some(4096));
        assert_eq!(config.max_array_size, Some(1024));
        assert_eq!(config.max_object_size, Some(256));
    }

    #[test]
    fn test_with_max_string_length_only() {
        let config = ParserConfig::new().with_max_string_length(Some(128));
        assert_eq!(config.max_string_length, Some(128));
        assert_eq!(config.max_depth, Some(32));
        assert_eq!(config.max_array_size, Some(1024));
        assert_eq!(config.max_object_size, Some(256));
    }

    #[test]
    fn test_with_max_array_size_only() {
        let config = ParserConfig::new().with_max_array_size(Some(8));
        assert_eq!(config.max_array_size, Some(8));
        assert_eq!(config.max_depth, Some(32));
        assert_eq!(config.max_string_length, Some(4096));
        assert_eq!(config.max_object_size, Some(256));
    }

    #[test]
    fn test_with_max_object_size_only() {
        let config = ParserConfig::new().with_max_object_size(Some(10));
        assert_eq!(config.max_object_size, Some(10));
        assert_eq!(config.max_depth, Some(32));
        assert_eq!(config.max_string_length, Some(4096));
        assert_eq!(config.max_array_size, Some(1024));
    }

    // --- builder: setting None to remove limits ---

    #[test]
    fn test_with_max_depth_none_removes_limit() {
        let config = ParserConfig::new().with_max_depth(None);
        assert_eq!(config.max_depth, None);
    }

    #[test]
    fn test_with_max_string_length_none_removes_limit() {
        let config = ParserConfig::new().with_max_string_length(None);
        assert_eq!(config.max_string_length, None);
    }

    #[test]
    fn test_with_max_array_size_none_removes_limit() {
        let config = ParserConfig::new().with_max_array_size(None);
        assert_eq!(config.max_array_size, None);
    }

    #[test]
    fn test_with_max_object_size_none_removes_limit() {
        let config = ParserConfig::new().with_max_object_size(None);
        assert_eq!(config.max_object_size, None);
    }

    // --- builder: chaining from unlimited() ---

    #[test]
    fn test_builder_from_unlimited_adds_limits() {
        let config = ParserConfig::unlimited()
            .with_max_depth(Some(64))
            .with_max_array_size(Some(512));
        assert_eq!(config.max_depth, Some(64));
        assert_eq!(config.max_array_size, Some(512));
        assert_eq!(config.max_string_length, None); // still unlimited
        assert_eq!(config.max_object_size, None); // still unlimited
    }

    // --- builder: chaining from strict() ---

    #[test]
    fn test_builder_from_strict_overrides_limit() {
        let config = ParserConfig::strict().with_max_depth(Some(100));
        assert_eq!(config.max_depth, Some(100));
        // Other strict values unchanged
        assert_eq!(config.max_string_length, Some(256));
        assert_eq!(config.max_array_size, Some(64));
        assert_eq!(config.max_object_size, Some(32));
    }

    // --- builder: all fields set to None from new() ---

    #[test]
    fn test_builder_all_none_equals_unlimited() {
        let built = ParserConfig::new()
            .with_max_depth(None)
            .with_max_string_length(None)
            .with_max_array_size(None)
            .with_max_object_size(None);
        let unlimited = ParserConfig::unlimited();
        assert_eq!(built.max_depth, unlimited.max_depth);
        assert_eq!(built.max_string_length, unlimited.max_string_length);
        assert_eq!(built.max_array_size, unlimited.max_array_size);
        assert_eq!(built.max_object_size, unlimited.max_object_size);
    }

    // --- builder: zero values ---

    #[test]
    fn test_builder_zero_limits() {
        let config = ParserConfig::new()
            .with_max_depth(Some(0))
            .with_max_string_length(Some(0))
            .with_max_array_size(Some(0))
            .with_max_object_size(Some(0));
        assert_eq!(config.max_depth, Some(0));
        assert_eq!(config.max_string_length, Some(0));
        assert_eq!(config.max_array_size, Some(0));
        assert_eq!(config.max_object_size, Some(0));
    }

    // --- builder: large values ---

    #[test]
    fn test_builder_large_limits() {
        let config = ParserConfig::unlimited()
            .with_max_depth(Some(usize::MAX))
            .with_max_string_length(Some(usize::MAX));
        assert_eq!(config.max_depth, Some(usize::MAX));
        assert_eq!(config.max_string_length, Some(usize::MAX));
    }

    // --- Debug trait ---

    #[test]
    fn test_debug_output_contains_field_names() {
        let config = ParserConfig::new();
        let s = format!("{:?}", config);
        assert!(s.contains("max_depth"));
        assert!(s.contains("max_string_length"));
        assert!(s.contains("max_array_size"));
        assert!(s.contains("max_object_size"));
    }

    #[test]
    fn test_debug_unlimited_shows_none() {
        let config = ParserConfig::unlimited();
        let s = format!("{:?}", config);
        assert!(s.contains("None"));
    }

    // --- strict(): values are tighter than new() ---

    #[test]
    fn test_strict_tighter_than_default() {
        let strict = ParserConfig::strict();
        let default = ParserConfig::new();
        assert!(strict.max_depth.unwrap() < default.max_depth.unwrap());
        assert!(strict.max_string_length.unwrap() < default.max_string_length.unwrap());
        assert!(strict.max_array_size.unwrap() < default.max_array_size.unwrap());
        assert!(strict.max_object_size.unwrap() < default.max_object_size.unwrap());
    }

    // --- builder is non-destructive: original is unaffected (Copy) ---

    #[test]
    fn test_builder_does_not_mutate_original() {
        let original = ParserConfig::new();
        let _modified = original.with_max_depth(Some(1));
        // original is Copy, so it's unaffected
        assert_eq!(original.max_depth, Some(32));
    }
}
