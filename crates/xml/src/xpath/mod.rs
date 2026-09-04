//! # XPath 1.0 Evaluation Subsystem
//!
//! Provides the top-level [`XPathEngine`] entry point for evaluating XPath expressions against DOM trees.

pub mod ast;
pub mod evaluator;
pub mod lexer;
pub mod parser;

pub use ast::{Axis, NodeTest, XPathExpr, XPathOperator};
pub use evaluator::{XPathEvaluator, XPathValue};
pub use lexer::{Token, XPathLexer};
pub use parser::XPathParser;

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::Result;
use crate::node::NodeId;

/// Main high-level XPath 1.0 engine interface.
pub struct XPathEngine<'a> {
    evaluator: XPathEvaluator<'a>,
}

impl<'a> XPathEngine<'a> {
    /// Instantiates a new [`XPathEngine`] for a target DOM [`Document`].
    pub fn new(doc: &'a Document) -> Self {
        Self {
            evaluator: XPathEvaluator::new(doc),
        }
    }

    /// Evaluates an XPath expression string against an optional `context_node` ID (defaults to root).
    pub fn evaluate(&self, expression: &str, context_node: Option<NodeId>) -> Result<XPathValue> {
        let mut parser = XPathParser::new(expression)?;
        let expr = parser.parse_expression()?;
        let ctx = context_node.unwrap_or_else(|| self.evaluator.doc.root_id().unwrap_or(0));
        self.evaluator.evaluate(&expr, ctx)
    }

    /// Evaluates an XPath expression string expecting a Node-Set result (`Vec<NodeId>`).
    pub fn evaluate_nodes(&self, expression: &str, context_node: Option<NodeId>) -> Result<Vec<NodeId>> {
        match self.evaluate(expression, context_node)? {
            XPathValue::NodeSet(ns) => Ok(ns),
            _ => Ok(Vec::new()),
        }
    }

    /// Sets or updates a variable binding reference ($var) in the XPath context.
    pub fn set_variable(&mut self, name: impl Into<String>, value: XPathValue) {
        self.evaluator.set_variable(name, value);
    }

    /// Registers a custom user-defined XPath function.
    pub fn register_function<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(&[XPathValue]) -> Result<XPathValue> + 'static,
    {
        self.evaluator.register_function(name, f);
    }
}
