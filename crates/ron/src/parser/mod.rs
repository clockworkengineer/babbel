//! RON (Rusty Object Notation / Readable Object Notation) parser coordinator.
//!
//! Parses RON documents into Babbel's universal `Value` AST.
//! Supports both Rusty Object Notation and language-neutral Readable Object Notation (starfederation/ron).

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::error::RonError;
use babbel_core::Value;

pub mod container;
pub mod number;
pub mod scalar;

/// Parse a RON string into a universal [`Value`] AST.
pub fn from_str(input: &str) -> Result<Value, RonError> {
    let mut parser = RonParser::new(input);
    let val = parser.parse_root()?;
    Ok(val)
}

/// Parse RON from a byte slice.
pub fn from_bytes(bytes: &[u8]) -> Result<Value, RonError> {
    let s = core::str::from_utf8(bytes).map_err(|_| RonError::InvalidUtf8)?;
    from_str(s)
}

/// Recursive-descent parser for RON.
pub struct RonParser<'a> {
    pub(crate) input: &'a str,
    pub(crate) chars: Vec<(usize, char)>,
    pub(crate) cursor: usize,
    pub(crate) depth: usize,
    pub(crate) max_depth: usize,
}

impl<'a> RonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            input,
            chars,
            cursor: 0,
            depth: 0,
            max_depth: 256,
        }
    }

    pub(crate) fn current_line_col(&self) -> (usize, usize) {
        let byte_pos = if self.cursor < self.chars.len() {
            self.chars[self.cursor].0
        } else {
            self.input.len()
        };

        let mut line = 1;
        let mut col = 1;
        for (i, b) in self.input.bytes().enumerate() {
            if i >= byte_pos {
                break;
            }
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    #[inline]
    pub(crate) fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|&(_, c)| c)
    }

    #[inline]
    pub(crate) fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|&(_, c)| c)
    }

    #[inline]
    pub(crate) fn next_char(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor].1;
            self.cursor += 1;
            Some(ch)
        } else {
            None
        }
    }

    pub(crate) fn skip_whitespace_and_comments(&mut self) -> Result<(), RonError> {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.cursor += 1;
                }
                Some('/') => {
                    match self.peek_next() {
                        Some('/') => {
                            // Single-line comment: // until \n or EOF
                            self.cursor += 2;
                            while let Some(ch) = self.peek() {
                                self.cursor += 1;
                                if ch == '\n' {
                                    break;
                                }
                            }
                        }
                        Some('*') => {
                            // Multi-line block comment with support for nesting: /* ... /* ... */ ... */
                            self.cursor += 2;
                            let mut comment_depth = 1;
                            while self.cursor < self.chars.len() {
                                if self.chars[self.cursor].1 == '/'
                                    && self.cursor + 1 < self.chars.len()
                                    && self.chars[self.cursor + 1].1 == '*'
                                {
                                    self.cursor += 2;
                                    comment_depth += 1;
                                } else if self.chars[self.cursor].1 == '*'
                                    && self.cursor + 1 < self.chars.len()
                                    && self.chars[self.cursor + 1].1 == '/'
                                {
                                    self.cursor += 2;
                                    comment_depth -= 1;
                                    if comment_depth == 0 {
                                        break;
                                    }
                                } else {
                                    self.cursor += 1;
                                }
                            }
                            if comment_depth > 0 {
                                return Err(RonError::UnexpectedEof);
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Root document parsing.
    /// Supports top-level object elision per ADR-0001:
    /// If document starts without `{`, `[`, or `(`, attempts to parse as an unbraced map.
    /// Falls back to single value parsing if elided map parsing fails.
    pub fn parse_root(&mut self) -> Result<Value, RonError> {
        self.skip_whitespace_and_comments()?;
        if self.cursor >= self.chars.len() {
            return Err(RonError::UnexpectedEof);
        }

        // If explicitly starts with map, array, or paren container, parse directly
        match self.peek() {
            Some('{') => {
                let val = self.parse_map()?;
                self.skip_whitespace_and_comments()?;
                if let Some(ch) = self.peek() {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::UnexpectedChar { ch, line, col });
                }
                return Ok(val);
            }
            Some('[') => {
                let val = self.parse_list()?;
                self.skip_whitespace_and_comments()?;
                if let Some(ch) = self.peek() {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::UnexpectedChar { ch, line, col });
                }
                return Ok(val);
            }
            Some('(') => {
                let val = self.parse_paren_container()?;
                self.skip_whitespace_and_comments()?;
                if let Some(ch) = self.peek() {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::UnexpectedChar { ch, line, col });
                }
                return Ok(val);
            }
            _ => {}
        }

        // Check if it's a Rust-style struct like `Config(...)` or `Point(x: 1)`,
        // or a raw/byte string literal like `b"..."` or `r#"..."#`
        if !self.is_named_struct_start() && !self.is_raw_or_byte_literal_start() {
            // Attempt elided map parsing from byte 0
            let checkpoint = self.cursor;
            let prev_depth = self.depth;
            if let Ok(elided) = self.parse_elided_map() {
                let check_cursor = self.cursor;
                if self.skip_whitespace_and_comments().is_ok() && self.cursor >= self.chars.len() {
                    return Ok(elided);
                }
                self.cursor = check_cursor;
            }
            self.cursor = checkpoint;
            self.depth = prev_depth;
        }

        // Fallback: parse single value
        let val = self.parse_value()?;
        self.skip_whitespace_and_comments()?;
        if let Some(ch) = self.peek() {
            let (line, col) = self.current_line_col();
            return Err(RonError::UnexpectedChar { ch, line, col });
        }
        Ok(val)
    }

    pub(crate) fn is_named_struct_start(&self) -> bool {
        let mut i = self.cursor;
        while i < self.chars.len() && self.chars[i].1.is_whitespace() {
            i += 1;
        }
        if i < self.chars.len() && is_ident_start(self.chars[i].1) {
            while i < self.chars.len()
                && (is_ident_part(self.chars[i].1)
                    || (self.chars[i].1 == ':'
                        && i + 1 < self.chars.len()
                        && self.chars[i + 1].1 == ':'))
            {
                if self.chars[i].1 == ':' {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            while i < self.chars.len() && self.chars[i].1.is_whitespace() {
                i += 1;
            }
            if i < self.chars.len() && self.chars[i].1 == '(' {
                return true;
            }
        }
        false
    }

    pub(crate) fn is_raw_or_byte_literal_start(&self) -> bool {
        let mut i = self.cursor;
        while i < self.chars.len() && self.chars[i].1.is_whitespace() {
            i += 1;
        }
        if i < self.chars.len() {
            let c = self.chars[i].1;
            let next = if i + 1 < self.chars.len() {
                Some(self.chars[i + 1].1)
            } else {
                None
            };
            if c == 'r' && (next == Some('"') || next == Some('#')) {
                return true;
            }
            if c == 'b' && (next == Some('"') || next == Some('\'') || next == Some('r')) {
                return true;
            }
        }
        false
    }

    pub fn parse_value(&mut self) -> Result<Value, RonError> {
        self.skip_whitespace_and_comments()?;
        let (line, col) = self.current_line_col();

        match self.peek() {
            Some('(') => self.parse_paren_container(),
            Some('[') => self.parse_list(),
            Some('{') => self.parse_map(),
            Some('"') | Some('\'') => {
                let s = self.parse_quoted_string()?;
                Ok(Value::String(s))
            }
            Some(',') => {
                let s = self.parse_comma_prefixed_token()?;
                Ok(Value::String(s))
            }
            Some('r') if self.peek_next() == Some('"') || self.peek_next() == Some('#') => {
                self.parse_raw_string()
            }
            Some('b')
                if self.peek_next() == Some('"')
                    || self.peek_next() == Some('r')
                    || self.peek_next() == Some('\'') =>
            {
                self.parse_byte_literal()
            }
            Some('+') | Some('-') | Some('.') | Some('0'..='9') => {
                self.parse_number_or_bare_token()
            }
            Some(c) if !c.is_whitespace() && !is_structural_delimiter(c) => self.parse_bare_value(),
            Some(ch) => Err(RonError::UnexpectedChar { ch, line, col }),
            None => Err(RonError::UnexpectedEof),
        }
    }

    pub(crate) fn consume_str(&mut self, s: &str) -> bool {
        let len = s.chars().count();
        if self.cursor + len <= self.chars.len() {
            let candidate: String = self.chars[self.cursor..self.cursor + len]
                .iter()
                .map(|&(_, c)| c)
                .collect();
            if candidate == s {
                self.cursor += len;
                return true;
            }
        }
        false
    }
}

#[inline]
pub(crate) fn is_structural_delimiter(ch: char) -> bool {
    matches!(ch, '{' | '}' | '[' | ']' | '"' | '\'' | ',' | '(' | ')')
}

#[inline]
pub(crate) fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

#[inline]
pub(crate) fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
