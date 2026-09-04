//! Configuration options for bencode parsing and encoding

/// Configuration for the bencode parser
#[derive(Debug, Clone, Copy)]
pub struct ParserConfig {
    /// Maximum depth of nested structures (default: 100)
    /// Set to prevent stack overflow from malicious deeply nested data
    pub max_depth: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self { max_depth: 100 }
    }
}

impl ParserConfig {
    /// Create a new parser configuration with default settings
    pub const fn new() -> Self {
        Self { max_depth: 100 }
    }

    /// Set the maximum nesting depth
    pub const fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }
}

/// Configuration for the bencode encoder
#[derive(Debug, Clone, Copy)]
pub struct EncoderConfig {
    /// Enforce canonical bencode format (default: true)
    /// - Dictionary keys must be sorted
    /// - No leading zeros in integers (except "0")
    pub enforce_canonical: bool,

    /// Verify dictionary key ordering during encoding (default: true)
    pub verify_dict_order: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            enforce_canonical: true,
            verify_dict_order: true,
        }
    }
}

impl EncoderConfig {
    /// Create a new encoder configuration with default settings
    pub const fn new() -> Self {
        Self {
            enforce_canonical: true,
            verify_dict_order: true,
        }
    }

    /// Set whether to enforce canonical bencode format
    pub const fn with_canonical(mut self, enforce: bool) -> Self {
        self.enforce_canonical = enforce;
        self
    }

    /// Set whether to verify dictionary key ordering
    pub const fn with_dict_order_verification(mut self, verify: bool) -> Self {
        self.verify_dict_order = verify;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_config_default() {
        let config = ParserConfig::default();
        assert_eq!(config.max_depth, 100);
    }

    #[test]
    fn parser_config_with_max_depth() {
        let config = ParserConfig::new().with_max_depth(50);
        assert_eq!(config.max_depth, 50);
    }

    #[test]
    fn encoder_config_default() {
        let config = EncoderConfig::default();
        assert!(config.enforce_canonical);
        assert!(config.verify_dict_order);
    }

    #[test]
    fn encoder_config_custom() {
        let config = EncoderConfig::new()
            .with_canonical(false)
            .with_dict_order_verification(false);
        assert!(!config.enforce_canonical);
        assert!(!config.verify_dict_order);
    }

    // --- ParserConfig ---

    #[test]
    fn parser_config_new_equals_default() {
        let via_new = ParserConfig::new();
        let via_default = ParserConfig::default();
        assert_eq!(via_new.max_depth, via_default.max_depth);
    }

    #[test]
    fn parser_config_with_max_depth_zero() {
        let config = ParserConfig::new().with_max_depth(0);
        assert_eq!(config.max_depth, 0);
    }

    #[test]
    fn parser_config_with_max_depth_one() {
        let config = ParserConfig::new().with_max_depth(1);
        assert_eq!(config.max_depth, 1);
    }

    #[test]
    fn parser_config_with_large_max_depth() {
        let config = ParserConfig::new().with_max_depth(usize::MAX);
        assert_eq!(config.max_depth, usize::MAX);
    }

    #[test]
    fn parser_config_chaining_overwrites_previous() {
        let config = ParserConfig::new().with_max_depth(10).with_max_depth(200);
        assert_eq!(config.max_depth, 200);
    }

    #[test]
    fn parser_config_is_copy() {
        let a = ParserConfig::new().with_max_depth(42);
        let b = a; // Copy
        assert_eq!(a.max_depth, b.max_depth);
    }

    #[test]
    fn parser_config_clone_equals_original() {
        let a = ParserConfig::new().with_max_depth(77);
        let b = a.clone();
        assert_eq!(a.max_depth, b.max_depth);
    }

    #[test]
    fn parser_config_debug_contains_max_depth() {
        let config = ParserConfig::new().with_max_depth(55);
        let s = format!("{:?}", config);
        assert!(s.contains("55"));
    }

    // --- EncoderConfig ---

    #[test]
    fn encoder_config_new_equals_default() {
        let via_new = EncoderConfig::new();
        let via_default = EncoderConfig::default();
        assert_eq!(via_new.enforce_canonical, via_default.enforce_canonical);
        assert_eq!(via_new.verify_dict_order, via_default.verify_dict_order);
    }

    #[test]
    fn encoder_config_with_canonical_false_only() {
        let config = EncoderConfig::new().with_canonical(false);
        assert!(!config.enforce_canonical);
        assert!(config.verify_dict_order); // unchanged
    }

    #[test]
    fn encoder_config_with_dict_order_false_only() {
        let config = EncoderConfig::new().with_dict_order_verification(false);
        assert!(config.enforce_canonical); // unchanged
        assert!(!config.verify_dict_order);
    }

    #[test]
    fn encoder_config_with_canonical_true_explicit() {
        let config = EncoderConfig::new()
            .with_canonical(false)
            .with_canonical(true);
        assert!(config.enforce_canonical);
    }

    #[test]
    fn encoder_config_with_dict_order_true_explicit() {
        let config = EncoderConfig::new()
            .with_dict_order_verification(false)
            .with_dict_order_verification(true);
        assert!(config.verify_dict_order);
    }

    #[test]
    fn encoder_config_is_copy() {
        let a = EncoderConfig::new().with_canonical(false);
        let b = a; // Copy
        assert_eq!(a.enforce_canonical, b.enforce_canonical);
        assert_eq!(a.verify_dict_order, b.verify_dict_order);
    }

    #[test]
    fn encoder_config_clone_equals_original() {
        let a = EncoderConfig::new()
            .with_canonical(false)
            .with_dict_order_verification(false);
        let b = a.clone();
        assert_eq!(a.enforce_canonical, b.enforce_canonical);
        assert_eq!(a.verify_dict_order, b.verify_dict_order);
    }

    #[test]
    fn encoder_config_debug_contains_field_values() {
        let config = EncoderConfig::new().with_canonical(false);
        let s = format!("{:?}", config);
        assert!(s.contains("enforce_canonical"));
        assert!(s.contains("false"));
    }
}
