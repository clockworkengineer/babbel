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

    /// Skip whitespace, single-line comments, and multiline comments.
    /// If `skip_newlines` is false, stops before newlines and semicolons.
    fn skip_whitespace_and_comments(&mut self, skip_newlines: bool) -> Result<(), KdlError> {
        while !self.is_eof() {
            match self.peek() {
                Some(' ') | Some('\t') => {
                    self.advance();
                }
                Some('\\') => {
                    // Line continuation: \ followed by optional whitespace and newline
                    let next = self.peek_at(1);
                    if next == Some('\r') || next == Some('\n') {
                        self.advance(); // consume \
                        if self.peek() == Some('\r') {
                            self.advance();
                        }
                        if self.peek() == Some('\n') {
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                Some('\r') | Some('\n') | Some(';') => {
                    if skip_newlines {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Some('/') => {
                    if self.peek_at(1) == Some('/') {
                        // Single-line comment
                        self.advance();
                        self.advance();
                        while let Some(ch) = self.peek() {
                            if ch == '\n' {
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
                        self.advance(); // consume /
                        self.advance(); // consume *
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
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_document(&mut self) -> Result<KdlDocument, KdlError> {
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
                self.skip_whitespace_and_comments(false)?;
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

        // Optional type annotation: `(type)`
        let type_annotation = if self.peek() == Some('(') {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.skip_whitespace_and_comments(false)?;

        // Node name: identifier
        let name = self.parse_identifier()?;
        let mut entries = Vec::new();
        let mut children = Vec::new();

        loop {
            self.skip_whitespace_and_comments(false)?;

            match self.peek() {
                None => break,
                Some('\n') | Some('\r') | Some(';') => {
                    self.advance();
                    break;
                }
                Some('}') => {
                    // Terminating children block from parent node
                    break;
                }
                Some('{') => {
                    // Children block
                    self.advance(); // consume {
                    children = self.parse_children()?;
                    // Children block finishes the node
                    break;
                }
                Some('/') => {
                    if self.peek_at(1) == Some('-') {
                        // Slashdash comment `/-` for entry or child block
                        self.advance(); // /
                        self.advance(); // -
                        self.skip_whitespace_and_comments(false)?;
                        if self.peek() == Some('{') {
                            // Discard children block
                            self.advance();
                            let _ = self.parse_children()?;
                        } else {
                            // Discard entry
                            let _ = self.parse_entry()?;
                        }
                        continue;
                    } else if self.peek_at(1) == Some('/') || self.peek_at(1) == Some('*') {
                        self.skip_whitespace_and_comments(false)?;
                        continue;
                    } else {
                        return Err(KdlError::UnexpectedChar {
                            ch: '/',
                            line: self.line,
                            col: self.col,
                        });
                    }
                }
                _ => {
                    let entry = self.parse_entry()?;
                    entries.push(entry);
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
                self.skip_whitespace_and_comments(false)?;
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

        // Check if there is a type annotation: `(type)`
        let type_annotation = if self.peek() == Some('(') {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.skip_whitespace_and_comments(false)?;

        // Now peek to see if this is `prop=val` or `arg`
        // We can inspect whether there's an identifier followed immediately by `=`
        if self.is_property_start() {
            let key = self.parse_identifier()?;
            self.skip_whitespace_and_comments(false)?;
            if self.peek() == Some('=') {
                self.advance(); // consume =
            }
            self.skip_whitespace_and_comments(false)?;

            let prop_type = if self.peek() == Some('(') {
                Some(self.parse_type_annotation()?)
            } else {
                type_annotation
            };

            let val = self.parse_value()?;
            Ok(KdlEntry::Prop(key, prop_type, val))
        } else {
            let val = self.parse_value()?;
            Ok(KdlEntry::Arg(type_annotation, val))
        }
    }

    fn is_property_start(&self) -> bool {
        // Look ahead to find if there's an identifier followed by `=`
        let mut idx = self.pos;
        if idx >= self.chars.len() {
            return false;
        }

        let first = self.chars[idx];
        if first == '"' || first == 'r' {
            // Check quoted string followed by '='
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
            } else {
                // Bare identifier
                while idx < self.chars.len() && is_bare_ident_char(self.chars[idx]) {
                    idx += 1;
                }
            }
        } else if is_bare_ident_char(first) {
            while idx < self.chars.len() && is_bare_ident_char(self.chars[idx]) {
                idx += 1;
            }
        } else {
            return false;
        }

        // Skip whitespace
        while idx < self.chars.len() && (self.chars[idx] == ' ' || self.chars[idx] == '\t') {
            idx += 1;
        }

        if idx < self.chars.len() && self.chars[idx] == '=' {
            // Must not be `==`
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
            Some('r') if self.peek_at(1) == Some('"') || self.peek_at(1) == Some('#') => {
                self.parse_raw_string()
            }
            Some(ch) if is_bare_ident_char(ch) => {
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if is_bare_ident_char(c) {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
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

        match self.peek() {
            Some('"') => {
                let s = self.parse_quoted_string()?;
                Ok(KdlValue::String(s))
            }
            Some('r') if self.peek_at(1) == Some('"') || self.peek_at(1) == Some('#') => {
                let s = self.parse_raw_string()?;
                Ok(KdlValue::String(s))
            }
            Some('t') if self.matches_keyword("true") => {
                self.consume_keyword("true");
                Ok(KdlValue::Bool(true))
            }
            Some('f') if self.matches_keyword("false") => {
                self.consume_keyword("false");
                Ok(KdlValue::Bool(false))
            }
            Some('n') if self.matches_keyword("null") => {
                self.consume_keyword("null");
                Ok(KdlValue::Null)
            }
            Some(ch) if ch.is_ascii_digit() || ch == '+' || ch == '-' => {
                self.parse_number()
            }
            Some(ch) if is_bare_ident_char(ch) => {
                // Bare string value
                let s = self.parse_identifier()?;
                Ok(KdlValue::String(s))
            }
            Some(ch) => Err(KdlError::UnexpectedChar {
                ch,
                line: self.line,
                col: self.col,
            }),
            None => Err(KdlError::UnexpectedEof),
        }
    }

    fn matches_keyword(&self, kw: &str) -> bool {
        for (i, c) in kw.chars().enumerate() {
            if self.peek_at(i) != Some(c) {
                return false;
            }
        }
        // Next character must not be a bare ident char
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

    fn parse_quoted_string(&mut self) -> Result<String, KdlError> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // consume opening "
        let mut s = String::new();

        while let Some(ch) = self.advance() {
            match ch {
                '"' => return Ok(s),
                '\\' => {
                    let esc = self.advance().ok_or(KdlError::UnexpectedEof)?;
                    match esc {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
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
                                let codepoint = u32::from_str_radix(&hex, 16).map_err(|_| {
                                    KdlError::InvalidEscape {
                                        sequence: format!("u{{{}}}", hex),
                                        line: start_line,
                                        col: start_col,
                                    }
                                })?;
                                let ch = char::from_u32(codepoint).ok_or_else(|| {
                                    KdlError::InvalidEscape {
                                        sequence: format!("u{{{}}}", hex),
                                        line: start_line,
                                        col: start_col,
                                    }
                                })?;
                                s.push(ch);
                            } else {
                                // \uXXXX (4 hex chars)
                                let mut hex = String::new();
                                for _ in 0..4 {
                                    hex.push(self.advance().ok_or(KdlError::UnexpectedEof)?);
                                }
                                let codepoint = u32::from_str_radix(&hex, 16).map_err(|_| {
                                    KdlError::InvalidEscape {
                                        sequence: format!("u{}", hex),
                                        line: start_line,
                                        col: start_col,
                                    }
                                })?;
                                let ch = char::from_u32(codepoint).ok_or_else(|| {
                                    KdlError::InvalidEscape {
                                        sequence: format!("u{}", hex),
                                        line: start_line,
                                        col: start_col,
                                    }
                                })?;
                                s.push(ch);
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
        self.advance(); // consume 'r'

        let mut hash_count = 0;
        while self.peek() == Some('#') {
            self.advance();
            hash_count += 1;
        }

        if self.advance() != Some('"') {
            return Err(KdlError::Expected {
                expected: "'\"' to start raw string",
                found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                line: start_line,
                col: start_col,
            });
        }

        let mut s = String::new();
        while !self.is_eof() {
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
                while let Some(c) = self.peek() {
                    if c.is_ascii_hexdigit() {
                        hex.push(c);
                        self.advance();
                    } else if c == '_' {
                        self.advance(); // ignore underscores
                    } else {
                        break;
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
                while let Some(c) = self.peek() {
                    if ('0'..='7').contains(&c) {
                        oct.push(c);
                        self.advance();
                    } else if c == '_' {
                        self.advance();
                    } else {
                        break;
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
                while let Some(c) = self.peek() {
                    if c == '0' || c == '1' {
                        bin.push(c);
                        self.advance();
                    } else if c == '_' {
                        self.advance();
                    } else {
                        break;
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
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                raw.push(c);
                self.advance();
            } else if c == '_' {
                self.advance(); // ignore separator
            } else if c == '.' && !is_float && self.peek_at(1).map_or(false, |n| n.is_ascii_digit()) {
                is_float = true;
                raw.push(c);
                self.advance();
            } else if (c == 'e' || c == 'E') && !is_float {
                is_float = true;
                raw.push(c);
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        raw.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
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

/// Helper to test if a character can be part of a bare KDL identifier.
fn is_bare_ident_char(ch: char) -> bool {
    !matches!(
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
            | ','
            | ' '
            | '\t'
            | '\r'
            | '\n'
    ) && !ch.is_control()
}
