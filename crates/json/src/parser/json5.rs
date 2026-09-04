//! JSON5 comment stripping
//!
//! Provides utilities to strip single-line (//) and multi-line (/* */) comments
//! from JSON5-style input before parsing as standard JSON.

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Strip JSON5-style comments from input
///
/// Removes:
/// - Single-line comments: // comment
/// - Multi-line comments: /* comment */
/// - Trailing commas are NOT handled (that requires full JSON5 parser)
///
/// # Arguments
/// * `input` - The JSON5 string with comments
///
/// # Returns
/// * JSON string with comments removed
///
/// # Examples
/// ```
/// use json_lib::parser::json5::strip_comments;
///
/// let input = r#"{
///     "name": "Alice", // This is a name
///     /* Multi-line
///        comment */
///     "age": 30
/// }"#;
/// let clean = strip_comments(input);
/// ```
pub fn strip_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Check for single-line comment
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Skip until end of line
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            // Keep the newline
            if i < bytes.len() {
                output.push('\n');
                i += 1;
            }
            continue;
        }

        // Check for multi-line comment
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Skip until */
            i += 2;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                // Preserve newlines in multi-line comments to maintain line numbers
                if bytes[i] == b'\n' {
                    output.push('\n');
                }
                i += 1;
            }
            continue;
        }

        // Check for string literals (don't strip comments inside strings)
        if bytes[i] == b'"' {
            output.push('"');
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    // Escaped character
                    output.push(bytes[i] as char);
                    output.push(bytes[i + 1] as char);
                    i += 2;
                } else if bytes[i] == b'"' {
                    // End of string
                    output.push('"');
                    i += 1;
                    break;
                } else {
                    output.push(bytes[i] as char);
                    i += 1;
                }
            }
            continue;
        }

        // Regular character
        output.push(bytes[i] as char);
        i += 1;
    }

    output
}

/// Strip JSON5 comments and parse as JSON
///
/// Convenience function that strips comments and then parses the result.
///
/// # Arguments
/// * `input` - JSON5 string with comments
///
/// # Returns
/// * Parsed Node or error
pub fn parse_json5(input: &str) -> Result<crate::nodes::node::Node, String> {
    let clean = strip_comments(input);
    crate::parser::default::from_str(&clean)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_single_line_comments() {
        let input = r#"{
            "name": "Alice", // This is a comment
            "age": 30
        }"#;
        let output = strip_comments(input);
        assert!(!output.contains("// This is a comment"));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"age\""));
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let input = r#"{
            "name": "Alice",
            /* This is a
               multi-line comment */
            "age": 30
        }"#;
        let output = strip_comments(input);
        assert!(!output.contains("/* This is a"));
        assert!(!output.contains("multi-line comment */"));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"age\""));
    }

    #[test]
    fn test_preserve_strings_with_comment_like_content() {
        let input = r#"{
            "url": "http://example.com",
            "comment": "This // is not a comment"
        }"#;
        let output = strip_comments(input);
        assert!(output.contains("http://example.com"));
        assert!(output.contains("This // is not a comment"));
    }

    #[test]
    fn test_multiple_comments() {
        let input = r#"{
            // First comment
            "a": 1,
            // Second comment
            "b": 2 /* inline comment */
        }"#;
        let output = strip_comments(input);
        assert!(!output.contains("First comment"));
        assert!(!output.contains("Second comment"));
        assert!(!output.contains("inline comment"));
        assert!(output.contains("\"a\""));
        assert!(output.contains("\"b\""));
    }

    #[test]
    fn test_preserve_line_numbers() {
        let input = "{\n// comment\n\"a\": 1\n}";
        let output = strip_comments(input);
        // Should have the same number of newlines
        assert_eq!(input.matches('\n').count(), output.matches('\n').count());
    }

    #[test]
    fn test_parse_json5() {
        let input = r#"{
            // Configuration file
            "host": "localhost",
            "port": 8080 /* default port */
        }"#;
        let result = parse_json5(input);
        assert!(result.is_ok());
        let node = result.unwrap();
        assert_eq!(node["host"].as_str(), Some("localhost"));
        assert_eq!(node["port"].as_i64(), Some(8080));
    }

    #[test]
    fn test_escaped_quotes_in_strings() {
        let input = r#"{"message": "He said \"hello\" // not a comment"}"#;
        let output = strip_comments(input);
        assert!(output.contains(r#"He said \"hello\" // not a comment"#));
    }

    // strip_comments edge cases
    #[test]
    fn test_strip_empty_input() {
        assert_eq!(strip_comments(""), "");
    }

    #[test]
    fn test_strip_no_comments() {
        let input = r#"{"a": 1, "b": "hello"}"#;
        let output = strip_comments(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_single_line_comment_at_start() {
        let input = "// leading comment\n{\"a\":1}";
        let output = strip_comments(input);
        assert!(!output.contains("leading comment"));
        assert!(output.contains("{\"a\":1}"));
    }

    #[test]
    fn test_single_line_comment_no_trailing_newline() {
        // Comment at end of file with no newline after
        let input = "{\"a\":1} // trailing comment";
        let output = strip_comments(input);
        assert!(!output.contains("trailing comment"));
        assert!(output.contains("{\"a\":1}"));
    }

    #[test]
    fn test_multi_line_comment_inline() {
        let input = r#"{"a": /* value */ 1}"#;
        let output = strip_comments(input);
        assert!(!output.contains("value"));
        assert!(output.contains("\"a\""));
        assert!(output.contains('1'));
    }

    #[test]
    fn test_multi_line_comment_preserves_newline_count() {
        let input = "{\n/* line1\nline2\nline3 */\n\"a\":1\n}";
        let output = strip_comments(input);
        // Newlines inside multi-line comments are preserved
        assert_eq!(input.matches('\n').count(), output.matches('\n').count());
    }

    #[test]
    fn test_slash_star_inside_string() {
        let input = r#"{"regex": "a /* b"}"#;
        let output = strip_comments(input);
        assert!(output.contains("a /* b"));
    }

    #[test]
    fn test_double_slash_inside_string() {
        let input = r#"{"url": "https://example.com/path"}"#;
        let output = strip_comments(input);
        assert!(output.contains("https://example.com/path"));
    }

    #[test]
    fn test_comment_after_string_value() {
        let input = r#"{"key": "value"} // end"#;
        let output = strip_comments(input);
        assert!(!output.contains("end"));
        assert!(output.contains("\"value\""));
    }

    #[test]
    fn test_output_capacity_hint() {
        // Output should never be longer than input (comments only removed)
        let input = "// comment\n{\"a\":1}";
        let output = strip_comments(input);
        assert!(output.len() <= input.len() + 5); // +5 for preserved newlines
    }

    #[test]
    fn test_strip_only_comments() {
        let input = "// just a comment";
        let output = strip_comments(input);
        assert!(!output.contains("just a comment"));
    }

    #[test]
    fn test_strip_block_comment_only() {
        let input = "/* entire file is a comment */";
        let output = strip_comments(input);
        assert!(!output.contains("entire"));
        assert_eq!(output.trim(), "");
    }

    #[test]
    fn test_adjacent_comments() {
        let input = "// comment1\n// comment2\n{\"a\":1}";
        let output = strip_comments(input);
        assert!(!output.contains("comment1"));
        assert!(!output.contains("comment2"));
        assert!(output.contains("{\"a\":1}"));
    }

    #[test]
    fn test_string_with_escaped_backslash_then_quote() {
        // "a\\" — escaped backslash followed by closing quote
        let input = r#"{"k": "a\\"}"#;
        let output = strip_comments(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_comment_between_keys() {
        let input = "{\"a\":1,/* between */\"b\":2}";
        let output = strip_comments(input);
        assert!(!output.contains("between"));
        assert!(output.contains("\"a\""));
        assert!(output.contains("\"b\""));
    }

    // parse_json5 edge cases
    #[test]
    fn test_parse_json5_empty_object() {
        let input = "{ /* nothing */ }";
        let result = parse_json5(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_json5_array_with_comments() {
        let input = "[\n  1, // first\n  2, // second\n  3\n]";
        let result = parse_json5(input);
        assert!(result.is_ok());
        let node = result.unwrap();
        assert_eq!(node.len(), Some(3));
    }

    #[test]
    fn test_parse_json5_nested_with_comments() {
        let input = r#"{
            "outer": {
                // inner comment
                "inner": true
            }
        }"#;
        let result = parse_json5(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_json5_all_types() {
        let input = r#"{
            "s": "hello", // string
            "n": 42,      /* number */
            "f": 3.14,
            "b": true,
            "nil": null,
            "arr": [1,2,3]
        }"#;
        let result = parse_json5(input);
        assert!(result.is_ok());
        let node = result.unwrap();
        assert_eq!(node["s"].as_str(), Some("hello"));
        assert_eq!(node["n"].as_i64(), Some(42));
        assert!(node["b"].is_boolean());
        assert!(node["nil"].is_null());
    }

    #[test]
    fn test_parse_json5_invalid_json_after_strip() {
        // Malformed JSON even after stripping comments
        let result = parse_json5("{ // comment\n bad }");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_json5_boolean_with_comments() {
        assert!(matches!(parse_json5("/* comment */ true"), Ok(n) if n.is_boolean()));
        assert!(matches!(parse_json5("false // comment"), Ok(n) if n.is_boolean()));
    }

    #[test]
    fn test_parse_json5_null_with_comment() {
        let result = parse_json5("/* before */ null");
        assert!(result.is_ok());
        assert!(result.unwrap().is_null());
    }

    #[test]
    fn test_parse_json5_url_in_value() {
        let input = r#"{"url": "https://example.com"}"#;
        let result = parse_json5(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["url"].as_str(), Some("https://example.com"));
    }
}
