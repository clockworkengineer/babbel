//! Recursive descent parser for HashiCorp HCL v2 syntax into universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use crate::error::HclError;
use babbel_core::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Directive {
    If,
    IfWithElse,
    For,
}

/// Parser state for HCL text input.
pub struct HclParser<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    idx: usize,
    line: usize,
    col: usize,
    paren_depth: usize,
    has_unterminated_comment: bool,
    heredoc_saw_newline: bool,
    last_was_heredoc: bool,
}

impl<'a> HclParser<'a> {
    fn is_hcl_whitespace(c: char) -> bool {
        matches!(c, ' ' | '\t')
    }

    fn is_hcl_id_start(c: char) -> bool {
        c != '\u{2E2F}'
            && (c.is_alphabetic()
                || c == '_'
                || matches!(
                    c,
                    '\u{2160}'..='\u{2188}'
                    | '\u{3007}' | '\u{3021}'..='\u{3029}' | '\u{3038}'..='\u{303A}'
                    | '\u{2118}' | '\u{212E}'
                    | '\u{309B}' | '\u{309C}'
                    | '\u{1885}' | '\u{1886}'
                ))
    }

    fn is_hcl_id_continue(c: char) -> bool {
        Self::is_hcl_id_start(c)
            || c.is_ascii_digit()
            || ('\u{0660}'..='\u{0669}').contains(&c)
            || ('\u{06F0}'..='\u{06F9}').contains(&c)
            || (c.is_numeric()
                && !matches!(
                    c,
                    '\u{00B2}' | '\u{00B3}' | '\u{00B9}'
                        | '\u{00BC}'..='\u{00BE}'
                        | '\u{2070}'..='\u{209F}'
                        | '\u{2150}'..='\u{218F}'
                        | '\u{2460}'..='\u{24FF}'
                        | '\u{2776}'..='\u{2793}'
                        | '\u{3220}'..='\u{325F}'
                        | '\u{3280}'..='\u{32BF}'
                ))
            || c == '-'
            || matches!(
                c,
                '\u{00B7}' | '\u{0387}'
                | '\u{0300}'..='\u{036F}'
                | '\u{0483}'..='\u{0489}'
                | '\u{0591}'..='\u{05BD}' | '\u{05BF}' | '\u{05C1}'..='\u{05C2}' | '\u{05C4}'..='\u{05C5}' | '\u{05C7}'
                | '\u{0610}'..='\u{061A}' | '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{06D6}'..='\u{06DC}'
                | '\u{06DF}'..='\u{06E4}' | '\u{06E7}'..='\u{06E8}' | '\u{06EA}'..='\u{06ED}'
                | '\u{0900}'..='\u{0D7F}'
                | '\u{203F}'..='\u{2040}' | '\u{2054}'
                | '\u{FE33}'..='\u{FE34}' | '\u{FE4D}'..='\u{FE4F}' | '\u{FF3F}'
            )
    }

    fn skip_expr_whitespace(&mut self) {
        if self.paren_depth > 0 {
            self.skip_whitespace_and_newlines();
        } else {
            self.skip_whitespace_and_comments();
        }
    }

    /// Create a new HCL parser.
    pub fn new(input: &'a str) -> Self {
        let input = input.strip_prefix('\u{FEFF}').unwrap_or(input);
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            input,
            chars,
            idx: 0,
            line: 1,
            col: 1,
            paren_depth: 0,
            has_unterminated_comment: false,
            heredoc_saw_newline: false,
            last_was_heredoc: false,
        }
    }

    /// Parse the entire HCL input into a top-level `Value::Object`.
    pub fn parse(&mut self) -> Result<Value, HclError> {
        self.skip_whitespace_and_newlines();
        let mut entries = Vec::new();
        let mut defined_attrs: Vec<String> = Vec::new();

        while !self.is_eof() {
            if self.has_unterminated_comment {
                return Err(self.error("unterminated block comment".into()));
            }

            let (name, is_block, val, _saw_nl) = self.parse_attribute_or_block_item(false)?;
            if !is_block {
                if defined_attrs.contains(&name) {
                    return Err(self.error(format!("Attribute '{}' redefined", name)));
                }
                defined_attrs.push(name.clone());
            }

            Self::merge_entry(&mut entries, name, val);
            self.skip_whitespace_and_newlines();
        }

        if self.has_unterminated_comment {
            return Err(self.error("unterminated block comment".into()));
        }

        Ok(Value::Object(entries))
    }

    fn merge_entry(entries: &mut Vec<(String, Value)>, key: String, val: Value) {
        if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == &key) {
            match (existing, val) {
                (Value::Object(existing_obj), Value::Object(new_obj)) => {
                    for (nk, nv) in new_obj {
                        Self::merge_entry(existing_obj, nk, nv);
                    }
                }
                (Value::Array(arr), new_val) => {
                    arr.push(new_val);
                }
                (existing_val, new_val) => {
                    let prev = core::mem::replace(existing_val, Value::Null);
                    *existing_val = Value::Array(alloc::vec![prev, new_val]);
                }
            }
        } else {
            entries.push((key, val));
        }
    }

    fn parse_attribute_or_block_item(
        &mut self,
        is_one_line: bool,
    ) -> Result<(String, bool, Value, bool), HclError> {
        self.skip_whitespace_and_comments();
        if self.has_unterminated_comment {
            return Err(self.error("unterminated block comment".into()));
        }

        let name = self.parse_identifier()?;
        self.skip_whitespace_and_comments();

        // Check if attribute assignment: `name = value`
        if self.peek_char() == Some('=') {
            self.bump(); // consume '='
            self.skip_whitespace_and_comments();
            self.last_was_heredoc = false;
            self.heredoc_saw_newline = false;
            let val = self.parse_expr()?;
            if is_one_line {
                self.skip_horizontal_whitespace_and_comments();
                if self.peek_char() == Some('\n')
                    || (self.peek_char() == Some('\r') && self.peek_ahead(1) == Some('\n'))
                    || self.peek_char() == Some('#')
                    || (self.peek_char() == Some('/') && self.peek_ahead(1) == Some('/'))
                {
                    return Err(self.error(
                        "one-line block must close on the same line without newlines".into(),
                    ));
                }
                if self.peek_char() != Some('}') {
                    return Err(self
                        .error("one-line block can only contain one attribute before '}'".into()));
                }
                return Ok((name, false, val, false));
            }
            let saw_nl = self.consume_attribute_separator()?;
            return Ok((name, false, val, saw_nl));
        }

        // Otherwise, it's a block: `name [labels...] { ... }`
        let mut labels = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            match self.peek_char() {
                Some('{') => {
                    self.bump(); // consume '{'
                    let block_is_one_line = !self.check_block_opening_newline()?;
                    let body = self.parse_block_body(block_is_one_line)?;
                    let wrapped = Self::nest_labels(labels, body);
                    let saw_nl = self.skip_whitespace_and_newlines();
                    if !saw_nl && !self.is_eof() && self.peek_char() != Some('}') {
                        return Err(self.error("block must be followed by newline".into()));
                    }
                    return Ok((name, true, wrapped, saw_nl));
                }
                Some('"') => {
                    let label = self.parse_quoted_string_literal()?;
                    labels.push(label);
                }
                Some(c) if Self::is_hcl_id_start(c) => {
                    let label = self.parse_identifier()?;
                    labels.push(label);
                }
                Some(c) => {
                    return Err(self.error(format!("unexpected character '{}' in block header", c)));
                }
                None => return Err(HclError::UnexpectedEof),
            }
        }
    }

    fn check_block_opening_newline(&mut self) -> Result<bool, HclError> {
        self.skip_horizontal_whitespace_and_comments();
        match self.peek_char() {
            Some('\n') => {
                self.bump();
                Ok(true)
            }
            Some('\r') if self.peek_ahead(1) == Some('\n') => {
                self.bump();
                self.bump();
                Ok(true)
            }
            Some('#') => {
                self.bump();
                while let Some(c) = self.bump() {
                    if c == '\n' {
                        return Ok(true);
                    }
                }
                Ok(true)
            }
            Some('/') if self.peek_ahead(1) == Some('/') => {
                self.bump();
                self.bump();
                while let Some(c) = self.bump() {
                    if c == '\n' {
                        return Ok(true);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn consume_attribute_separator(&mut self) -> Result<bool, HclError> {
        if self.heredoc_saw_newline {
            return Ok(true);
        }
        let saw_newline = self.skip_whitespace_and_newlines();
        if self.is_eof() || self.peek_char() == Some('}') {
            return Ok(saw_newline);
        }
        if saw_newline {
            return Ok(true);
        }
        Err(self.error("expected newline separating attributes".into()))
    }

    fn nest_labels(labels: Vec<String>, body: Value) -> Value {
        let mut current = body;
        for label in labels.into_iter().rev() {
            current = Value::Object(alloc::vec![(label, current)]);
        }
        current
    }

    fn parse_block_body(&mut self, is_one_line: bool) -> Result<Value, HclError> {
        let mut entries = Vec::new();
        let mut defined_attrs: Vec<String> = Vec::new();
        let mut attr_count = 0;
        let mut last_ended_with_newline = true;

        loop {
            if is_one_line {
                self.skip_horizontal_whitespace_and_comments();
                if self.peek_char() == Some('\n')
                    || (self.peek_char() == Some('\r') && self.peek_ahead(1) == Some('\n'))
                    || self.peek_char() == Some('#')
                    || (self.peek_char() == Some('/') && self.peek_ahead(1) == Some('/'))
                {
                    return Err(self.error(
                        "one-line block must close on the same line without newlines".into(),
                    ));
                }
            } else {
                let saw_nl = self.skip_whitespace_and_newlines();
                if saw_nl {
                    last_ended_with_newline = true;
                }
            }

            if self.is_eof() {
                return Err(HclError::UnexpectedEof);
            }

            if self.peek_char() == Some('}') {
                if !is_one_line && !last_ended_with_newline {
                    return Err(self.error(
                        "closing brace of multi-line block must be on its own line".into(),
                    ));
                }
                self.bump(); // consume '}'
                return Ok(Value::Object(entries));
            }

            let (name, is_block, val, saw_nl) = self.parse_attribute_or_block_item(is_one_line)?;
            if is_one_line {
                if is_block {
                    return Err(self.error("one-line block cannot contain a block".into()));
                }
                attr_count += 1;
                if attr_count > 1 {
                    return Err(self.error("one-line block can only contain one attribute".into()));
                }
                if self.last_was_heredoc {
                    return Err(self.error("one-line block cannot contain heredoc value".into()));
                }
            }

            if !is_block {
                if defined_attrs.contains(&name) {
                    return Err(self.error(format!("Attribute '{}' redefined", name)));
                }
                defined_attrs.push(name.clone());
            }

            last_ended_with_newline = saw_nl;

            Self::merge_entry(&mut entries, name, val);
        }
    }

    fn value_to_expr_string(v: &Value) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Bool(b) => alloc::format!("{}", b),
            Value::Integer(i) => alloc::format!("{}", i),
            Value::Float(f) => alloc::format!("{}", f),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let inner = arr
                    .iter()
                    .map(Self::value_to_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                alloc::format!("[{}]", inner)
            }
            Value::Object(obj) => {
                let inner = obj
                    .iter()
                    .map(|(k, val)| alloc::format!("{} = {}", k, Self::value_to_expr_string(val)))
                    .collect::<Vec<_>>()
                    .join(", ");
                alloc::format!("{{{}}}", inner)
            }
            Value::Bytes(b) => alloc::format!("{:?}", b),
        }
    }

    fn parse_expr(&mut self) -> Result<Value, HclError> {
        self.parse_ternary_expr()
    }

    fn parse_ternary_expr(&mut self) -> Result<Value, HclError> {
        let cond = self.parse_binary_expr(0)?;
        if self.heredoc_saw_newline && self.paren_depth == 0 {
            return Ok(cond);
        }
        self.skip_expr_whitespace();
        if self.peek_char() == Some('?') {
            self.bump(); // consume '?'
            self.skip_expr_whitespace();
            let true_val = self.parse_expr()?;
            if self.heredoc_saw_newline && self.paren_depth == 0 {
                return Err(self.error("Missing false expression in conditional: expression cannot continue after heredoc closing marker".into()));
            }
            self.skip_expr_whitespace();
            if self.peek_char() == Some(':') {
                self.bump(); // consume ':'
            } else {
                return Err(self.error("expected ':' in ternary expression".into()));
            }
            self.skip_expr_whitespace();
            let false_val = self.parse_expr()?;
            match cond {
                Value::Bool(true) => Ok(true_val),
                Value::Bool(false) => Ok(false_val),
                _ => Ok(Value::String(alloc::format!(
                    "{} ? {} : {}",
                    Self::value_to_expr_string(&cond),
                    Self::value_to_expr_string(&true_val),
                    Self::value_to_expr_string(&false_val)
                ))),
            }
        } else {
            Ok(cond)
        }
    }

    fn get_op_precedence(op: &str) -> Option<u8> {
        match op {
            "||" => Some(1),
            "&&" => Some(2),
            "==" | "!=" => Some(3),
            "<" | "<=" | ">" | ">=" => Some(4),
            "+" | "-" => Some(5),
            "*" | "/" | "%" => Some(6),
            _ => None,
        }
    }

    fn peek_binary_op(&self) -> Option<(&'static str, usize)> {
        if self.idx >= self.chars.len() {
            return None;
        }
        match self.peek_char() {
            Some('|') if self.peek_ahead(1) == Some('|') => Some(("||", 2)),
            Some('&') if self.peek_ahead(1) == Some('&') => Some(("&&", 2)),
            Some('=') if self.peek_ahead(1) == Some('=') => Some(("==", 2)),
            Some('!') if self.peek_ahead(1) == Some('=') => Some(("!=", 2)),
            Some('<') if self.peek_ahead(1) == Some('=') => Some(("<=", 2)),
            Some('>') if self.peek_ahead(1) == Some('=') => Some((">=", 2)),
            Some('<') => Some(("<", 1)),
            Some('>') => Some((">", 1)),
            Some('+') => Some(("+", 1)),
            Some('-') => Some(("-", 1)),
            Some('*') => Some(("*", 1)),
            Some('/') if self.peek_ahead(1) != Some('/') && self.peek_ahead(1) != Some('*') => {
                Some(("/", 1))
            }
            Some('%') => Some(("%", 1)),
            _ => None,
        }
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Value, HclError> {
        let mut lhs = self.parse_unary_or_postfix()?;

        loop {
            self.skip_expr_whitespace();
            let (op, op_len) = match self.peek_binary_op() {
                Some((op, len)) => {
                    let prec = match Self::get_op_precedence(op) {
                        Some(p) if p >= min_prec => p,
                        _ => break,
                    };
                    (op, (len, prec))
                }
                None => break,
            };

            self.advance_by(op_len.0);
            self.skip_expr_whitespace();
            let rhs = self.parse_binary_expr(op_len.1 + 1)?;

            lhs = Self::eval_binary_op(op, lhs, rhs);
        }

        Ok(lhs)
    }

    fn eval_binary_op(op: &str, lhs: Value, rhs: Value) -> Value {
        match (op, &lhs, &rhs) {
            ("+", Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_add(*b)),
            ("+", Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            ("+", Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 + b),
            ("+", Value::Float(a), Value::Integer(b)) => Value::Float(a + *b as f64),
            ("-", Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_sub(*b)),
            ("-", Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            ("-", Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 - b),
            ("-", Value::Float(a), Value::Integer(b)) => Value::Float(a - *b as f64),
            ("*", Value::Integer(a), Value::Integer(b)) => Value::Integer(a.wrapping_mul(*b)),
            ("*", Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            ("*", Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 * b),
            ("*", Value::Float(a), Value::Integer(b)) => Value::Float(a * *b as f64),
            ("/", Value::Integer(a), Value::Integer(b)) if *b != 0 => Value::Integer(a / b),
            ("/", Value::Float(a), Value::Float(b)) if *b != 0.0 => Value::Float(a / b),
            ("/", Value::Integer(a), Value::Float(b)) if *b != 0.0 => Value::Float(*a as f64 / b),
            ("/", Value::Float(a), Value::Integer(b)) if *b != 0 => Value::Float(a / *b as f64),
            ("%", Value::Integer(a), Value::Integer(b)) if *b != 0 => Value::Integer(a % b),
            ("==", a, b) => Value::Bool(a == b),
            ("!=", a, b) => Value::Bool(a != b),
            ("<", Value::Integer(a), Value::Integer(b)) => Value::Bool(a < b),
            ("<", Value::Float(a), Value::Float(b)) => Value::Bool(a < b),
            ("<=", Value::Integer(a), Value::Integer(b)) => Value::Bool(a <= b),
            ("<=", Value::Float(a), Value::Float(b)) => Value::Bool(a <= b),
            (">", Value::Integer(a), Value::Integer(b)) => Value::Bool(a > b),
            (">", Value::Float(a), Value::Float(b)) => Value::Bool(a > b),
            (">=", Value::Integer(a), Value::Integer(b)) => Value::Bool(a >= b),
            (">=", Value::Float(a), Value::Float(b)) => Value::Bool(a >= b),
            ("&&", Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && *b),
            ("||", Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || *b),
            _ => Value::String(alloc::format!(
                "{} {} {}",
                Self::value_to_expr_string(&lhs),
                op,
                Self::value_to_expr_string(&rhs)
            )),
        }
    }

    fn parse_unary_or_postfix(&mut self) -> Result<Value, HclError> {
        self.skip_expr_whitespace();
        if self.peek_char() == Some('!') {
            self.bump();
            let operand = self.parse_unary_or_postfix()?;
            return match operand {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Ok(Value::String(alloc::format!(
                    "!{}",
                    Self::value_to_expr_string(&operand)
                ))),
            };
        }
        if self.peek_char() == Some('-') && self.peek_ahead(1).map_or(true, |c| !c.is_ascii_digit())
        {
            self.bump();
            let operand = self.parse_unary_or_postfix()?;
            return match operand {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Ok(Value::String(alloc::format!(
                    "-{}",
                    Self::value_to_expr_string(&operand)
                ))),
            };
        }

        let mut base = self.parse_primary()?;
        let mut saw_legacy_index = false;

        loop {
            self.skip_expr_whitespace();
            match self.peek_char() {
                Some('.') => {
                    if self.peek_ahead(1) == Some('.') && self.peek_ahead(2) == Some('.') {
                        break;
                    }
                    if matches!(base, Value::Float(_)) {
                        break;
                    }
                    if matches!(base, Value::Integer(_)) {
                        let is_ident_next = self.peek_ahead(1).map_or(false, Self::is_hcl_id_start);
                        if !is_ident_next {
                            break;
                        }
                    }
                    self.bump(); // consume '.'
                    self.skip_expr_whitespace();
                    if self.peek_char() == Some('*') {
                        self.bump();
                        base = Value::String(alloc::format!(
                            "{}.*",
                            Self::value_to_expr_string(&base)
                        ));
                        saw_legacy_index = false;
                    } else if self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
                        if saw_legacy_index {
                            return Err(self.error("Legacy index syntax cannot be chained, because 0.0 reads as a number".into()));
                        }
                        let mut num = String::new();
                        while let Some(c) = self.peek_char() {
                            if c.is_ascii_digit() {
                                num.push(c);
                                self.bump();
                            } else {
                                break;
                            }
                        }
                        base = Value::String(alloc::format!(
                            "{}.{}",
                            Self::value_to_expr_string(&base),
                            num
                        ));
                        saw_legacy_index = true;
                    } else {
                        let member = self.parse_identifier()?;
                        base = Value::String(alloc::format!(
                            "{}.{}",
                            Self::value_to_expr_string(&base),
                            member
                        ));
                        saw_legacy_index = false;
                    }
                }
                Some('[') => {
                    self.bump(); // consume '['
                    self.paren_depth += 1;
                    let allow_newlines = self.paren_depth > 1;

                    let mut is_splat = false;
                    let mut advance_to = 0;
                    let mut k = 0;
                    while let Some(c) = self.peek_ahead(k) {
                        if c == ' ' || c == '\t' || (allow_newlines && (c == '\n' || c == '\r')) {
                            k += 1;
                        } else if c == '/' && self.peek_ahead(k + 1) == Some('*') {
                            k += 2;
                            while let Some(ic) = self.peek_ahead(k) {
                                if ic == '*' && self.peek_ahead(k + 1) == Some('/') {
                                    k += 2;
                                    break;
                                }
                                k += 1;
                            }
                        } else {
                            break;
                        }
                    }

                    if self.peek_ahead(k) == Some('*') {
                        let mut k2 = k + 1;
                        while let Some(c) = self.peek_ahead(k2) {
                            if c == ' ' || c == '\t' || (allow_newlines && (c == '\n' || c == '\r'))
                            {
                                k2 += 1;
                            } else if c == '/' && self.peek_ahead(k2 + 1) == Some('*') {
                                k2 += 2;
                                while let Some(ic) = self.peek_ahead(k2) {
                                    if ic == '*' && self.peek_ahead(k2 + 1) == Some('/') {
                                        k2 += 2;
                                        break;
                                    }
                                    k2 += 1;
                                }
                            } else {
                                break;
                            }
                        }
                        if self.peek_ahead(k2) == Some(']') {
                            is_splat = true;
                            advance_to = k2 + 1;
                        }
                    }

                    if is_splat {
                        self.advance_by(advance_to);
                        self.paren_depth -= 1;
                        base = Value::String(alloc::format!(
                            "{}[*]",
                            Self::value_to_expr_string(&base)
                        ));
                        saw_legacy_index = false;
                        continue;
                    }

                    self.skip_whitespace_and_newlines();
                    let idx = self.parse_expr()?;
                    self.skip_whitespace_and_newlines();
                    if self.peek_char() == Some(']') {
                        self.bump();
                        self.paren_depth -= 1;
                    } else {
                        self.paren_depth -= 1;
                        return Err(self.error("expected ']' after index".into()));
                    }
                    base = Value::String(alloc::format!(
                        "{}[{}]",
                        Self::value_to_expr_string(&base),
                        Self::value_to_expr_string(&idx)
                    ));
                    saw_legacy_index = false;
                }
                _ => break,
            }
        }

        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Value, HclError> {
        self.skip_expr_whitespace();
        if self.has_unterminated_comment {
            return Err(self.error("unterminated block comment".into()));
        }

        match self.peek_char() {
            Some('(') => {
                self.bump(); // consume '('
                self.paren_depth += 1;
                self.skip_whitespace_and_newlines();
                let val = self.parse_expr()?;
                self.skip_whitespace_and_newlines();
                if self.peek_char() == Some(')') {
                    self.bump(); // consume ')'
                    self.paren_depth -= 1;
                    Ok(val)
                } else {
                    self.paren_depth -= 1;
                    Err(self.error("expected ')' closing expression".into()))
                }
            }
            Some('"') => self.parse_quoted_string().map(Value::String),
            Some('<') if self.peek_ahead(1) == Some('<') => self.parse_heredoc().map(Value::String),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some('-') | Some('0'..='9') => self.parse_number(),
            Some(c) if Self::is_hcl_id_start(c) => {
                let mut id = self.parse_identifier()?;
                let mut is_namespaced = false;
                loop {
                    let mark = self.idx;
                    self.skip_whitespace_and_newlines();
                    if self.peek_char() == Some(':') && self.peek_ahead(1) == Some(':') {
                        self.bump();
                        self.bump();
                        self.skip_horizontal_whitespace_and_comments();
                        let next_id = self.parse_identifier()?;
                        id.push_str("::");
                        id.push_str(&next_id);
                        is_namespaced = true;
                    } else {
                        self.idx = mark;
                        break;
                    }
                }
                self.skip_expr_whitespace();
                if self.peek_char() == Some('(') {
                    self.bump(); // consume '('
                    self.paren_depth += 1;
                    let mut args = Vec::new();
                    self.skip_whitespace_and_newlines();
                    let mut first = true;
                    while !self.is_eof() && self.peek_char() != Some(')') {
                        if !first {
                            if self.peek_char() == Some(',') {
                                self.bump();
                                self.skip_whitespace_and_newlines();
                            } else {
                                self.paren_depth -= 1;
                                return Err(
                                    self.error("expected ',' separating function arguments".into())
                                );
                            }
                        }
                        first = false;
                        self.skip_whitespace_and_newlines();
                        if self.peek_char() == Some(')') {
                            break;
                        }
                        let arg = self.parse_expr()?;
                        self.skip_whitespace_and_newlines();
                        let is_expand = if self.peek_char() == Some('.')
                            && self.peek_ahead(1) == Some('.')
                            && self.peek_ahead(2) == Some('.')
                        {
                            self.bump();
                            self.bump();
                            self.bump();
                            true
                        } else {
                            false
                        };
                        args.push(if is_expand {
                            Value::String(alloc::format!("{}...", Self::value_to_expr_string(&arg)))
                        } else {
                            arg
                        });
                        self.skip_whitespace_and_newlines();
                        if is_expand {
                            if self.peek_char() != Some(')') {
                                self.paren_depth -= 1;
                                return Err(self.error("expansion '...' can only appear on the last argument and cannot be followed by a comma".into()));
                            }
                            break;
                        }
                    }
                    if self.peek_char() == Some(')') {
                        self.bump();
                        self.paren_depth -= 1;
                    } else {
                        self.paren_depth -= 1;
                        return Err(self.error("expected ')' closing function call".into()));
                    }
                    let args_str = args
                        .iter()
                        .map(Self::value_to_expr_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    Ok(Value::String(alloc::format!("{}({})", id, args_str)))
                } else if is_namespaced {
                    Err(self.error("namespaced function name must be followed by '('".into()))
                } else if id == "true" {
                    Ok(Value::Bool(true))
                } else if id == "false" {
                    Ok(Value::Bool(false))
                } else if id == "null" {
                    Ok(Value::Null)
                } else {
                    Ok(Value::String(id))
                }
            }
            Some(c) => Err(self.error(format!("unexpected character '{}' starting expression", c))),
            None => Err(HclError::UnexpectedEof),
        }
    }

    fn parse_for_tuple(&mut self) -> Result<Value, HclError> {
        self.advance_by(3); // consume "for"
        self.skip_whitespace_and_newlines();
        let var1 = self.parse_identifier()?;
        self.skip_whitespace_and_newlines();
        let mut var2 = None;
        if self.peek_char() == Some(',') {
            self.bump();
            self.skip_whitespace_and_newlines();
            var2 = Some(self.parse_identifier()?);
            self.skip_whitespace_and_newlines();
        }
        if !self.check_keyword("in") {
            return Err(self.error("expected 'in' after for variables".into()));
        }
        self.advance_by(2); // consume "in"
        self.skip_whitespace_and_newlines();
        let coll = self.parse_expr()?;
        self.skip_whitespace_and_newlines();
        if self.peek_char() != Some(':') {
            return Err(self.error("expected ':' in for expression".into()));
        }
        self.bump(); // consume ':'
        self.skip_whitespace_and_newlines();
        let elem = self.parse_expr()?;
        self.skip_whitespace_and_newlines();
        let mut if_cond = None;
        if self.check_keyword("if") {
            self.advance_by(2);
            self.skip_whitespace_and_newlines();
            if_cond = Some(self.parse_expr()?);
            self.skip_whitespace_and_newlines();
        }
        if self.peek_char() != Some(']') {
            return Err(self.error("expected ']' closing for expression".into()));
        }
        self.bump(); // consume ']'

        let vars = if let Some(v2) = var2 {
            alloc::format!("{}, {}", var1, v2)
        } else {
            var1
        };
        let cond_str = if let Some(c) = if_cond {
            alloc::format!(" if {}", Self::value_to_expr_string(&c))
        } else {
            "".to_string()
        };
        Ok(Value::String(alloc::format!(
            "[for {} in {}: {}{}]",
            vars,
            Self::value_to_expr_string(&coll),
            Self::value_to_expr_string(&elem),
            cond_str
        )))
    }

    fn parse_for_object(&mut self) -> Result<Value, HclError> {
        self.advance_by(3); // consume "for"
        self.skip_whitespace_and_newlines();
        let var1 = self.parse_identifier()?;
        self.skip_whitespace_and_newlines();
        let mut var2 = None;
        if self.peek_char() == Some(',') {
            self.bump();
            self.skip_whitespace_and_newlines();
            var2 = Some(self.parse_identifier()?);
            self.skip_whitespace_and_newlines();
        }
        if !self.check_keyword("in") {
            return Err(self.error("expected 'in' after for variables".into()));
        }
        self.advance_by(2); // consume "in"
        self.skip_whitespace_and_newlines();
        let coll = self.parse_expr()?;
        self.skip_whitespace_and_newlines();
        if self.peek_char() != Some(':') {
            return Err(self.error("expected ':' in for expression".into()));
        }
        self.bump(); // consume ':'
        self.skip_whitespace_and_newlines();
        let key_expr = self.parse_expr()?;
        self.skip_whitespace_and_newlines();
        if self.peek_char() != Some('=') || self.peek_ahead(1) != Some('>') {
            return Err(self.error("expected '=>' in object for expression".into()));
        }
        self.bump(); // '='
        self.bump(); // '>'
        self.skip_whitespace_and_newlines();
        let val_expr = self.parse_expr()?;
        self.skip_whitespace_and_newlines();
        let is_ellipsis = if self.peek_char() == Some('.')
            && self.peek_ahead(1) == Some('.')
            && self.peek_ahead(2) == Some('.')
        {
            self.bump();
            self.bump();
            self.bump();
            true
        } else {
            false
        };
        self.skip_whitespace_and_newlines();
        let mut if_cond = None;
        if self.check_keyword("if") {
            self.advance_by(2);
            self.skip_whitespace_and_newlines();
            if_cond = Some(self.parse_expr()?);
            self.skip_whitespace_and_newlines();
        }
        if self.peek_char() != Some('}') {
            return Err(self.error("expected '}' closing object for expression".into()));
        }
        self.bump(); // consume '}'

        let vars = if let Some(v2) = var2 {
            alloc::format!("{}, {}", var1, v2)
        } else {
            var1
        };
        let cond_str = if let Some(c) = if_cond {
            alloc::format!(" if {}", Self::value_to_expr_string(&c))
        } else {
            "".to_string()
        };
        let ell_str = if is_ellipsis { "..." } else { "" };
        Ok(Value::String(alloc::format!(
            "{{for {} in {}: {} => {}{}{}}}",
            vars,
            Self::value_to_expr_string(&coll),
            Self::value_to_expr_string(&key_expr),
            Self::value_to_expr_string(&val_expr),
            ell_str,
            cond_str
        )))
    }

    fn parse_array(&mut self) -> Result<Value, HclError> {
        self.bump(); // consume '['
        self.paren_depth += 1;
        self.skip_whitespace_and_newlines();
        if self.check_keyword("for") {
            let res = self.parse_for_tuple();
            self.paren_depth -= 1;
            return res;
        }

        let mut items = Vec::new();
        let mut first = true;
        let mut saw_comma = false;
        while !self.is_eof() {
            self.skip_whitespace_and_newlines();
            if self.peek_char() == Some(']') {
                self.bump();
                self.paren_depth -= 1;
                return Ok(Value::Array(items));
            }

            if !first && !saw_comma {
                self.paren_depth -= 1;
                return Err(self.error("expected ',' separating tuple elements".into()));
            }
            first = false;

            let item = self.parse_expr()?;
            items.push(item);

            self.skip_whitespace_and_newlines();
            if self.peek_char() == Some(',') {
                self.bump();
                saw_comma = true;
            } else {
                saw_comma = false;
            }
        }

        self.paren_depth -= 1;
        Err(HclError::UnexpectedEof)
    }

    fn parse_object(&mut self) -> Result<Value, HclError> {
        self.bump(); // consume '{'
        self.paren_depth += 1;
        self.skip_whitespace_and_newlines();
        if self.check_keyword("for") {
            let res = self.parse_for_object();
            self.paren_depth -= 1;
            return res;
        }

        let mut entries = Vec::new();
        let mut first = true;
        let mut saw_comma = false;
        while !self.is_eof() {
            let saw_newline = self.skip_whitespace_and_newlines();
            if self.peek_char() == Some('}') {
                self.bump();
                self.paren_depth -= 1;
                return Ok(Value::Object(entries));
            }

            if !first && !saw_newline && !saw_comma {
                self.paren_depth -= 1;
                return Err(
                    self.error("expected newline or comma separating object elements".into())
                );
            }
            first = false;

            let key_val = self.parse_expr()?;
            let key = Self::value_to_expr_string(&key_val);
            self.skip_whitespace_and_comments();

            if self.peek_char() == Some('=') || self.peek_char() == Some(':') {
                self.bump();
            } else {
                self.paren_depth -= 1;
                return Err(self.error("expected '=' or ':' after object key".into()));
            }

            self.skip_whitespace_and_comments();
            let val = self.parse_expr()?;
            entries.push((key, val));

            self.skip_whitespace_and_comments();
            if self.peek_char() == Some(',') {
                self.bump();
                saw_comma = true;
            } else {
                saw_comma = false;
            }
        }

        self.paren_depth -= 1;
        Err(HclError::UnexpectedEof)
    }

    fn parse_number(&mut self) -> Result<Value, HclError> {
        let start = self.idx;
        if self.peek_char() == Some('-') {
            self.bump();
        }

        let mut has_decimal = false;
        let mut has_exponent = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.bump();
            } else if c == '.'
                && !has_decimal
                && !has_exponent
                && self
                    .peek_ahead(1)
                    .map_or(false, |next| next.is_ascii_digit())
            {
                has_decimal = true;
                self.bump();
            } else if (c == 'e' || c == 'E') && !has_exponent {
                has_exponent = true;
                self.bump();
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.bump();
                }
            } else {
                break;
            }
        }

        if (has_decimal || has_exponent) && self.peek_char() == Some('.') {
            return Err(
                self.error("unexpected point after exponent or second fractional part".into())
            );
        }

        let is_float = has_decimal || has_exponent;
        let slice = self.get_slice(start, self.idx);
        if is_float {
            slice
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| self.error(format!("invalid float: {}", slice)))
        } else {
            slice
                .parse::<i128>()
                .map(Value::Integer)
                .or_else(|_| slice.parse::<f64>().map(Value::Float))
                .map_err(|_| self.error(format!("invalid integer: {}", slice)))
        }
    }

    fn parse_quoted_string(&mut self) -> Result<String, HclError> {
        self.bump(); // consume opening '"'
        let mut s = String::new();
        let mut dir_stack: Vec<Directive> = Vec::new();

        while !self.is_eof() {
            match self.bump() {
                Some('"') => {
                    if !dir_stack.is_empty() {
                        return Err(self.error("unclosed template directive".into()));
                    }
                    return Ok(s);
                }
                Some('\n') => return Err(self.error("unescaped newline in string literal".into())),
                Some('\r') => return Err(self.error("unescaped CR in string literal".into())),
                Some('\\') => match self.bump() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('u') => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let hex_c = self.bump().ok_or(HclError::UnexpectedEof)?;
                            let digit = hex_c
                                .to_digit(16)
                                .ok_or_else(|| self.error("invalid unicode escape".into()))?;
                            code = (code << 4) | digit;
                        }
                        let ch = char::from_u32(code)
                            .ok_or_else(|| self.error("invalid unicode scalar".into()))?;
                        s.push(ch);
                    }
                    Some('U') => {
                        let mut code = 0u32;
                        for _ in 0..8 {
                            let hex_c = self.bump().ok_or(HclError::UnexpectedEof)?;
                            let digit = hex_c
                                .to_digit(16)
                                .ok_or_else(|| self.error("invalid unicode escape".into()))?;
                            code = (code << 4) | digit;
                        }
                        let ch = char::from_u32(code)
                            .ok_or_else(|| self.error("invalid unicode scalar".into()))?;
                        s.push(ch);
                    }
                    Some(c) => return Err(self.error(format!("invalid escape sequence '\\{}'", c))),
                    None => return Err(HclError::UnexpectedEof),
                },
                Some('$') if self.peek_char() == Some('$') && self.peek_ahead(1) == Some('{') => {
                    self.bump(); // consume second '$'
                    s.push('$');
                    s.push('{');
                    self.bump(); // consume '{'
                }
                Some('%') if self.peek_char() == Some('%') && self.peek_ahead(1) == Some('{') => {
                    self.bump(); // consume second '%'
                    s.push('%');
                    s.push('{');
                    self.bump(); // consume '{'
                }
                Some('$') if self.peek_char() == Some('{') => {
                    self.bump(); // consume '{'
                    let interp = self.parse_template_interpolation()?;
                    s.push_str(&interp);
                }
                Some('%') if self.peek_char() == Some('{') => {
                    self.bump(); // consume '{'
                    self.parse_template_directive(&mut dir_stack)?;
                }
                Some(c) => s.push(c),
                None => return Err(HclError::UnexpectedEof),
            }
        }

        Err(HclError::UnexpectedEof)
    }

    fn parse_template_interpolation(&mut self) -> Result<String, HclError> {
        let _strip_start = if self.peek_char() == Some('~') {
            self.bump();
            true
        } else {
            false
        };

        self.paren_depth += 1;
        self.skip_whitespace_and_newlines();

        if self.peek_char() == Some('}')
            || (self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}'))
        {
            self.paren_depth -= 1;
            return Err(self.error("empty interpolation is not allowed".into()));
        }

        let expr = self.parse_expr()?;
        self.skip_whitespace_and_newlines();

        let _strip_end = if self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}') {
            self.bump();
            true
        } else {
            false
        };

        if self.peek_char() == Some('}') {
            self.bump();
            self.paren_depth -= 1;
            Ok(alloc::format!("${{{}}}", Self::value_to_expr_string(&expr)))
        } else {
            self.paren_depth -= 1;
            Err(self.error("expected '}' closing interpolation".into()))
        }
    }

    fn parse_template_directive(&mut self, stack: &mut Vec<Directive>) -> Result<(), HclError> {
        let _strip_start = if self.peek_char() == Some('~') {
            self.bump();
            true
        } else {
            false
        };

        self.paren_depth += 1;
        self.skip_whitespace_and_newlines();

        if self.peek_char() == Some('}')
            || (self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}'))
        {
            self.paren_depth -= 1;
            return Err(self.error("empty directive is not allowed".into()));
        }

        let kw = self.parse_identifier()?;
        match kw.as_str() {
            "if" => {
                self.skip_whitespace_and_newlines();
                let _cond = self.parse_expr()?;
                self.skip_whitespace_and_newlines();
                if self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}') {
                    self.bump();
                }
                if self.peek_char() == Some('}') {
                    self.bump();
                    stack.push(Directive::If);
                } else {
                    self.paren_depth -= 1;
                    return Err(self.error("expected '}' closing if directive".into()));
                }
            }
            "else" => {
                self.skip_whitespace_and_newlines();
                match stack.last() {
                    Some(Directive::If) => {
                        *stack.last_mut().unwrap() = Directive::IfWithElse;
                    }
                    Some(Directive::IfWithElse) => {
                        self.paren_depth -= 1;
                        return Err(self.error("second else in if directive".into()));
                    }
                    _ => {
                        self.paren_depth -= 1;
                        return Err(self.error("else without if".into()));
                    }
                }
                if self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}') {
                    self.bump();
                }
                if self.peek_char() == Some('}') {
                    self.bump();
                } else {
                    self.paren_depth -= 1;
                    return Err(self.error("expected '}' closing else directive".into()));
                }
            }
            "endif" => {
                self.skip_whitespace_and_newlines();
                match stack.pop() {
                    Some(Directive::If) | Some(Directive::IfWithElse) => {}
                    Some(Directive::For) => {
                        self.paren_depth -= 1;
                        return Err(self.error("endif cannot close for directive".into()));
                    }
                    None => {
                        self.paren_depth -= 1;
                        return Err(self.error("endif without if".into()));
                    }
                }
                if self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}') {
                    self.bump();
                }
                if self.peek_char() == Some('}') {
                    self.bump();
                } else {
                    self.paren_depth -= 1;
                    return Err(self.error("expected '}' closing endif directive".into()));
                }
            }
            "for" => {
                self.skip_whitespace_and_newlines();
                let _var1 = self.parse_identifier()?;
                self.skip_whitespace_and_newlines();
                if self.peek_char() == Some(',') {
                    self.bump();
                    self.skip_whitespace_and_newlines();
                    let _var2 = self.parse_identifier()?;
                    self.skip_whitespace_and_newlines();
                }
                let in_kw = self.parse_identifier()?;
                if in_kw != "in" {
                    self.paren_depth -= 1;
                    return Err(self.error("expected 'in' in for directive".into()));
                }
                self.skip_whitespace_and_newlines();
                let _coll = self.parse_expr()?;
                self.skip_whitespace_and_newlines();
                if self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}') {
                    self.bump();
                }
                if self.peek_char() == Some('}') {
                    self.bump();
                    stack.push(Directive::For);
                } else {
                    self.paren_depth -= 1;
                    return Err(self.error("expected '}' closing for directive".into()));
                }
            }
            "endfor" => {
                self.skip_whitespace_and_newlines();
                match stack.pop() {
                    Some(Directive::For) => {}
                    Some(Directive::If) | Some(Directive::IfWithElse) => {
                        self.paren_depth -= 1;
                        return Err(self.error("endfor cannot close if directive".into()));
                    }
                    None => {
                        self.paren_depth -= 1;
                        return Err(self.error("endfor without for".into()));
                    }
                }
                if self.peek_char() == Some('~') && self.peek_ahead(1) == Some('}') {
                    self.bump();
                }
                if self.peek_char() == Some('}') {
                    self.bump();
                } else {
                    self.paren_depth -= 1;
                    return Err(self.error("expected '}' closing endfor directive".into()));
                }
            }
            _ => {
                self.paren_depth -= 1;
                return Err(self.error(format!("unknown directive keyword '{}'", kw)));
            }
        }

        self.paren_depth -= 1;
        Ok(())
    }

    fn parse_quoted_string_literal(&mut self) -> Result<String, HclError> {
        self.bump(); // consume opening '"'
        let mut s = String::new();

        while !self.is_eof() {
            match self.bump() {
                Some('"') => return Ok(s),
                Some('\n') => return Err(self.error("unescaped newline in string literal".into())),
                Some('\r') => return Err(self.error("unescaped CR in string literal".into())),
                Some('\\') => match self.bump() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('u') => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let hex_c = self.bump().ok_or(HclError::UnexpectedEof)?;
                            let digit = hex_c
                                .to_digit(16)
                                .ok_or_else(|| self.error("invalid unicode escape".into()))?;
                            code = (code << 4) | digit;
                        }
                        let ch = char::from_u32(code)
                            .ok_or_else(|| self.error("invalid unicode scalar".into()))?;
                        s.push(ch);
                    }
                    Some('U') => {
                        let mut code = 0u32;
                        for _ in 0..8 {
                            let hex_c = self.bump().ok_or(HclError::UnexpectedEof)?;
                            let digit = hex_c
                                .to_digit(16)
                                .ok_or_else(|| self.error("invalid unicode escape".into()))?;
                            code = (code << 4) | digit;
                        }
                        let ch = char::from_u32(code)
                            .ok_or_else(|| self.error("invalid unicode scalar".into()))?;
                        s.push(ch);
                    }
                    Some(c) => return Err(self.error(format!("invalid escape sequence '\\{}'", c))),
                    None => return Err(HclError::UnexpectedEof),
                },
                Some('$') if self.peek_char() == Some('$') && self.peek_ahead(1) == Some('{') => {
                    self.bump();
                    s.push('$');
                    s.push('{');
                    self.bump();
                }
                Some('%') if self.peek_char() == Some('%') && self.peek_ahead(1) == Some('{') => {
                    self.bump();
                    s.push('%');
                    s.push('{');
                    self.bump();
                }
                Some('$') if self.peek_char() == Some('{') => {
                    return Err(self.error("block label cannot contain interpolation".into()));
                }
                Some('%') if self.peek_char() == Some('{') => {
                    return Err(self.error("block label cannot contain directive".into()));
                }
                Some(c) => s.push(c),
                None => return Err(HclError::UnexpectedEof),
            }
        }

        Err(HclError::UnexpectedEof)
    }

    fn validate_template_content(&self, text: &str) -> Result<(), HclError> {
        let mut parser = HclParser::new(text);
        let mut dir_stack: Vec<Directive> = Vec::new();
        while !parser.is_eof() {
            match parser.bump() {
                Some('$')
                    if parser.peek_char() == Some('$') && parser.peek_ahead(1) == Some('{') =>
                {
                    parser.bump();
                    parser.bump();
                }
                Some('%')
                    if parser.peek_char() == Some('%') && parser.peek_ahead(1) == Some('{') =>
                {
                    parser.bump();
                    parser.bump();
                }
                Some('$') if parser.peek_char() == Some('{') => {
                    parser.bump();
                    let _ = parser.parse_template_interpolation()?;
                }
                Some('%') if parser.peek_char() == Some('{') => {
                    parser.bump();
                    parser.parse_template_directive(&mut dir_stack)?;
                }
                _ => {}
            }
        }
        if !dir_stack.is_empty() {
            return Err(self.error("unclosed template directive in heredoc".into()));
        }
        Ok(())
    }

    fn parse_heredoc(&mut self) -> Result<String, HclError> {
        self.bump(); // '<'
        self.bump(); // '<'
        let indented = if self.peek_char() == Some('-') {
            self.bump();
            true
        } else {
            false
        };

        let marker = self.parse_identifier()?;
        // Require newline after heredoc opening marker
        let mut saw_newline = false;
        while let Some(c) = self.bump() {
            if c == '\n' {
                saw_newline = true;
                break;
            } else if c == '\r' && self.peek_char() == Some('\n') {
                self.bump();
                saw_newline = true;
                break;
            } else if !Self::is_hcl_whitespace(c) {
                return Err(self.error("unexpected character after heredoc marker".into()));
            }
        }
        if !saw_newline {
            return Err(HclError::UnexpectedEof);
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut closed = false;

        while !self.is_eof() {
            match self.bump() {
                Some('\n') => {
                    let trimmed = current_line.trim();
                    if trimmed == marker {
                        closed = true;
                        break;
                    }
                    lines.push(current_line);
                    current_line = String::new();
                }
                Some('\r') if self.peek_char() != Some('\n') => {
                    return Err(self.error("lone CR in heredoc is invalid".into()));
                }
                Some(c) => current_line.push(c),
                None => {
                    if current_line.trim() == marker {
                        closed = true;
                        break;
                    }
                    return Err(HclError::UnexpectedEof);
                }
            }
        }

        if !closed {
            return Err(HclError::UnexpectedEof);
        }

        self.last_was_heredoc = true;
        self.heredoc_saw_newline = true;

        let result_body = if indented {
            let min_indent = lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| {
                    l.chars()
                        .take_while(|c| Self::is_hcl_whitespace(*c))
                        .count()
                })
                .min()
                .unwrap_or(0);

            let mut result = String::new();
            for line in lines {
                let stripped: String = line.chars().skip(min_indent).collect();
                result.push_str(&stripped);
                result.push('\n');
            }
            if result.ends_with('\n') {
                result.pop();
            }
            result
        } else {
            let mut result = lines.join("\n");
            if result.ends_with('\n') {
                result.pop();
            }
            result
        };

        self.validate_template_content(&result_body)?;
        Ok(result_body)
    }

    fn parse_identifier(&mut self) -> Result<String, HclError> {
        let start = self.idx;
        if let Some(c) = self.peek_char() {
            if Self::is_hcl_id_start(c) {
                self.bump();
            } else {
                return Err(
                    self.error("expected identifier starting with letter or underscore".into())
                );
            }
        } else {
            return Err(HclError::UnexpectedEof);
        }

        while let Some(c) = self.peek_char() {
            if Self::is_hcl_id_continue(c) {
                self.bump();
            } else {
                break;
            }
        }
        Ok(self.get_slice(start, self.idx).to_string())
    }

    fn skip_whitespace_and_newlines(&mut self) -> bool {
        let mut saw_newline = false;
        while !self.is_eof() {
            match self.peek_char() {
                Some('\n') => {
                    saw_newline = true;
                    self.bump();
                }
                Some('\r') if self.peek_ahead(1) == Some('\n') => {
                    saw_newline = true;
                    self.bump();
                    self.bump();
                }
                Some(c) if Self::is_hcl_whitespace(c) => {
                    self.bump();
                }
                Some('#') => {
                    self.bump();
                    while let Some(c) = self.bump() {
                        if c == '\n' {
                            saw_newline = true;
                            break;
                        }
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('/') => {
                    self.bump();
                    self.bump();
                    while let Some(c) = self.bump() {
                        if c == '\n' {
                            saw_newline = true;
                            break;
                        }
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('*') => {
                    self.bump();
                    self.bump();
                    let mut terminated = false;
                    while !self.is_eof() {
                        if self.peek_char() == Some('*') && self.peek_ahead(1) == Some('/') {
                            self.bump();
                            self.bump();
                            terminated = true;
                            break;
                        }
                        if self.peek_char() == Some('\n') {
                            saw_newline = true;
                        }
                        self.bump();
                    }
                    if !terminated {
                        self.has_unterminated_comment = true;
                        break;
                    }
                }
                _ => break,
            }
        }
        saw_newline
    }

    fn check_keyword(&self, kw: &str) -> bool {
        let chars_left = self.chars.len() - self.idx;
        if chars_left < kw.len() {
            return false;
        }
        let kw_chars: Vec<char> = kw.chars().collect();
        for i in 0..kw.len() {
            if self.chars[self.idx + i].1 != kw_chars[i] {
                return false;
            }
        }
        // Ensure not part of longer identifier
        if chars_left > kw.len() {
            let next_c = self.chars[self.idx + kw.len()].1;
            if Self::is_hcl_id_continue(next_c) {
                return false;
            }
        }
        true
    }

    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn skip_horizontal_whitespace_and_comments(&mut self) {
        while !self.is_eof() {
            match self.peek_char() {
                Some(c) if Self::is_hcl_whitespace(c) => {
                    self.bump();
                }
                Some('/') if self.peek_ahead(1) == Some('*') => {
                    let mut look = 2;
                    let mut has_nl = false;
                    while let Some(c) = self.peek_ahead(look) {
                        if c == '\n' {
                            has_nl = true;
                            break;
                        }
                        if c == '*' && self.peek_ahead(look + 1) == Some('/') {
                            break;
                        }
                        look += 1;
                    }
                    if has_nl {
                        break;
                    }
                    self.bump();
                    self.bump();
                    let mut terminated = false;
                    while !self.is_eof() {
                        if self.peek_char() == Some('*') && self.peek_ahead(1) == Some('/') {
                            self.bump();
                            self.bump();
                            terminated = true;
                            break;
                        }
                        self.bump();
                    }
                    if !terminated {
                        self.has_unterminated_comment = true;
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_eof() {
            match self.peek_char() {
                Some(c) if Self::is_hcl_whitespace(c) => {
                    self.bump();
                }
                Some('#') => {
                    self.bump();
                    while let Some(c) = self.peek_char() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('/') => {
                    self.bump();
                    self.bump();
                    while let Some(c) = self.peek_char() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('*') => {
                    self.bump();
                    self.bump();
                    let mut terminated = false;
                    while !self.is_eof() {
                        if self.peek_char() == Some('*') && self.peek_ahead(1) == Some('/') {
                            self.bump();
                            self.bump();
                            terminated = true;
                            break;
                        }
                        self.bump();
                    }
                    if !terminated {
                        self.has_unterminated_comment = true;
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.idx >= self.chars.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.idx).map(|(_, c)| *c)
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.chars.get(self.idx + n).map(|(_, c)| *c)
    }

    fn bump(&mut self) -> Option<char> {
        if self.idx < self.chars.len() {
            let (_, c) = self.chars[self.idx];
            self.idx += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn get_slice(&self, start_idx: usize, end_idx: usize) -> &'a str {
        if start_idx >= self.chars.len() {
            return "";
        }
        let byte_start = self.chars[start_idx].0;
        let byte_end = if end_idx < self.chars.len() {
            self.chars[end_idx].0
        } else {
            self.input.len()
        };
        &self.input[byte_start..byte_end]
    }

    fn error(&self, msg: String) -> HclError {
        HclError::InvalidSyntax {
            line: self.line,
            col: self.col,
            msg,
        }
    }
}

/// Convenience function parsing HCL text into a universal `Value`.
pub fn from_str(input: &str) -> Result<Value, HclError> {
    let mut parser = HclParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hcl_basic() {
        let src = r#"
            # Terraform style configuration
            variable "region" {
                default     = "us-west-2"
                description = "AWS region"
                count       = 3
                enabled     = true
            }

            output "ip" {
                value = ["10.0.0.1", "10.0.0.2"]
            }
        "#;

        let val = from_str(src).unwrap();
        assert!(matches!(val, Value::Object(_)));
        let obj = val.as_object().unwrap();
        assert!(obj.iter().any(|(k, _)| k == "variable"));
        assert!(obj.iter().any(|(k, _)| k == "output"));
    }
}
