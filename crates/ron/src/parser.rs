//! RON (Rusty Object Notation) parser.
//!
//! Parses RON documents into Babbel's universal `Value` AST.

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
    let val = parser.parse_value()?;
    parser.skip_whitespace_and_comments()?;
    if let Some(ch) = parser.peek() {
        let (line, col) = parser.current_line_col();
        return Err(RonError::UnexpectedChar { ch, line, col });
    }
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

    pub fn parse_value(&mut self) -> Result<Value, RonError> {
        self.skip_whitespace_and_comments()?;
        let (line, col) = self.current_line_col();

        match self.peek() {
            Some('(') => self.parse_paren_container(),
            Some('[') => self.parse_list(),
            Some('{') => self.parse_map(),
            Some('"') => self.parse_string(),
            Some('\'') => self.parse_char(),
            Some('r') if self.peek_next() == Some('"') || self.peek_next() == Some('#') => {
                self.parse_raw_string()
            }
            Some('b') if self.peek_next() == Some('"') || self.peek_next() == Some('r') || self.peek_next() == Some('\'') => {
                self.parse_byte_literal()
            }
            Some('+') | Some('-') | Some('.') | Some('0'..='9') => self.parse_number(),
            Some(c) if is_ident_start(c) => self.parse_ident_or_struct(),
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
        // Look ahead to check if the first item has a colon
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
                    return Err(RonError::Expected { expected: "':' after field name", found: self.peek().map(|c| c.to_string()).unwrap_or_default(), line, col });
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
                    Some(ch) => {
                        let (line, col) = self.current_line_col();
                        return Err(RonError::Expected { expected: "',' or ')'", found: ch.to_string(), line, col });
                    }
                    None => return Err(RonError::UnexpectedEof),
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
                    Some(ch) => {
                        let (line, col) = self.current_line_col();
                        return Err(RonError::Expected { expected: "',' or ')'", found: ch.to_string(), line, col });
                    }
                    None => return Err(RonError::UnexpectedEof),
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
            match self.peek() {
                Some(',') => {
                    self.cursor += 1;
                }
                Some(']') => {
                    self.cursor += 1;
                    self.depth -= 1;
                    return Ok(Value::Array(items));
                }
                Some(ch) => {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::Expected { expected: "',' or ']'", found: ch.to_string(), line, col });
                }
                None => return Err(RonError::UnexpectedEof),
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

            let key_val = self.parse_value()?;
            let key = match key_val {
                Value::String(s) => s,
                Value::Integer(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                other => format!("{:?}", other),
            };

            self.skip_whitespace_and_comments()?;
            if self.peek() != Some(':') {
                let (line, col) = self.current_line_col();
                return Err(RonError::Expected { expected: "':' after map key", found: self.peek().map(|c| c.to_string()).unwrap_or_default(), line, col });
            }
            self.cursor += 1; // consume ':'

            let value = self.parse_value()?;
            entries.push((key, value));

            self.skip_whitespace_and_comments()?;
            match self.peek() {
                Some(',') => {
                    self.cursor += 1;
                }
                Some('}') => {
                    self.cursor += 1;
                    self.depth -= 1;
                    return Ok(Value::Object(entries));
                }
                Some(ch) => {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::Expected { expected: "',' or '}'", found: ch.to_string(), line, col });
                }
                None => return Err(RonError::UnexpectedEof),
            }
        }
    }

    fn parse_ident_or_struct(&mut self) -> Result<Value, RonError> {
        let ident = self.parse_ident_name()?;

        // Check special identifier values
        match ident.as_str() {
            "true" => return Ok(Value::Bool(true)),
            "false" => return Ok(Value::Bool(false)),
            "None" => return Ok(Value::Null),
            "inf" => return Ok(Value::Float(core::f64::INFINITY)),
            "NaN" => return Ok(Value::Float(core::f64::NAN)),
            _ => {}
        }

        self.skip_whitespace_and_comments()?;

        // Named struct or enum tuple: `Ident(...)`
        if self.peek() == Some('(') {
            if ident == "Some" {
                // `Some(inner)`
                self.cursor += 1; // consume '('
                let inner = self.parse_value()?;
                self.skip_whitespace_and_comments()?;
                if self.peek() == Some(',') {
                    self.cursor += 1;
                }
                self.skip_whitespace_and_comments()?;
                if self.peek() != Some(')') {
                    let (line, col) = self.current_line_col();
                    return Err(RonError::Expected { expected: "')' closing Some()", found: self.peek().map(|c| c.to_string()).unwrap_or_default(), line, col });
                }
                self.cursor += 1; // consume ')'
                return Ok(inner);
            }

            // Normal named struct/tuple: `Point(x: 1, y: 2)`
            let container = self.parse_paren_container()?;
            return Ok(container);
        }

        // Just an identifier / unit variant (e.g. `Active`, `Direction::North`)
        Ok(Value::String(ident))
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

    fn parse_string(&mut self) -> Result<Value, RonError> {
        self.cursor += 1; // consume opening '"'
        let mut s = String::new();

        while let Some(ch) = self.next_char() {
            if ch == '"' {
                return Ok(Value::String(s));
            } else if ch == '\\' {
                match self.next_char() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('/') => s.push('/'),
                    Some('b') => s.push('\u{0008}'),
                    Some('f') => s.push('\u{000C}'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('0') => s.push('\0'),
                    Some('x') => {
                        let h1 = self.next_char().ok_or(RonError::UnexpectedEof)?;
                        let h2 = self.next_char().ok_or(RonError::UnexpectedEof)?;
                        let hex_str = format!("{}{}", h1, h2);
                        let byte = u8::from_str_radix(&hex_str, 16).map_err(|_| {
                            let (line, col) = self.current_line_col();
                            RonError::InvalidEscape { sequence: hex_str, line, col }
                        })?;
                        s.push(byte as char);
                    }
                    Some('u') => {
                        if self.peek() == Some('{') {
                            self.next_char(); // consume '{'
                            let mut hex = String::new();
                            while let Some(c) = self.next_char() {
                                if c == '}' {
                                    break;
                                }
                                hex.push(c);
                            }
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| {
                                let (line, col) = self.current_line_col();
                                RonError::InvalidEscape { sequence: hex.clone(), line, col }
                            })?;
                            let decoded = char::from_u32(code).ok_or_else(|| {
                                let (line, col) = self.current_line_col();
                                RonError::InvalidEscape { sequence: hex, line, col }
                            })?;
                            s.push(decoded);
                        } else {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                hex.push(self.next_char().ok_or(RonError::UnexpectedEof)?);
                            }
                            let code = u32::from_str_radix(&hex, 16).map_err(|_| {
                                let (line, col) = self.current_line_col();
                                RonError::InvalidEscape { sequence: hex.clone(), line, col }
                            })?;
                            let decoded = char::from_u32(code).ok_or_else(|| {
                                let (line, col) = self.current_line_col();
                                RonError::InvalidEscape { sequence: hex, line, col }
                            })?;
                            s.push(decoded);
                        }
                    }
                    Some(other) => s.push(other),
                    None => return Err(RonError::UnexpectedEof),
                }
            } else {
                s.push(ch);
            }
        }

        Err(RonError::UnexpectedEof)
    }

    fn parse_char(&mut self) -> Result<Value, RonError> {
        self.cursor += 1; // consume opening '\''
        let ch = self.next_char().ok_or(RonError::UnexpectedEof)?;
        let val = if ch == '\\' {
            match self.next_char() {
                Some('n') => '\n',
                Some('r') => '\r',
                Some('t') => '\t',
                Some('\\') => '\\',
                Some('\'') => '\'',
                Some('0') => '\0',
                Some(other) => other,
                None => return Err(RonError::UnexpectedEof),
            }
        } else {
            ch
        };

        if self.next_char() != Some('\'') {
            let (line, col) = self.current_line_col();
            return Err(RonError::Expected { expected: "closing '\''", found: String::new(), line, col });
        }

        Ok(Value::String(val.to_string()))
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
            return Err(RonError::Expected { expected: "'\"' starting raw string", found: self.peek().map(|c| c.to_string()).unwrap_or_default(), line, col });
        }
        self.cursor += 1; // consume '"'

        let mut content = String::new();
        while let Some(ch) = self.next_char() {
            if ch == '"' {
                // Check if followed by hash_count '#'
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
            // raw byte string: br"..."
            let str_val = self.parse_raw_string()?;
            if let Value::String(s) = str_val {
                return Ok(Value::Bytes(s.into_bytes()));
            }
        } else if self.peek() == Some('"') {
            // normal byte string: b"..."
            let str_val = self.parse_string()?;
            if let Value::String(s) = str_val {
                return Ok(Value::Bytes(s.into_bytes()));
            }
        } else if self.peek() == Some('\'') {
            // byte char: b'x'
            self.cursor += 1; // consume '\''
            let b = self.next_char().ok_or(RonError::UnexpectedEof)?;
            if self.next_char() != Some('\'') {
                let (line, col) = self.current_line_col();
                return Err(RonError::Expected { expected: "closing '\''", found: String::new(), line, col });
            }
            return Ok(Value::Integer(b as u8 as i128));
        }

        let (line, col) = self.current_line_col();
        Err(RonError::Expected { expected: "byte literal (b\"...\" or b'...)", found: self.peek().map(|c| c.to_string()).unwrap_or_default(), line, col })
    }

    fn parse_number(&mut self) -> Result<Value, RonError> {
        let (line, col) = self.current_line_col();
        let mut token = String::new();

        // Optional sign
        if let Some(c) = self.peek() {
            if c == '+' || c == '-' {
                token.push(c);
                self.cursor += 1;
            }
        }

        // Check for special float literals inf, NaN
        if self.consume_str("inf") {
            let is_neg = token.starts_with('-');
            let f = if is_neg { -core::f64::INFINITY } else { core::f64::INFINITY };
            return Ok(Value::Float(f));
        }
        if self.consume_str("NaN") {
            return Ok(Value::Float(core::f64::NAN));
        }

        // Check for radices: 0x (hex), 0b (bin), 0o (oct)
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
                                self.cursor += 1; // skip underscore
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

        // Standard decimal / float
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = self.peek() {
            match ch {
                '_' => {
                    self.cursor += 1; // skip underscore separator
                }
                '.' => {
                    // Check if it's a field access or tuple dot, like in `0.field`
                    if let Some(next) = self.peek_next() {
                        if !next.is_ascii_digit() && next != 'e' && next != 'E' {
                            // Trailing dot like `5.` is allowed in RON
                        }
                    }
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

#[inline]
fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

#[inline]
fn is_ident_part(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
