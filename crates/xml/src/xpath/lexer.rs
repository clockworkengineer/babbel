//! # XPath 1.0 Lexer / Tokenizer
//!
//! Tokenizes raw XPath expression strings into discrete tokens ([`Token`]).

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};
use crate::io::{is_xml_name_char, is_xml_name_start};

/// XPath 1.0 token variants.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// `/`
    Slash,
    /// `//`
    DoubleSlash,
    /// `@`
    At,
    /// `.`
    Dot,
    /// `..`
    DoubleDot,
    /// `*`
    Star,
    /// `::`
    ColonColon,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `|`
    Pipe,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `=`
    Eq,
    /// `!=`
    NotEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// Identifier or keyword name.
    Name(String),
    /// Literal string enclosed in quotes.
    LiteralString(String),
    /// Literal number.
    LiteralNumber(f64),
    /// End of File / input stream.
    Eof,
    /// Variable reference ($var).
    Variable(String),
}

/// Lexer scanning XPath string slice character by character without intermediate allocations.
#[derive(Debug, Clone)]
pub struct XPathLexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> XPathLexer<'a> {
    /// Instantiates a new [`XPathLexer`] for a given XPath input string slice.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Peeks at current character.
    pub fn peek(&self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            self.input[self.pos..].chars().next()
        }
    }

    /// Peeks character at `n` positions ahead.
    pub fn peek_nth(&self, n: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(n)
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            None
        } else {
            let ch = self.input[self.pos..].chars().next()?;
            self.pos += ch.len_utf8();
            Some(ch)
        }
    }

    fn skip_whitespace(&mut self) {
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Scans and returns the next [`Token`].
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::Eof),
        };

        match ch {
            '/' => {
                self.advance();
                if self.peek() == Some('/') {
                    self.advance();
                    Ok(Token::DoubleSlash)
                } else {
                    Ok(Token::Slash)
                }
            }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    Ok(Token::DoubleDot)
                } else if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    // Backtrack 1 byte for leading dot in number
                    self.pos -= 1;
                    self.read_number()
                } else {
                    Ok(Token::Dot)
                }
            }
            '@' => {
                self.advance();
                Ok(Token::At)
            }
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '|' => {
                self.advance();
                Ok(Token::Pipe)
            }
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => {
                self.advance();
                Ok(Token::Minus)
            }
            '=' => {
                self.advance();
                Ok(Token::Eq)
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::NotEq)
                } else {
                    Err(XmlError::XPathError("Expected '=' after '!'".into()))
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::LtEq)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::GtEq)
                } else {
                    Ok(Token::Gt)
                }
            }
            ':' => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    Ok(Token::ColonColon)
                } else {
                    Ok(Token::Name(":".into()))
                }
            }
            '$' => {
                self.advance();
                let var_name = match self.read_name()? {
                    Token::Name(n) => n,
                    _ => return Err(XmlError::XPathError("Expected variable name after '$'".into())),
                };
                Ok(Token::Variable(var_name))
            }
            '"' | '\'' => self.read_string(ch),
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if is_xml_name_start(ch) => self.read_name(),
            _ => Err(XmlError::XPathError(format!(
                "Unexpected character in XPath expression: '{ch}'"
            ))),
        }
    }

    fn read_string(&mut self, quote: char) -> Result<Token> {
        self.advance(); // consume quote
        let start_pos = self.pos;
        while let Some(ch) = self.peek() {
            if ch == quote {
                let s = self.input[start_pos..self.pos].to_string();
                self.advance(); // consume quote
                return Ok(Token::LiteralString(s));
            }
            self.advance();
        }
        Err(XmlError::XPathError("Unterminated string literal in XPath".into()))
    }

    fn read_number(&mut self) -> Result<Token> {
        let start_pos = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                self.advance();
            } else {
                break;
            }
        }
        let num_str = &self.input[start_pos..self.pos];
        num_str
            .parse::<f64>()
            .map(Token::LiteralNumber)
            .map_err(|_| XmlError::XPathError(format!("Invalid number literal '{num_str}'")))
    }

    fn read_name(&mut self) -> Result<Token> {
        let start_pos = self.pos;
        while let Some(ch) = self.peek() {
            if ch == ':' {
                if self.peek_nth(1) == Some(':') {
                    // Axis specifier '::', stop reading name
                    break;
                } else {
                    // Single ':' is namespace prefix separator in QName (e.g. soap:Envelope)
                    self.advance();
                }
            } else if is_xml_name_char(ch) {
                self.advance();
            } else {
                break;
            }
        }
        let name = self.input[start_pos..self.pos].to_string();
        Ok(Token::Name(name))
    }
}
