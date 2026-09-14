//! KDL recursive-descent parser supporting full KDL syntax.

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::ast::{KdlDocument, KdlEntry, KdlNode, KdlValue};
use crate::error::KdlError;

/// Parse a KDL document string into a `KdlDocument` AST.
pub fn parse_document(input: &str) -> Result<KdlDocument, KdlError> {
    let mut parser = Parser::new(input);
    parser.parse_document()
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    _marker: core::marker::PhantomData<&'a str>,
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
                message: format!("Forbidden bidirectional unicode character U+{:04X}", ch as u32),
                line: self.line,
                col: self.col,
            });
        }
        if ch == '\u{FEFF}' && self.pos > 0 {
            return Err(KdlError::Syntax {
                message: "Byte order mark (BOM) is only allowed at the beginning of the document".to_string(),
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
                            if self.peek_at(idx) == Some('/') && self.peek_at(idx + 1) == Some('*') {
                                idx += 2;
                                depth += 1;
                            } else if self.peek_at(idx) == Some('*') && self.peek_at(idx + 1) == Some('/') {
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

    fn parse_node(&mut self) -> Result<KdlNode, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        if self.peek() == Some('}') {
            return Err(KdlError::UnexpectedChar {
                ch: '}',
                line: self.line,
                col: self.col,
            });
        }

        // Optional type annotation: `(type)`
        let type_annotation = if self.peek() == Some('(') {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let had_space = self.skip_whitespace_and_comments(false)?;

        // Node name: identifier
        let name = self.parse_identifier()?;
        let mut entries = Vec::new();
        let mut children = Vec::new();
        let mut seen_any_child_block = false;
        let seen_active_children = false;
        let mut prev_space = had_space;

        loop {
            let had_space_before_token = self.skip_whitespace_and_comments(false)?;
            prev_space = prev_space || had_space_before_token;

            match self.peek() {
                None => break,
                Some(ch) if is_kdl_newline(ch) || ch == ';' => {
                    self.advance();
                    break;
                }
                Some('}') => {
                    // Terminating children block from parent
                    break;
                }
                Some('{') => {
                    if seen_active_children {
                        return Err(KdlError::Syntax {
                            message: "Node cannot have multiple children blocks".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    self.advance(); // consume {
                    children = self.parse_children()?;
                    let _ = seen_active_children;
                    let _ = seen_any_child_block;

                    // After children block `}`, only slashdashed child block, whitespace, newline, or ; allowed
                    let _ = self.skip_whitespace_and_comments(false)?;
                    while self.peek() == Some('/') && self.peek_at(1) == Some('-') {
                        let mut lookahead = 2;
                        while let Some(c) = self.peek_at(lookahead) {
                            if is_kdl_whitespace(c) || is_kdl_newline(c) {
                                lookahead += 1;
                            } else {
                                break;
                            }
                        }
                        if self.peek_at(lookahead) == Some('{') {
                            // Discard slashdash children
                            self.advance(); // /
                            self.advance(); // -
                            self.skip_whitespace_and_comments(true)?;
                            self.advance(); // {
                            let _ = self.parse_children()?;
                            let _ = self.skip_whitespace_and_comments(false)?;
                        } else {
                            break;
                        }
                    }

                    // Must be followed by newline, EOF, or ; or }
                    let _ = self.skip_whitespace_and_comments(false)?;
                    match self.peek() {
                        None => break,
                        Some(c) if is_kdl_newline(c) || c == ';' => {
                            self.advance();
                            break;
                        }
                        Some('}') => break,
                        Some(other) => {
                            return Err(KdlError::Syntax {
                                message: format!("Expected newline or ';' after children block, found '{}'", other),
                                line: self.line,
                                col: self.col,
                            });
                        }
                    }
                }
                Some('/') if self.peek_at(1) == Some('-') => {
                    self.advance(); // /
                    self.advance(); // -
                    self.skip_whitespace_and_comments(true)?;
                    if self.peek() == Some('{') {
                        // Discard slashdashed children block
                        self.advance(); // {
                        let _ = self.parse_children()?;
                        seen_any_child_block = true;
                    } else {
                        if seen_any_child_block {
                            return Err(KdlError::Syntax {
                                message: "Cannot place entry after children block".to_string(),
                                line: self.line,
                                col: self.col,
                            });
                        }
                        // Discard entry
                        let _ = self.parse_entry()?;
                    }
                    prev_space = true;
                    continue;
                }
                _ => {
                    if seen_any_child_block {
                        return Err(KdlError::Syntax {
                            message: "Cannot place entry after children block".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    if !prev_space {
                        return Err(KdlError::Syntax {
                            message: "Whitespace required between tokens".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    let entry = self.parse_entry()?;
                    entries.push(entry);
                    prev_space = false;
                }
            }
        }

        Ok(KdlNode {
            type_annotation,
            name,
            entries,
            children,
        })
    }

    fn parse_children(&mut self) -> Result<Vec<KdlNode>, KdlError> {
        let mut nodes = Vec::new();

        loop {
            self.skip_whitespace_and_comments(true)?;
            if self.is_eof() {
                return Err(KdlError::UnexpectedEof);
            }

            if self.peek() == Some('}') {
                self.advance(); // consume }
                break;
            }

            // Check for slashdash comment `/-`
            if self.peek() == Some('/') && self.peek_at(1) == Some('-') {
                self.advance(); // /
                self.advance(); // -
                self.skip_whitespace_and_comments(true)?;
                let _ = self.parse_node()?;
                continue;
            }

            let node = self.parse_node()?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    fn parse_entry(&mut self) -> Result<KdlEntry, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        // In KDL, type annotations on properties cannot appear before property key:
        // `(type)prop=val` is ILLEGAL!
        let has_type = self.peek() == Some('(');
        let type_annotation = if has_type {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.skip_whitespace_and_comments(false)?;

        if self.is_property_start() {
            if has_type {
                return Err(KdlError::Syntax {
                    message: "Type annotation cannot precede property key; use key=(type)val".to_string(),
                    line: self.line,
                    col: self.col,
                });
            }

            let key = self.parse_identifier()?;
            self.skip_whitespace_and_comments(false)?;
            if self.peek() == Some('=') {
                self.advance(); // consume =
            } else {
                return Err(KdlError::Expected {
                    expected: "'=' in property",
                    found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                    line: self.line,
                    col: self.col,
                });
            }
            self.skip_whitespace_and_comments(false)?;

            let prop_type = if self.peek() == Some('(') {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            self.skip_whitespace_and_comments(false)?;

            let val = self.parse_value()?;
            Ok(KdlEntry::Prop(key, prop_type, val))
        } else {
            let val = self.parse_value()?;
            Ok(KdlEntry::Arg(type_annotation, val))
        }
    }

    fn is_property_start(&self) -> bool {
        let mut idx = self.pos;
        if idx >= self.chars.len() {
            return false;
        }

        let first = self.chars[idx];
        if first == '"' {
            idx += 1;
            while idx < self.chars.len() && self.chars[idx] != '"' {
                if self.chars[idx] == '\\' {
                    idx += 1;
                }
                idx += 1;
            }
            if idx < self.chars.len() && self.chars[idx] == '"' {
                idx += 1;
            }
        } else if first == '#' {
            // Raw string key: #..."...#...
            let mut hashes = 0;
            while idx < self.chars.len() && self.chars[idx] == '#' {
                hashes += 1;
                idx += 1;
            }
            if idx < self.chars.len() && self.chars[idx] == '"' {
                idx += 1;
                while idx < self.chars.len() {
                    if self.chars[idx] == '"' {
                        idx += 1;
                        let mut match_h = 0;
                        while idx < self.chars.len() && self.chars[idx] == '#' && match_h < hashes {
                            match_h += 1;
                            idx += 1;
                        }
                        if match_h == hashes {
                            break;
                        }
                    } else {
                        idx += 1;
                    }
                }
            } else {
                return false;
            }
        } else if is_bare_ident_char(first) {
            while idx < self.chars.len() && is_bare_ident_char(self.chars[idx]) {
                idx += 1;
            }
        } else {
            return false;
        }

        while idx < self.chars.len() && is_kdl_whitespace(self.chars[idx]) {
            idx += 1;
        }

        if idx < self.chars.len() && self.chars[idx] == '=' {
            if idx + 1 < self.chars.len() && self.chars[idx + 1] == '=' {
                return false;
            }
            return true;
        }

        false
    }

    fn parse_type_annotation(&mut self) -> Result<String, KdlError> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // consume (
        self.skip_whitespace_and_comments(false)?;

        // Disallow slashdash inside type
        if self.peek() == Some('/') && self.peek_at(1) == Some('-') {
            return Err(KdlError::Syntax {
                message: "Slashdash is not allowed inside type annotation".to_string(),
                line: self.line,
                col: self.col,
            });
        }

        if self.peek() == Some(')') {
            return Err(KdlError::Syntax {
                message: "Empty type annotation is not allowed".to_string(),
                line: start_line,
                col: start_col,
            });
        }

        let ty = self.parse_identifier()?;
        self.skip_whitespace_and_comments(false)?;

        if self.peek() == Some(')') {
            self.advance(); // consume )
            Ok(ty)
        } else {
            Err(KdlError::Expected {
                expected: "')' to close type annotation",
                found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                line: start_line,
                col: start_col,
            })
        }
    }

    fn parse_identifier(&mut self) -> Result<String, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        match self.peek() {
            Some('"') => self.parse_quoted_string(),
            Some('#') => self.parse_raw_string(),
            Some(ch) if is_bare_ident_char(ch) => {
                let start_line = self.line;
                let start_col = self.col;
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if is_bare_ident_char(c) {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Check identifier validity
                let first = ident.chars().next().unwrap();
                if first.is_ascii_digit() {
                    return Err(KdlError::Syntax {
                        message: "Bare identifier cannot start with a digit".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                if (first == '+' || first == '-') && ident.len() > 1 {
                    let second = ident.chars().nth(1).unwrap();
                    if second.is_ascii_digit() {
                        return Err(KdlError::Syntax {
                            message: "Bare identifier cannot start with sign followed by digit".to_string(),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                if first == '.' && ident.len() > 1 && ident.chars().nth(1).unwrap().is_ascii_digit() {
                    return Err(KdlError::Syntax {
                        message: "Bare identifier cannot start with dot followed by digit".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                if matches!(
                    ident.as_str(),
                    "true" | "false" | "null" | "inf" | "-inf" | "nan"
                ) {
                    return Err(KdlError::Syntax {
                        message: format!("Reserved keyword '{}' cannot be used as bare identifier", ident),
                        line: start_line,
                        col: start_col,
                    });
                }

                Ok(ident)
            }
            Some(ch) => Err(KdlError::UnexpectedChar {
                ch,
                line: self.line,
                col: self.col,
            }),
            None => Err(KdlError::UnexpectedEof),
        }
    }

    fn parse_value(&mut self) -> Result<KdlValue, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        // Keywords starting with #
        if self.peek() == Some('#') {
            if self.matches_keyword("#true") {
                self.consume_keyword("#true");
                return Ok(KdlValue::Bool(true));
            } else if self.matches_keyword("#false") {
                self.consume_keyword("#false");
                return Ok(KdlValue::Bool(false));
            } else if self.matches_keyword("#null") {
                self.consume_keyword("#null");
                return Ok(KdlValue::Null);
            } else if self.matches_keyword("#inf") {
                self.consume_keyword("#inf");
                return Ok(KdlValue::Float(f64::INFINITY));
            } else if self.matches_keyword("#-inf") {
                self.consume_keyword("#-inf");
                return Ok(KdlValue::Float(f64::NEG_INFINITY));
            } else if self.matches_keyword("#nan") {
                self.consume_keyword("#nan");
                return Ok(KdlValue::Float(f64::NAN));
            }
        }

        // Bare keywords (backwards compatibility with KDL v1)
        if self.matches_keyword("true") {
            self.consume_keyword("true");
            return Ok(KdlValue::Bool(true));
        } else if self.matches_keyword("false") {
            self.consume_keyword("false");
            return Ok(KdlValue::Bool(false));
        } else if self.matches_keyword("null") {
            self.consume_keyword("null");
            return Ok(KdlValue::Null);
        }

        // Multiline string: """ or #...#"""
        if self.is_multiline_string_start() {
            let s = self.parse_multiline_string()?;
            return Ok(KdlValue::String(s));
        }

        // Raw string: #..."..."#
        if self.peek() == Some('#') {
            let s = self.parse_raw_string()?;
            return Ok(KdlValue::String(s));
        }

        // Standard quoted string
        if self.peek() == Some('"') {
            let s = self.parse_quoted_string()?;
            return Ok(KdlValue::String(s));
        }

        // Numbers starting with digit or sign
        if let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                return self.parse_number();
            }
            if ch == '+' || ch == '-' {
                if let Some(next) = self.peek_at(1) {
                    if next.is_ascii_digit() {
                        return self.parse_number();
                    }
                }
            }
        }

        // Bare identifier as string value (or error if keyword)
        let s = self.parse_identifier()?;
        Ok(KdlValue::String(s))
    }

    fn matches_keyword(&self, kw: &str) -> bool {
        for (i, c) in kw.chars().enumerate() {
            if self.peek_at(i) != Some(c) {
                return false;
            }
        }
        if let Some(next) = self.peek_at(kw.len()) {
            if is_bare_ident_char(next) {
                return false;
            }
        }
        true
    }

    fn consume_keyword(&mut self, kw: &str) {
        for _ in 0..kw.len() {
            self.advance();
        }
    }

    fn is_multiline_string_start(&self) -> bool {
        let mut idx = 0;
        while self.peek_at(idx) == Some('#') {
            idx += 1;
        }
        self.peek_at(idx) == Some('"')
            && self.peek_at(idx + 1) == Some('"')
            && self.peek_at(idx + 2) == Some('"')
    }

    fn parse_multiline_string(&mut self) -> Result<String, KdlError> {
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
                    message: "Multiline string opening '\"\"\"' must be followed by a newline".to_string(),
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
                    message: "Multiline string line does not match closing indentation prefix".to_string(),
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
                        let cp = u32::from_str_radix(&hex, 16).map_err(|_| KdlError::InvalidEscape {
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

    fn parse_quoted_string(&mut self) -> Result<String, KdlError> {
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
                                let ch = char::from_u32(cp).ok_or_else(|| {
                                    KdlError::InvalidEscape {
                                        sequence: format!("u{{{}}}", hex),
                                        line: start_line,
                                        col: start_col,
                                    }
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

    fn parse_raw_string(&mut self) -> Result<String, KdlError> {
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
                        message: "Literal newline not allowed in single-line raw string".to_string(),
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

    fn parse_number(&mut self) -> Result<KdlValue, KdlError> {
        let start_line = self.line;
        let start_col = self.col;
        let mut raw = String::new();

        if let Some(sign) = self.peek() {
            if sign == '+' || sign == '-' {
                raw.push(sign);
                self.advance();
            }
        }

        // Check for base prefixes: 0x, 0o, 0b
        if self.peek() == Some('0') {
            let next = self.peek_at(1);
            if next == Some('x') || next == Some('X') {
                self.advance(); // 0
                self.advance(); // x
                let mut hex = String::new();
                let mut first_digit = true;
                while let Some(c) = self.peek() {
                    if c.is_ascii_hexdigit() {
                        hex.push(c);
                        self.advance();
                        first_digit = false;
                    } else if c == '_' {
                        if first_digit {
                            return Err(KdlError::InvalidNumber {
                                literal: "0x_".to_string(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                if hex.is_empty() {
                    return Err(KdlError::InvalidNumber {
                        literal: "0x".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                // Check no trailing invalid chars
                if let Some(tail) = self.peek() {
                    if is_bare_ident_char(tail) {
                        return Err(KdlError::InvalidNumber {
                            literal: format!("0x{}{}", hex, tail),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                let val = i128::from_str_radix(&hex, 16).map_err(|_| KdlError::InvalidNumber {
                    literal: format!("0x{}", hex),
                    line: start_line,
                    col: start_col,
                })?;
                let val = if raw == "-" { -val } else { val };
                return Ok(KdlValue::Integer(val));
            } else if next == Some('o') || next == Some('O') {
                self.advance(); // 0
                self.advance(); // o
                let mut oct = String::new();
                let mut first_digit = true;
                while let Some(c) = self.peek() {
                    if ('0'..='7').contains(&c) {
                        oct.push(c);
                        self.advance();
                        first_digit = false;
                    } else if c == '_' {
                        if first_digit {
                            return Err(KdlError::InvalidNumber {
                                literal: "0o_".to_string(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                if oct.is_empty() {
                    return Err(KdlError::InvalidNumber {
                        literal: "0o".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                if let Some(tail) = self.peek() {
                    if is_bare_ident_char(tail) {
                        return Err(KdlError::InvalidNumber {
                            literal: format!("0o{}{}", oct, tail),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                let val = i128::from_str_radix(&oct, 8).map_err(|_| KdlError::InvalidNumber {
                    literal: format!("0o{}", oct),
                    line: start_line,
                    col: start_col,
                })?;
                let val = if raw == "-" { -val } else { val };
                return Ok(KdlValue::Integer(val));
            } else if next == Some('b') || next == Some('B') {
                self.advance(); // 0
                self.advance(); // b
                let mut bin = String::new();
                let mut first_digit = true;
                while let Some(c) = self.peek() {
                    if c == '0' || c == '1' {
                        bin.push(c);
                        self.advance();
                        first_digit = false;
                    } else if c == '_' {
                        if first_digit {
                            return Err(KdlError::InvalidNumber {
                                literal: "0b_".to_string(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                if bin.is_empty() {
                    return Err(KdlError::InvalidNumber {
                        literal: "0b".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                if let Some(tail) = self.peek() {
                    if is_bare_ident_char(tail) {
                        return Err(KdlError::InvalidNumber {
                            literal: format!("0b{}{}", bin, tail),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                let val = i128::from_str_radix(&bin, 2).map_err(|_| KdlError::InvalidNumber {
                    literal: format!("0b{}", bin),
                    line: start_line,
                    col: start_col,
                })?;
                let val = if raw == "-" { -val } else { val };
                return Ok(KdlValue::Integer(val));
            }
        }

        // Standard decimal integer or float
        let mut is_float = false;
        let mut has_exp = false;
        let mut has_integer_digits = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                raw.push(c);
                self.advance();
                if !is_float && !has_exp {
                    has_integer_digits = true;
                }
            } else if c == '_' {
                self.advance();
            } else if c == '.' && !is_float && !has_exp {
                // Must have digits after '.'
                if let Some(next) = self.peek_at(1) {
                    if next.is_ascii_digit() {
                        is_float = true;
                        raw.push(c);
                        self.advance();
                    } else {
                        return Err(KdlError::InvalidNumber {
                            literal: format!("{}.", raw),
                            line: start_line,
                            col: start_col,
                        });
                    }
                } else {
                    return Err(KdlError::InvalidNumber {
                        literal: format!("{}.", raw),
                        line: start_line,
                        col: start_col,
                    });
                }
            } else if (c == 'e' || c == 'E') && !has_exp && has_integer_digits {
                is_float = true;
                has_exp = true;
                raw.push(c);
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        raw.push(sign);
                        self.advance();
                    }
                }
                // Must have at least one digit in exponent
                if let Some(exp_first) = self.peek() {
                    if !exp_first.is_ascii_digit() {
                        return Err(KdlError::InvalidNumber {
                            literal: raw,
                            line: start_line,
                            col: start_col,
                        });
                    }
                } else {
                    return Err(KdlError::InvalidNumber {
                        literal: raw,
                        line: start_line,
                        col: start_col,
                    });
                }
            } else {
                break;
            }
        }

        if !has_integer_digits {
            return Err(KdlError::InvalidNumber {
                literal: raw,
                line: start_line,
                col: start_col,
            });
        }

        // Reject if immediately followed by trailing characters
        if let Some(tail) = self.peek() {
            if is_bare_ident_char(tail) || tail == '.' {
                return Err(KdlError::InvalidNumber {
                    literal: format!("{}{}", raw, tail),
                    line: start_line,
                    col: start_col,
                });
            }
        }

        if is_float {
            let f = raw.parse::<f64>().map_err(|_| KdlError::InvalidNumber {
                literal: raw.clone(),
                line: start_line,
                col: start_col,
            })?;
            Ok(KdlValue::Float(f))
        } else {
            let i = raw.parse::<i128>().map_err(|_| KdlError::InvalidNumber {
                literal: raw.clone(),
                line: start_line,
                col: start_col,
            })?;
            Ok(KdlValue::Integer(i))
        }
    }
}

/// Helper to test if a character is valid KDL whitespace.
fn is_kdl_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

/// Helper to test if a character is a KDL newline.
fn is_kdl_newline(c: char) -> bool {
    matches!(
        c,
        '\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{2028}' | '\u{2029}'
    )
}

/// Helper to test if a character is a forbidden bidirectional unicode character.
fn is_bidi_char(c: char) -> bool {
    matches!(
        c,
        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// Helper to test if a character can be part of a bare KDL identifier.
fn is_bare_ident_char(ch: char) -> bool {
    !is_kdl_whitespace(ch)
        && !is_kdl_newline(ch)
        && !matches!(
            ch,
            '(' | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '/'
                | '\\'
                | '"'
                | '='
                | ';'
                | '#'
        )
        && !ch.is_control()
        && !is_bidi_char(ch)
}
