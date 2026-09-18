//! RFC 9535 JSONPath AST definitions.

use crate::model::Value;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// A compiled RFC 9535 JSONPath query expression.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPath {
    pub segments: Vec<Segment>,
}

/// A path segment/step in a JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    /// Direct child by property name: `.key` or `['key']`
    Child(String),
    /// Numeric index: `[0]` or `[-1]`
    Index(i64),
    /// Wildcard matching all children or array elements: `.*` or `[*]`
    Wildcard,
    /// Array slice `[start:end:step]`
    Slice {
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },
    /// Recursive descent into all descendants: `..`
    Descendant(Box<Segment>),
    /// Filter expression: `[?(<expr>)]`
    Filter(FilterExpr),
}

/// A filter expression inside `[?(...)]`.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpr {
    /// Path existence test: `@.key`
    Exists(PathQuery),
    /// Binary comparison: `left op right`
    Comparison {
        op: ComparisonOp,
        left: FilterOperand,
        right: FilterOperand,
    },
    /// Logical AND: `left && right`
    And(Box<FilterExpr>, Box<FilterExpr>),
    /// Logical OR: `left || right`
    Or(Box<FilterExpr>, Box<FilterExpr>),
    /// Logical NOT: `!expr`
    Not(Box<FilterExpr>),
}

/// Binary comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

/// Operands for comparisons in filter expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperand {
    /// Relative path from current node: `@.name`
    RelativePath(PathQuery),
    /// Absolute path from root: `$.threshold`
    RootPath(PathQuery),
    /// Literal constant value (string, number, bool, null)
    Literal(Value),
    /// Built-in function call, e.g. `length(@.items)`
    FunctionCall {
        name: String,
        arg: Box<FilterOperand>,
    },
}

/// A sub-path query used in filter expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct PathQuery {
    pub segments: Vec<Segment>,
}
