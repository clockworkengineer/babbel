//! KDL recursive-descent parser supporting full KDL syntax.

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::ast::KdlDocument;
use crate::error::KdlError;

/// Parse a KDL document string into a `KdlDocument` AST.
pub fn parse_document(input: &str) -> Result<KdlDocument, KdlError> {
    let mut parser = Parser::new(input);
    parser.parse_document()
}
pub mod node;
pub mod number;
pub mod string;
pub mod value;

pub(crate) struct Parser<'a> {
    pub(crate) chars: Vec<char>,
    pub(crate) pos: usize,
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) _marker: core::marker::PhantomData<&'a str>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            _marker: core::marker::PhantomData,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    /// Check for illegal Bidi characters or BOM.
    fn check_valid_char(&self, ch: char) -> Result<(), KdlError> {
        if is_bidi_char(ch) {
            return Err(KdlError::Syntax {
                message: format!(
                    "Forbidden bidirectional unicode character U+{:04X}",
                    ch as u32
                ),
                line: self.line,
                col: self.col,
            });
        }
        if ch == '\u{FEFF}' && self.pos > 0 {
            return Err(KdlError::Syntax {
                message: "Byte order mark (BOM) is only allowed at the beginning of the document"
                    .to_string(),
                line: self.line,
                col: self.col,
            });
        }
        Ok(())
    }

    /// Skip whitespace and comments.
    /// Returns true if at least one whitespace, comment, or line-break was consumed.
    fn skip_whitespace_and_comments(&mut self, skip_newlines: bool) -> Result<bool, KdlError> {
        let mut consumed = false;

        while !self.is_eof() {
            let ch = match self.peek() {
                Some(c) => c,
                None => break,
            };

            self.check_valid_char(ch)?;

            if is_kdl_whitespace(ch) {
                self.advance();
                consumed = true;
            } else if ch == '\\' {
                // Line continuation outside strings: \ followed by optional whitespace/comments, then newline or EOF
                let mut idx = 1;
                let mut valid_continuation = false;
                while let Some(next) = self.peek_at(idx) {
                    if is_kdl_whitespace(next) {
                        idx += 1;
                    } else if next == '/' && self.peek_at(idx + 1) == Some('/') {
                        // Single line comment
                        idx += 2;
                        while let Some(c) = self.peek_at(idx) {
                            if is_kdl_newline(c) {
                                break;
                            }
                            idx += 1;
                        }
                    } else if next == '/' && self.peek_at(idx + 1) == Some('*') {
                        // Block comment
                        idx += 2;
                        let mut depth = 1;
                        while depth > 0 && self.pos + idx < self.chars.len() {
                            if self.peek_at(idx) == Some('/') && self.peek_at(idx + 1) == Some('*')
                            {
                                idx += 2;
                                depth += 1;
                            } else if self.peek_at(idx) == Some('*')
                                && self.peek_at(idx + 1) == Some('/')
                            {
                                idx += 2;
                                depth -= 1;
                            } else {
                                idx += 1;
                            }
                        }
                    } else {
                        break;
                    }
                }

                if self.pos + idx >= self.chars.len() {
                    // EOF after \ and optional comments/whitespace
                    valid_continuation = true;
                } else if let Some(next) = self.peek_at(idx) {
                    if is_kdl_newline(next) {
                        valid_continuation = true;
                        idx += 1;
                        if next == '\r' && self.peek_at(idx) == Some('\n') {
                            idx += 1;
                        }
                    }
                }

                if valid_continuation {
                    for _ in 0..idx {
                        self.advance();
                    }
                    consumed = true;
                    continue;
                }
                break;
            } else if is_kdl_newline(ch) || ch == ';' {
                if skip_newlines {
                    self.advance();
                    consumed = true;
                } else {
                    break;
                }
            } else if ch == '/' {
                if self.peek_at(1) == Some('/') {
                    // Single-line comment
                    self.advance();
                    self.advance();
                    consumed = true;
                    while let Some(c) = self.peek() {
                        if is_kdl_newline(c) {
                            if skip_newlines {
                                self.advance();
                            }
                            break;
                        }
                        self.advance();
                    }
                } else if self.peek_at(1) == Some('*') {
                    // Multi-line block comment with nesting support
                    let start_line = self.line;
                    let start_col = self.col;
                    self.advance(); // /
                    self.advance(); // *
                    consumed = true;
                    let mut depth = 1;
                    while depth > 0 && !self.is_eof() {
                        if self.peek() == Some('/') && self.peek_at(1) == Some('*') {
                            self.advance();
                            self.advance();
                            depth += 1;
                        } else if self.peek() == Some('*') && self.peek_at(1) == Some('/') {
                            self.advance();
                            self.advance();
                            depth -= 1;
                        } else {
                            self.advance();
                        }
                    }
                    if depth > 0 {
                        return Err(KdlError::Syntax {
                            message: "Unterminated multiline block comment".to_string(),
                            line: start_line,
                            col: start_col,
                        });
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(consumed)
    }

    fn parse_document(&mut self) -> Result<KdlDocument, KdlError> {
        // Strip leading BOM if present
        if self.peek() == Some('\u{FEFF}') {
            self.advance();
        }

        let mut doc = KdlDocument::new();

        loop {
            self.skip_whitespace_and_comments(true)?;
            if self.is_eof() {
                break;
            }

            // Check for slashdash comment `/-` before node
            if self.peek() == Some('/') && self.peek_at(1) == Some('-') {
                self.advance(); // consume /
                self.advance(); // consume -
                self.skip_whitespace_and_comments(true)?;
                // Parse and discard next node
                let _ = self.parse_node()?;
                continue;
            }

            let node = self.parse_node()?;
            doc.nodes.push(node);
        }

        Ok(doc)
    }
}

/// Helper to test if a character is valid KDL whitespace.
#[inline]
pub(crate) fn is_kdl_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// Helper to test if a character is a KDL newline.
#[inline]
pub(crate) fn is_kdl_newline(c: char) -> bool {
    matches!(
        c,
        '\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{2028}' | '\u{2029}'
    )
}

/// Helper to test if a character is a forbidden bidirectional unicode character.
#[inline]
pub(crate) fn is_bidi_char(c: char) -> bool {
    matches!(
        c,
        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// Helper to test if a character can be part of a bare KDL identifier.
#[inline]
pub(crate) fn is_bare_ident_char(ch: char) -> bool {
    !is_kdl_whitespace(ch)
        && !is_kdl_newline(ch)
        && !matches!(
            ch,
            '(' | ')' | '{' | '}' | '[' | ']' | '/' | '\\' | '"' | '=' | ';' | '#'
        )
        && !ch.is_control()
        && !is_bidi_char(ch)
}
