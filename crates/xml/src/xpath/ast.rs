//! # XPath 1.0 AST Definitions
//!
//! Abstract Syntax Tree types representing XPath expressions, operators, node tests, and 13 XPath axes.

use crate::alloc_prelude::*;

/// Enum representing all 13 standard XPath 1.0 axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Virtual Root axis.
    Root,
    /// `child::` axis.
    Child,
    /// `descendant::` axis.
    Descendant,
    /// `parent::` axis.
    Parent,
    /// `ancestor::` axis.
    Ancestor,
    /// `following-sibling::` axis.
    FollowingSibling,
    /// `preceding-sibling::` axis.
    PrecedingSibling,
    /// `following::` axis.
    Following,
    /// `preceding::` axis.
    Preceding,
    /// `attribute::` or `@` axis.
    Attribute,
    /// `namespace::` axis.
    Namespace,
    /// `self::` or `.` axis.
    SelfAxis,
    /// `descendant-or-self::` or `//` axis.
    DescendantOrSelf,
    /// `ancestor-or-self::` axis.
    AncestorOrSelf,
}

/// XPath node test pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeTest {
    /// Wildcard `*` node test.
    Wildcard,
    /// `node()` node test.
    Node,
    /// `text()` node test.
    Text,
    /// `comment()` node test.
    Comment,
    /// Explicit tag or attribute name test (e.g. `book`, `@id`).
    Name(String),
    /// Attribute wildcard `@*`.
    AttributeWildcard,
}

/// XPath binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XPathOperator {
    /// `or`
    Or,
    /// `and`
    And,
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
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Multiply,
    /// `div`
    Div,
    /// `mod`
    Mod,
    /// `|`
    Union,
}

/// Abstract Syntax Tree node for XPath 1.0 expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum XPathExpr {
    /// Literal string constant (e.g., `'cooking'`).
    LiteralString(String),
    /// Literal numeric constant (e.g., `29.99`).
    LiteralNumber(f64),
    /// Single path step with axis, node test, and predicates.
    Step {
        axis: Axis,
        test: NodeTest,
        predicates: Vec<XPathExpr>,
    },
    /// Multi-step path expression (`/bookstore/book[1]/title`).
    Path(Vec<XPathExpr>),
    /// Binary operation expression (`a + b`, `x = y`, `cond1 and cond2`).
    BinaryOp {
        op: XPathOperator,
        left: Box<XPathExpr>,
        right: Box<XPathExpr>,
    },
    /// Function call expression (`count(//book)`, `contains(title, 'XML')`).
    FunctionCall {
        name: String,
        args: Vec<XPathExpr>,
    },
    /// Variable reference (`$var`).
    VariableRef(String),
}
