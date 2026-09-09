//! TOML Document parser constructing an AST `Node::Table`.

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::error::TomlError;
use crate::nodes::Node;
use super::lexer::Lexer;
use super::tokens::{SpannedToken, Token};

/// Parser for TOML documents.
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    root: Node,
    // Tracks current table path for subsequent key-value pairs
    current_table_path: Vec<String>,
    // Tracks whether current target is in an array of tables
    in_array_of_tables: bool,
    explicit_tables: Vec<Vec<String>>,
    array_tables: Vec<Vec<String>>,
    static_keys: Vec<(Vec<String>, String)>,
}

impl Parser {
    /// Create a new parser from a raw TOML string.
    pub fn new(input: &str) -> Result<Self, TomlError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self {
            tokens,
            pos: 0,
            root: Node::new_table(),
            current_table_path: Vec::new(),
            in_array_of_tables: false,
            explicit_tables: Vec::new(),
            array_tables: Vec::new(),
            static_keys: Vec::new(),
        })
    }

    fn peek(&self) -> &SpannedToken {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn advance(&mut self) -> &SpannedToken {
        let p = self.pos;
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        &self.tokens[p]
    }

    fn expect_line_end(&mut self) -> Result<(), TomlError> {
        match self.peek().token {
            Token::Newline | Token::Eof => {
                if self.peek().token == Token::Newline {
                    self.advance();
                }
                Ok(())
            }
            _ => {
                let tok = self.peek();
                Err(TomlError::syntax(
                    format!("Expected newline or EOF after statement, found {:?}", tok.token),
                    tok.line,
                    tok.column,
                    tok.position,
                ))
            }
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek().token == Token::Newline {
            self.advance();
        }
    }

    /// Parse the document into a root `Node::Table`.
    pub fn parse(&mut self) -> Result<Node, TomlError> {
        self.skip_newlines();

        while self.peek().token != Token::Eof {
            match self.peek().token {
                Token::Newline => {
                    self.advance();
                }
                Token::LBracket => {
                    let first = self.advance().clone();
                    if self.peek().token == Token::LBracket {
                        let second = self.advance().clone();
                        if second.position != first.position + 1 {
                            return Err(TomlError::syntax(
                                "Whitespace is not permitted between opening brackets in array of tables header",
                                second.line,
                                second.column,
                                second.position,
                            ));
                        }
                        self.parse_array_of_tables_header()?;
                    } else {
                        self.parse_table_header()?;
                    }
                }
                _ => {
                    self.parse_key_value_pair()?;
                }
            }
            self.skip_newlines();
        }

        Ok(self.root.clone())
    }

    fn parse_table_header(&mut self) -> Result<(), TomlError> {
        let path = self.parse_key_path()?;

        let tok = self.advance().clone();
        if tok.token != Token::RBracket {
            return Err(TomlError::syntax(
                "Expected ']' closing table header",
                tok.line,
                tok.column,
                tok.position,
            ));
        }

        // Must be followed by newline or EOF
        self.expect_line_end()?;

        if self.explicit_tables.contains(&path) {
            return Err(TomlError::syntax(
                format!("Table '{:?}' is defined more than once", path),
                tok.line,
                tok.column,
                tok.position,
            ));
        }
        if self.array_tables.contains(&path) {
            return Err(TomlError::syntax(
                format!("Table '{:?}' conflicts with existing array of tables", path),
                tok.line,
                tok.column,
                tok.position,
            ));
        }
        if !path.is_empty() {
            let parent = &path[..path.len() - 1];
            let key = &path[path.len() - 1];
            if self.static_keys.iter().any(|(p, k)| p == parent && k == key) {
                return Err(TomlError::syntax(
                    format!("Table '{:?}' conflicts with statically defined key", path),
                    tok.line,
                    tok.column,
                    tok.position,
                ));
            }
        }

        // Ensure table path exists in root
        ensure_table_exists(&mut self.root, &path)?;

        self.explicit_tables.push(path.clone());
        self.current_table_path = path;
        self.in_array_of_tables = false;
        Ok(())
    }

    fn parse_array_of_tables_header(&mut self) -> Result<(), TomlError> {
        let path = self.parse_key_path()?;

        let tok1 = self.advance().clone();
        let tok2 = self.advance().clone();
        if tok1.token != Token::RBracket || tok2.token != Token::RBracket {
            return Err(TomlError::syntax(
                "Expected ']]' closing array of tables header",
                tok1.line,
                tok1.column,
                tok1.position,
            ));
        }
        if tok2.position != tok1.position + 1 {
            return Err(TomlError::syntax(
                "Whitespace is not permitted between closing brackets in array of tables header",
                tok2.line,
                tok2.column,
                tok2.position,
            ));
        }

        self.expect_line_end()?;

        if self.explicit_tables.contains(&path) {
            return Err(TomlError::syntax(
                format!("Array of tables '{:?}' conflicts with existing table", path),
                tok1.line,
                tok1.column,
                tok1.position,
            ));
        }
        if !path.is_empty() {
            let parent = &path[..path.len() - 1];
            let key = &path[path.len() - 1];
            if self.static_keys.iter().any(|(p, k)| p == parent && k == key) {
                return Err(TomlError::syntax(
                    format!("Array of tables '{:?}' conflicts with statically defined key", path),
                    tok1.line,
                    tok1.column,
                    tok1.position,
                ));
            }
        }

        // Append new table to array of tables
        append_array_table(&mut self.root, &path)?;
        if !self.array_tables.contains(&path) {
            self.array_tables.push(path.clone());
        }

        // Reset descendant explicit tables and static keys for the new array-of-tables element
        self.explicit_tables.retain(|t| !t.starts_with(&path));
        self.static_keys.retain(|(p, _)| !p.starts_with(&path));

        self.current_table_path = path;
        self.in_array_of_tables = true;
        Ok(())
    }

    fn parse_key_path(&mut self) -> Result<Vec<String>, TomlError> {
        let mut path = Vec::new();

        loop {
            let key = self.parse_single_key()?;
            path.push(key);

            if self.peek().token == Token::Period {
                self.advance(); // eat '.'
            } else {
                break;
            }
        }

        if path.is_empty() {
            let tok = self.peek();
            return Err(TomlError::syntax(
                "Table or key path cannot be empty",
                tok.line,
                tok.column,
                tok.position,
            ));
        }

        Ok(path)
    }

    fn parse_single_key(&mut self) -> Result<String, TomlError> {
        let tok = self.advance();
        match &tok.token {
            Token::Key(k) => Ok(k.clone()),
            Token::String(s) => Ok(s.clone()),
            Token::Integer(i) => Ok(i.to_string()),
            Token::Boolean(b) => Ok(if *b { "true".to_string() } else { "false".to_string() }),
            _ => Err(TomlError::syntax(
                format!("Expected key, found {:?}", tok.token),
                tok.line,
                tok.column,
                tok.position,
            )),
        }
    }

    fn parse_key_value_pair(&mut self) -> Result<(), TomlError> {
        let key_path = self.parse_key_path()?;

        let eq_tok = self.advance();
        if eq_tok.token != Token::Equals {
            return Err(TomlError::syntax(
                "Expected '=' after key",
                eq_tok.line,
                eq_tok.column,
                eq_tok.position,
            ));
        }

        let value = self.parse_value()?;
        self.expect_line_end()?;

        // Insert into current table scope
        insert_into_scope(
            &mut self.root,
            &self.current_table_path,
            self.in_array_of_tables,
            &key_path,
            value,
        )?;

        self.static_keys.push((self.current_table_path.clone(), key_path[0].clone()));

        Ok(())
    }



    fn parse_value(&mut self) -> Result<Node, TomlError> {
        let tok = self.peek().clone();
        match tok.token {
            Token::String(s) => {
                self.advance();
                Ok(Node::String(s))
            }
            Token::Integer(i) => {
                self.advance();
                Ok(Node::Integer(i))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Node::Float(f))
            }
            Token::Boolean(b) => {
                self.advance();
                Ok(Node::Boolean(b))
            }
            Token::Datetime(dt) => {
                self.advance();
                Ok(Node::Datetime(dt))
            }
            Token::LBracket => self.parse_array(),
            Token::LBrace => self.parse_inline_table(),
            _ => Err(TomlError::syntax(
                format!("Unexpected token for value: {:?}", tok.token),
                tok.line,
                tok.column,
                tok.position,
            )),
        }
    }

    fn parse_array(&mut self) -> Result<Node, TomlError> {
        // Eat '['
        self.advance();
        let mut items = Vec::new();

        loop {
            self.skip_newlines();
            if self.peek().token == Token::RBracket {
                self.advance(); // eat ']'
                break;
            }

            let val = self.parse_value()?;
            items.push(val);

            self.skip_newlines();
            if self.peek().token == Token::Comma {
                self.advance(); // eat ','
                self.skip_newlines();
            } else if self.peek().token == Token::RBracket {
                self.advance();
                break;
            } else {
                let tok = self.peek();
                return Err(TomlError::syntax(
                    "Expected ',' or ']' inside array",
                    tok.line,
                    tok.column,
                    tok.position,
                ));
            }
        }

        Ok(Node::Array(items))
    }

    fn parse_inline_table(&mut self) -> Result<Node, TomlError> {
        // Eat '{'
        self.advance();
        let mut entries = Vec::new();

        loop {
            self.skip_newlines();
            if self.peek().token == Token::RBrace {
                self.advance();
                break;
            }

            let key_path = self.parse_key_path()?;
            self.skip_newlines();
            let eq = self.advance().clone();
            if eq.token != Token::Equals {
                return Err(TomlError::syntax(
                    "Expected '=' after key in inline table",
                    eq.line,
                    eq.column,
                    eq.position,
                ));
            }
            self.skip_newlines();

            let val = self.parse_value()?;
            // Simple inline insertion
            if key_path.len() == 1 {
                let key = &key_path[0];
                if entries.iter().any(|(k, _)| k == key) {
                    return Err(TomlError::syntax(
                        format!("Duplicate key '{}' in inline table", key),
                        eq.line,
                        eq.column,
                        eq.position,
                    ));
                }
                entries.push((key.clone(), val));
            } else {
                // Nested dotted key inside inline table
                insert_nested_entry(&mut entries, &key_path, val)?;
            }

            self.skip_newlines();
            if self.peek().token == Token::Comma {
                self.advance();
                self.skip_newlines();
                if self.peek().token == Token::RBrace {
                    self.advance();
                    break;
                }
            } else if self.peek().token == Token::RBrace {
                self.advance();
                break;
            } else {
                let tok = self.peek();
                return Err(TomlError::syntax(
                    "Expected ',' or '}' in inline table",
                    tok.line,
                    tok.column,
                    tok.position,
                ));
            }
        }

        Ok(Node::Table(entries))
    }
}

fn insert_nested_entry(
    entries: &mut Vec<(String, Node)>,
    path: &[String],
    val: Node,
) -> Result<(), TomlError> {
    if path.is_empty() {
        return Ok(());
    }
    if path.len() == 1 {
        let key = &path[0];
        if entries.iter().any(|(k, _)| k == key) {
            return Err(TomlError::custom(format!("Duplicate key '{}'", key)));
        }
        entries.push((key.clone(), val));
        return Ok(());
    }

    let head = &path[0];
    if let Some((_, existing)) = entries.iter_mut().find(|(k, _)| k == head) {
        if let Node::Table(sub) = existing {
            insert_nested_entry(sub, &path[1..], val)?;
        } else {
            return Err(TomlError::custom(format!(
                "Cannot insert dotted key into existing scalar '{}'",
                head
            )));
        }
    } else {
        let mut sub = Vec::new();
        insert_nested_entry(&mut sub, &path[1..], val)?;
        entries.push((head.clone(), Node::Table(sub)));
    }
    Ok(())
}

fn navigate_table_mut<'a>(
    node: &'a mut Node,
    path: &[String],
) -> Result<&'a mut Node, TomlError> {
    if path.is_empty() {
        return Ok(node);
    }
    let segment = &path[0];
    let rest = &path[1..];

    match node {
        Node::Table(entries) => {
            let next_node = if let Some(pos) = entries.iter().position(|(k, _)| k == segment) {
                entries[pos].1.get_nested_target_mut()
            } else {
                entries.push((segment.clone(), Node::new_table()));
                let len = entries.len();
                &mut entries[len - 1].1
            };
            navigate_table_mut(next_node, rest)
        }
        _ => Err(TomlError::custom(format!(
            "Expected table along path segment '{}'",
            segment
        ))),
    }
}

fn ensure_table_exists(root: &mut Node, path: &[String]) -> Result<(), TomlError> {
    navigate_table_mut(root, path).map(|_| ())
}

fn append_array_table(root: &mut Node, path: &[String]) -> Result<(), TomlError> {
    if path.is_empty() {
        return Ok(());
    }
    let parent_path = &path[..path.len() - 1];
    let last_segment = &path[path.len() - 1];

    let parent = navigate_table_mut(root, parent_path)?;
    match parent {
        Node::Table(entries) => {
            if let Some(pos) = entries.iter().position(|(k, _)| k == last_segment) {
                match &mut entries[pos].1 {
                    Node::Array(arr) => {
                        arr.push(Node::new_table());
                        Ok(())
                    }
                    _ => Err(TomlError::custom(format!(
                        "Array of tables '{}' conflicts with existing key",
                        last_segment
                    ))),
                }
            } else {
                entries.push((last_segment.clone(), Node::Array(vec![Node::new_table()])));
                Ok(())
            }
        }
        _ => Err(TomlError::custom("Parent is not a table")),
    }
}

fn insert_into_scope(
    root: &mut Node,
    table_path: &[String],
    in_array_of_tables: bool,
    key_path: &[String],
    value: Node,
) -> Result<(), TomlError> {
    let target_table = if table_path.is_empty() {
        root
    } else if in_array_of_tables {
        let parent_path = &table_path[..table_path.len() - 1];
        let last_segment = &table_path[table_path.len() - 1];
        let parent = navigate_table_mut(root, parent_path)?;
        match parent {
            Node::Table(entries) => {
                let pos = entries.iter().position(|(k, _)| k == last_segment).ok_or_else(|| {
                    TomlError::custom(format!("Array of tables '{}' not found", last_segment))
                })?;
                match &mut entries[pos].1 {
                    Node::Array(items) => items.last_mut().ok_or_else(|| {
                        TomlError::custom("Array of tables is empty")
                    })?,
                    _ => return Err(TomlError::custom("Expected array of tables")),
                }
            }
            _ => return Err(TomlError::custom("Target parent is not a table")),
        }
    } else {
        navigate_table_mut(root, table_path)?
    };

    match target_table {
        Node::Table(entries) => insert_nested_entry(entries, key_path, value),
        _ => Err(TomlError::custom("Target scope is not a table")),
    }
}
