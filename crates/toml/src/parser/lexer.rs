//! Lexical scanner implementing the TOML v1.0.0 specification.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

use crate::error::TomlError;
use crate::nodes::TomlDatetime;
use super::tokens::{SpannedToken, Token};

/// Lexer state scanning characters and yielding tokens.
pub struct Lexer<'a> {
    _input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given string.
    pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,
            bytes: input.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// Tokenize the entire input into a list of spanned tokens.
    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, TomlError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.token == Token::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            Some(self.bytes[self.pos])
        } else {
            None
        }
    }

    fn peek_ahead(&self, offset: usize) -> Option<u8> {
        let idx = self.pos + offset;
        if idx < self.bytes.len() {
            Some(self.bytes[idx])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            if b == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(b)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b'\t' | b' ' | b'\r' => {
                    self.advance();
                }
                b'#' => {
                    // Comment until newline or EOF
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    /// Read the next token from input.
    pub fn next_token(&mut self) -> Result<SpannedToken, TomlError> {
        self.skip_whitespace_and_comments();

        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.col;

        let b = match self.peek() {
            Some(b) => b,
            None => {
                return Ok(SpannedToken {
                    token: Token::Eof,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                });
            }
        };

        match b {
            b'\n' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::Newline,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b'=' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::Equals,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b'.' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::Period,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b',' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::Comma,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b'{' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::LBrace,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b'}' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::RBrace,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b'[' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::LBracket,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b']' => {
                self.advance();
                Ok(SpannedToken {
                    token: Token::RBracket,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                })
            }
            b'"' => self.scan_basic_string(start_pos, start_line, start_col),
            b'\'' => self.scan_literal_string(start_pos, start_line, start_col),
            _ => self.scan_bare_or_literal(start_pos, start_line, start_col),
        }
    }

    fn scan_basic_string(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<SpannedToken, TomlError> {
        // Check for multiline """
        if self.peek_ahead(1) == Some(b'"') && self.peek_ahead(2) == Some(b'"') {
            self.advance();
            self.advance();
            self.advance();

            // Strip first newline immediately following """
            if self.peek() == Some(b'\r') && self.peek_ahead(1) == Some(b'\n') {
                self.advance();
                self.advance();
            } else if self.peek() == Some(b'\n') {
                self.advance();
            }

            let mut out = String::new();
            loop {
                if self.pos >= self.bytes.len() {
                    return Err(TomlError::syntax(
                        "Unclosed multi-line basic string",
                        start_line,
                        start_col,
                        start_pos,
                    ));
                }

                if self.peek() == Some(b'"')
                    && self.peek_ahead(1) == Some(b'"')
                    && self.peek_ahead(2) == Some(b'"')
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    break;
                }

                if self.peek() == Some(b'\\') {
                    self.advance();
                    let mut is_eol_backslash = false;
                    let mut lookahead = 0;
                    while let Some(b) = self.peek_ahead(lookahead) {
                        if b == b' ' || b == b'\t' {
                            lookahead += 1;
                        } else if b == b'\r' || b == b'\n' {
                            is_eol_backslash = true;
                            break;
                        } else {
                            break;
                        }
                    }

                    if is_eol_backslash {
                        // Line ending backslash: trim whitespace and newlines
                        while let Some(w) = self.peek() {
                            if w == b' ' || w == b'\t' || w == b'\r' || w == b'\n' {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    } else {
                        let ch = self.parse_escape_sequence(start_line, start_col, start_pos)?;
                        out.push(ch);
                    }
                } else if self.peek() == Some(b'\r') && self.peek_ahead(1) == Some(b'\n') {
                    self.advance();
                    self.advance();
                    out.push('\n');
                } else {
                    let b = self.advance().unwrap();
                    out.push(b as char);
                }
            }

            Ok(SpannedToken {
                token: Token::String(out),
                line: start_line,
                column: start_col,
                position: start_pos,
            })
        } else {
            // Single-line basic string "..."
            self.advance();
            let mut out = String::new();
            loop {
                let b = match self.advance() {
                    Some(b) => b,
                    None => {
                        return Err(TomlError::syntax(
                            "Unclosed string literal",
                            start_line,
                            start_col,
                            start_pos,
                        ));
                    }
                };

                match b {
                    b'"' => break,
                    b'\n' => {
                        return Err(TomlError::syntax(
                            "Newline in single-line string literal",
                            start_line,
                            start_col,
                            start_pos,
                        ));
                    }
                    b'\\' => {
                        let ch = self.parse_escape_sequence(start_line, start_col, start_pos)?;
                        out.push(ch);
                    }
                    other => out.push(other as char),
                }
            }

            Ok(SpannedToken {
                token: Token::String(out),
                line: start_line,
                column: start_col,
                position: start_pos,
            })
        }
    }

    fn scan_literal_string(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<SpannedToken, TomlError> {
        // Check for multiline '''
        if self.peek_ahead(1) == Some(b'\'') && self.peek_ahead(2) == Some(b'\'') {
            self.advance();
            self.advance();
            self.advance();

            // Strip initial newline immediately following '''
            if self.peek() == Some(b'\r') && self.peek_ahead(1) == Some(b'\n') {
                self.advance();
                self.advance();
            } else if self.peek() == Some(b'\n') {
                self.advance();
            }

            let mut out = String::new();
            loop {
                if self.pos >= self.bytes.len() {
                    return Err(TomlError::syntax(
                        "Unclosed multi-line literal string",
                        start_line,
                        start_col,
                        start_pos,
                    ));
                }

                if self.peek() == Some(b'\'')
                    && self.peek_ahead(1) == Some(b'\'')
                    && self.peek_ahead(2) == Some(b'\'')
                {
                    self.advance();
                    self.advance();
                    self.advance();
                    break;
                }

                if self.peek() == Some(b'\r') && self.peek_ahead(1) == Some(b'\n') {
                    self.advance();
                    self.advance();
                    out.push('\n');
                } else {
                    let b = self.advance().unwrap();
                    out.push(b as char);
                }
            }

            Ok(SpannedToken {
                token: Token::String(out),
                line: start_line,
                column: start_col,
                position: start_pos,
            })
        } else {
            // Single-line literal string '...'
            self.advance();
            let mut out = String::new();
            loop {
                let b = match self.advance() {
                    Some(b) => b,
                    None => {
                        return Err(TomlError::syntax(
                            "Unclosed literal string",
                            start_line,
                            start_col,
                            start_pos,
                        ));
                    }
                };

                match b {
                    b'\'' => break,
                    b'\n' => {
                        return Err(TomlError::syntax(
                            "Newline in single-line literal string",
                            start_line,
                            start_col,
                            start_pos,
                        ));
                    }
                    other => out.push(other as char),
                }
            }

            Ok(SpannedToken {
                token: Token::String(out),
                line: start_line,
                column: start_col,
                position: start_pos,
            })
        }
    }

    fn parse_escape_sequence(
        &mut self,
        line: usize,
        col: usize,
        pos: usize,
    ) -> Result<char, TomlError> {
        match self.advance() {
            Some(b'"') => Ok('"'),
            Some(b'\\') => Ok('\\'),
            Some(b'b') => Ok('\x08'),
            Some(b't') => Ok('\t'),
            Some(b'n') => Ok('\n'),
            Some(b'f') => Ok('\x0C'),
            Some(b'r') => Ok('\r'),
            Some(b'e') => Ok('\x1B'),
            Some(b'x') => self.parse_hex_escape(2, line, col, pos),
            Some(b'u') => self.parse_hex_escape(4, line, col, pos),
            Some(b'U') => self.parse_hex_escape(8, line, col, pos),
            Some(other) => Err(TomlError::syntax(
                format!("Invalid escape sequence '\\{}'", other as char),
                line,
                col,
                pos,
            )),
            None => Err(TomlError::syntax("Unfinished escape sequence", line, col, pos)),
        }
    }

    fn parse_hex_escape(
        &mut self,
        digits: usize,
        line: usize,
        col: usize,
        pos: usize,
    ) -> Result<char, TomlError> {
        let mut val = 0u32;
        for _ in 0..digits {
            let b = self.advance().ok_or_else(|| {
                TomlError::syntax("Incomplete unicode escape", line, col, pos)
            })?;
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                _ => {
                    return Err(TomlError::syntax(
                        "Invalid hex character in unicode escape",
                        line,
                        col,
                        pos,
                    ));
                }
            };
            val = (val << 4) | digit;
        }

        char::from_u32(val).ok_or_else(|| {
            TomlError::syntax("Invalid unicode scalar value", line, col, pos)
        })
    }

    fn scan_bare_or_literal(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<SpannedToken, TomlError> {
        let first_byte = match self.peek() {
            Some(b) => b,
            None => {
                return Ok(SpannedToken {
                    token: Token::Eof,
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                });
            }
        };

        let mut text = String::new();

        // If it starts with + or - or a digit, it could be a number, special float, or datetime
        let is_numeric_or_sign = first_byte.is_ascii_digit() || first_byte == b'+' || first_byte == b'-';

        if is_numeric_or_sign {
            while let Some(b) = self.peek() {
                if b.is_ascii_alphanumeric()
                    || b == b'_'
                    || b == b'-'
                    || b == b'+'
                    || b == b':'
                    || b == b'.'
                {
                    text.push(b as char);
                    self.advance();
                } else if b == b' '
                    && text.len() == 10
                    && text.as_bytes()[4] == b'-'
                    && text.as_bytes()[7] == b'-'
                    && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit())
                    && self.peek_ahead(2).map_or(false, |c| c.is_ascii_digit())
                    && self.peek_ahead(3) == Some(b':')
                {
                    text.push(' ');
                    self.advance();
                } else {
                    break;
                }
            }
        } else {
            // Bare identifier (keys, booleans, inf, nan) - only [A-Za-z0-9_-]
            while let Some(b) = self.peek() {
                if b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                    text.push(b as char);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if text.is_empty() {
            let bad_char = self.advance().unwrap_or(0);
            return Err(TomlError::syntax(
                format!("Unexpected character '{}'", bad_char as char),
                start_line,
                start_col,
                start_pos,
            ));
        }

        // 1. Boolean check
        if text == "true" {
            return Ok(SpannedToken {
                token: Token::Boolean(true),
                line: start_line,
                column: start_col,
                position: start_pos,
            });
        }
        if text == "false" {
            return Ok(SpannedToken {
                token: Token::Boolean(false),
                line: start_line,
                column: start_col,
                position: start_pos,
            });
        }

        // 2. Special Floats
        if text == "inf" || text == "+inf" {
            return Ok(SpannedToken {
                token: Token::Float(f64::INFINITY),
                line: start_line,
                column: start_col,
                position: start_pos,
            });
        }
        if text == "-inf" {
            return Ok(SpannedToken {
                token: Token::Float(f64::NEG_INFINITY),
                line: start_line,
                column: start_col,
                position: start_pos,
            });
        }
        if text == "nan" || text == "+nan" || text == "-nan" {
            return Ok(SpannedToken {
                token: Token::Float(f64::NAN),
                line: start_line,
                column: start_col,
                position: start_pos,
            });
        }

        // 3. Datetime check
        if let Some(dt) = TomlDatetime::parse(&text) {
            return Ok(SpannedToken {
                token: Token::Datetime(dt),
                line: start_line,
                column: start_col,
                position: start_pos,
            });
        }

        // 4. Hex, Octal, Binary Integers
        if text.starts_with("0x") || text.starts_with("0X") {
            if let Err(e) = validate_toml_number_syntax(&text) {
                return Err(TomlError::syntax(e, start_line, start_col, start_pos));
            }
            let clean = text[2..].replace('_', "");
            if let Ok(i) = i64::from_str_radix(&clean, 16) {
                return Ok(SpannedToken {
                    token: Token::Integer(i),
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                });
            }
        }
        if text.starts_with("0o") || text.starts_with("0O") {
            if let Err(e) = validate_toml_number_syntax(&text) {
                return Err(TomlError::syntax(e, start_line, start_col, start_pos));
            }
            let clean = text[2..].replace('_', "");
            if let Ok(i) = i64::from_str_radix(&clean, 8) {
                return Ok(SpannedToken {
                    token: Token::Integer(i),
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                });
            }
        }
        if text.starts_with("0b") || text.starts_with("0B") {
            if let Err(e) = validate_toml_number_syntax(&text) {
                return Err(TomlError::syntax(e, start_line, start_col, start_pos));
            }
            let clean = text[2..].replace('_', "");
            if let Ok(i) = i64::from_str_radix(&clean, 2) {
                return Ok(SpannedToken {
                    token: Token::Integer(i),
                    line: start_line,
                    column: start_col,
                    position: start_pos,
                });
            }
        }

        // 5. Decimal integer or float
        if is_numeric_or_sign {
            match validate_toml_number_syntax(&text) {
                Ok(()) => {
                    let clean = text.replace('_', "");
                    if let Ok(i) = clean.parse::<i64>() {
                        return Ok(SpannedToken {
                            token: Token::Integer(i),
                            line: start_line,
                            column: start_col,
                            position: start_pos,
                        });
                    }
                    if (clean.contains('.') || clean.contains('e') || clean.contains('E'))
                        && !clean.contains(':')
                    {
                        if let Ok(f) = clean.parse::<f64>() {
                            return Ok(SpannedToken {
                                token: Token::Float(f),
                                line: start_line,
                                column: start_col,
                                position: start_pos,
                            });
                        }
                    }
                }
                Err(e) => {
                    let has_non_key_chars = text.chars().any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '-');
                    if has_non_key_chars {
                        return Err(TomlError::syntax(e, start_line, start_col, start_pos));
                    }
                }
            }
        }

        // 6. Bare key fallback
        Ok(SpannedToken {
            token: Token::Key(text),
            line: start_line,
            column: start_col,
            position: start_pos,
        })
    }
}

fn validate_toml_number_syntax(text: &str) -> Result<(), &'static str> {
    if text.starts_with('_') || text.ends_with('_') {
        return Err("Underscore must be surrounded by digits");
    }
    if text.contains("__") {
        return Err("Double underscore is not permitted");
    }
    if text.starts_with("0x") || text.starts_with("0X")
        || text.starts_with("0o") || text.starts_with("0O")
        || text.starts_with("0b") || text.starts_with("0B")
    {
        if text.len() <= 2 || text.as_bytes()[2] == b'_' {
            return Err("Underscore directly after base prefix is not permitted");
        }
        return Ok(());
    }

    if text.contains("._") || text.contains("_.")
        || text.contains("e_") || text.contains("_e")
        || text.contains("E_") || text.contains("_E")
    {
        return Err("Underscore cannot be adjacent to decimal point or exponent");
    }

    let clean = text.replace('_', "");
    let mut num_str = clean.as_str();
    if num_str.starts_with('+') || num_str.starts_with('-') {
        num_str = &num_str[1..];
    }
    if num_str.is_empty() {
        return Err("Missing digits");
    }

    let (int_and_frac, _has_exp) = if let Some(pos) = num_str.find(|c| c == 'e' || c == 'E') {
        let exp_part = &num_str[pos + 1..];
        let exp_digits = if exp_part.starts_with('+') || exp_part.starts_with('-') {
            &exp_part[1..]
        } else {
            exp_part
        };
        if exp_digits.is_empty() || !exp_digits.chars().all(|c| c.is_ascii_digit()) {
            return Err("Malformed exponent");
        }
        (&num_str[..pos], true)
    } else {
        (num_str, false)
    };

    let (int_part, _has_frac) = if let Some(pos) = int_and_frac.find('.') {
        let frac_part = &int_and_frac[pos + 1..];
        if frac_part.is_empty() || !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err("Fractional part must contain digits after decimal point");
        }
        (&int_and_frac[..pos], true)
    } else {
        (int_and_frac, false)
    };

    if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
        return Err("Malformed integer part");
    }

    if int_part.len() > 1 && int_part.starts_with('0') {
        return Err("Leading zero is not permitted in decimal numbers");
    }

    Ok(())
}
