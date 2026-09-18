//! RON composite container parsing (structs, tuples, lists, maps, elided maps).

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use super::scalar::decode_escapes;
use super::{RonParser, is_ident_part, is_ident_start};
use crate::error::RonError;
use babbel_core::Value;

impl<'a> RonParser<'a> {
    /// Handles both `()` (unit/null), `(field: val, ...)` (struct), and `(val, val)` (tuple).
    pub(crate) fn parse_paren_container(&mut self) -> Result<Value, RonError> {
        if self.depth >= self.max_depth {
            return Err(RonError::RecursionLimitExceeded {
                depth: self.depth,
                max: self.max_depth,
            });
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

    pub(crate) fn peek_is_struct_field(&mut self) -> Result<bool, RonError> {
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

    pub(crate) fn parse_list(&mut self) -> Result<Value, RonError> {
        if self.depth >= self.max_depth {
            return Err(RonError::RecursionLimitExceeded {
                depth: self.depth,
                max: self.max_depth,
            });
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

    pub(crate) fn parse_map(&mut self) -> Result<Value, RonError> {
        if self.depth >= self.max_depth {
            return Err(RonError::RecursionLimitExceeded {
                depth: self.depth,
                max: self.max_depth,
            });
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

    pub(crate) fn parse_elided_map(&mut self) -> Result<Value, RonError> {
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

    pub(crate) fn parse_key_string(&mut self) -> Result<String, RonError> {
        self.skip_whitespace_and_comments()?;
        let (line, col) = self.current_line_col();

        match self.peek() {
            Some('"') | Some('\'') => self.parse_quoted_string(),
            Some(',') => self.parse_comma_prefixed_token(),
            Some('{') | Some('}') | Some('[') | Some(']') => Err(RonError::UnexpectedChar {
                ch: self.peek().unwrap(),
                line,
                col,
            }),
            Some(_) => {
                let atom = self.scan_bare_atom()?;
                decode_escapes(&atom, line, col)
            }
            None => Err(RonError::UnexpectedEof),
        }
    }
}
