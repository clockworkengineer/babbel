#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::{Parser, is_kdl_newline, is_kdl_whitespace};
use crate::error::KdlError;

impl<'a> Parser<'a> {
    pub(crate) fn is_multiline_string_start(&self) -> bool {
        let mut idx = 0;
        while self.peek_at(idx) == Some('#') {
            idx += 1;
        }
        self.peek_at(idx) == Some('"')
            && self.peek_at(idx + 1) == Some('"')
            && self.peek_at(idx + 2) == Some('"')
    }

    pub(crate) fn parse_multiline_string(&mut self) -> Result<String, KdlError> {
        let start_line = self.line;
        let start_col = self.col;

        let mut hash_count = 0;
        while self.peek() == Some('#') {
            self.advance();
            hash_count += 1;
        }

        // Consume """
        for _ in 0..3 {
            self.advance();
        }

        // After opening """, must be followed by optional whitespace, then newline
        while let Some(c) = self.peek() {
            if is_kdl_whitespace(c) {
                self.advance();
            } else {
                break;
            }
        }

        if let Some(c) = self.peek() {
            if c == '\r' {
                self.advance();
                if self.peek() == Some('\n') {
                    self.advance();
                }
            } else if c == '\n' || is_kdl_newline(c) {
                self.advance();
            } else {
                return Err(KdlError::Syntax {
                    message: "Multiline string opening '\"\"\"' must be followed by a newline"
                        .to_string(),
                    line: start_line,
                    col: start_col,
                });
            }
        } else {
            return Err(KdlError::UnexpectedEof);
        }

        // Collect lines until closing delimiter
        let mut raw_lines: Vec<String> = Vec::new();
        let mut continuation_lines: Vec<bool> = Vec::new();
        let indent_prefix: String;

        loop {
            if self.is_eof() {
                return Err(KdlError::Syntax {
                    message: "Unclosed multiline string".to_string(),
                    line: start_line,
                    col: start_col,
                });
            }

            // Check if current line contains closing delimiter
            let line_start_pos = self.pos;
            let mut prefix = String::new();
            while let Some(c) = self.peek() {
                if is_kdl_whitespace(c) {
                    prefix.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let mut lookahead = 0;
            let mut has_escaped_indent = false;
            if self.peek() == Some('\\') && is_kdl_whitespace(self.peek_at(1).unwrap_or('\0')) {
                lookahead = 1;
                while let Some(c) = self.peek_at(lookahead) {
                    if is_kdl_whitespace(c) {
                        lookahead += 1;
                    } else {
                        break;
                    }
                }
                has_escaped_indent = true;
            }

            let is_close = self.peek_at(lookahead) == Some('"')
                && self.peek_at(lookahead + 1) == Some('"')
                && self.peek_at(lookahead + 2) == Some('"')
                && (0..hash_count).all(|h| self.peek_at(lookahead + 3 + h) == Some('#'));

            if is_close {
                // If previous line ended with \, check whether it had non-whitespace before \
                if let Some(prev) = raw_lines.last() {
                    let trimmed = prev.trim_end_matches(|c| is_kdl_whitespace(c));
                    if trimmed.ends_with('\\') {
                        let before_bs = trimmed[..trimmed.len() - 1].trim();
                        if !before_bs.is_empty() {
                            return Err(KdlError::Syntax {
                                message: "Closing multiline string delimiter cannot be escaped by preceding line".to_string(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                    }
                }

                if has_escaped_indent {
                    for _ in 0..lookahead {
                        self.advance();
                    }
                }
                for _ in 0..(3 + hash_count) {
                    self.advance();
                }

                // If previous line was only whitespace followed by \, combine into prefix
                if let Some(prev) = raw_lines.pop() {
                    let trimmed = prev.trim_end_matches(|c| is_kdl_whitespace(c));
                    if trimmed.ends_with('\\') && trimmed[..trimmed.len() - 1].trim().is_empty() {
                        prefix = trimmed[..trimmed.len() - 1].to_string();
                        continuation_lines.pop();
                    } else {
                        raw_lines.push(prev);
                    }
                }

                indent_prefix = prefix;
                break;
            } else {
                // Rewind to start of line and read the entire line
                self.pos = line_start_pos;
                let mut line = String::new();
                while let Some(c) = self.peek() {
                    if c == '\r' {
                        self.advance();
                        if self.peek() == Some('\n') {
                            self.advance();
                        }
                        break;
                    } else if c == '\n' || is_kdl_newline(c) {
                        self.advance();
                        break;
                    } else {
                        line.push(c);
                        self.advance();
                    }
                }

                let is_cont = raw_lines.last().map_or(false, |prev| {
                    let trimmed = prev.trim_end_matches(|c| is_kdl_whitespace(c));
                    let bs_count = trimmed.chars().rev().take_while(|c| *c == '\\').count();
                    bs_count % 2 == 1
                });
                continuation_lines.push(is_cont);
                raw_lines.push(line);
            }
        }

        // Strip indent_prefix from all lines
        let mut dedented_lines = Vec::new();
        for (i, line) in raw_lines.iter().enumerate() {
            let is_cont = continuation_lines.get(i).copied().unwrap_or(false);
            let is_all_ws = line.chars().all(|c| is_kdl_whitespace(c));
            if is_all_ws {
                dedented_lines.push(String::new());
            } else if is_cont {
                dedented_lines.push(line.clone());
            } else if line.starts_with(&indent_prefix) {
                dedented_lines.push(line[indent_prefix.len()..].to_string());
            } else {
                return Err(KdlError::Syntax {
                    message: "Multiline string line does not match closing indentation prefix"
                        .to_string(),
                    line: start_line,
                    col: start_col,
                });
            }
        }

        if hash_count > 0 {
            // Raw multiline string: join with \n verbatim
            Ok(dedented_lines.join("\n"))
        } else {
            // Standard multiline string: process escapes and line continuations
            self.process_multiline_escapes(&dedented_lines, start_line, start_col)
        }
    }

    fn process_multiline_escapes(
        &self,
        lines: &[String],
        start_line: usize,
        start_col: usize,
    ) -> Result<String, KdlError> {
        // Pass 1: merge line continuations
        let mut merged_lines = Vec::new();
        let mut idx = 0;
        while idx < lines.len() {
            let mut curr = lines[idx].clone();
            while idx + 1 < lines.len() {
                let trimmed = curr.trim_end_matches(|c| is_kdl_whitespace(c));
                let bs_count = trimmed.chars().rev().take_while(|c| *c == '\\').count();
                if bs_count % 2 == 1 {
                    // Line ends with unescaped '\'
                    let cut = trimmed.len() - 1;
                    curr = trimmed[..cut].to_string();
                    idx += 1;
                    let next = lines[idx].trim_start_matches(|c| is_kdl_whitespace(c));
                    curr.push_str(next);
                } else {
                    break;
                }
            }
            merged_lines.push(curr);
            idx += 1;
        }

        // Pass 2: decode string escapes and join with '\n'
        let mut decoded_lines = Vec::new();
        for line in &merged_lines {
            decoded_lines.push(self.decode_string_escapes(line, start_line, start_col)?);
        }

        Ok(decoded_lines.join("\n"))
    }

    fn decode_string_escapes(
        &self,
        s: &str,
        start_line: usize,
        start_col: usize,
    ) -> Result<String, KdlError> {
        let mut res = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                let esc = chars.next().ok_or(KdlError::UnexpectedEof)?;
                if is_kdl_whitespace(esc) || is_kdl_newline(esc) {
                    // Consume all consecutive whitespace and newline
                    while let Some(&next) = chars.peek() {
                        if is_kdl_whitespace(next) || is_kdl_newline(next) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
                match esc {
                    '"' => res.push('"'),
                    '\\' => res.push('\\'),
                    'b' => res.push('\x08'),
                    'f' => res.push('\x0C'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    's' => res.push(' '),
                    'u' => {
                        if chars.next() != Some('{') {
                            return Err(KdlError::InvalidEscape {
                                sequence: "u without {".to_string(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                        let mut hex = String::new();
                        while let Some(c) = chars.next() {
                            if c == '}' {
                                break;
                            }
                            hex.push(c);
                        }
                        if hex.is_empty() || hex.len() > 6 {
                            return Err(KdlError::InvalidEscape {
                                sequence: format!("u{{{}}}", hex),
                                line: start_line,
                                col: start_col,
                            });
                        }
                        let cp =
                            u32::from_str_radix(&hex, 16).map_err(|_| KdlError::InvalidEscape {
                                sequence: format!("u{{{}}}", hex),
                                line: start_line,
                                col: start_col,
                            })?;
                        let c = char::from_u32(cp).ok_or_else(|| KdlError::InvalidEscape {
                            sequence: format!("u{{{}}}", hex),
                            line: start_line,
                            col: start_col,
                        })?;
                        res.push(c);
                    }
                    other => {
                        return Err(KdlError::InvalidEscape {
                            sequence: other.to_string(),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
            } else {
                res.push(ch);
            }
        }

        Ok(res)
    }

    pub(crate) fn parse_quoted_string(&mut self) -> Result<String, KdlError> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // consume opening "
        let mut s = String::new();

        while let Some(ch) = self.advance() {
            match ch {
                '"' => return Ok(s),
                '\r' | '\n' => {
                    return Err(KdlError::Syntax {
                        message: "Literal newline not allowed in single-line string".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                '\\' => {
                    // Line continuation / whitespace escape inside string
                    if let Some(next) = self.peek() {
                        if is_kdl_whitespace(next) || is_kdl_newline(next) {
                            while let Some(c) = self.peek() {
                                if is_kdl_whitespace(c) || is_kdl_newline(c) {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            continue;
                        }
                    }

                    let esc = self.advance().ok_or(KdlError::UnexpectedEof)?;
                    match esc {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        'b' => s.push('\x08'),
                        'f' => s.push('\x0C'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        's' => s.push(' '),
                        'u' => {
                            if self.peek() == Some('{') {
                                self.advance();
                                let mut hex = String::new();
                                while let Some(c) = self.advance() {
                                    if c == '}' {
                                        break;
                                    }
                                    hex.push(c);
                                }
                                if hex.is_empty() || hex.len() > 6 {
                                    return Err(KdlError::InvalidEscape {
                                        sequence: format!("u{{{}}}", hex),
                                        line: start_line,
                                        col: start_col,
                                    });
                                }
                                let cp = u32::from_str_radix(&hex, 16).map_err(|_| {
                                    KdlError::InvalidEscape {
                                        sequence: format!("u{{{}}}", hex),
                                        line: start_line,
                                        col: start_col,
                                    }
                                })?;
                                let ch =
                                    char::from_u32(cp).ok_or_else(|| KdlError::InvalidEscape {
                                        sequence: format!("u{{{}}}", hex),
                                        line: start_line,
                                        col: start_col,
                                    })?;
                                s.push(ch);
                            } else {
                                return Err(KdlError::InvalidEscape {
                                    sequence: "u".to_string(),
                                    line: start_line,
                                    col: start_col,
                                });
                            }
                        }
                        other => {
                            return Err(KdlError::InvalidEscape {
                                sequence: other.to_string(),
                                line: self.line,
                                col: self.col,
                            });
                        }
                    }
                }
                other => s.push(other),
            }
        }

        Err(KdlError::Syntax {
            message: "Unclosed quoted string literal".to_string(),
            line: start_line,
            col: start_col,
        })
    }

    pub(crate) fn parse_raw_string(&mut self) -> Result<String, KdlError> {
        let start_line = self.line;
        let start_col = self.col;

        let mut hash_count = 0;
        while self.peek() == Some('#') {
            self.advance();
            hash_count += 1;
        }

        if hash_count == 0 || self.peek() != Some('"') {
            return Err(KdlError::Expected {
                expected: "'\"' to start raw string",
                found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                line: start_line,
                col: start_col,
            });
        }
        self.advance(); // consume opening "

        let mut s = String::new();
        while !self.is_eof() {
            if let Some(ch) = self.peek() {
                if ch == '\r' || ch == '\n' || is_kdl_newline(ch) {
                    return Err(KdlError::Syntax {
                        message: "Literal newline not allowed in single-line raw string"
                            .to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
            }

            if self.peek() == Some('"') {
                self.advance();
                let mut matched_hashes = 0;
                while matched_hashes < hash_count && self.peek() == Some('#') {
                    self.advance();
                    matched_hashes += 1;
                }
                if matched_hashes == hash_count {
                    return Ok(s);
                } else {
                    s.push('"');
                    for _ in 0..matched_hashes {
                        s.push('#');
                    }
                }
            } else {
                s.push(self.advance().unwrap());
            }
        }

        Err(KdlError::Syntax {
            message: "Unclosed raw string literal".to_string(),
            line: start_line,
            col: start_col,
        })
    }
}
