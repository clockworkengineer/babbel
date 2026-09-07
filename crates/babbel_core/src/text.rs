//! Line-oriented text processing, frontmatter extraction, and indentation manipulation.

use alloc::string::String;
use alloc::vec::Vec;

/// Supported frontmatter header formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFormat {
    /// YAML frontmatter delimited by opening and closing `---`.
    Yaml,
    /// TOML frontmatter delimited by opening and closing `+++`.
    Toml,
    /// Custom or unknown delimiter format.
    Custom(&'static str),
}

/// Represents a parsed document containing optional frontmatter metadata and body content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentWithFrontmatter<'a> {
    /// Raw frontmatter text, if present (excluding the delimiters).
    pub frontmatter: Option<&'a str>,
    /// Document body content following the closing frontmatter delimiter.
    pub content: &'a str,
    /// Format of the frontmatter, if detected.
    pub format: Option<FrontmatterFormat>,
}

/// Splits a document into frontmatter metadata and main body content.
///
/// Supports:
/// - YAML frontmatter: bounded by `---` on the first line and `---` (or `...`) on a subsequent line.
/// - TOML frontmatter: bounded by `+++` on the first line and `+++` on a subsequent line.
///
/// If no frontmatter delimiter is detected on the very first line, the entire text is returned as `content`.
///
/// # Examples
/// ```
/// use babbel_core::text::{split_frontmatter, FrontmatterFormat};
///
/// let doc = "---\ntitle: Hello\n---\n# World";
/// let parsed = split_frontmatter(doc);
/// assert_eq!(parsed.format, Some(FrontmatterFormat::Yaml));
/// assert_eq!(parsed.frontmatter, Some("title: Hello"));
/// assert_eq!(parsed.content, "# World");
/// ```
pub fn split_frontmatter(input: &str) -> DocumentWithFrontmatter<'_> {
    let trimmed = input.trim_start_matches('\u{feff}'); // strip BOM if any

    // Detect delimiter
    let (delim, format) = if trimmed.starts_with("---\r\n") || trimmed.starts_with("---\n") || trimmed == "---" {
        ("---", FrontmatterFormat::Yaml)
    } else if trimmed.starts_with("+++\r\n") || trimmed.starts_with("+++\n") || trimmed == "+++" {
        ("+++", FrontmatterFormat::Toml)
    } else {
        return DocumentWithFrontmatter {
            frontmatter: None,
            content: trimmed,
            format: None,
        };
    };

    // Find the end of the first line
    let first_line_end = match trimmed.find('\n') {
        Some(idx) => idx + 1,
        None => {
            // Only the delimiter exists
            return DocumentWithFrontmatter {
                frontmatter: None,
                content: trimmed,
                format: None,
            };
        }
    };

    let rest = &trimmed[first_line_end..];

    // Search for closing delimiter on its own line
    // Closing delimiter can be `---` or `...` for YAML, `+++` for TOML.
    let mut offset = 0;
    while offset < rest.len() {
        let line_slice = &rest[offset..];
        let line_len = line_slice.find('\n').map(|i| i + 1).unwrap_or(line_slice.len());
        let current_line = &line_slice[..line_len];
        let trimmed_line = current_line.trim_end_matches(&['\r', '\n'][..]);

        let is_closing = if format == FrontmatterFormat::Yaml {
            trimmed_line == delim || trimmed_line == "..."
        } else {
            trimmed_line == delim
        };

        if is_closing {
            // Frontmatter is rest[..offset], trimmed of trailing \r\n
            let fm_raw = &rest[..offset];
            let fm = fm_raw.trim_end_matches(&['\r', '\n'][..]);
            let content_start = offset + line_len;
            let content = if content_start <= rest.len() {
                // Strip leading single newline after closing delimiter if present
                let rem = &rest[content_start..];
                rem
            } else {
                ""
            };

            return DocumentWithFrontmatter {
                frontmatter: Some(fm),
                content,
                format: Some(format),
            };
        }

        offset += line_len;
    }

    // Closing delimiter was never found; entire input is treated as content
    DocumentWithFrontmatter {
        frontmatter: None,
        content: trimmed,
        format: None,
    }
}

/// Indents each non-empty line of the given string by `prefix`.
///
/// # Examples
/// ```
/// use babbel_core::text::indent;
///
/// let s = "foo\nbar\n";
/// assert_eq!(indent(s, "  "), "  foo\n  bar\n");
/// ```
pub fn indent(text: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(text.len() + prefix.len() * 4);
    let ends_with_newline = text.ends_with('\n');
    let content = if ends_with_newline {
        &text[..text.len() - 1]
    } else {
        text
    };

    let mut first = true;
    for line in content.split('\n') {
        let line_clean = line.strip_suffix('\r').unwrap_or(line);
        if !first {
            result.push('\n');
        }
        first = false;

        if !line_clean.is_empty() {
            result.push_str(prefix);
            result.push_str(line_clean);
        }
    }

    if ends_with_newline {
        result.push('\n');
    }

    result
}

/// Removes any common leading whitespace indentation from every line in `text`.
///
/// Empty and whitespace-only lines are ignored when calculating common indentation.
///
/// # Examples
/// ```
/// use babbel_core::text::dedent;
///
/// let s = "    foo\n    bar\n";
/// assert_eq!(dedent(s), "foo\nbar\n");
/// ```
pub fn dedent(text: &str) -> String {
    let ends_with_newline = text.ends_with('\n');
    let content = if ends_with_newline {
        &text[..text.len() - 1]
    } else {
        text
    };

    let lines: Vec<&str> = content
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();

    // Find common indentation among non-blank lines
    let mut min_indent: Option<usize> = None;

    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        min_indent = Some(match min_indent {
            Some(curr) => curr.min(indent),
            None => indent,
        });
    }

    let cut = min_indent.unwrap_or(0);
    if cut == 0 {
        return text.into();
    }

    let mut result = String::with_capacity(text.len());
    let mut first = true;

    for line in lines {
        if !first {
            result.push('\n');
        }
        first = false;

        if line.trim().is_empty() {
            // Keep blank lines empty
        } else if line.len() >= cut {
            result.push_str(&line[cut..]);
        } else {
            result.push_str(line);
        }
    }

    if ends_with_newline {
        result.push('\n');
    }

    result
}

/// Trims trailing whitespace from each line and trims leading and trailing empty lines from the document.
pub fn trim_lines(text: &str) -> String {
    let mut lines: Vec<&str> = text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l).trim_end())
        .collect();

    // Trim leading blank lines
    while !lines.is_empty() && lines[0].is_empty() {
        lines.remove(0);
    }

    // Trim trailing blank lines
    while !lines.is_empty() && lines[lines.len() - 1].is_empty() {
        lines.pop();
    }

    lines.join("\n")
}

/// Counts total number of lines in `text`.
pub fn line_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.split('\n').count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_frontmatter_yaml() {
        let input = "---\ntitle: Rust Guide\nauthor: Ferris\n---\n# Chapter 1\nHello world.";
        let doc = split_frontmatter(input);
        assert_eq!(doc.format, Some(FrontmatterFormat::Yaml));
        assert_eq!(doc.frontmatter, Some("title: Rust Guide\nauthor: Ferris"));
        assert_eq!(doc.content, "# Chapter 1\nHello world.");
    }

    #[test]
    fn test_split_frontmatter_yaml_with_crlf() {
        let input = "---\r\ntitle: Windows\r\n---\r\nBody text";
        let doc = split_frontmatter(input);
        assert_eq!(doc.format, Some(FrontmatterFormat::Yaml));
        assert_eq!(doc.frontmatter, Some("title: Windows"));
        assert_eq!(doc.content, "Body text");
    }

    #[test]
    fn test_split_frontmatter_toml() {
        let input = "+++\ntitle = \"Config\"\n+++\nContent here";
        let doc = split_frontmatter(input);
        assert_eq!(doc.format, Some(FrontmatterFormat::Toml));
        assert_eq!(doc.frontmatter, Some("title = \"Config\""));
        assert_eq!(doc.content, "Content here");
    }

    #[test]
    fn test_split_frontmatter_none() {
        let input = "Just markdown content\nwith multiple lines.";
        let doc = split_frontmatter(input);
        assert_eq!(doc.format, None);
        assert_eq!(doc.frontmatter, None);
        assert_eq!(doc.content, input);
    }

    #[test]
    fn test_indent() {
        let text = "alpha\nbeta\n\ngamma";
        let indented = indent(text, "  ");
        assert_eq!(indented, "  alpha\n  beta\n\n  gamma");
    }

    #[test]
    fn test_dedent() {
        let text = "    func hello() {\n        return 42;\n    }";
        let dedented = dedent(text);
        assert_eq!(dedented, "func hello() {\n    return 42;\n}");
    }

    #[test]
    fn test_trim_lines() {
        let text = "\n\n  hello world   \n  foo bar \t \n\n\n";
        let trimmed = trim_lines(text);
        assert_eq!(trimmed, "  hello world\n  foo bar");
    }

    #[test]
    fn test_line_count() {
        assert_eq!(line_count(""), 0);
        assert_eq!(line_count("hello"), 1);
        assert_eq!(line_count("hello\nworld"), 2);
    }
}
