//! RFC 9535 JSONPath parser.

use super::ast::{ComparisonOp, FilterExpr, FilterOperand, PathQuery, QueryPath, Segment};
use crate::error::BabbelError;
use crate::model::Value;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Parser for RFC 9535 JSONPath expressions.
pub struct JsonPathParser<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
}

impl<'a> JsonPathParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            input,
            chars,
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<QueryPath, BabbelError> {
        self.skip_whitespace();

        // Optional leading root token '$'
        if self.peek_char() == Some('$') {
            self.advance();
        }

        let mut segments = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            match self.peek_char() {
                Some('.') => {
                    self.advance();
                    if self.peek_char() == Some('.') {
                        // Recursive descent: '..'
                        self.advance();
                        self.skip_whitespace();
                        let target = self.parse_descendant_target()?;
                        segments.push(Segment::Descendant(Box::new(target)));
                    } else if self.peek_char() == Some('*') {
                        self.advance();
                        segments.push(Segment::Wildcard);
                    } else if self.peek_char() == Some('[') {
                        // Dot followed by bracket notation (e.g. .[0])
                        let segs = self.parse_bracket_segments()?;
                        segments.extend(segs);
                    } else {
                        let ident = self.parse_identifier()?;
                        segments.push(Segment::Child(ident));
                    }
                }
                Some('[') => {
                    let segs = self.parse_bracket_segments()?;
                    segments.extend(segs);
                }
                Some(c) => {
                    return Err(BabbelError::syntax(format!(
                        "unexpected character '{}' in JSONPath expression at index {}",
                        c,
                        self.current_offset()
                    )));
                }
                None => break,
            }
        }

        Ok(QueryPath { segments })
    }

    fn parse_descendant_target(&mut self) -> Result<Segment, BabbelError> {
        if self.peek_char() == Some('*') {
            self.advance();
            Ok(Segment::Wildcard)
        } else if self.peek_char() == Some('[') {
            let mut segs = self.parse_bracket_segments()?;
            if segs.is_empty() {
                Err(BabbelError::syntax("empty bracket in descendant segment"))
            } else {
                Ok(segs.remove(0))
            }
        } else {
            let ident = self.parse_identifier()?;
            Ok(Segment::Child(ident))
        }
    }

    fn parse_bracket_segments(&mut self) -> Result<Vec<Segment>, BabbelError> {
        self.expect('[')?;
        self.skip_whitespace();

        if self.peek_char() == Some('?') {
            // Filter expression: [?(...)]
            self.advance();
            self.skip_whitespace();
            let filter = self.parse_filter_expr()?;
            self.skip_whitespace();
            self.expect(']')?;
            return Ok(alloc::vec![Segment::Filter(filter)]);
        }

        if self.peek_char() == Some('*') {
            self.advance();
            self.skip_whitespace();
            self.expect(']')?;
            return Ok(alloc::vec![Segment::Wildcard]);
        }

        // Check if slice: e.g. ":", "1:", ":3", "1:3", "1:3:1"
        if self.peek_char() == Some(':') || self.is_slice_lookahead() {
            let slice = self.parse_slice()?;
            self.skip_whitespace();
            self.expect(']')?;
            return Ok(alloc::vec![slice]);
        }

        // Bracket items (could be comma-separated keys or indices: ['a', 'b'])
        let mut segments = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some('\'') || self.peek_char() == Some('"') {
                let s = self.parse_quoted_string()?;
                segments.push(Segment::Child(s));
            } else if let Some(idx) = self.try_parse_integer() {
                segments.push(Segment::Index(idx));
            } else {
                let s = self.parse_unquoted_bracket_prop()?;
                segments.push(Segment::Child(s));
            }

            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(']')?;
        Ok(segments)
    }

    fn is_slice_lookahead(&self) -> bool {
        let mut i = self.pos;
        if i < self.chars.len() && (self.chars[i].1 == '-' || self.chars[i].1.is_ascii_digit()) {
            while i < self.chars.len()
                && (self.chars[i].1.is_ascii_digit() || self.chars[i].1 == '-')
            {
                i += 1;
            }
            while i < self.chars.len() && self.chars[i].1.is_whitespace() {
                i += 1;
            }
            return i < self.chars.len() && self.chars[i].1 == ':';
        }
        false
    }

    fn parse_slice(&mut self) -> Result<Segment, BabbelError> {
        let start = if self.peek_char() != Some(':') {
            Some(self.parse_integer()?)
        } else {
            None
        };

        self.expect(':')?;
        self.skip_whitespace();

        let mut end = None;
        let mut step = None;

        if self.peek_char() != Some(':') && self.peek_char() != Some(']') {
            end = Some(self.parse_integer()?);
            self.skip_whitespace();
        }

        if self.peek_char() == Some(':') {
            self.advance();
            self.skip_whitespace();
            if self.peek_char() != Some(']') {
                step = Some(self.parse_integer()?);
            }
        }

        Ok(Segment::Slice { start, end, step })
    }

    fn parse_filter_expr(&mut self) -> Result<FilterExpr, BabbelError> {
        self.skip_whitespace();
        let mut left_paren = false;
        if self.peek_char() == Some('(') {
            self.advance();
            left_paren = true;
            self.skip_whitespace();
        }

        let expr = self.parse_or_expr()?;

        if left_paren {
            self.skip_whitespace();
            self.expect(')')?;
        }

        Ok(expr)
    }

    fn parse_or_expr(&mut self) -> Result<FilterExpr, BabbelError> {
        let mut left = self.parse_and_expr()?;
        self.skip_whitespace();
        while self.peek_str("||") {
            self.advance_by(2);
            self.skip_whitespace();
            let right = self.parse_and_expr()?;
            left = FilterExpr::Or(Box::new(left), Box::new(right));
            self.skip_whitespace();
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<FilterExpr, BabbelError> {
        let mut left = self.parse_unary_expr()?;
        self.skip_whitespace();
        while self.peek_str("&&") {
            self.advance_by(2);
            self.skip_whitespace();
            let right = self.parse_unary_expr()?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
            self.skip_whitespace();
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<FilterExpr, BabbelError> {
        self.skip_whitespace();
        if self.peek_char() == Some('!') && !self.peek_str("!=") {
            self.advance();
            let expr = self.parse_unary_expr()?;
            return Ok(FilterExpr::Not(Box::new(expr)));
        }

        if self.peek_char() == Some('(') {
            self.advance();
            let inner = self.parse_or_expr()?;
            self.skip_whitespace();
            self.expect(')')?;
            return Ok(inner);
        }

        self.parse_comparison_or_exists()
    }

    fn parse_comparison_or_exists(&mut self) -> Result<FilterExpr, BabbelError> {
        let left_op = self.parse_filter_operand()?;
        self.skip_whitespace();

        if let Some(op) = self.try_parse_comparison_op() {
            self.skip_whitespace();
            let right_op = self.parse_filter_operand()?;
            Ok(FilterExpr::Comparison {
                op,
                left: left_op,
                right: right_op,
            })
        } else {
            // Path existence check
            match left_op {
                FilterOperand::RelativePath(path) => Ok(FilterExpr::Exists(path)),
                FilterOperand::RootPath(path) => Ok(FilterExpr::Exists(path)),
                _ => Err(BabbelError::syntax(
                    "expected comparison operator or path existence check in filter",
                )),
            }
        }
    }

    fn try_parse_comparison_op(&mut self) -> Option<ComparisonOp> {
        if self.peek_str("==") {
            self.advance_by(2);
            Some(ComparisonOp::Equal)
        } else if self.peek_str("!=") {
            self.advance_by(2);
            Some(ComparisonOp::NotEqual)
        } else if self.peek_str("<=") {
            self.advance_by(2);
            Some(ComparisonOp::LessOrEqual)
        } else if self.peek_str(">=") {
            self.advance_by(2);
            Some(ComparisonOp::GreaterOrEqual)
        } else if self.peek_char() == Some('<') {
            self.advance();
            Some(ComparisonOp::LessThan)
        } else if self.peek_char() == Some('>') {
            self.advance();
            Some(ComparisonOp::GreaterThan)
        } else {
            None
        }
    }

    fn parse_filter_operand(&mut self) -> Result<FilterOperand, BabbelError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('@') => {
                self.advance();
                let query = self.parse_subpath()?;
                Ok(FilterOperand::RelativePath(query))
            }
            Some('$') => {
                self.advance();
                let query = self.parse_subpath()?;
                Ok(FilterOperand::RootPath(query))
            }
            Some('\'') | Some('"') => {
                let s = self.parse_quoted_string()?;
                Ok(FilterOperand::Literal(Value::String(s)))
            }
            Some(c) if c == '-' || c.is_ascii_digit() => {
                let num = self.parse_number_literal()?;
                Ok(FilterOperand::Literal(num))
            }
            Some('t') if self.peek_str("true") => {
                self.advance_by(4);
                Ok(FilterOperand::Literal(Value::Bool(true)))
            }
            Some('f') if self.peek_str("false") => {
                self.advance_by(5);
                Ok(FilterOperand::Literal(Value::Bool(false)))
            }
            Some('n') if self.peek_str("null") => {
                self.advance_by(4);
                Ok(FilterOperand::Literal(Value::Null))
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let ident = self.parse_identifier()?;
                self.skip_whitespace();
                if self.peek_char() == Some('(') {
                    // Function call, e.g. length(...)
                    self.advance();
                    self.skip_whitespace();
                    let arg = self.parse_filter_operand()?;
                    self.skip_whitespace();
                    self.expect(')')?;
                    Ok(FilterOperand::FunctionCall {
                        name: ident,
                        arg: Box::new(arg),
                    })
                } else {
                    Ok(FilterOperand::Literal(Value::String(ident)))
                }
            }
            Some(c) => Err(BabbelError::syntax(format!(
                "unexpected token '{}' in filter operand at offset {}",
                c,
                self.current_offset()
            ))),
            None => Err(BabbelError::syntax("unexpected end of filter operand")),
        }
    }

    fn parse_subpath(&mut self) -> Result<PathQuery, BabbelError> {
        let mut segments = Vec::new();
        while !self.is_eof() {
            self.skip_whitespace();
            if self.peek_char() == Some('.') {
                self.advance();
                if self.peek_char() == Some('*') {
                    self.advance();
                    segments.push(Segment::Wildcard);
                } else {
                    let id = self.parse_identifier()?;
                    segments.push(Segment::Child(id));
                }
            } else if self.peek_char() == Some('[') {
                let segs = self.parse_bracket_segments()?;
                segments.extend(segs);
            } else {
                break;
            }
        }
        Ok(PathQuery { segments })
    }

    fn parse_identifier(&mut self) -> Result<String, BabbelError> {
        let mut s = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if s.is_empty() {
            Err(BabbelError::syntax(format!(
                "expected identifier at offset {}",
                self.current_offset()
            )))
        } else {
            Ok(s)
        }
    }

    fn parse_unquoted_bracket_prop(&mut self) -> Result<String, BabbelError> {
        let mut s = String::new();
        while let Some(c) = self.peek_char() {
            if c != ']' && c != ',' && !c.is_whitespace() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if s.is_empty() {
            Err(BabbelError::syntax("expected property in brackets"))
        } else {
            Ok(s)
        }
    }

    fn parse_quoted_string(&mut self) -> Result<String, BabbelError> {
        let quote = self.peek_char().unwrap();
        self.advance();
        let mut s = String::new();
        let mut escaped = false;

        while let Some(c) = self.peek_char() {
            self.advance();
            if escaped {
                match c {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '\\' => s.push('\\'),
                    '\'' => s.push('\''),
                    '"' => s.push('"'),
                    _ => s.push(c),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                return Ok(s);
            } else {
                s.push(c);
            }
        }
        Err(BabbelError::syntax("unclosed quote in string literal"))
    }

    fn parse_integer(&mut self) -> Result<i64, BabbelError> {
        let mut s = String::new();
        if self.peek_char() == Some('-') {
            s.push('-');
            self.advance();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s.parse::<i64>()
            .map_err(|_| BabbelError::syntax("invalid integer"))
    }

    fn try_parse_integer(&mut self) -> Option<i64> {
        let start = self.pos;
        let mut s = String::new();
        if self.peek_char() == Some('-') {
            s.push('-');
            self.advance();
        }
        let mut found = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
                found = true;
            } else {
                break;
            }
        }
        if found {
            if let Ok(i) = s.parse::<i64>() {
                return Some(i);
            }
        }
        self.pos = start;
        None
    }

    fn parse_number_literal(&mut self) -> Result<Value, BabbelError> {
        let mut s = String::new();
        let mut is_float = false;
        if self.peek_char() == Some('-') {
            s.push('-');
            self.advance();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                // Check if next char is digit to avoid eating '..'
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].1.is_ascii_digit() {
                    is_float = true;
                    s.push(c);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if is_float {
            s.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| BabbelError::syntax("invalid float literal"))
        } else {
            s.parse::<i128>()
                .map(Value::Integer)
                .map_err(|_| BabbelError::syntax("invalid integer literal"))
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    fn peek_str(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() > self.chars.len() {
            return false;
        }
        for (idx, &ch) in chars.iter().enumerate() {
            if self.chars[self.pos + idx].1 != ch {
                return false;
            }
        }
        true
    }

    fn advance(&mut self) {
        if self.pos < self.chars.len() {
            self.pos += 1;
        }
    }

    fn advance_by(&mut self, count: usize) {
        self.pos = (self.pos + count).min(self.chars.len());
    }

    fn expect(&mut self, expected: char) -> Result<(), BabbelError> {
        match self.peek_char() {
            Some(c) if c == expected => {
                self.advance();
                Ok(())
            }
            Some(c) => Err(BabbelError::syntax(format!(
                "expected '{}', found '{}' at offset {}",
                expected,
                c,
                self.current_offset()
            ))),
            None => Err(BabbelError::syntax(format!(
                "expected '{}', found EOF",
                expected
            ))),
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn current_offset(&self) -> usize {
        self.chars
            .get(self.pos)
            .map(|&(idx, _)| idx)
            .unwrap_or(self.input.len())
    }
}
