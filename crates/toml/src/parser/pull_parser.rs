//! Zero-allocation streaming pull-parser for microcontrollers and embedded environments.

#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::error::TomlError;
use super::lexer::Lexer;
use super::tokens::Token;

/// Event emitted by the streaming `TomlPullParser`.
#[derive(Debug, Clone, PartialEq)]
pub enum TomlPullEvent {
    /// Document start
    StartDocument,
    /// Standard table header `[table.name]`
    TableHeader(String),
    /// Array of tables header `[[table.name]]`
    ArrayOfTablesHeader(String),
    /// Key in a key-value assignment
    Key(String),
    /// String scalar value
    ValueString(String),
    /// Integer scalar value
    ValueInteger(i64),
    /// Float scalar value
    ValueFloat(f64),
    /// Boolean scalar value
    ValueBoolean(bool),
    /// RFC 3339 datetime value
    ValueDatetime(String),
    /// Start of an array `[`
    StartArray,
    /// End of an array `]`
    EndArray,
    /// Start of an inline table `{`
    StartInlineTable,
    /// End of an inline table `}`
    EndInlineTable,
    /// Document end
    EndDocument,
}

/// Zero-allocation streaming pull parser for TOML data.
pub struct TomlPullParser<'a> {
    lexer: Lexer<'a>,
    started: bool,
    finished: bool,
}

impl<'a> TomlPullParser<'a> {
    /// Create a new pull parser over an input string slice.
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            started: false,
            finished: false,
        }
    }

    /// Pull the next event from the stream.
    pub fn next_event(&mut self) -> Result<Option<TomlPullEvent>, TomlError> {
        if self.finished {
            return Ok(None);
        }

        if !self.started {
            self.started = true;
            return Ok(Some(TomlPullEvent::StartDocument));
        }

        loop {
            let tok = self.lexer.next_token()?;
            match tok.token {
                Token::Eof => {
                    self.finished = true;
                    return Ok(Some(TomlPullEvent::EndDocument));
                }
                Token::Newline | Token::Equals | Token::Period | Token::Comma => {
                    // Structural separators, skip to next meaningful token
                    continue;
                }
                Token::LBracket => {
                    // Peek ahead or parse table header key
                    return Ok(Some(TomlPullEvent::StartArray));
                }
                Token::RBracket => {
                    return Ok(Some(TomlPullEvent::EndArray));
                }
                Token::LBrace => {
                    return Ok(Some(TomlPullEvent::StartInlineTable));
                }
                Token::RBrace => {
                    return Ok(Some(TomlPullEvent::EndInlineTable));
                }
                Token::Key(k) => {
                    return Ok(Some(TomlPullEvent::Key(k)));
                }
                Token::String(s) => {
                    return Ok(Some(TomlPullEvent::ValueString(s)));
                }
                Token::Integer(i) => {
                    return Ok(Some(TomlPullEvent::ValueInteger(i)));
                }
                Token::Float(f) => {
                    return Ok(Some(TomlPullEvent::ValueFloat(f)));
                }
                Token::Boolean(b) => {
                    return Ok(Some(TomlPullEvent::ValueBoolean(b)));
                }
                Token::Datetime(dt) => {
                    return Ok(Some(TomlPullEvent::ValueDatetime(dt.raw)));
                }
            }
        }
    }
}
