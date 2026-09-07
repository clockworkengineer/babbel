//! # Zero-Allocation Streaming JSON Pull Parser (`JsonPullParser`)
//!
//! Designed specifically for resource-constrained microcontrollers, RTOS, and embedded systems.
//! Emits streaming SAX-style JSON tokens (`StartObject`, `EndObject`, `Key(&'a str)`, `Value(JsonScalar<'a>)`, `StartArray`, `EndArray`)
//! with zero dynamic heap allocation ($O(1)$ stack memory).

use babbel_core::ErrorCode;

/// Scalar value borrowed from the input stream with zero allocation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonScalar<'a> {
    /// JSON `null` literal
    Null,
    /// JSON boolean (`true` or `false`)
    Bool(bool),
    /// Unparsed numeric string slice (e.g. `"42"`, `"-3.14"`, `"1e6"`)
    Number(&'a str),
    /// Unescaped or raw string content slice without surrounding quotes
    String(&'a str),
}

impl<'a> JsonScalar<'a> {
    /// Attempts to parse the scalar as an `i64`.
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Self::Number(s) => s.parse::<i64>().ok(),
            _ => None,
        }
    }

    /// Attempts to parse the scalar as an `i32` (embedded integer).
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Self::Number(s) => s.parse::<i32>().ok(),
            _ => None,
        }
    }

    /// Attempts to parse the scalar as an `f64`.
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Number(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Returns the string slice if this scalar is a string.
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the boolean value if this scalar is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Streaming event emitted by [`JsonPullParser`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsonPullEvent<'a> {
    /// Opening curly brace `{`
    StartObject,
    /// Closing curly brace `}`
    EndObject,
    /// Object key string slice (without surrounding quotes)
    Key(&'a str),
    /// Scalar value (null, bool, number, string)
    Value(JsonScalar<'a>),
    /// Opening bracket `[`
    StartArray,
    /// Closing bracket `]`
    EndArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Container {
    Object { expecting_key: bool },
    Array { expecting_value: bool },
}

/// Zero-allocation streaming JSON pull parser.
///
/// Tracks up to 16 levels of nesting on the stack without any heap allocation.
pub struct JsonPullParser<'a> {
    input: &'a str,
    pos: usize,
    stack: [Option<Container>; 16],
    depth: usize,
}

impl<'a> JsonPullParser<'a> {
    /// Creates a new streaming pull parser over a borrowed JSON string.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            stack: [None; 16],
            depth: 0,
        }
    }

    /// Returns current byte position in input.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Advances the parser and returns the next [`JsonPullEvent`].
    /// Returns `Ok(None)` when EOF is reached.
    pub fn next_event(&mut self) -> Result<Option<JsonPullEvent<'a>>, ErrorCode> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            if self.depth > 0 {
                return Err(ErrorCode::UnexpectedEof);
            }
            return Ok(None);
        }

        let bytes = self.input.as_bytes();

        // Check if inside object
        if self.depth > 0 {
            if let Some(Container::Object { expecting_key }) = self.stack[self.depth - 1] {
                if expecting_key {
                    if bytes[self.pos] == b'}' {
                        self.pos += 1;
                        self.depth -= 1;
                        self.on_value_consumed();
                        return Ok(Some(JsonPullEvent::EndObject));
                    }
                    if bytes[self.pos] == b'"' {
                        let key = self.parse_string_slice()?;
                        self.skip_whitespace();
                        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b':' {
                            return Err(ErrorCode::SyntaxError);
                        }
                        self.pos += 1; // skip ':'
                        self.stack[self.depth - 1] = Some(Container::Object { expecting_key: false });
                        return Ok(Some(JsonPullEvent::Key(key)));
                    } else {
                        return Err(ErrorCode::SyntaxError);
                    }
                }
            }
        }

        let ch = bytes[self.pos];
        match ch {
            b'{' => {
                self.pos += 1;
                self.push_container(Container::Object { expecting_key: true })?;
                Ok(Some(JsonPullEvent::StartObject))
            }
            b'}' => {
                self.pos += 1;
                if self.depth == 0 {
                    return Err(ErrorCode::SyntaxError);
                }
                self.depth -= 1;
                self.on_value_consumed();
                Ok(Some(JsonPullEvent::EndObject))
            }
            b'[' => {
                self.pos += 1;
                self.push_container(Container::Array { expecting_value: true })?;
                Ok(Some(JsonPullEvent::StartArray))
            }
            b']' => {
                self.pos += 1;
                if self.depth == 0 {
                    return Err(ErrorCode::SyntaxError);
                }
                self.depth -= 1;
                self.on_value_consumed();
                Ok(Some(JsonPullEvent::EndArray))
            }
            b'"' => {
                let s = self.parse_string_slice()?;
                self.on_value_consumed();
                Ok(Some(JsonPullEvent::Value(JsonScalar::String(s))))
            }
            b't' => {
                if self.consume_literal("true") {
                    self.on_value_consumed();
                    Ok(Some(JsonPullEvent::Value(JsonScalar::Bool(true))))
                } else {
                    Err(ErrorCode::SyntaxError)
                }
            }
            b'f' => {
                if self.consume_literal("false") {
                    self.on_value_consumed();
                    Ok(Some(JsonPullEvent::Value(JsonScalar::Bool(false))))
                } else {
                    Err(ErrorCode::SyntaxError)
                }
            }
            b'n' => {
                if self.consume_literal("null") {
                    self.on_value_consumed();
                    Ok(Some(JsonPullEvent::Value(JsonScalar::Null)))
                } else {
                    Err(ErrorCode::SyntaxError)
                }
            }
            b'-' | b'0'..=b'9' => {
                let num_str = self.parse_number_slice()?;
                self.on_value_consumed();
                Ok(Some(JsonPullEvent::Value(JsonScalar::Number(num_str))))
            }
            _ => Err(ErrorCode::SyntaxError),
        }
    }

    fn push_container(&mut self, c: Container) -> Result<(), ErrorCode> {
        if self.depth >= self.stack.len() {
            return Err(ErrorCode::SyntaxError); // Max nesting depth exceeded
        }
        self.stack[self.depth] = Some(c);
        self.depth += 1;
        Ok(())
    }

    fn on_value_consumed(&mut self) {
        if self.depth > 0 {
            match self.stack[self.depth - 1] {
                Some(Container::Object { .. }) => {
                    self.consume_comma_or_end();
                    self.stack[self.depth - 1] = Some(Container::Object { expecting_key: true });
                }
                Some(Container::Array { .. }) => {
                    self.consume_comma_or_end();
                }
                None => {}
            }
        }
    }

    fn consume_comma_or_end(&mut self) {
        self.skip_whitespace();
        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b',' {
            self.pos += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() {
            match bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn consume_literal(&mut self, lit: &str) -> bool {
        let b = lit.as_bytes();
        if self.pos + b.len() <= self.input.len() && &self.input.as_bytes()[self.pos..self.pos + b.len()] == b {
            self.pos += b.len();
            true
        } else {
            false
        }
    }

    fn parse_string_slice(&mut self) -> Result<&'a str, ErrorCode> {
        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b'"' {
            return Err(ErrorCode::SyntaxError);
        }
        self.pos += 1;
        let start = self.pos;
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() {
            if bytes[self.pos] == b'\\' {
                self.pos += 2;
                continue;
            }
            if bytes[self.pos] == b'"' {
                let slice = &self.input[start..self.pos];
                self.pos += 1; // skip closing quote
                return Ok(slice);
            }
            self.pos += 1;
        }
        Err(ErrorCode::UnexpectedEof)
    }

    fn parse_number_slice(&mut self) -> Result<&'a str, ErrorCode> {
        let start = self.pos;
        let bytes = self.input.as_bytes();
        if self.pos < bytes.len() && bytes[self.pos] == b'-' {
            self.pos += 1;
        }
        while self.pos < bytes.len() {
            match bytes[self.pos] {
                b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-' => self.pos += 1,
                _ => break,
            }
        }
        if self.pos == start {
            return Err(ErrorCode::SyntaxError);
        }
        Ok(&self.input[start..self.pos])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_pull_parser_basic() {
        let json = r#"{"name":"sensor_1","val":42,"active":true,"tags":["iot","temp"]}"#;
        let mut parser = JsonPullParser::new(json);

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::StartObject)));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("name"))));
        assert_eq!(
            parser.next_event(),
            Ok(Some(JsonPullEvent::Value(JsonScalar::String("sensor_1"))))
        );

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("val"))));
        let num_event = parser.next_event().unwrap().unwrap();
        match num_event {
            JsonPullEvent::Value(s) => {
                assert_eq!(s.to_i64(), Some(42));
                assert_eq!(s.to_i32(), Some(42));
            }
            _ => panic!("expected number"),
        }

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("active"))));
        assert_eq!(
            parser.next_event(),
            Ok(Some(JsonPullEvent::Value(JsonScalar::Bool(true))))
        );

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("tags"))));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::StartArray)));
        assert_eq!(
            parser.next_event(),
            Ok(Some(JsonPullEvent::Value(JsonScalar::String("iot"))))
        );
        assert_eq!(
            parser.next_event(),
            Ok(Some(JsonPullEvent::Value(JsonScalar::String("temp"))))
        );
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::EndArray)));

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::EndObject)));
        assert_eq!(parser.next_event(), Ok(None));
    }

    #[test]
    fn test_json_pull_parser_empty_structures() {
        let json = r#"{"empty_arr":[],"empty_obj":{}}"#;
        let mut parser = JsonPullParser::new(json);

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::StartObject)));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("empty_arr"))));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::StartArray)));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::EndArray)));

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::Key("empty_obj"))));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::StartObject)));
        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::EndObject)));

        assert_eq!(parser.next_event(), Ok(Some(JsonPullEvent::EndObject)));
        assert_eq!(parser.next_event(), Ok(None));
    }
}
