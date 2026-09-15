#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use crate::ast::KdlValue;
use crate::error::KdlError;
use super::{is_bare_ident_char, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_value(&mut self) -> Result<KdlValue, KdlError> {
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
}
