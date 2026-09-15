//! Recursive descent parser for HashiCorp HCL v2 syntax into universal `Value` AST.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

use babbel_core::Value;
use crate::error::HclError;

/// Parser state for HCL text input.
pub struct HclParser<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    idx: usize,
    line: usize,
    col: usize,
}

impl<'a> HclParser<'a> {
    /// Create a new HCL parser.
    pub fn new(input: &'a str) -> Self {
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            input,
            chars,
            idx: 0,
            line: 1,
            col: 1,
        }
    }

    /// Parse the entire HCL input into a top-level `Value::Object`.
    pub fn parse(&mut self) -> Result<Value, HclError> {
        self.skip_whitespace_and_comments();
        let mut entries = Vec::new();

        while !self.is_eof() {
            let (key, val) = self.parse_attribute_or_block()?;
            Self::merge_entry(&mut entries, key, val);
            self.skip_whitespace_and_comments();
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

    fn parse_attribute_or_block(&mut self) -> Result<(String, Value), HclError> {
        self.skip_whitespace_and_comments();
        let name = self.parse_identifier_or_string()?;
        self.skip_whitespace_and_comments();

        // Check if attribute assignment: `name = value`
        if self.peek_char() == Some('=') {
            self.bump(); // consume '='
            self.skip_whitespace_and_comments();
            let val = self.parse_expr()?;
            self.consume_optional_separator();
            return Ok((name, val));
        }

        // Otherwise, it's a block: `name [labels...] { ... }`
        let mut labels = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            match self.peek_char() {
                Some('{') => {
                    self.bump(); // consume '{'
                    let body = self.parse_block_body()?;
                    let wrapped = Self::nest_labels(labels, body);
                    return Ok((name, wrapped));
                }
                Some('"') => {
                    let label = self.parse_quoted_string()?;
                    labels.push(label);
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
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

    fn nest_labels(labels: Vec<String>, body: Value) -> Value {
        let mut current = body;
        for label in labels.into_iter().rev() {
            current = Value::Object(alloc::vec![(label, current)]);
        }
        current
    }

    fn parse_block_body(&mut self) -> Result<Value, HclError> {
        let mut entries = Vec::new();
        self.skip_whitespace_and_comments();

        while !self.is_eof() {
            if self.peek_char() == Some('}') {
                self.bump();
                return Ok(Value::Object(entries));
            }

            let (k, v) = self.parse_attribute_or_block()?;
            Self::merge_entry(&mut entries, k, v);
            self.skip_whitespace_and_comments();
        }

        Err(HclError::UnexpectedEof)
    }

    fn value_to_expr_string(v: &Value) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Bool(b) => alloc::format!("{}", b),
            Value::Integer(i) => alloc::format!("{}", i),
            Value::Float(f) => alloc::format!("{}", f),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let inner = arr.iter().map(Self::value_to_expr_string).collect::<Vec<_>>().join(", ");
                alloc::format!("[{}]", inner)
            }
            Value::Object(obj) => {
                let inner = obj.iter().map(|(k, val)| alloc::format!("{} = {}", k, Self::value_to_expr_string(val))).collect::<Vec<_>>().join(", ");
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
        self.skip_whitespace_and_comments();
        if self.peek_char() == Some('?') {
            self.bump(); // consume '?'
            self.skip_whitespace_and_comments();
            let true_val = self.parse_expr()?;
            self.skip_whitespace_and_comments();
            if self.peek_char() == Some(':') {
                self.bump(); // consume ':'
            } else {
                return Err(self.error("expected ':' in ternary expression".into()));
            }
            self.skip_whitespace_and_comments();
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

    fn peek_binary_op(&mut self) -> Option<(&'static str, usize)> {
        self.skip_whitespace_and_comments();
        match self.peek_char() {
            Some('|') if self.peek_ahead(1) == Some('|') => Some(("||", 2)),
            Some('&') if self.peek_ahead(1) == Some('&') => Some(("&&", 2)),
            Some('=') if self.peek_ahead(1) == Some('=') => Some(("==", 2)),
            Some('!') if self.peek_ahead(1) == Some('=') => Some(("!=", 2)),
            Some('<') if self.peek_ahead(1) == Some('=') => Some(("<=", 2)),
            Some('>') if self.peek_ahead(1) == Some('=') => Some((">=", 2)),
            Some('<') if self.peek_ahead(1) != Some('<') => Some(("<", 1)),
            Some('>') => Some((">", 1)),
            Some('+') => Some(("+", 1)),
            Some('-') => Some(("-", 1)),
            Some('*') => Some(("*", 1)),
            Some('/') if self.peek_ahead(1) != Some('/') && self.peek_ahead(1) != Some('*') => Some(("/", 1)),
            Some('%') => Some(("%", 1)),
            _ => None,
        }
    }

    fn parse_binary_expr(&mut self, min_prec: u8) -> Result<Value, HclError> {
        let mut lhs = self.parse_unary_or_postfix()?;

        loop {
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
            self.skip_whitespace_and_comments();
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
        self.skip_whitespace_and_comments();
        if self.peek_char() == Some('!') {
            self.bump();
            let operand = self.parse_unary_or_postfix()?;
            return match operand {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Ok(Value::String(alloc::format!("!{}", Self::value_to_expr_string(&operand)))),
            };
        }
        if self.peek_char() == Some('-') && self.peek_ahead(1).map_or(true, |c| !c.is_ascii_digit()) {
            self.bump();
            let operand = self.parse_unary_or_postfix()?;
            return match operand {
                Value::Integer(i) => Ok(Value::Integer(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Ok(Value::String(alloc::format!("-{}", Self::value_to_expr_string(&operand)))),
            };
        }

        let mut base = self.parse_primary()?;

        loop {
            self.skip_whitespace_and_comments();
            match self.peek_char() {
                Some('.') => {
                    self.bump(); // consume '.'
                    self.skip_whitespace_and_comments();
                    if self.peek_char() == Some('*') {
                        self.bump();
                        base = Value::String(alloc::format!("{}.*", Self::value_to_expr_string(&base)));
                    } else {
                        let member = self.parse_identifier()?;
                        base = Value::String(alloc::format!("{}.{}", Self::value_to_expr_string(&base), member));
                    }
                }
                Some('[') => {
                    self.bump(); // consume '['
                    self.skip_whitespace_and_comments();
                    if self.peek_char() == Some('*') && self.peek_ahead(1) == Some(']') {
                        self.bump();
                        self.bump();
                        base = Value::String(alloc::format!("{}[*]", Self::value_to_expr_string(&base)));
                    } else {
                        let idx = self.parse_expr()?;
                        self.skip_whitespace_and_comments();
                        if self.peek_char() == Some(']') {
                            self.bump();
                        } else {
                            return Err(self.error("expected ']' after index".into()));
                        }
                        base = Value::String(alloc::format!(
                            "{}[{}]",
                            Self::value_to_expr_string(&base),
                            Self::value_to_expr_string(&idx)
                        ));
                    }
                }
                _ => break,
            }
        }

        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Value, HclError> {
        self.skip_whitespace_and_comments();
        match self.peek_char() {
            Some('(') => {
                self.bump(); // consume '('
                let val = self.parse_expr()?;
                self.skip_whitespace_and_comments();
                if self.peek_char() == Some(')') {
                    self.bump(); // consume ')'
                    Ok(val)
                } else {
                    Err(self.error("expected ')' closing expression".into()))
                }
            }
            Some('"') => self.parse_quoted_string().map(Value::String),
            Some('<') if self.peek_ahead(1) == Some('<') => self.parse_heredoc().map(Value::String),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some('-') | Some('0'..='9') => self.parse_number(),
            Some('t') if self.check_keyword("true") => {
                self.advance_by(4);
                Ok(Value::Bool(true))
            }
            Some('f') if self.check_keyword("false") => {
                self.advance_by(5);
                Ok(Value::Bool(false))
            }
            Some('n') if self.check_keyword("null") => {
                self.advance_by(4);
                Ok(Value::Null)
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let id = self.parse_identifier()?;
                self.skip_whitespace_and_comments();
                if self.peek_char() == Some('(') {
                    self.bump(); // consume '('
                    let mut args = Vec::new();
                    self.skip_whitespace_and_comments();
                    while !self.is_eof() && self.peek_char() != Some(')') {
                        let arg = self.parse_expr()?;
                        args.push(arg);
                        self.skip_whitespace_and_comments();
                        if self.peek_char() == Some(',') {
                            self.bump();
                            self.skip_whitespace_and_comments();
                        }
                    }
                    if self.peek_char() == Some(')') {
                        self.bump();
                    } else {
                        return Err(self.error("expected ')' closing function call".into()));
                    }
                    let args_str = args.iter().map(Self::value_to_expr_string).collect::<Vec<_>>().join(", ");
                    Ok(Value::String(alloc::format!("{}({})", id, args_str)))
                } else {
                    Ok(Value::String(id))
                }
            }
            Some(c) => Err(self.error(format!("unexpected character '{}' starting expression", c))),
            None => Err(HclError::UnexpectedEof),
        }
    }

    fn parse_array(&mut self) -> Result<Value, HclError> {
        self.bump(); // consume '['
        self.skip_whitespace_and_comments();
        if self.check_keyword("for") {
            let start_idx = self.idx;
            let mut depth = 1;
            while !self.is_eof() {
                match self.bump() {
                    Some('[') => depth += 1,
                    Some(']') => {
                        depth -= 1;
                        if depth == 0 {
                            let s = self.get_slice(start_idx, self.idx - 1);
                            return Ok(Value::String(alloc::format!("[{}]", s.trim())));
                        }
                    }
                    _ => {}
                }
            }
            return Err(HclError::UnexpectedEof);
        }

        let mut items = Vec::new();
        while !self.is_eof() {
            if self.peek_char() == Some(']') {
                self.bump();
                return Ok(Value::Array(items));
            }

            let item = self.parse_expr()?;
            items.push(item);
            self.skip_whitespace_and_comments();

            if self.peek_char() == Some(',') {
                self.bump();
                self.skip_whitespace_and_comments();
            }
        }

        Err(HclError::UnexpectedEof)
    }

    fn parse_object(&mut self) -> Result<Value, HclError> {
        self.bump(); // consume '{'
        self.skip_whitespace_and_comments();
        if self.check_keyword("for") {
            let start_idx = self.idx;
            let mut depth = 1;
            while !self.is_eof() {
                match self.bump() {
                    Some('{') => depth += 1,
                    Some('}') => {
                        depth -= 1;
                        if depth == 0 {
                            let s = self.get_slice(start_idx, self.idx - 1);
                            return Ok(Value::String(alloc::format!("{{{}}}", s.trim())));
                        }
                    }
                    _ => {}
                }
            }
            return Err(HclError::UnexpectedEof);
        }

        let mut entries = Vec::new();
        while !self.is_eof() {
            if self.peek_char() == Some('}') {
                self.bump();
                return Ok(Value::Object(entries));
            }

            let key = self.parse_identifier_or_string()?;
            self.skip_whitespace_and_comments();

            if self.peek_char() == Some('=') || self.peek_char() == Some(':') {
                self.bump();
            } else {
                return Err(self.error("expected '=' or ':' after object key".into()));
            }

            self.skip_whitespace_and_comments();
            let val = self.parse_expr()?;
            entries.push((key, val));
            self.skip_whitespace_and_comments();

            if self.peek_char() == Some(',') {
                self.bump();
                self.skip_whitespace_and_comments();
            }
        }

        Err(HclError::UnexpectedEof)
    }

    fn parse_number(&mut self) -> Result<Value, HclError> {
        let start = self.idx;
        if self.peek_char() == Some('-') {
            self.bump();
        }

        let mut is_float = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.bump();
            } else if c == '.' && !is_float {
                is_float = true;
                self.bump();
            } else if (c == 'e' || c == 'E') && !is_float {
                is_float = true;
                self.bump();
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.bump();
                }
            } else {
                break;
            }
        }

        let slice = self.get_slice(start, self.idx);
        if is_float {
            slice.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| self.error(format!("invalid float: {}", slice)))
        } else {
            slice.parse::<i128>()
                .map(Value::Integer)
                .map_err(|_| self.error(format!("invalid integer: {}", slice)))
        }
    }

    fn parse_quoted_string(&mut self) -> Result<String, HclError> {
        self.bump(); // consume opening '"'
        let mut s = String::new();

        while !self.is_eof() {
            match self.bump() {
                Some('"') => return Ok(s),
                Some('\n') => return Err(self.error("unescaped newline in string literal".into())),
                Some('\\') => match self.bump() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('/') => s.push('/'),
                    Some('b') => s.push('\x08'),
                    Some('f') => s.push('\x0C'),
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('u') => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let hex_c = self.bump().ok_or(HclError::UnexpectedEof)?;
                            let digit = hex_c.to_digit(16).ok_or_else(|| self.error("invalid unicode escape".into()))?;
                            code = (code << 4) | digit;
                        }
                        let ch = char::from_u32(code).ok_or_else(|| self.error("invalid unicode scalar".into()))?;
                        s.push(ch);
                    }
                    Some('U') => {
                        let mut code = 0u32;
                        for _ in 0..8 {
                            let hex_c = self.bump().ok_or(HclError::UnexpectedEof)?;
                            let digit = hex_c.to_digit(16).ok_or_else(|| self.error("invalid unicode escape".into()))?;
                            code = (code << 4) | digit;
                        }
                        let ch = char::from_u32(code).ok_or_else(|| self.error("invalid unicode scalar".into()))?;
                        s.push(ch);
                    }
                    Some(c) => return Err(self.error(format!("invalid escape sequence '\\{}'", c))),
                    None => return Err(HclError::UnexpectedEof),
                },
                Some('$') if self.peek_char() == Some('{') => {
                    self.bump(); // consume '{'
                    s.push_str("${");
                    let mut depth = 1;
                    while !self.is_eof() {
                        match self.bump() {
                            Some('{') => {
                                depth += 1;
                                s.push('{');
                            }
                            Some('}') => {
                                depth -= 1;
                                s.push('}');
                                if depth == 0 {
                                    break;
                                }
                            }
                            Some('"') => {
                                s.push('"');
                                let inner_s = self.parse_quoted_string()?;
                                s.push_str(&inner_s);
                                s.push('"');
                            }
                            Some(c) => s.push(c),
                            None => return Err(HclError::UnexpectedEof),
                        }
                    }
                    if depth != 0 {
                        return Err(self.error("unterminated interpolation".into()));
                    }
                }
                Some('%') if self.peek_char() == Some('{') => {
                    self.bump(); // consume '{'
                    s.push_str("%{");
                    let mut depth = 1;
                    while !self.is_eof() {
                        match self.bump() {
                            Some('{') => {
                                depth += 1;
                                s.push('{');
                            }
                            Some('}') => {
                                depth -= 1;
                                s.push('}');
                                if depth == 0 {
                                    break;
                                }
                            }
                            Some('"') => {
                                s.push('"');
                                let inner_s = self.parse_quoted_string()?;
                                s.push_str(&inner_s);
                                s.push('"');
                            }
                            Some(c) => s.push(c),
                            None => return Err(HclError::UnexpectedEof),
                        }
                    }
                    if depth != 0 {
                        return Err(self.error("unterminated directive".into()));
                    }
                }
                Some(c) => s.push(c),
                None => return Err(HclError::UnexpectedEof),
            }
        }

        Err(HclError::UnexpectedEof)
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
        // Skip until newline
        while let Some(c) = self.bump() {
            if c == '\n' {
                break;
            }
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        while !self.is_eof() {
            match self.bump() {
                Some('\n') => {
                    let trimmed = current_line.trim();
                    if trimmed == marker {
                        break;
                    }
                    lines.push(current_line);
                    current_line = String::new();
                }
                Some(c) => current_line.push(c),
                None => {
                    if current_line.trim() == marker {
                        break;
                    }
                    return Err(HclError::UnexpectedEof);
                }
            }
        }

        if indented {
            // Find minimum indentation
            let min_indent = lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
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
            Ok(result)
        } else {
            let mut result = lines.join("\n");
            if result.ends_with('\n') {
                result.pop();
            }
            Ok(result)
        }
    }

    fn parse_identifier(&mut self) -> Result<String, HclError> {
        let start = self.idx;
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                self.bump();
            } else {
                break;
            }
        }
        if start == self.idx {
            return Err(self.error("expected identifier".into()));
        }
        Ok(self.get_slice(start, self.idx).to_string())
    }

    fn parse_identifier_or_string(&mut self) -> Result<String, HclError> {
        if self.peek_char() == Some('"') {
            self.parse_quoted_string()
        } else {
            self.parse_identifier()
        }
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
            if next_c.is_alphanumeric() || next_c == '_' {
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

    fn consume_optional_separator(&mut self) {
        self.skip_whitespace_and_comments();
        if self.peek_char() == Some(',') || self.peek_char() == Some(';') {
            self.bump();
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_eof() {
            match self.peek_char() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('#') => {
                    self.bump();
                    while let Some(c) = self.bump() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('/') => {
                    self.bump();
                    self.bump();
                    while let Some(c) = self.bump() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                Some('/') if self.peek_ahead(1) == Some('*') => {
                    self.bump();
                    self.bump();
                    while !self.is_eof() {
                        if self.peek_char() == Some('*') && self.peek_ahead(1) == Some('/') {
                            self.bump();
                            self.bump();
                            break;
                        }
                        self.bump();
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
