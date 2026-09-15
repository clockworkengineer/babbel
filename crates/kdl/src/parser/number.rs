#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

use crate::ast::KdlValue;
use crate::error::KdlError;
use super::{is_bare_ident_char, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_number(&mut self) -> Result<KdlValue, KdlError> {
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
