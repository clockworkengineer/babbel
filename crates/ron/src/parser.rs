//! RON (Rusty Object Notation / Readable Object Notation) parser.
//!
//! Parses RON documents into Babbel's universal `Value` AST.
//! Supports both Rusty Object Notation and language-neutral Readable Object Notation (starfederation/ron).

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use babbel_core::Value;
use crate::error::RonError;

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
    input: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    depth: usize,
    max_depth: usize,
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

    fn current_line_col(&self) -> (usize, usize) {
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
    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|&(_, c)| c)
    }

    #[inline]
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|&(_, c)| c)
    }

    #[inline]
    fn next_char(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor].1;
            self.cursor += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), RonError> {
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

    fn is_named_struct_start(&self) -> bool {
        let mut i = self.cursor;
        while i < self.chars.len() && self.chars[i].1.is_whitespace() {
            i += 1;
        }
        if i < self.chars.len() && is_ident_start(self.chars[i].1) {
            while i < self.chars.len()
                && (is_ident_part(self.chars[i].1)
                    || (self.chars[i].1 == ':' && i + 1 < self.chars.len() && self.chars[i + 1].1 == ':'))
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

    fn is_raw_or_byte_literal_start(&self) -> bool {
        let mut i = self.cursor;
        while i < self.chars.len() && self.chars[i].1.is_whitespace() {
            i += 1;
        }
        if i < self.chars.len() {
            let c = self.chars[i].1;
            let next = if i + 1 < self.chars.len() { Some(self.chars[i + 1].1) } else { None };
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
            Some('+') | Some('-') | Some('.') | Some('0'..='9') => self.parse_number_or_bare_token(),
            Some(c) if !c.is_whitespace() && !is_structural_delimiter(c) => self.parse_bare_value(),
            Some(ch) => Err(RonError::UnexpectedChar { ch, line, col }),
            None => Err(RonError::UnexpectedEof),
        }
    }

    /// Handles both `()` (unit/null), `(field: val, ...)` (struct), and `(val, val)` (tuple).
    fn parse_paren_container(&mut self) -> Result<Value, RonError> {
        if self.depth >= self.max_depth {
            return Err(RonError::RecursionLimitExceeded { depth: self.depth, max: self.max_depth });
        }
        self.depth += 1;
        self.cursor += 1; // consume '('

        self.skip_whitespace_and_comments()?;
        if self.peek() == Some(')') {
            self.cursor += 1; // consume ')'
            self.depth -= 1;
            return Ok(Value::Null); // Unit type `()`
        }

        // Determine if this is a named-field struct `(a: 1)` or a tuple `(1, 2)`
        let checkpoint = self.cursor;
        let is_struct = self.peek_is_struct_field()?;
        self.cursor = checkpoint;

        if is_struct {
            let mut fields = Vec::new();
            loop {
                self.skip_whitespace_and_comments()?;
                if self.peek() == Some(')') {
                    self.cursor += 1;
                    self.depth -= 1;
                    return Ok(Value::Object(fields));
                }

                let key = self.parse_ident_name()?;
                self.skip_whitespace_and_comments()?;
                if self.peek() != Some(':') {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::Expected {
                        expected: "':' after field name",
                        found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                        line,
                        col,
                    });
                }
                self.cursor += 1; // consume ':'
                let val = self.parse_value()?;
                fields.push((key, val));

                self.skip_whitespace_and_comments()?;
                match self.peek() {
                    Some(',') => {
                        self.cursor += 1; // consume ','
                    }
                    Some(')') => {
                        self.cursor += 1;
                        self.depth -= 1;
                        return Ok(Value::Object(fields));
                    }
                    _ => {}
                }
            }
        } else {
            // Tuple: `(elem1, elem2, ...)` -> Array
            let mut items = Vec::new();
            loop {
                self.skip_whitespace_and_comments()?;
                if self.peek() == Some(')') {
                    self.cursor += 1;
                    self.depth -= 1;
                    return Ok(Value::Array(items));
                }

                let item = self.parse_value()?;
                items.push(item);

                self.skip_whitespace_and_comments()?;
                match self.peek() {
                    Some(',') => {
                        self.cursor += 1;
                    }
                    Some(')') => {
                        self.cursor += 1;
                        self.depth -= 1;
                        return Ok(Value::Array(items));
                    }
                    _ => {}
                }
            }
        }
    }

    fn peek_is_struct_field(&mut self) -> Result<bool, RonError> {
        self.skip_whitespace_and_comments()?;
        if let Some(c) = self.peek() {
            if is_ident_start(c) {
                while let Some(ch) = self.peek() {
                    if is_ident_part(ch) {
                        self.cursor += 1;
                    } else if ch == ':' && self.peek_next() == Some(':') {
                        self.cursor += 2;
                    } else {
                        break;
                    }
                }
                self.skip_whitespace_and_comments()?;
                return Ok(self.peek() == Some(':') && self.peek_next() != Some(':'));
            }
        }
        Ok(false)
    }

    fn parse_list(&mut self) -> Result<Value, RonError> {
        if self.depth >= self.max_depth {
            return Err(RonError::RecursionLimitExceeded { depth: self.depth, max: self.max_depth });
        }
        self.depth += 1;
        self.cursor += 1; // consume '['

        let mut items = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(']') {
                self.cursor += 1;
                self.depth -= 1;
                return Ok(Value::Array(items));
            }

            let item = self.parse_value()?;
            items.push(item);

            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(',') {
                self.cursor += 1;
            }
        }
    }

    fn parse_map(&mut self) -> Result<Value, RonError> {
        if self.depth >= self.max_depth {
            return Err(RonError::RecursionLimitExceeded { depth: self.depth, max: self.max_depth });
        }
        self.depth += 1;
        self.cursor += 1; // consume '{'

        let mut entries = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;
            if self.peek() == Some('}') {
                self.cursor += 1;
                self.depth -= 1;
                return Ok(Value::Object(entries));
            }

            let key = self.parse_key_string()?;

            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(':') {
                self.cursor += 1;
            }

            self.skip_whitespace_and_comments()?;
            if self.peek() == Some('}') || self.peek().is_none() {
                let (line, col) = self.current_line_col();
                return Err(RonError::Expected {
                    expected: "value after map key",
                    found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                    line,
                    col,
                });
            }

            let value = self.parse_value()?;
            entries.push((key, value));

            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(',') {
                self.cursor += 1;
            }
        }
    }

    fn parse_elided_map(&mut self) -> Result<Value, RonError> {
        let mut entries = Vec::new();

        while self.cursor < self.chars.len() {
            self.skip_whitespace_and_comments()?;
            if self.cursor >= self.chars.len() {
                break;
            }

            if let Some(c) = self.peek() {
                if c == '{' || c == '}' || c == '[' || c == ']' {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::UnexpectedChar { ch: c, line, col });
                }
            }

            let key = self.parse_key_string()?;

            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(':') {
                self.cursor += 1;
            }

            self.skip_whitespace_and_comments()?;
            if self.cursor >= self.chars.len() {
                return Err(RonError::UnexpectedEof);
            }

            let value = self.parse_value()?;
            entries.push((key, value));

            self.skip_whitespace_and_comments()?;
            if self.peek() == Some(',') {
                self.cursor += 1;
            }
        }

        if entries.is_empty() {
            Err(RonError::UnexpectedEof)
        } else {
            Ok(Value::Object(entries))
        }
    }

    fn parse_key_string(&mut self) -> Result<String, RonError> {
        self.skip_whitespace_and_comments()?;
        let (line, col) = self.current_line_col();

        match self.peek() {
            Some('"') | Some('\'') => self.parse_quoted_string(),
            Some(',') => self.parse_comma_prefixed_token(),
            Some('{') | Some('}') | Some('[') | Some(']') => {
                Err(RonError::UnexpectedChar { ch: self.peek().unwrap(), line, col })
            }
            Some(_) => {
                let atom = self.scan_bare_atom()?;
                decode_escapes(&atom, line, col)
            }
            None => Err(RonError::UnexpectedEof),
        }
    }

    fn scan_bare_atom(&mut self) -> Result<String, RonError> {
        let mut atom = String::new();
        let (line, col) = self.current_line_col();

        while let Some(ch) = self.peek() {
            if ch == '\\' {
                atom.push(self.next_char().unwrap());
                if let Some(esc) = self.peek() {
                    if esc == 'u' {
                        atom.push(self.next_char().unwrap());
                        if self.peek() == Some('{') {
                            atom.push(self.next_char().unwrap());
                            while let Some(h) = self.peek() {
                                atom.push(self.next_char().unwrap());
                                if h == '}' {
                                    break;
                                }
                            }
                        } else {
                            for _ in 0..4 {
                                if let Some(h) = self.next_char() {
                                    atom.push(h);
                                } else {
                                    return Err(RonError::UnexpectedEof);
                                }
                            }
                        }
                    } else if esc == 'x' {
                        atom.push(self.next_char().unwrap());
                        for _ in 0..2 {
                            if let Some(h) = self.next_char() {
                                atom.push(h);
                            } else {
                                return Err(RonError::UnexpectedEof);
                            }
                        }
                    } else {
                        atom.push(self.next_char().unwrap());
                    }
                } else {
                    return Err(RonError::UnexpectedEof);
                }
            } else if ch == ':' {
                if self.peek_next() == Some(':') {
                    atom.push(self.next_char().unwrap());
                    atom.push(self.next_char().unwrap());
                } else {
                    break;
                }
            } else if ch.is_whitespace() || is_structural_delimiter(ch) {
                break;
            } else if (ch as u32) < 0x20 {
                return Err(RonError::UnexpectedChar { ch, line, col });
            } else {
                atom.push(self.next_char().unwrap());
            }
        }

        if atom.is_empty() {
            Err(RonError::Expected { expected: "bare token", found: String::new(), line, col })
        } else {
            Ok(atom)
        }
    }

    fn parse_bare_value(&mut self) -> Result<Value, RonError> {
        let (line, col) = self.current_line_col();
        let atom = self.scan_bare_atom()?;

        // Check if followed by '(' -> Rust struct or Some(...)
        if self.peek() == Some('(') {
            if atom == "Some" {
                self.cursor += 1; // consume '('
                let inner = self.parse_value()?;
                self.skip_whitespace_and_comments()?;
                if self.peek() == Some(',') {
                    self.cursor += 1;
                }
                self.skip_whitespace_and_comments()?;
                if self.peek() != Some(')') {
                    let (l, c) = self.current_line_col();
                    return Err(RonError::Expected {
                        expected: "')' closing Some()",
                        found: self.peek().map(|ch| ch.to_string()).unwrap_or_default(),
                        line: l,
                        col: c,
                    });
                }
                self.cursor += 1; // consume ')'
                return Ok(inner);
            } else {
                let container = self.parse_paren_container()?;
                return Ok(container);
            }
        }

        // Exact unescaped keywords
        if !atom.contains('\\') {
            match atom.as_str() {
                "true" => return Ok(Value::Bool(true)),
                "false" => return Ok(Value::Bool(false)),
                "null" | "None" => return Ok(Value::Null),
                "inf" => return Ok(Value::Float(core::f64::INFINITY)),
                "NaN" => return Ok(Value::Float(core::f64::NAN)),
                _ => {}
            }
        }

        let decoded = decode_escapes(&atom, line, col)?;
        Ok(Value::String(decoded))
    }

    fn parse_comma_prefixed_token(&mut self) -> Result<String, RonError> {
        let (line, col) = self.current_line_col();
        let mut atom = String::new();
        atom.push(self.next_char().unwrap()); // consume ','

        while let Some(ch) = self.peek() {
            if ch == '\\' {
                atom.push(self.next_char().unwrap());
                if let Some(esc) = self.peek() {
                    if esc == 'u' {
                        atom.push(self.next_char().unwrap());
                        if self.peek() == Some('{') {
                            atom.push(self.next_char().unwrap());
                            while let Some(h) = self.peek() {
                                atom.push(self.next_char().unwrap());
                                if h == '}' {
                                    break;
                                }
                            }
                        } else {
                            for _ in 0..4 {
                                if let Some(h) = self.next_char() {
                                    atom.push(h);
                                } else {
                                    return Err(RonError::UnexpectedEof);
                                }
                            }
                        }
                    } else if esc == 'x' {
                        atom.push(self.next_char().unwrap());
                        for _ in 0..2 {
                            if let Some(h) = self.next_char() {
                                atom.push(h);
                            } else {
                                return Err(RonError::UnexpectedEof);
                            }
                        }
                    } else {
                        atom.push(self.next_char().unwrap());
                    }
                } else {
                    return Err(RonError::UnexpectedEof);
                }
            } else if ch == ':' {
                if self.peek_next() == Some(':') {
                    atom.push(self.next_char().unwrap());
                    atom.push(self.next_char().unwrap());
                } else {
                    break;
                }
            } else if ch.is_whitespace() || is_structural_delimiter(ch) {
                break;
            } else if (ch as u32) < 0x20 {
                return Err(RonError::UnexpectedChar { ch, line, col });
            } else {
                atom.push(self.next_char().unwrap());
            }
        }

        decode_escapes(&atom, line, col)
    }

    fn parse_quoted_string(&mut self) -> Result<String, RonError> {
        let (line, col) = self.current_line_col();
        let quote = self.peek().ok_or(RonError::UnexpectedEof)?;
        if quote != '"' && quote != '\'' {
            return Err(RonError::UnexpectedChar { ch: quote, line, col });
        }

        // Count opening run length n
        let mut n = 0;
        while self.peek() == Some(quote) {
            n += 1;
            self.cursor += 1;
        }

        let next_is_delim = match self.peek() {
            None => true,
            Some(c) if c.is_whitespace() || c == ',' || c == ']' || c == '}' || c == ':' => true,
            _ => false,
        };

        if n % 2 == 0 && next_is_delim {
            return Ok(String::new());
        }

        if quote == '\'' && n >= 5 && (n - 2) % 3 == 0 && next_is_delim {
            let count = (n - 2) / 3;
            let mut res = String::new();
            for _ in 0..count {
                res.push('\'');
            }
            return Ok(res);
        }

        if quote == '\'' && n == 1 && self.peek().map_or(true, |c| c.is_whitespace()) {
            return Ok("'".to_string());
        }

        // Content starts after opening run
        let mut content = String::new();

        while self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor].1;
            if ch == '\\' {
                content.push(self.next_char().unwrap());
                if let Some(esc) = self.peek() {
                    if esc == 'u' {
                        content.push(self.next_char().unwrap());
                        if self.peek() == Some('{') {
                            content.push(self.next_char().unwrap());
                            while let Some(h) = self.peek() {
                                content.push(self.next_char().unwrap());
                                if h == '}' {
                                    break;
                                }
                            }
                        } else {
                            for _ in 0..4 {
                                if let Some(h) = self.next_char() {
                                    content.push(h);
                                } else {
                                    return Err(RonError::UnexpectedEof);
                                }
                            }
                        }
                    } else if esc == 'x' {
                        content.push(self.next_char().unwrap());
                        for _ in 0..2 {
                            if let Some(h) = self.next_char() {
                                content.push(h);
                            } else {
                                return Err(RonError::UnexpectedEof);
                            }
                        }
                    } else {
                        content.push(self.next_char().unwrap());
                    }
                } else {
                    return Err(RonError::UnexpectedEof);
                }
            } else if ch == quote {
                let mut run_len = 0;
                let check_cursor = self.cursor;
                while check_cursor + run_len < self.chars.len() && self.chars[check_cursor + run_len].1 == quote {
                    run_len += 1;
                }
                if run_len >= n {
                    self.cursor += n;
                    return decode_escapes(&content, line, col);
                } else {
                    for _ in 0..run_len {
                        content.push(self.next_char().unwrap());
                    }
                }
            } else if (ch as u32) < 0x20 {
                return Err(RonError::UnexpectedChar { ch, line, col });
            } else {
                content.push(self.next_char().unwrap());
            }
        }

        Err(RonError::UnexpectedEof)
    }

    fn parse_number_or_bare_token(&mut self) -> Result<Value, RonError> {
        let checkpoint = self.cursor;
        match self.parse_number() {
            Ok(num) => {
                if let Some(ch) = self.peek() {
                    if !ch.is_whitespace() && !is_structural_delimiter(ch) && ch != ':' {
                        self.cursor = checkpoint;
                        return self.parse_bare_value();
                    }
                }
                Ok(num)
            }
            Err(_) => {
                self.cursor = checkpoint;
                self.parse_bare_value()
            }
        }
    }

    fn parse_ident_name(&mut self) -> Result<String, RonError> {
        let mut name = String::new();
        let (line, col) = self.current_line_col();

        while let Some(ch) = self.peek() {
            if is_ident_part(ch) {
                name.push(ch);
                self.cursor += 1;
            } else if ch == ':' && self.peek_next() == Some(':') {
                name.push_str("::");
                self.cursor += 2;
            } else {
                break;
            }
        }

        if name.is_empty() {
            Err(RonError::Expected { expected: "identifier", found: String::new(), line, col })
        } else {
            Ok(name)
        }
    }

    fn parse_raw_string(&mut self) -> Result<Value, RonError> {
        self.cursor += 1; // consume 'r'
        let mut hash_count = 0;
        while self.peek() == Some('#') {
            hash_count += 1;
            self.cursor += 1;
        }

        if self.peek() != Some('"') {
            let (line, col) = self.current_line_col();
            return Err(RonError::Expected {
                expected: "'\"' starting raw string",
                found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                line,
                col,
            });
        }
        self.cursor += 1; // consume '"'

        let mut content = String::new();
        while let Some(ch) = self.next_char() {
            if ch == '"' {
                let mut matched_hashes = 0;
                while matched_hashes < hash_count && self.peek() == Some('#') {
                    matched_hashes += 1;
                    self.cursor += 1;
                }
                if matched_hashes == hash_count {
                    return Ok(Value::String(content));
                } else {
                    content.push('"');
                    for _ in 0..matched_hashes {
                        content.push('#');
                    }
                }
            } else {
                content.push(ch);
            }
        }

        Err(RonError::UnexpectedEof)
    }

    fn parse_byte_literal(&mut self) -> Result<Value, RonError> {
        self.cursor += 1; // consume 'b'
        if self.peek() == Some('r') {
            let str_val = self.parse_raw_string()?;
            if let Value::String(s) = str_val {
                return Ok(Value::Bytes(s.into_bytes()));
            }
        } else if self.peek() == Some('"') {
            let s = self.parse_quoted_string()?;
            return Ok(Value::Bytes(s.into_bytes()));
        } else if self.peek() == Some('\'') {
            self.cursor += 1; // consume '\''
            let b = self.next_char().ok_or(RonError::UnexpectedEof)?;
            if self.next_char() != Some('\'') {
                let (line, col) = self.current_line_col();
                return Err(RonError::Expected { expected: "closing '\''", found: String::new(), line, col });
            }
            return Ok(Value::Integer(b as u8 as i128));
        }

        let (line, col) = self.current_line_col();
        Err(RonError::Expected {
            expected: "byte literal (b\"...\" or b'...)",
            found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
            line,
            col,
        })
    }

    fn parse_number(&mut self) -> Result<Value, RonError> {
        let (line, col) = self.current_line_col();
        let mut token = String::new();

        if let Some(c) = self.peek() {
            if c == '+' || c == '-' {
                token.push(c);
                self.cursor += 1;
            }
        }

        if self.consume_str("inf") {
            let is_neg = token.starts_with('-');
            let f = if is_neg { -core::f64::INFINITY } else { core::f64::INFINITY };
            return Ok(Value::Float(f));
        }
        if self.consume_str("NaN") {
            return Ok(Value::Float(core::f64::NAN));
        }

        if self.peek() == Some('0') {
            if let Some(radix_char) = self.peek_next() {
                match radix_char {
                    'x' | 'X' => {
                        self.cursor += 2;
                        let mut digits = String::new();
                        while let Some(c) = self.peek() {
                            if c.is_ascii_hexdigit() {
                                digits.push(c);
                                self.cursor += 1;
                            } else if c == '_' {
                                self.cursor += 1;
                            } else {
                                break;
                            }
                        }
                        if digits.is_empty() {
                            return Err(RonError::InvalidNumber { literal: format!("{}0x", token), line, col });
                        }
                        let raw = i128::from_str_radix(&digits, 16).map_err(|_| {
                            RonError::InvalidNumber { literal: format!("{}0x{}", token, digits), line, col }
                        })?;
                        let val = if token.starts_with('-') { -raw } else { raw };
                        return Ok(Value::Integer(val));
                    }
                    'b' | 'B' => {
                        self.cursor += 2;
                        let mut digits = String::new();
                        while let Some(c) = self.peek() {
                            if c == '0' || c == '1' {
                                digits.push(c);
                                self.cursor += 1;
                            } else if c == '_' {
                                self.cursor += 1;
                            } else {
                                break;
                            }
                        }
                        if digits.is_empty() {
                            return Err(RonError::InvalidNumber { literal: format!("{}0b", token), line, col });
                        }
                        let raw = i128::from_str_radix(&digits, 2).map_err(|_| {
                            RonError::InvalidNumber { literal: format!("{}0b{}", token, digits), line, col }
                        })?;
                        let val = if token.starts_with('-') { -raw } else { raw };
                        return Ok(Value::Integer(val));
                    }
                    'o' | 'O' => {
                        self.cursor += 2;
                        let mut digits = String::new();
                        while let Some(c) = self.peek() {
                            if ('0'..='7').contains(&c) {
                                digits.push(c);
                                self.cursor += 1;
                            } else if c == '_' {
                                self.cursor += 1;
                            } else {
                                break;
                            }
                        }
                        if digits.is_empty() {
                            return Err(RonError::InvalidNumber { literal: format!("{}0o", token), line, col });
                        }
                        let raw = i128::from_str_radix(&digits, 8).map_err(|_| {
                            RonError::InvalidNumber { literal: format!("{}0o{}", token, digits), line, col }
                        })?;
                        let val = if token.starts_with('-') { -raw } else { raw };
                        return Ok(Value::Integer(val));
                    }
                    _ => {}
                }
            }
        }

        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = self.peek() {
            match ch {
                '_' => {
                    self.cursor += 1;
                }
                '.' => {
                    if has_dot || has_exp {
                        break;
                    }
                    has_dot = true;
                    token.push(ch);
                    self.cursor += 1;
                }
                '0'..='9' => {
                    token.push(ch);
                    self.cursor += 1;
                }
                'e' | 'E' => {
                    if has_exp {
                        break;
                    }
                    has_exp = true;
                    token.push(ch);
                    self.cursor += 1;
                    if let Some(s) = self.peek() {
                        if s == '+' || s == '-' {
                            token.push(s);
                            self.cursor += 1;
                        }
                    }
                }
                _ => break,
            }
        }

        let clean = token.strip_prefix('+').unwrap_or(&token);

        if has_dot || has_exp {
            let f = clean.parse::<f64>().map_err(|_| RonError::InvalidNumber { literal: token.clone(), line, col })?;
            Ok(Value::Float(f))
        } else if let Ok(i) = clean.parse::<i128>() {
            Ok(Value::Integer(i))
        } else if let Ok(f) = clean.parse::<f64>() {
            Ok(Value::Float(f))
        } else {
            Err(RonError::InvalidNumber { literal: token, line, col })
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
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

fn decode_escapes(raw: &str, start_line: usize, start_col: usize) -> Result<String, RonError> {
    let mut result = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' {
            i += 1;
            if i >= chars.len() {
                return Err(RonError::UnexpectedEof);
            }
            match chars[i] {
                '"' => {
                    result.push('"');
                    i += 1;
                }
                '\\' => {
                    result.push('\\');
                    i += 1;
                }
                '/' => {
                    result.push('/');
                    i += 1;
                }
                'b' => {
                    result.push('\u{0008}');
                    i += 1;
                }
                'f' => {
                    result.push('\u{000C}');
                    i += 1;
                }
                'n' => {
                    result.push('\n');
                    i += 1;
                }
                'r' => {
                    result.push('\r');
                    i += 1;
                }
                't' => {
                    result.push('\t');
                    i += 1;
                }
                '\'' => {
                    result.push('\'');
                    i += 1;
                }
                '0' => {
                    result.push('\0');
                    i += 1;
                }
                'u' => {
                    i += 1;
                    if i < chars.len() && chars[i] == '{' {
                        i += 1;
                        let mut hex = String::new();
                        while i < chars.len() && chars[i] != '}' {
                            hex.push(chars[i]);
                            i += 1;
                        }
                        if i >= chars.len() || chars[i] != '}' {
                            return Err(RonError::UnexpectedEof);
                        }
                        i += 1; // consume '}'
                        let code = u32::from_str_radix(&hex, 16).map_err(|_| {
                            RonError::InvalidEscape { sequence: hex.clone(), line: start_line, col: start_col }
                        })?;
                        let decoded = char::from_u32(code).ok_or_else(|| {
                            RonError::InvalidEscape { sequence: hex, line: start_line, col: start_col }
                        })?;
                        result.push(decoded);
                    } else {
                        if i + 4 > chars.len() {
                            return Err(RonError::InvalidEscape {
                                sequence: chars[i..].iter().collect(),
                                line: start_line,
                                col: start_col,
                            });
                        }
                        let hex_str: String = chars[i..i + 4].iter().collect();
                        if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
                            return Err(RonError::InvalidEscape {
                                sequence: hex_str,
                                line: start_line,
                                col: start_col,
                            });
                        }
                        i += 4;
                        let code = u32::from_str_radix(&hex_str, 16).map_err(|_| {
                            RonError::InvalidEscape { sequence: hex_str.clone(), line: start_line, col: start_col }
                        })?;

                        if (0xD800..=0xDBFF).contains(&code) {
                            if i + 6 <= chars.len() && chars[i] == '\\' && chars[i + 1] == 'u' {
                                let low_hex: String = chars[i + 2..i + 6].iter().collect();
                                if low_hex.chars().all(|c| c.is_ascii_hexdigit()) {
                                    let low_code = u32::from_str_radix(&low_hex, 16).unwrap_or(0);
                                    if (0xDC00..=0xDFFF).contains(&low_code) {
                                        i += 6;
                                        let combined = 0x10000 + (((code - 0xD800) << 10) | (low_code - 0xDC00));
                                        if let Some(c) = char::from_u32(combined) {
                                            result.push(c);
                                            continue;
                                        }
                                    }
                                }
                            }
                            return Err(RonError::InvalidEscape {
                                sequence: hex_str,
                                line: start_line,
                                col: start_col,
                            });
                        } else if (0xDC00..=0xDFFF).contains(&code) {
                            return Err(RonError::InvalidEscape {
                                sequence: hex_str,
                                line: start_line,
                                col: start_col,
                            });
                        } else {
                            let decoded = char::from_u32(code).ok_or_else(|| {
                                RonError::InvalidEscape { sequence: hex_str.clone(), line: start_line, col: start_col }
                            })?;
                            result.push(decoded);
                        }
                    }
                }
                'x' => {
                    i += 1;
                    if i + 2 > chars.len() {
                        return Err(RonError::UnexpectedEof);
                    }
                    let hex_str: String = chars[i..i + 2].iter().collect();
                    i += 2;
                    let byte = u8::from_str_radix(&hex_str, 16).map_err(|_| {
                        RonError::InvalidEscape { sequence: hex_str.clone(), line: start_line, col: start_col }
                    })?;
                    result.push(byte as char);
                }
                other => {
                    return Err(RonError::InvalidEscape {
                        sequence: other.to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
            }
        } else if (ch as u32) < 0x20 {
            return Err(RonError::UnexpectedChar { ch, line: start_line, col: start_col });
        } else {
            result.push(ch);
            i += 1;
        }
    }
    Ok(result)
}

#[inline]
fn is_structural_delimiter(ch: char) -> bool {
    matches!(ch, '{' | '}' | '[' | ']' | '"' | '\'' | ',' | '(' | ')')
}

#[inline]
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

#[inline]
fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
