//! JSON5 and JSONC parser implementation
//!
//! Provides full specification-compliant JSON5 (json5.org) and JSONC parsing:
//! - Single-line (`//`) and multi-line (`/* */`) comments
//! - Unquoted object keys (ECMAScript 5.1 IdentifierName) and single-quoted keys
//! - Trailing commas in objects and arrays
//! - Single-quoted strings and multi-line strings via escaped newlines
//! - Hexadecimal numbers (`0x...`), leading/trailing decimal points (`.5`, `5.`), explicit plus sign (`+42`)
//! - IEEE 754 specials: `Infinity`, `-Infinity`, `NaN`
//! - Legacy comment-stripping utility `strip_comments` for backward compatibility

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
#[cfg(feature = "std")]
use std::collections::HashMap;

use crate::nodes::node::{Node, Numeric};

/// Strip JSON5-style comments from input (retained for backward compatibility).
pub fn strip_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Check for single-line comment
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            if i < bytes.len() {
                output.push('\n');
                i += 1;
            }
            continue;
        }

        // Check for multi-line comment
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    break;
                }
                if bytes[i] == b'\n' {
                    output.push('\n');
                }
                i += 1;
            }
            continue;
        }

        // Check for double-quoted string literals
        if bytes[i] == b'"' {
            output.push('"');
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    output.push(bytes[i] as char);
                    output.push(bytes[i + 1] as char);
                    i += 2;
                } else if bytes[i] == b'"' {
                    output.push('"');
                    i += 1;
                    break;
                } else {
                    output.push(bytes[i] as char);
                    i += 1;
                }
            }
            continue;
        }

        // Check for single-quoted string literals
        if bytes[i] == b'\'' {
            output.push('\'');
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    output.push(bytes[i] as char);
                    output.push(bytes[i + 1] as char);
                    i += 2;
                } else if bytes[i] == b'\'' {
                    output.push('\'');
                    i += 1;
                    break;
                } else {
                    output.push(bytes[i] as char);
                    i += 1;
                }
            }
            continue;
        }

        // Regular character
        output.push(bytes[i] as char);
        i += 1;
    }

    output
}

/// Parses a JSON5/JSONC string into a Babbel [`Node`] tree.
pub fn parse_json5(input: &str) -> Result<Node, String> {
    let mut parser = Json5Parser::new(input);
    let node = parser.parse()?;
    parser.skip_whitespace_and_comments()?;
    if let Some(ch) = parser.peek() {
        return Err(format!(
            "Unexpected trailing character '{}' at byte {}",
            ch, parser.cursor
        ));
    }
    Ok(node)
}

/// Convenience alias for `parse_json5`.
pub fn from_str(input: &str) -> Result<Node, String> {
    parse_json5(input)
}

/// Convenience function to parse JSON5 from a byte slice.
pub fn from_bytes(bytes: &[u8]) -> Result<Node, String> {
    let s =
        core::str::from_utf8(bytes).map_err(|_| "Invalid UTF-8 in JSON5 payload".to_string())?;
    parse_json5(s)
}

/// Recursive-descent parser for JSON5.
pub struct Json5Parser {
    chars: Vec<(usize, char)>,
    cursor: usize,
    depth: usize,
    max_depth: usize,
}

impl Json5Parser {
    pub fn new(input: &str) -> Self {
        let chars: Vec<(usize, char)> = input.char_indices().collect();
        Self {
            chars,
            cursor: 0,
            depth: 0,
            max_depth: 256,
        }
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).map(|&(_, c)| c)
    }

    #[inline]
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).map(|&(_, c)| c)
    }

    #[inline]
    fn next_char(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor].1;
            self.cursor += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), String> {
        loop {
            match self.peek() {
                Some(c) if is_json5_whitespace(c) => {
                    self.cursor += 1;
                }
                Some('/') => {
                    match self.peek_next() {
                        Some('/') => {
                            // Single line comment: advance past // and consume until newline or EOF
                            self.cursor += 2;
                            while let Some(ch) = self.peek() {
                                self.cursor += 1;
                                if ch == '\n' {
                                    break;
                                }
                            }
                        }
                        Some('*') => {
                            // Multi-line block comment: advance past /* and consume until */
                            self.cursor += 2;
                            let mut terminated = false;
                            while self.cursor < self.chars.len() {
                                if self.chars[self.cursor].1 == '*'
                                    && self.cursor + 1 < self.chars.len()
                                    && self.chars[self.cursor + 1].1 == '/'
                                {
                                    self.cursor += 2;
                                    terminated = true;
                                    break;
                                }
                                self.cursor += 1;
                            }
                            if !terminated {
                                return Err("Unterminated multi-line comment in JSON5".to_string());
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        self.skip_whitespace_and_comments()?;
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') | Some('\'') => self.parse_string(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null_or_nan(),
            Some('+') | Some('-') | Some('.') | Some('0'..='9') | Some('I') | Some('N') => {
                self.parse_number()
            }
            Some(c) => Err(format!("Unexpected character '{}' in JSON5 value", c)),
            None => Err("Unexpected end of JSON5 input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<Node, String> {
        if self.depth >= self.max_depth {
            return Err("JSON5 maximum recursion depth exceeded".to_string());
        }
        self.depth += 1;
        self.cursor += 1; // consume '{'

        let mut map = HashMap::new();

        loop {
            self.skip_whitespace_and_comments()?;
            match self.peek() {
                Some('}') => {
                    self.cursor += 1; // consume '}'
                    self.depth -= 1;
                    return Ok(Node::Object(map));
                }
                Some(_) => {
                    let key = self.parse_key()?;
                    self.skip_whitespace_and_comments()?;
                    if self.peek() != Some(':') {
                        return Err(format!(
                            "Expected ':' after key in JSON5 object at byte {}",
                            self.cursor
                        ));
                    }
                    self.cursor += 1; // consume ':'
                    let value = self.parse()?;
                    map.insert(key, value);

                    self.skip_whitespace_and_comments()?;
                    match self.peek() {
                        Some(',') => {
                            self.cursor += 1; // consume ','
                            // Trailing comma check: loop will check for '}'
                        }
                        Some('}') => {
                            self.cursor += 1; // consume '}'
                            self.depth -= 1;
                            return Ok(Node::Object(map));
                        }
                        Some(c) => {
                            return Err(format!(
                                "Expected ',' or '}}' in JSON5 object, found '{}'",
                                c
                            ));
                        }
                        None => return Err("Unexpected EOF in JSON5 object".to_string()),
                    }
                }
                None => return Err("Unexpected EOF in JSON5 object".to_string()),
            }
        }
    }

    fn parse_key(&mut self) -> Result<String, String> {
        self.skip_whitespace_and_comments()?;
        match self.peek() {
            Some('"') | Some('\'') => {
                let node = self.parse_string()?;
                if let Node::Str(s) = node {
                    Ok(s)
                } else {
                    unreachable!()
                }
            }
            Some(c) if is_identifier_start(c) => {
                let mut key = String::new();
                while let Some(ch) = self.peek() {
                    if is_identifier_part(ch) {
                        key.push(ch);
                        self.cursor += 1;
                    } else {
                        break;
                    }
                }
                Ok(key)
            }
            Some(c) => Err(format!(
                "Invalid character '{}' starting JSON5 object key",
                c
            )),
            None => Err("Unexpected EOF while parsing JSON5 key".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<Node, String> {
        if self.depth >= self.max_depth {
            return Err("JSON5 maximum recursion depth exceeded".to_string());
        }
        self.depth += 1;
        self.cursor += 1; // consume '['

        let mut items = Vec::new();

        loop {
            self.skip_whitespace_and_comments()?;
            match self.peek() {
                Some(']') => {
                    self.cursor += 1; // consume ']'
                    self.depth -= 1;
                    return Ok(Node::Array(items));
                }
                Some(_) => {
                    let item = self.parse()?;
                    items.push(item);

                    self.skip_whitespace_and_comments()?;
                    match self.peek() {
                        Some(',') => {
                            self.cursor += 1; // consume ','
                            // Trailing comma allowed; next iteration checks for ']'
                        }
                        Some(']') => {
                            self.cursor += 1; // consume ']'
                            self.depth -= 1;
                            return Ok(Node::Array(items));
                        }
                        Some(c) => {
                            return Err(format!(
                                "Expected ',' or ']' in JSON5 array, found '{}'",
                                c
                            ));
                        }
                        None => return Err("Unexpected EOF in JSON5 array".to_string()),
                    }
                }
                None => return Err("Unexpected EOF in JSON5 array".to_string()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<Node, String> {
        let quote = self
            .next_char()
            .ok_or("Unexpected EOF reading string quote")?;
        if quote != '"' && quote != '\'' {
            return Err(format!("Expected quote (' or \"), found '{}'", quote));
        }

        let mut s = String::new();

        while let Some(ch) = self.next_char() {
            if ch == quote {
                return Ok(Node::Str(s));
            } else if ch == '\\' {
                match self.next_char() {
                    Some('"') => s.push('"'),
                    Some('\'') => s.push('\''),
                    Some('\\') => s.push('\\'),
                    Some('/') => s.push('/'),
                    Some('b') => s.push('\u{0008}'),
                    Some('f') => s.push('\u{000C}'),
                    Some('n') => s.push('\n'),
                    Some('r') => s.push('\r'),
                    Some('t') => s.push('\t'),
                    Some('v') => s.push('\u{000B}'),
                    Some('0') => {
                        // In JSON5: \0 is null character only if not followed by a digit
                        if let Some(next) = self.peek() {
                            if next.is_ascii_digit() {
                                return Err(
                                    "Octal escape sequences are not allowed in JSON5".to_string()
                                );
                            }
                        }
                        s.push('\0');
                    }
                    Some('x') => {
                        // Hexadecimal escape: \xHH
                        let h1 = self.next_char().ok_or("Incomplete \\x escape sequence")?;
                        let h2 = self.next_char().ok_or("Incomplete \\x escape sequence")?;
                        let val = u8::from_str_radix(&format!("{}{}", h1, h2), 16)
                            .map_err(|_| "Invalid \\x hexadecimal escape".to_string())?;
                        s.push(val as char);
                    }
                    Some('u') => {
                        // Unicode escape: \uHHHH
                        let mut hex_str = String::with_capacity(4);
                        for _ in 0..4 {
                            hex_str.push(self.next_char().ok_or("Incomplete \\u escape sequence")?);
                        }
                        let code = u32::from_str_radix(&hex_str, 16)
                            .map_err(|_| "Invalid \\u unicode escape".to_string())?;
                        let decoded = char::from_u32(code).ok_or_else(|| {
                            format!("Invalid unicode scalar value U+{:04X}", code)
                        })?;
                        s.push(decoded);
                    }
                    Some('\n') => {
                        // Escaped newline: line continuation, do not emit character
                    }
                    Some('\r') => {
                        // Escaped CRLF line continuation
                        if self.peek() == Some('\n') {
                            self.next_char();
                        }
                    }
                    Some(c) => {
                        // Escaped normal character in JSON5 (e.g. \a -> a)
                        s.push(c);
                    }
                    None => return Err("Unexpected EOF in string escape sequence".to_string()),
                }
            } else if ch == '\n' || ch == '\r' {
                return Err("Unescaped newline in JSON5 string literal".to_string());
            } else {
                s.push(ch);
            }
        }

        Err("Unterminated string in JSON5".to_string())
    }

    fn parse_boolean(&mut self) -> Result<Node, String> {
        if self.consume_str("true") {
            Ok(Node::Boolean(true))
        } else if self.consume_str("false") {
            Ok(Node::Boolean(false))
        } else {
            Err(format!("Expected boolean at byte {}", self.cursor))
        }
    }

    fn parse_null_or_nan(&mut self) -> Result<Node, String> {
        if self.consume_str("null") {
            Ok(Node::None)
        } else if self.consume_str("NaN") {
            Ok(Node::Number(Numeric::Float(core::f64::NAN)))
        } else {
            Err(format!("Unexpected token at byte {}", self.cursor))
        }
    }

    fn parse_number(&mut self) -> Result<Node, String> {
        // Collect number token characters
        let mut token = String::new();

        // Optional sign: + or -
        if let Some(c) = self.peek() {
            if c == '+' || c == '-' {
                token.push(c);
                self.cursor += 1;
            }
        }

        // Check for Infinity or NaN
        if self.consume_str("Infinity") {
            let is_neg = token.starts_with('-');
            let val = if is_neg {
                -core::f64::INFINITY
            } else {
                core::f64::INFINITY
            };
            return Ok(Node::Number(Numeric::Float(val)));
        }
        if self.consume_str("NaN") {
            return Ok(Node::Number(Numeric::Float(core::f64::NAN)));
        }

        // Check for Hexadecimal: 0x or 0X
        if self.peek() == Some('0')
            && (self.peek_next() == Some('x') || self.peek_next() == Some('X'))
        {
            self.cursor += 2; // consume 0x
            let mut hex_digits = String::new();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_hexdigit() {
                    hex_digits.push(ch);
                    self.cursor += 1;
                } else {
                    break;
                }
            }
            if hex_digits.is_empty() {
                return Err("Expected hexadecimal digits after '0x'".to_string());
            }
            let raw_int = i64::from_str_radix(&hex_digits, 16)
                .map_err(|_| "Hexadecimal integer out of 64-bit range".to_string())?;
            let signed_int = if token.starts_with('-') {
                -raw_int
            } else {
                raw_int
            };
            return Ok(Node::Number(Numeric::Integer(signed_int)));
        }

        // Decimal number (supports .5, 5., 1.2e3, etc.)
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = self.peek() {
            match ch {
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
                    if let Some(sign) = self.peek() {
                        if sign == '+' || sign == '-' {
                            token.push(sign);
                            self.cursor += 1;
                        }
                    }
                }
                _ => break,
            }
        }

        if token.is_empty() || token == "+" || token == "-" || token == "." {
            return Err("Invalid JSON5 number syntax".to_string());
        }

        // Strip leading '+' for rust standard float/int parsers
        let parse_target = if let Some(stripped) = token.strip_prefix('+') {
            stripped
        } else {
            &token
        };

        if has_dot || has_exp {
            let f = parse_target
                .parse::<f64>()
                .map_err(|_| format!("Failed to parse float: '{}'", token))?;
            Ok(Node::Number(Numeric::Float(f)))
        } else if let Ok(i) = parse_target.parse::<i64>() {
            Ok(Node::Number(Numeric::Integer(i)))
        } else if let Ok(u) = parse_target.parse::<u64>() {
            Ok(Node::Number(Numeric::UInteger(u)))
        } else if let Ok(f) = parse_target.parse::<f64>() {
            Ok(Node::Number(Numeric::Float(f)))
        } else {
            Err(format!("Number out of range: '{}'", token))
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let len = s.chars().count();
        if self.cursor + len <= self.chars.len() {
            let candidate: String = self.chars[self.cursor..self.cursor + len]
                .iter()
                .map(|&(_, c)| c)
                .collect();
            if candidate == s {
                self.cursor += len;
                return true;
            }
        }
        false
    }
}

#[inline]
fn is_json5_whitespace(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t' | '\r' | '\n' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}'
    )
}

#[inline]
fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '$' || c == '_'
}

#[inline]
fn is_identifier_part(c: char) -> bool {
    c.is_alphanumeric() || c == '$' || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_single_line_comments() {
        let input = r#"{
            "name": "Alice", // This is a comment
            "age": 30
        }"#;
        let output = strip_comments(input);
        assert!(!output.contains("// This is a comment"));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"age\""));
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let input = r#"{
            "name": "Alice",
            /* This is a
               multi-line comment */
            "age": 30
        }"#;
        let output = strip_comments(input);
        assert!(!output.contains("/* This is a"));
        assert!(!output.contains("multi-line comment */"));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"age\""));
    }

    #[test]
    fn test_unquoted_keys() {
        let input = r#"{
            service: "web",
            port: 8080,
            _internal_id: 1234,
            $ref: "schema",
        }"#;
        let node = parse_json5(input).expect("Failed to parse unquoted keys");
        assert_eq!(node["service"].as_str(), Some("web"));
        assert_eq!(node["port"].as_i64(), Some(8080));
        assert_eq!(node["_internal_id"].as_i64(), Some(1234));
        assert_eq!(node["$ref"].as_str(), Some("schema"));
    }

    #[test]
    fn test_single_quoted_strings() {
        let input = r#"{
            'title': 'Babbel JSON5',
            'desc': 'Single\'s quote test',
        }"#;
        let node = parse_json5(input).expect("Failed to parse single quoted strings");
        assert_eq!(node["title"].as_str(), Some("Babbel JSON5"));
        assert_eq!(node["desc"].as_str(), Some("Single's quote test"));
    }

    #[test]
    fn test_trailing_commas() {
        let input = r#"{
            items: [
                1,
                2,
                3,
            ],
            enabled: true,
        }"#;
        let node = parse_json5(input).expect("Failed to parse trailing commas");
        assert_eq!(node["items"].len(), Some(3));
        assert_eq!(node["enabled"].as_bool(), Some(true));
    }

    #[test]
    fn test_hexadecimal_numbers() {
        let input = r#"{
            positive_hex: 0xdecaf,
            uppercase_hex: 0XFF,
            negative_hex: -0x10,
        }"#;
        let node = parse_json5(input).expect("Failed to parse hex numbers");
        assert_eq!(node["positive_hex"].as_i64(), Some(0xdecaf));
        assert_eq!(node["uppercase_hex"].as_i64(), Some(255));
        assert_eq!(node["negative_hex"].as_i64(), Some(-16));
    }

    #[test]
    fn test_special_floats_and_signs() {
        let input = r#"{
            leading_dot: .5,
            trailing_dot: 5.,
            explicit_plus: +42,
            explicit_float: +3.14,
            inf: Infinity,
            neg_inf: -Infinity,
            nan: NaN,
        }"#;
        let node = parse_json5(input).expect("Failed to parse special floats");
        assert_eq!(node["leading_dot"].as_f64(), Some(0.5));
        assert_eq!(node["trailing_dot"].as_f64(), Some(5.0));
        assert_eq!(node["explicit_plus"].as_i64(), Some(42));
        assert_eq!(node["explicit_float"].as_f64(), Some(3.14));
        assert_eq!(node["inf"].as_f64(), Some(f64::INFINITY));
        assert_eq!(node["neg_inf"].as_f64(), Some(-f64::INFINITY));
        assert!(node["nan"].as_f64().map_or(false, |f| f.is_nan()));
    }

    #[test]
    fn test_escaped_newlines_in_strings() {
        let input = "{\n  message: 'Hello, \\\nWorld!'\n}";
        let node = parse_json5(input).expect("Failed to parse multi-line string");
        assert_eq!(node["message"].as_str(), Some("Hello, World!"));
    }

    #[test]
    fn test_comments_inside_and_outside() {
        let input = r#"
        // Configuration header
        {
            /* General server settings */
            host: '127.0.0.1', // loopback
            port: 3000, /* default port */
            endpoints: [
                '/v1', // api
                '/v2', /* beta */
            ],
        }
        // Footer note
        "#;
        let node = parse_json5(input).expect("Failed to parse fully commented JSON5");
        assert_eq!(node["host"].as_str(), Some("127.0.0.1"));
        assert_eq!(node["port"].as_i64(), Some(3000));
        assert_eq!(node["endpoints"].len(), Some(2));
    }
}
