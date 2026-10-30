//! Zero-dependency regular expression engine for JSON Schema `pattern` and RFC 9535 JSONPath `match()` / `search()`.
//!
//! Supports standard ECMA-262 / I-Regexp syntax:
//! - Literals and escapes (`\d`, `\D`, `\w`, `\W`, `\s`, `\S`, `\n`, `\r`, `\t`, `\.`, etc.)
//! - Wildcard `.` (any character except `\n`)
//! - Character classes `[a-z0-9_]`, negated classes `[^...]`
//! - Anchors `^`, `$`, `\b`, `\B`
//! - Quantifiers `*`, `+`, `?`, `{m}`, `{m,}`, `{m,n}` (including non-greedy `*?`, `+?`, `??`)
//! - Grouping `(...)`, `(?:...)` and alternation `|`

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// A compiled regular expression AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regex {
    node: RegexNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegexNode {
    Empty,
    AnchorStart,
    AnchorEnd,
    WordBoundary {
        negated: bool,
    },
    Char(char),
    AnyChar,
    BuiltinClass {
        kind: BuiltinClassKind,
        negated: bool,
    },
    BracketClass {
        items: Vec<ClassItem>,
        negated: bool,
    },
    Seq(Vec<RegexNode>),
    Alt(Vec<RegexNode>),
    Repeat {
        inner: Box<RegexNode>,
        min: usize,
        max: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuiltinClassKind {
    Digit,
    Word,
    Space,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClassItem {
    Single(char),
    Range(char, char),
    Builtin {
        kind: BuiltinClassKind,
        negated: bool,
    },
}

impl BuiltinClassKind {
    #[inline]
    fn matches(self, c: char) -> bool {
        match self {
            Self::Digit => c.is_ascii_digit(),
            Self::Word => c.is_ascii_alphanumeric() || c == '_',
            Self::Space => c.is_whitespace(),
        }
    }
}

impl ClassItem {
    #[inline]
    fn matches(&self, c: char) -> bool {
        match self {
            Self::Single(ch) => *ch == c,
            Self::Range(start, end) => c >= *start && c <= *end,
            Self::Builtin { kind, negated } => {
                let m = kind.matches(c);
                if *negated { !m } else { m }
            }
        }
    }
}

const DEFAULT_STEP_BUDGET: usize = 100_000;

impl Regex {
    /// Compile a regular expression pattern string.
    pub fn compile(pattern: &str) -> Result<Self, String> {
        let mut parser = RegexParser::new(pattern);
        let node = parser.parse_alternation()?;
        if !parser.is_eof() {
            return Err("unexpected closing parenthesis in regex".to_string());
        }
        Ok(Self { node })
    }

    /// Returns `true` if the pattern matches anywhere within `text` (JSON Schema `pattern` & JSONPath `search()`).
    pub fn is_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let mut budget = DEFAULT_STEP_BUDGET;
        for start in 0..=chars.len() {
            let mut end_positions = Vec::new();
            self.eval_node(&self.node, &chars, start, &mut end_positions, &mut budget);
            if !end_positions.is_empty() {
                return true;
            }
            if budget == 0 {
                break;
            }
        }
        false
    }

    /// Returns `true` if the pattern matches the entire `text` from start to end (RFC 9535 `match()`).
    pub fn is_full_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let mut end_positions = Vec::new();
        let mut budget = DEFAULT_STEP_BUDGET;
        self.eval_node(&self.node, &chars, 0, &mut end_positions, &mut budget);
        end_positions.contains(&chars.len())
    }

    fn eval_node(
        &self,
        node: &RegexNode,
        chars: &[char],
        pos: usize,
        out: &mut Vec<usize>,
        budget: &mut usize,
    ) {
        if *budget == 0 {
            return;
        }
        *budget -= 1;

        match node {
            RegexNode::Empty => push_unique(out, pos),
            RegexNode::AnchorStart => {
                if pos == 0 {
                    push_unique(out, pos);
                }
            }
            RegexNode::AnchorEnd => {
                if pos == chars.len() {
                    push_unique(out, pos);
                }
            }
            RegexNode::WordBoundary { negated } => {
                let prev_word = pos > 0 && BuiltinClassKind::Word.matches(chars[pos - 1]);
                let next_word = pos < chars.len() && BuiltinClassKind::Word.matches(chars[pos]);
                let is_boundary = prev_word != next_word;
                if is_boundary != *negated {
                    push_unique(out, pos);
                }
            }
            RegexNode::Char(expected) => {
                if pos < chars.len() && chars[pos] == *expected {
                    push_unique(out, pos + 1);
                }
            }
            RegexNode::AnyChar => {
                if pos < chars.len() && chars[pos] != '\n' && chars[pos] != '\r' {
                    push_unique(out, pos + 1);
                }
            }
            RegexNode::BuiltinClass { kind, negated } => {
                if pos < chars.len() {
                    let m = kind.matches(chars[pos]);
                    if m != *negated {
                        push_unique(out, pos + 1);
                    }
                }
            }
            RegexNode::BracketClass { items, negated } => {
                if pos < chars.len() {
                    let c = chars[pos];
                    let any_match = items.iter().any(|item| item.matches(c));
                    if any_match != *negated {
                        push_unique(out, pos + 1);
                    }
                }
            }
            RegexNode::Seq(nodes) => {
                let mut current_positions = alloc::vec![pos];
                for sub in nodes {
                    let mut next_positions = Vec::new();
                    for p in current_positions {
                        self.eval_node(sub, chars, p, &mut next_positions, budget);
                        if *budget == 0 {
                            return;
                        }
                    }
                    if next_positions.is_empty() {
                        return;
                    }
                    current_positions = next_positions;
                }
                for p in current_positions {
                    push_unique(out, p);
                }
            }
            RegexNode::Alt(branches) => {
                for branch in branches {
                    self.eval_node(branch, chars, pos, out, budget);
                    if *budget == 0 {
                        return;
                    }
                }
            }
            RegexNode::Repeat { inner, min, max } => {
                self.eval_repeat(inner, chars, pos, 0, *min, *max, out, budget);
            }
        }
    }

    fn eval_repeat(
        &self,
        inner: &RegexNode,
        chars: &[char],
        pos: usize,
        count: usize,
        min: usize,
        max: Option<usize>,
        out: &mut Vec<usize>,
        budget: &mut usize,
    ) {
        if *budget == 0 {
            return;
        }
        *budget -= 1;

        if count >= min {
            push_unique(out, pos);
        }
        if let Some(m) = max {
            if count >= m {
                return;
            }
        }

        let mut next_positions = Vec::new();
        self.eval_node(inner, chars, pos, &mut next_positions, budget);
        for next_pos in next_positions {
            // Guard against infinite zero-width loops on unbounded repetitions
            if next_pos == pos && count >= min {
                continue;
            }
            self.eval_repeat(inner, chars, next_pos, count + 1, min, max, out, budget);
            if *budget == 0 {
                return;
            }
        }
    }
}

#[inline]
fn push_unique(vec: &mut Vec<usize>, val: usize) {
    if !vec.contains(&val) {
        vec.push(val);
    }
}

struct RegexParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> RegexParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn parse_alternation(&mut self) -> Result<RegexNode, String> {
        let mut branches = Vec::new();
        branches.push(self.parse_sequence()?);

        while self.peek() == Some('|') {
            self.next_char();
            branches.push(self.parse_sequence()?);
        }

        if branches.len() == 1 {
            Ok(branches.remove(0))
        } else {
            Ok(RegexNode::Alt(branches))
        }
    }

    fn parse_sequence(&mut self) -> Result<RegexNode, String> {
        let mut nodes = Vec::new();

        while let Some(c) = self.peek() {
            if c == ')' || c == '|' {
                break;
            }
            let atom = self.parse_atom()?;
            let quantified = self.parse_quantifier(atom)?;
            nodes.push(quantified);
        }

        if nodes.is_empty() {
            Ok(RegexNode::Empty)
        } else if nodes.len() == 1 {
            Ok(nodes.remove(0))
        } else {
            Ok(RegexNode::Seq(nodes))
        }
    }

    fn parse_atom(&mut self) -> Result<RegexNode, String> {
        match self.next_char() {
            Some('^') => Ok(RegexNode::AnchorStart),
            Some('$') => Ok(RegexNode::AnchorEnd),
            Some('.') => Ok(RegexNode::AnyChar),
            Some('(') => {
                if self.input[self.pos..].starts_with("?:") {
                    self.pos += 2;
                }
                let inner = self.parse_alternation()?;
                if self.next_char() != Some(')') {
                    return Err("unclosed group in regex".to_string());
                }
                Ok(inner)
            }
            Some('[') => self.parse_bracket_class(),
            Some('\\') => self.parse_escape(),
            Some(c) => Ok(RegexNode::Char(c)),
            None => Ok(RegexNode::Empty),
        }
    }

    fn parse_escape(&mut self) -> Result<RegexNode, String> {
        match self.next_char() {
            Some('d') => Ok(RegexNode::BuiltinClass {
                kind: BuiltinClassKind::Digit,
                negated: false,
            }),
            Some('D') => Ok(RegexNode::BuiltinClass {
                kind: BuiltinClassKind::Digit,
                negated: true,
            }),
            Some('w') => Ok(RegexNode::BuiltinClass {
                kind: BuiltinClassKind::Word,
                negated: false,
            }),
            Some('W') => Ok(RegexNode::BuiltinClass {
                kind: BuiltinClassKind::Word,
                negated: true,
            }),
            Some('s') => Ok(RegexNode::BuiltinClass {
                kind: BuiltinClassKind::Space,
                negated: false,
            }),
            Some('S') => Ok(RegexNode::BuiltinClass {
                kind: BuiltinClassKind::Space,
                negated: true,
            }),
            Some('b') => Ok(RegexNode::WordBoundary { negated: false }),
            Some('B') => Ok(RegexNode::WordBoundary { negated: true }),
            Some('n') => Ok(RegexNode::Char('\n')),
            Some('r') => Ok(RegexNode::Char('\r')),
            Some('t') => Ok(RegexNode::Char('\t')),
            Some(c) => Ok(RegexNode::Char(c)),
            None => Err("trailing backslash in regex".to_string()),
        }
    }

    fn parse_bracket_class(&mut self) -> Result<RegexNode, String> {
        let mut negated = false;
        if self.peek() == Some('^') {
            self.next_char();
            negated = true;
        }

        let mut items = Vec::new();
        let mut first = true;

        while let Some(c) = self.peek() {
            if c == ']' && !first {
                self.next_char();
                return Ok(RegexNode::BracketClass { items, negated });
            }
            first = false;

            let item = self.parse_class_atom()?;
            if let ClassItem::Single(start_ch) = item {
                if self.peek() == Some('-') {
                    // Check if '-' is followed by something other than ']'
                    let saved = self.pos;
                    self.next_char(); // consume '-'
                    if self.peek().is_some() && self.peek() != Some(']') {
                        if let ClassItem::Single(end_ch) = self.parse_class_atom()? {
                            items.push(ClassItem::Range(start_ch, end_ch));
                            continue;
                        }
                    }
                    self.pos = saved;
                }
            }
            items.push(item);
        }

        Err("unclosed character class '[' in regex".to_string())
    }

    fn parse_class_atom(&mut self) -> Result<ClassItem, String> {
        match self.next_char() {
            Some('\\') => match self.next_char() {
                Some('d') => Ok(ClassItem::Builtin {
                    kind: BuiltinClassKind::Digit,
                    negated: false,
                }),
                Some('D') => Ok(ClassItem::Builtin {
                    kind: BuiltinClassKind::Digit,
                    negated: true,
                }),
                Some('w') => Ok(ClassItem::Builtin {
                    kind: BuiltinClassKind::Word,
                    negated: false,
                }),
                Some('W') => Ok(ClassItem::Builtin {
                    kind: BuiltinClassKind::Word,
                    negated: true,
                }),
                Some('s') => Ok(ClassItem::Builtin {
                    kind: BuiltinClassKind::Space,
                    negated: false,
                }),
                Some('S') => Ok(ClassItem::Builtin {
                    kind: BuiltinClassKind::Space,
                    negated: true,
                }),
                Some('n') => Ok(ClassItem::Single('\n')),
                Some('r') => Ok(ClassItem::Single('\r')),
                Some('t') => Ok(ClassItem::Single('\t')),
                Some(c) => Ok(ClassItem::Single(c)),
                None => Err("trailing backslash in character class".to_string()),
            },
            Some(c) => Ok(ClassItem::Single(c)),
            None => Err("unexpected EOF in character class".to_string()),
        }
    }

    fn parse_quantifier(&mut self, inner: RegexNode) -> Result<RegexNode, String> {
        let (min, max) = match self.peek() {
            Some('*') => {
                self.next_char();
                if self.peek() == Some('?') {
                    self.next_char();
                }
                (0, None)
            }
            Some('+') => {
                self.next_char();
                if self.peek() == Some('?') {
                    self.next_char();
                }
                (1, None)
            }
            Some('?') => {
                self.next_char();
                if self.peek() == Some('?') {
                    self.next_char();
                }
                (0, Some(1))
            }
            Some('{') => {
                let saved = self.pos;
                self.next_char(); // consume '{'
                if let Some((min_v, max_v)) = self.try_parse_brace_quantifier() {
                    if self.peek() == Some('?') {
                        self.next_char();
                    }
                    (min_v, max_v)
                } else {
                    self.pos = saved;
                    return Ok(inner);
                }
            }
            _ => return Ok(inner),
        };

        Ok(RegexNode::Repeat {
            inner: Box::new(inner),
            min,
            max,
        })
    }

    fn try_parse_brace_quantifier(&mut self) -> Option<(usize, Option<usize>)> {
        let min = self.try_parse_usize()?;
        match self.peek()? {
            '}' => {
                self.next_char();
                Some((min, Some(min)))
            }
            ',' => {
                self.next_char();
                if self.peek() == Some('}') {
                    self.next_char();
                    Some((min, None))
                } else {
                    let max = self.try_parse_usize()?;
                    if self.next_char() == Some('}') {
                        Some((min, Some(max)))
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn try_parse_usize(&mut self) -> Option<usize> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        if self.pos > start {
            self.input[start..self.pos].parse::<usize>().ok()
        } else {
            None
        }
    }
}

/// Convenience helper for JSON Schema `pattern` / JSONPath `search()` (substring regex match).
pub fn regex_is_match(pattern: &str, text: &str) -> bool {
    match Regex::compile(pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => text.contains(pattern),
    }
}

/// Convenience helper for RFC 9535 JSONPath `match()` (full-string regex match).
pub fn regex_is_full_match(pattern: &str, text: &str) -> bool {
    match Regex::compile(pattern) {
        Ok(re) => re.is_full_match(text),
        Err(_) => text == pattern,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_character_classes_and_quantifiers() {
        assert!(regex_is_match(r"^[a-zA-Z0-9_]{3,8}$", "user_01"));
        assert!(!regex_is_match(r"^[a-zA-Z0-9_]{3,8}$", "ab"));
        assert!(!regex_is_match(r"^[a-zA-Z0-9_]{3,8}$", "too_long_username"));

        assert!(regex_is_match(r"^\d{4}-\d{2}-\d{2}$", "2026-10-28"));
        assert!(!regex_is_match(r"^\d{4}-\d{2}-\d{2}$", "2026-1-28"));
    }

    #[test]
    fn test_regex_alternation_and_groups() {
        assert!(regex_is_match(r"^(dev|staging|prod)$", "staging"));
        assert!(!regex_is_match(r"^(dev|staging|prod)$", "local"));

        assert!(regex_is_full_match(r"1974-05-..", "1974-05-27"));
        assert!(!regex_is_full_match(r"1974-05", "1974-05-27"));
        assert!(regex_is_match(r"1974-05", "1974-05-27"));
    }

    #[test]
    fn test_regex_negated_classes_and_boundaries() {
        assert!(regex_is_match(r"^[^0-9]+$", "hello_world"));
        assert!(!regex_is_match(r"^[^0-9]+$", "hello123"));
        assert!(regex_is_match(r"\bcat\b", "the cat sat"));
        assert!(!regex_is_match(r"\bcat\b", "wildcat"));
    }

    #[test]
    fn test_regex_redos_step_budget_protection() {
        // Classic catastrophic backtracking pattern: (a+)+$ against "aaaa...!"
        let pattern = r"^(a+)+$";
        let malicious_input = "aaaaaaaaaaaaaaaaaaaaaaaaaaaa!";
        let re = Regex::compile(pattern).expect("compile regex");
        // Must safely abort evaluation and return false without hanging or stack overflow
        assert!(!re.is_full_match(malicious_input));
    }
}
