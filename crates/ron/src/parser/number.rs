#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

use babbel_core::Value;
use crate::error::RonError;
use super::RonParser;

impl<'a> RonParser<'a> {
    pub(crate) fn parse_number(&mut self) -> Result<Value, RonError> {
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
                                oct_push(&mut digits, c);
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
}

#[inline]
fn oct_push(digits: &mut String, c: char) {
    digits.push(c);
}
