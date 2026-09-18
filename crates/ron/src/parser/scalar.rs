//! RON scalar and literal parsing (numbers, identifiers, strings, raw strings, bytes).

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::{RonParser, is_ident_part, is_structural_delimiter};
use crate::error::RonError;
use babbel_core::Value;

impl<'a> RonParser<'a> {
    pub(crate) fn scan_bare_atom(&mut self) -> Result<String, RonError> {
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
            Err(RonError::Expected {
                expected: "bare token",
                found: String::new(),
                line,
                col,
            })
        } else {
            Ok(atom)
        }
    }

    pub(crate) fn parse_bare_value(&mut self) -> Result<Value, RonError> {
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

    pub(crate) fn parse_comma_prefixed_token(&mut self) -> Result<String, RonError> {
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

    pub(crate) fn parse_quoted_string(&mut self) -> Result<String, RonError> {
        let (line, col) = self.current_line_col();
        let quote = self.peek().ok_or(RonError::UnexpectedEof)?;
        if quote != '"' && quote != '\'' {
            return Err(RonError::UnexpectedChar {
                ch: quote,
                line,
                col,
            });
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
                                atom_helper_push(&mut content, h);
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
                while check_cursor + run_len < self.chars.len()
                    && self.chars[check_cursor + run_len].1 == quote
                {
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

    pub(crate) fn parse_number_or_bare_token(&mut self) -> Result<Value, RonError> {
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

    pub(crate) fn parse_ident_name(&mut self) -> Result<String, RonError> {
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
            Err(RonError::Expected {
                expected: "identifier",
                found: String::new(),
                line,
                col,
            })
        } else {
            Ok(name)
        }
    }

    pub(crate) fn parse_raw_string(&mut self) -> Result<Value, RonError> {
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

    pub(crate) fn parse_byte_literal(&mut self) -> Result<Value, RonError> {
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
                return Err(RonError::Expected {
                    expected: "closing '\''",
                    found: String::new(),
                    line,
                    col,
                });
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
}

#[inline]
fn atom_helper_push(content: &mut String, h: char) {
    content.push(h);
}

pub(crate) fn decode_escapes(
    raw: &str,
    start_line: usize,
    start_col: usize,
) -> Result<String, RonError> {
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
                        let code =
                            u32::from_str_radix(&hex, 16).map_err(|_| RonError::InvalidEscape {
                                sequence: hex.clone(),
                                line: start_line,
                                col: start_col,
                            })?;
                        let decoded =
                            char::from_u32(code).ok_or_else(|| RonError::InvalidEscape {
                                sequence: hex,
                                line: start_line,
                                col: start_col,
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
                            RonError::InvalidEscape {
                                sequence: hex_str.clone(),
                                line: start_line,
                                col: start_col,
                            }
                        })?;

                        if (0xD800..=0xDBFF).contains(&code) {
                            if i + 6 <= chars.len() && chars[i] == '\\' && chars[i + 1] == 'u' {
                                let low_hex: String = chars[i + 2..i + 6].iter().collect();
                                if low_hex.chars().all(|c| c.is_ascii_hexdigit()) {
                                    let low_code = u32::from_str_radix(&low_hex, 16).unwrap_or(0);
                                    if (0xDC00..=0xDFFF).contains(&low_code) {
                                        i += 6;
                                        let combined = 0x10000
                                            + (((code - 0xD800) << 10) | (low_code - 0xDC00));
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
                            let decoded =
                                char::from_u32(code).ok_or_else(|| RonError::InvalidEscape {
                                    sequence: hex_str.clone(),
                                    line: start_line,
                                    col: start_col,
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
                    let byte =
                        u8::from_str_radix(&hex_str, 16).map_err(|_| RonError::InvalidEscape {
                            sequence: hex_str.clone(),
                            line: start_line,
                            col: start_col,
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
            return Err(RonError::UnexpectedChar {
                ch,
                line: start_line,
                col: start_col,
            });
        } else {
            result.push(ch);
            i += 1;
        }
    }
    Ok(result)
}
