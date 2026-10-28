//! RFC 9535 JSONPath parser (zero-allocation byte-offset cursor over `&str`).

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
    pos: usize,
}

impl<'a> JsonPathParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
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

        // Bracket items (could be comma-separated keys or indices: ['a', 'b'] or [0, 2])
        let mut segments = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some('\'') || self.peek_char() == Some('"') {
                let s = self.parse_quoted_string()?;
                segments.push(Segment::Child(s));
            } else if self.peek_char() == Some(':') || self.is_slice_lookahead() {
                let slice = self.parse_slice()?;
                segments.push(slice);
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
        if segments.len() > 1 {
            Ok(alloc::vec![Segment::Union(segments)])
        } else {
            Ok(segments)
        }
    }

    fn is_slice_lookahead(&self) -> bool {
        let rest = &self.input[self.pos..];
        let mut chars = rest.chars().peekable();
        if let Some(&first) = chars.peek() {
            if first == '-' || first.is_ascii_digit() {
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '-' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                return chars.peek() == Some(&':');
            }
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

        if self.peek_char() != Some(':') && self.peek_char() != Some(']') && self.peek_char() != Some(',') {
            end = Some(self.parse_integer()?);
            self.skip_whitespace();
        }

        if self.peek_char() == Some(':') {
            self.advance();
            self.skip_whitespace();
            if self.peek_char() != Some(']') && self.peek_char() != Some(',') {
                step = Some(self.parse_integer()?);
            }
        }

        Ok(Segment::Slice { start, end, step })
    }

    fn parse_filter_expr(&mut self) -> Result<FilterExpr, BabbelError> {
        self.skip_whitespace();
        let mut left_paren = false;
        if self.peek_char() == Some('(') {
            let saved = self.pos;
            self.advance();
            self.skip_whitespace();
            if let Ok(expr) = self.parse_or_expr() {
                self.skip_whitespace();
                if self.peek_char() == Some(')') {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek_char() == Some(']') {
                        return Ok(expr);
                    }
                }
            }
            self.pos = saved;
            left_paren = false;
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
            self.advance_bytes(2);
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
            self.advance_bytes(2);
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
            // Path existence check or boolean function call (e.g. match/search)
            match left_op {
                FilterOperand::RelativePath(path) => Ok(FilterExpr::Exists(path)),
                FilterOperand::RootPath(path) => Ok(FilterExpr::Exists(path)),
                FilterOperand::FunctionCall { .. } => Ok(FilterExpr::Comparison {
                    op: ComparisonOp::Equal,
                    left: left_op,
                    right: FilterOperand::Literal(Value::Bool(true)),
                }),
                _ => Err(BabbelError::syntax(
                    "expected comparison operator or path existence check in filter",
                )),
            }
        }
    }

    fn try_parse_comparison_op(&mut self) -> Option<ComparisonOp> {
        if self.peek_str("==") {
            self.advance_bytes(2);
            Some(ComparisonOp::Equal)
        } else if self.peek_str("!=") {
            self.advance_bytes(2);
            Some(ComparisonOp::NotEqual)
        } else if self.peek_str("<=") {
            self.advance_bytes(2);
            Some(ComparisonOp::LessOrEqual)
        } else if self.peek_str(">=") {
            self.advance_bytes(2);
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
            Some('t') if self.peek_keyword("true") => {
                self.advance_bytes(4);
                Ok(FilterOperand::Literal(Value::Bool(true)))
            }
            Some('f') if self.peek_keyword("false") => {
                self.advance_bytes(5);
                Ok(FilterOperand::Literal(Value::Bool(false)))
            }
            Some('n') if self.peek_keyword("null") => {
                self.advance_bytes(4);
                Ok(FilterOperand::Literal(Value::Null))
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                let ident = self.parse_identifier()?;
                self.skip_whitespace();
                if self.peek_char() == Some('(') {
                    // Function call, e.g. length(@.x) or match(@.x, "^a.*")
                    self.advance();
                    self.skip_whitespace();
                    let mut args = Vec::new();
                    if self.peek_char() != Some(')') {
                        loop {
                            self.skip_whitespace();
                            let arg = self.parse_filter_operand()?;
                            args.push(arg);
                            self.skip_whitespace();
                            if self.peek_char() == Some(',') {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.skip_whitespace();
                    self.expect(')')?;
                    Ok(FilterOperand::FunctionCall { name: ident, args })
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
                if self.peek_char() == Some('.') {
                    self.advance();
                    let target = self.parse_descendant_target()?;
                    segments.push(Segment::Descendant(Box::new(target)));
                } else if self.peek_char() == Some('*') {
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
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || (!c.is_ascii() && !c.is_whitespace()) {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            Err(BabbelError::syntax(format!(
                "expected identifier at offset {}",
                self.current_offset()
            )))
        } else {
            Ok(String::from(&self.input[start..self.pos]))
        }
    }

    fn parse_unquoted_bracket_prop(&mut self) -> Result<String, BabbelError> {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c != ']' && c != ',' && !c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            Err(BabbelError::syntax("expected property in brackets"))
        } else {
            Ok(String::from(&self.input[start..self.pos]))
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
                    'b' => s.push('\u{0008}'),
                    'f' => s.push('\u{000C}'),
                    '\\' => s.push('\\'),
                    '/' => s.push('/'),
                    '\'' => s.push('\''),
                    '"' => s.push('"'),
                    'u' => {
                        let hex_start = self.pos;
                        for _ in 0..4 {
                            if self.peek_char().is_some_and(|ch| ch.is_ascii_hexdigit()) {
                                self.advance();
                            } else {
                                return Err(BabbelError::syntax("invalid \\uXXXX escape in JSONPath"));
                            }
                        }
                        let code = u32::from_str_radix(&self.input[hex_start..self.pos], 16)
                            .map_err(|_| BabbelError::syntax("invalid \\uXXXX hex"))?;
                        if let Some(ch) = char::from_u32(code) {
                            s.push(ch);
                        }
                    }
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
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos]
            .parse::<i64>()
            .map_err(|_| BabbelError::syntax("invalid integer"))
    }

    fn try_parse_integer(&mut self) -> Option<i64> {
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.advance();
        }
        let digit_start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos > digit_start {
            if let Ok(i) = self.input[start..self.pos].parse::<i64>() {
                return Some(i);
            }
        }
        self.pos = start;
        None
    }

    fn parse_number_literal(&mut self) -> Result<Value, BabbelError> {
        let start = self.pos;
        let mut is_float = false;
        if self.peek_char() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.advance();
            } else if c == '.' && !is_float {
                let rest = &self.input[self.pos + 1..];
                if rest.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
                    is_float = true;
                    self.advance();
                } else {
                    break;
                }
            } else if (c == 'e' || c == 'E') && !self.input[start..self.pos].is_empty() {
                is_float = true;
                self.advance();
                if matches!(self.peek_char(), Some('+') | Some('-')) {
                    self.advance();
                }
            } else {
                break;
            }
        }
        let slice = &self.input[start..self.pos];
        if is_float {
            slice
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| BabbelError::syntax("invalid float literal"))
        } else {
            slice
                .parse::<i128>()
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

    #[inline]
    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    #[inline]
    fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    #[inline]
    fn peek_keyword(&self, kw: &str) -> bool {
        let rest = &self.input[self.pos..];
        if !rest.starts_with(kw) {
            return false;
        }
        !rest[kw.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    #[inline]
    fn advance(&mut self) {
        if let Some(c) = self.peek_char() {
            self.pos += c.len_utf8();
        }
    }

    #[inline]
    fn advance_bytes(&mut self, count: usize) {
        self.pos = (self.pos + count).min(self.input.len());
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

    #[inline]
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    #[inline]
    fn current_offset(&self) -> usize {
        self.pos
    }
}
