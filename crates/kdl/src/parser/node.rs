#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::ast::{KdlEntry, KdlNode};
use crate::error::KdlError;
use super::{is_bare_ident_char, is_kdl_newline, is_kdl_whitespace, Parser};

impl<'a> Parser<'a> {
    pub(crate) fn parse_node(&mut self) -> Result<KdlNode, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        if self.peek() == Some('}') {
            return Err(KdlError::UnexpectedChar {
                ch: '}',
                line: self.line,
                col: self.col,
            });
        }

        // Optional type annotation: `(type)`
        let type_annotation = if self.peek() == Some('(') {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let had_space = self.skip_whitespace_and_comments(false)?;

        // Node name: identifier
        let name = self.parse_identifier()?;
        let mut entries = Vec::new();
        let mut children = Vec::new();
        let mut seen_any_child_block = false;
        let seen_active_children = false;
        let mut prev_space = had_space;

        loop {
            let had_space_before_token = self.skip_whitespace_and_comments(false)?;
            prev_space = prev_space || had_space_before_token;

            match self.peek() {
                None => break,
                Some(ch) if is_kdl_newline(ch) || ch == ';' => {
                    self.advance();
                    break;
                }
                Some('}') => {
                    // Terminating children block from parent
                    break;
                }
                Some('{') => {
                    if seen_active_children {
                        return Err(KdlError::Syntax {
                            message: "Node cannot have multiple children blocks".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    self.advance(); // consume {
                    children = self.parse_children()?;
                    let _ = seen_active_children;
                    let _ = seen_any_child_block;

                    // After children block `}`, only slashdashed child block, whitespace, newline, or ; allowed
                    let _ = self.skip_whitespace_and_comments(false)?;
                    while self.peek() == Some('/') && self.peek_at(1) == Some('-') {
                        let mut lookahead = 2;
                        while let Some(c) = self.peek_at(lookahead) {
                            if is_kdl_whitespace(c) || is_kdl_newline(c) {
                                lookahead += 1;
                            } else {
                                break;
                            }
                        }
                        if self.peek_at(lookahead) == Some('{') {
                            // Discard slashdash children
                            self.advance(); // /
                            self.advance(); // -
                            self.skip_whitespace_and_comments(true)?;
                            self.advance(); // {
                            let _ = self.parse_children()?;
                            let _ = self.skip_whitespace_and_comments(false)?;
                        } else {
                            break;
                        }
                    }

                    // Must be followed by newline, EOF, or ; or }
                    let _ = self.skip_whitespace_and_comments(false)?;
                    match self.peek() {
                        None => break,
                        Some(c) if is_kdl_newline(c) || c == ';' => {
                            self.advance();
                            break;
                        }
                        Some('}') => break,
                        Some(other) => {
                            return Err(KdlError::Syntax {
                                message: format!("Expected newline or ';' after children block, found '{}'", other),
                                line: self.line,
                                col: self.col,
                            });
                        }
                    }
                }
                Some('/') if self.peek_at(1) == Some('-') => {
                    self.advance(); // /
                    self.advance(); // -
                    self.skip_whitespace_and_comments(true)?;
                    if self.peek() == Some('{') {
                        // Discard slashdashed children block
                        self.advance(); // {
                        let _ = self.parse_children()?;
                        seen_any_child_block = true;
                    } else {
                        if seen_any_child_block {
                            return Err(KdlError::Syntax {
                                message: "Cannot place entry after children block".to_string(),
                                line: self.line,
                                col: self.col,
                            });
                        }
                        // Discard entry
                        let _ = self.parse_entry()?;
                    }
                    prev_space = true;
                    continue;
                }
                _ => {
                    if seen_any_child_block {
                        return Err(KdlError::Syntax {
                            message: "Cannot place entry after children block".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    if !prev_space {
                        return Err(KdlError::Syntax {
                            message: "Whitespace required between tokens".to_string(),
                            line: self.line,
                            col: self.col,
                        });
                    }
                    let entry = self.parse_entry()?;
                    entries.push(entry);
                    prev_space = false;
                }
            }
        }

        Ok(KdlNode {
            type_annotation,
            name,
            entries,
            children,
        })
    }

    fn parse_children(&mut self) -> Result<Vec<KdlNode>, KdlError> {
        let mut nodes = Vec::new();

        loop {
            self.skip_whitespace_and_comments(true)?;
            if self.is_eof() {
                return Err(KdlError::UnexpectedEof);
            }

            if self.peek() == Some('}') {
                self.advance(); // consume }
                break;
            }

            // Check for slashdash comment `/-`
            if self.peek() == Some('/') && self.peek_at(1) == Some('-') {
                self.advance(); // /
                self.advance(); // -
                self.skip_whitespace_and_comments(true)?;
                let _ = self.parse_node()?;
                continue;
            }

            let node = self.parse_node()?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    fn parse_entry(&mut self) -> Result<KdlEntry, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        // In KDL, type annotations on properties cannot appear before property key:
        // `(type)prop=val` is ILLEGAL!
        let has_type = self.peek() == Some('(');
        let type_annotation = if has_type {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        self.skip_whitespace_and_comments(false)?;

        if self.is_property_start() {
            if has_type {
                return Err(KdlError::Syntax {
                    message: "Type annotation cannot precede property key; use key=(type)val".to_string(),
                    line: self.line,
                    col: self.col,
                });
            }

            let key = self.parse_identifier()?;
            self.skip_whitespace_and_comments(false)?;
            if self.peek() == Some('=') {
                self.advance(); // consume =
            } else {
                return Err(KdlError::Expected {
                    expected: "'=' in property",
                    found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                    line: self.line,
                    col: self.col,
                });
            }
            self.skip_whitespace_and_comments(false)?;

            let prop_type = if self.peek() == Some('(') {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            self.skip_whitespace_and_comments(false)?;

            let val = self.parse_value()?;
            Ok(KdlEntry::Prop(key, prop_type, val))
        } else {
            let val = self.parse_value()?;
            Ok(KdlEntry::Arg(type_annotation, val))
        }
    }

    fn is_property_start(&self) -> bool {
        let mut idx = self.pos;
        if idx >= self.chars.len() {
            return false;
        }

        let first = self.chars[idx];
        if first == '"' {
            idx += 1;
            while idx < self.chars.len() && self.chars[idx] != '"' {
                if self.chars[idx] == '\\' {
                    idx += 1;
                }
                idx += 1;
            }
            if idx < self.chars.len() && self.chars[idx] == '"' {
                idx += 1;
            }
        } else if first == '#' {
            // Raw string key: #..."...#...
            let mut hashes = 0;
            while idx < self.chars.len() && self.chars[idx] == '#' {
                hashes += 1;
                idx += 1;
            }
            if idx < self.chars.len() && self.chars[idx] == '"' {
                idx += 1;
                while idx < self.chars.len() {
                    if self.chars[idx] == '"' {
                        idx += 1;
                        let mut match_h = 0;
                        while idx < self.chars.len() && self.chars[idx] == '#' && match_h < hashes {
                            match_h += 1;
                            idx += 1;
                        }
                        if match_h == hashes {
                            break;
                        }
                    } else {
                        idx += 1;
                    }
                }
            } else {
                return false;
            }
        } else if is_bare_ident_char(first) {
            while idx < self.chars.len() && is_bare_ident_char(self.chars[idx]) {
                idx += 1;
            }
        } else {
            return false;
        }

        while idx < self.chars.len() && is_kdl_whitespace(self.chars[idx]) {
            idx += 1;
        }

        if idx < self.chars.len() && self.chars[idx] == '=' {
            if idx + 1 < self.chars.len() && self.chars[idx + 1] == '=' {
                return false;
            }
            return true;
        }

        false
    }

    fn parse_type_annotation(&mut self) -> Result<String, KdlError> {
        let start_line = self.line;
        let start_col = self.col;
        self.advance(); // consume (
        self.skip_whitespace_and_comments(false)?;

        // Disallow slashdash inside type
        if self.peek() == Some('/') && self.peek_at(1) == Some('-') {
            return Err(KdlError::Syntax {
                message: "Slashdash is not allowed inside type annotation".to_string(),
                line: self.line,
                col: self.col,
            });
        }

        if self.peek() == Some(')') {
            return Err(KdlError::Syntax {
                message: "Empty type annotation is not allowed".to_string(),
                line: start_line,
                col: start_col,
            });
        }

        let ty = self.parse_identifier()?;
        self.skip_whitespace_and_comments(false)?;

        if self.peek() == Some(')') {
            self.advance(); // consume )
            Ok(ty)
        } else {
            Err(KdlError::Expected {
                expected: "')' to close type annotation",
                found: self.peek().map(|c| c.to_string()).unwrap_or_default(),
                line: start_line,
                col: start_col,
            })
        }
    }

    pub(crate) fn parse_identifier(&mut self) -> Result<String, KdlError> {
        self.skip_whitespace_and_comments(false)?;

        match self.peek() {
            Some('"') => self.parse_quoted_string(),
            Some('#') => self.parse_raw_string(),
            Some(ch) if is_bare_ident_char(ch) => {
                let start_line = self.line;
                let start_col = self.col;
                let mut ident = String::new();
                while let Some(c) = self.peek() {
                    if is_bare_ident_char(c) {
                        ident.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }

                // Check identifier validity
                let first = ident.chars().next().unwrap();
                if first.is_ascii_digit() {
                    return Err(KdlError::Syntax {
                        message: "Bare identifier cannot start with a digit".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                if (first == '+' || first == '-') && ident.len() > 1 {
                    let second = ident.chars().nth(1).unwrap();
                    if second.is_ascii_digit() {
                        return Err(KdlError::Syntax {
                            message: "Bare identifier cannot start with sign followed by digit".to_string(),
                            line: start_line,
                            col: start_col,
                        });
                    }
                }
                if first == '.' && ident.len() > 1 && ident.chars().nth(1).unwrap().is_ascii_digit() {
                    return Err(KdlError::Syntax {
                        message: "Bare identifier cannot start with dot followed by digit".to_string(),
                        line: start_line,
                        col: start_col,
                    });
                }
                if matches!(
                    ident.as_str(),
                    "true" | "false" | "null" | "inf" | "-inf" | "nan"
                ) {
                    return Err(KdlError::Syntax {
                        message: format!("Reserved keyword '{}' cannot be used as bare identifier", ident),
                        line: start_line,
                        col: start_col,
                    });
                }

                Ok(ident)
            }
            Some(ch) => Err(KdlError::UnexpectedChar {
                ch,
                line: self.line,
                col: self.col,
            }),
            None => Err(KdlError::UnexpectedEof),
        }
    }

}

