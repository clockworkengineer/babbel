//! # XPath 1.0 Evaluator
//!
//! Evaluates [`XPathExpr`] ASTs against a DOM [`Document`] and context node ID.

use crate::alloc_prelude::*;
use crate::document::Document;
use crate::error::{Result, XmlError};
use crate::node::{NodeId, NodeKind};
use crate::xpath::ast::{Axis, NodeTest, XPathExpr, XPathOperator};

/// Dynamic value type resulting from an XPath 1.0 expression evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum XPathValue {
    /// Node-set matching query (`Vec<NodeId>`).
    NodeSet(Vec<NodeId>),
    /// Boolean result.
    Boolean(bool),
    /// Numeric result (`f64`).
    Number(f64),
    /// String result.
    String(String),
}

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Callback signature for custom user-defined XPath functions.
pub type XPathCustomFn = alloc::boxed::Box<dyn Fn(&[XPathValue]) -> Result<XPathValue>>;

/// Evaluator computing XPath ASTs against a DOM [`Document`].
pub struct XPathEvaluator<'a> {
    /// Reference to target [`Document`].
    pub doc: &'a Document,
    /// Variable bindings table ($var).
    pub variables: HashMap<String, XPathValue>,
    /// Custom function callbacks.
    pub custom_functions: HashMap<String, XPathCustomFn>,
    depth: core::cell::Cell<usize>,
}

impl<'a> XPathEvaluator<'a> {
    /// Maximum recursive call depth during XPath evaluation (256 frames).
    pub const MAX_EVAL_DEPTH: usize = 256;

    /// Instantiates a new [`XPathEvaluator`] for a target [`Document`].
    pub fn new(doc: &'a Document) -> Self {
        Self {
            doc,
            variables: HashMap::new(),
            custom_functions: HashMap::new(),
            depth: core::cell::Cell::new(0),
        }
    }

    /// Sets or updates a variable binding reference ($var).
    pub fn set_variable(&mut self, name: impl Into<String>, value: XPathValue) {
        self.variables.insert(name.into(), value);
    }

    /// Registers a custom user-defined XPath function.
    pub fn register_function<F>(&mut self, name: impl Into<String>, f: F)
    where
        F: Fn(&[XPathValue]) -> Result<XPathValue> + 'static,
    {
        self.custom_functions.insert(name.into(), Box::new(f));
    }

    /// Evaluates an [`XPathExpr`] AST relative to a given `context_node` ID.
    pub fn evaluate(&self, expr: &XPathExpr, context_node: NodeId) -> Result<XPathValue> {
        self.evaluate_internal(expr, context_node, 1, 1)
    }

    /// Evaluates an [`XPathExpr`] AST with dynamic context position and context size.
    pub fn evaluate_internal(
        &self,
        expr: &XPathExpr,
        context_node: NodeId,
        pos: usize,
        size: usize,
    ) -> Result<XPathValue> {
        let current_depth = self.depth.get();
        if current_depth >= Self::MAX_EVAL_DEPTH {
            return Err(XmlError::XPathError(
                "XPath evaluation exceeded maximum recursion depth (256)".into(),
            ));
        }
        self.depth.set(current_depth + 1);
        struct DepthGuard<'b>(&'b core::cell::Cell<usize>);
        impl<'b> Drop for DepthGuard<'b> {
            fn drop(&mut self) {
                self.0.set(self.0.get().saturating_sub(1));
            }
        }
        let _guard = DepthGuard(&self.depth);

        match expr {
            XPathExpr::LiteralString(s) => Ok(XPathValue::String(s.clone())),
            XPathExpr::LiteralNumber(n) => Ok(XPathValue::Number(*n)),
            XPathExpr::Step {
                axis,
                test,
                predicates,
            } => {
                if *axis == Axis::Attribute {
                    if let NodeTest::Name(attr_name) = test {
                        if let Some(val) = self.doc.get_attribute(context_node, attr_name) {
                            return Ok(XPathValue::String(val.to_string()));
                        }
                    } else if let NodeTest::Wildcard | NodeTest::AttributeWildcard = test {
                        if let Some(node) = self.doc.get_node(context_node) {
                            if let NodeKind::Element { attributes, .. } = &node.kind {
                                if !attributes.is_empty() {
                                    return Ok(XPathValue::NodeSet(vec![context_node]));
                                }
                            }
                        }
                    }
                    return Ok(XPathValue::NodeSet(Vec::new()));
                }

                let initial_nodes = self.evaluate_axis(axis, test, context_node)?;
                let filtered = self.apply_predicates(initial_nodes, predicates)?;
                Ok(XPathValue::NodeSet(filtered))
            }
            XPathExpr::Path(steps) => {
                let mut current_nodes = vec![context_node];
                for step in steps {
                    let mut next_nodes = Vec::new();
                    let step_size = current_nodes.len();
                    for (step_idx, &ctx) in current_nodes.iter().enumerate() {
                        if let XPathValue::NodeSet(ns) = self.evaluate_internal(step, ctx, step_idx + 1, step_size)? {
                            next_nodes.extend(ns);
                        }
                    }
                    next_nodes.sort_unstable();
                    next_nodes.dedup();
                    current_nodes = next_nodes;
                }
                Ok(XPathValue::NodeSet(current_nodes))
            }
            XPathExpr::BinaryOp { op, left, right } => {
                let lval = self.evaluate_internal(left, context_node, pos, size)?;
                let rval = self.evaluate_internal(right, context_node, pos, size)?;
                self.evaluate_binary(op, lval, rval)
            }
            XPathExpr::FunctionCall { name, args } => {
                self.evaluate_function(name, args, context_node, pos, size)
            }
            XPathExpr::VariableRef(var) => {
                if let Some(val) = self.variables.get(var) {
                    Ok(val.clone())
                } else {
                    Err(XmlError::XPathError(format!(
                        "Unbound variable reference '${var}'"
                    )))
                }
            }
        }
    }

    fn evaluate_axis(&self, axis: &Axis, test: &NodeTest, context_node: NodeId) -> Result<Vec<NodeId>> {
        let candidates = match axis {
            Axis::Root => vec![self.doc.root_id().unwrap_or(0)],
            Axis::Child => {
                self.doc.get_node(context_node).map_or(Vec::new(), |n| n.children.clone())
            }
            Axis::SelfAxis => vec![context_node],
            Axis::Parent => self.doc.get_node(context_node).and_then(|n| n.parent).into_iter().collect(),
            Axis::Descendant => {
                let mut desc = Vec::new();
                self.collect_descendants(context_node, &mut desc, false);
                desc
            }
            Axis::DescendantOrSelf => {
                let mut desc = Vec::new();
                self.collect_descendants(context_node, &mut desc, true);
                desc
            }
            Axis::Ancestor => {
                let mut anc = Vec::new();
                let mut curr = context_node;
                while let Some(parent) = self.doc.get_node(curr).and_then(|n| n.parent) {
                    anc.push(parent);
                    curr = parent;
                }
                anc
            }
            Axis::AncestorOrSelf => {
                let mut anc = vec![context_node];
                let mut curr = context_node;
                while let Some(parent) = self.doc.get_node(curr).and_then(|n| n.parent) {
                    anc.push(parent);
                    curr = parent;
                }
                anc
            }
            Axis::FollowingSibling => {
                if let Some(parent_id) = self.doc.get_node(context_node).and_then(|n| n.parent) {
                    if let Some(parent) = self.doc.get_node(parent_id) {
                        if let Some(pos) = parent.children.iter().position(|&id| id == context_node) {
                            parent.children[pos + 1..].to_vec()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Axis::PrecedingSibling => {
                if let Some(parent_id) = self.doc.get_node(context_node).and_then(|n| n.parent) {
                    if let Some(parent) = self.doc.get_node(parent_id) {
                        if let Some(pos) = parent.children.iter().position(|&id| id == context_node) {
                            parent.children[..pos].to_vec()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Axis::Following => {
                let mut nodes = Vec::new();
                for id in context_node + 1..self.doc.len() as NodeId {
                    nodes.push(id);
                }
                nodes
            }
            Axis::Preceding => {
                let mut nodes = Vec::new();
                for id in (0..context_node).rev() {
                    nodes.push(id);
                }
                nodes
            }
            _ => {
                let mut desc = Vec::new();
                self.collect_descendants(context_node, &mut desc, false);
                desc
            }
        };

        Ok(candidates
            .into_iter()
            .filter(|&nid| self.matches_node_test(nid, test))
            .collect())
    }

    fn collect_descendants(&self, node_id: NodeId, out: &mut Vec<NodeId>, include_self: bool) {
        if include_self {
            out.push(node_id);
        }
        if let Some(node) = self.doc.get_node(node_id) {
            for &c_id in &node.children {
                self.collect_descendants(c_id, out, true);
            }
        }
    }

    fn matches_node_test(&self, node_id: NodeId, test: &NodeTest) -> bool {
        let node = match self.doc.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };

        match test {
            NodeTest::Wildcard => matches!(node.kind, NodeKind::Element { .. }),
            NodeTest::Node => true,
            NodeTest::Text => matches!(node.kind, NodeKind::Text(_)),
            NodeTest::Comment => matches!(node.kind, NodeKind::Comment(_)),
            NodeTest::Name(target_name) => match &node.kind {
                NodeKind::Element { name, .. } => &**name == target_name,
                NodeKind::ProcessingInstruction { target, .. } => &**target == target_name,
                NodeKind::DocTypeDefinition { name, .. } => &**name == target_name,
                _ => false,
            },
            _ => true,
        }
    }

    fn apply_predicates(&self, mut nodes: Vec<NodeId>, predicates: &[XPathExpr]) -> Result<Vec<NodeId>> {
        for pred in predicates {
            let mut filtered = Vec::new();
            let size = nodes.len();
            for (idx, &nid) in nodes.iter().enumerate() {
                let pval = self.evaluate_internal(pred, nid, idx + 1, size)?;
                let is_match = match pval {
                    XPathValue::Number(n) => (n as usize) == (idx + 1),
                    XPathValue::Boolean(b) => b,
                    XPathValue::NodeSet(ns) => !ns.is_empty(),
                    XPathValue::String(s) => !s.is_empty(),
                };
                if is_match {
                    filtered.push(nid);
                }
            }
            nodes = filtered;
        }
        Ok(nodes)
    }

    fn evaluate_binary(&self, op: &XPathOperator, left: XPathValue, right: XPathValue) -> Result<XPathValue> {
        match op {
            XPathOperator::Or => Ok(XPathValue::Boolean(self.to_bool(&left) || self.to_bool(&right))),
            XPathOperator::And => Ok(XPathValue::Boolean(self.to_bool(&left) && self.to_bool(&right))),
            XPathOperator::Eq => Ok(XPathValue::Boolean(self.to_string(&left) == self.to_string(&right))),
            XPathOperator::NotEq => Ok(XPathValue::Boolean(self.to_string(&left) != self.to_string(&right))),
            XPathOperator::Lt => Ok(XPathValue::Boolean(self.to_number(&left) < self.to_number(&right))),
            XPathOperator::LtEq => Ok(XPathValue::Boolean(self.to_number(&left) <= self.to_number(&right))),
            XPathOperator::Gt => Ok(XPathValue::Boolean(self.to_number(&left) > self.to_number(&right))),
            XPathOperator::GtEq => Ok(XPathValue::Boolean(self.to_number(&left) >= self.to_number(&right))),
            XPathOperator::Plus => Ok(XPathValue::Number(self.to_number(&left) + self.to_number(&right))),
            XPathOperator::Minus => Ok(XPathValue::Number(self.to_number(&left) - self.to_number(&right))),
            XPathOperator::Multiply => Ok(XPathValue::Number(self.to_number(&left) * self.to_number(&right))),
            XPathOperator::Div => Ok(XPathValue::Number(self.to_number(&left) / self.to_number(&right))),
            XPathOperator::Mod => Ok(XPathValue::Number(self.to_number(&left) % self.to_number(&right))),
            XPathOperator::Union => {
                let mut ns1 = match left {
                    XPathValue::NodeSet(ns) => ns,
                    _ => Vec::new(),
                };
                let ns2 = match right {
                    XPathValue::NodeSet(ns) => ns,
                    _ => Vec::new(),
                };
                ns1.extend(ns2);
                ns1.sort_unstable();
                ns1.dedup();
                Ok(XPathValue::NodeSet(ns1))
            }
        }
    }

    fn evaluate_function(
        &self,
        name: &str,
        args: &[XPathExpr],
        ctx: NodeId,
        pos: usize,
        size: usize,
    ) -> Result<XPathValue> {
        match name {
            "position" => {
                if !args.is_empty() {
                    return Err(XmlError::XPathError("position() expects 0 arguments".into()));
                }
                Ok(XPathValue::Number(pos as f64))
            }
            "last" => {
                if !args.is_empty() {
                    return Err(XmlError::XPathError("last() expects 0 arguments".into()));
                }
                Ok(XPathValue::Number(size as f64))
            }
            "id" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("id() expects 1 argument".into()));
                }
                let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                let id_str = self.to_string(&val);
                let mut res_nodes = Vec::new();
                for token in id_str.split_whitespace() {
                    if let Some(nid) = self.doc.get_element_by_id(token) {
                        res_nodes.push(nid);
                    }
                }
                Ok(XPathValue::NodeSet(res_nodes))
            }
            "namespace-uri" => {
                let target_node = if args.is_empty() {
                    ctx
                } else if let XPathValue::NodeSet(ns) = self.evaluate_internal(&args[0], ctx, pos, size)? {
                    *ns.first().unwrap_or(&ctx)
                } else {
                    ctx
                };
                let uri = self.doc.get_namespace_uri(target_node).unwrap_or_default();
                Ok(XPathValue::String(uri))
            }
            "lang" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("lang() expects 1 argument".into()));
                }
                let target_lang = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?).to_ascii_lowercase();
                let mut curr = Some(ctx);
                let mut is_lang = false;
                while let Some(nid) = curr {
                    if let Some(node_lang) = self.doc.get_attribute(nid, "xml:lang") {
                        let nl = node_lang.to_ascii_lowercase();
                        if nl == target_lang || nl.starts_with(&format!("{}-", target_lang)) {
                            is_lang = true;
                        }
                        break;
                    }
                    curr = self.doc.parent_id(nid);
                }
                Ok(XPathValue::Boolean(is_lang))
            }
            "ends-with" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("ends-with() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let s2 = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                Ok(XPathValue::Boolean(s1.ends_with(&s2)))
            }
            "lower-case" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("lower-case() expects 1 argument".into()));
                }
                let s = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                Ok(XPathValue::String(s.to_lowercase()))
            }
            "upper-case" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("upper-case() expects 1 argument".into()));
                }
                let s = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                Ok(XPathValue::String(s.to_uppercase()))
            }
            "replace" => {
                if args.len() != 3 {
                    return Err(XmlError::XPathError("replace() expects 3 arguments".into()));
                }
                let input = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let pattern = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                let replacement = self.to_string(&self.evaluate_internal(&args[2], ctx, pos, size)?);
                Ok(XPathValue::String(input.replace(&pattern, &replacement)))
            }
            "count" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("count() expects 1 argument".into()));
                }
                let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                let count = match val {
                    XPathValue::NodeSet(ns) => ns.len(),
                    _ => 0,
                };
                Ok(XPathValue::Number(count as f64))
            }
            "name" | "local-name" => {
                let target_node = if args.is_empty() {
                    ctx
                } else if let XPathValue::NodeSet(ns) = self.evaluate_internal(&args[0], ctx, pos, size)? {
                    *ns.first().unwrap_or(&ctx)
                } else {
                    ctx
                };
                let name_str = self
                    .doc
                    .get_node(target_node)
                    .map(|n| n.kind.name().to_string())
                    .unwrap_or_default();
                Ok(XPathValue::String(name_str))
            }
            "string" => {
                if args.is_empty() {
                    Ok(XPathValue::String(self.get_node_text(ctx)))
                } else {
                    let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                    Ok(XPathValue::String(self.to_string(&val)))
                }
            }
            "boolean" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("boolean() expects 1 argument".into()));
                }
                let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                Ok(XPathValue::Boolean(self.to_bool(&val)))
            }
            "not" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("not() expects 1 argument".into()));
                }
                let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                Ok(XPathValue::Boolean(!self.to_bool(&val)))
            }
            "number" => {
                if args.is_empty() {
                    Ok(XPathValue::Number(self.to_number(&XPathValue::String(self.get_node_text(ctx)))))
                } else {
                    let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                    Ok(XPathValue::Number(self.to_number(&val)))
                }
            }
            "sum" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("sum() expects 1 argument".into()));
                }
                let val = self.evaluate_internal(&args[0], ctx, pos, size)?;
                let total = match val {
                    XPathValue::NodeSet(ns) => ns
                        .iter()
                        .map(|&nid| self.get_node_text(nid).parse::<f64>().unwrap_or(0.0))
                        .sum(),
                    _ => 0.0,
                };
                Ok(XPathValue::Number(total))
            }
            "contains" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("contains() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let s2 = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                Ok(XPathValue::Boolean(s1.contains(&s2)))
            }
            "starts-with" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("starts-with() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let s2 = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                Ok(XPathValue::Boolean(s1.starts_with(&s2)))
            }
            "string-length" => {
                let s = if args.is_empty() {
                    self.get_node_text(ctx)
                } else {
                    self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?)
                };
                Ok(XPathValue::Number(s.chars().count() as f64))
            }
            "concat" => {
                let mut res = String::new();
                for arg in args {
                    let v = self.evaluate_internal(arg, ctx, pos, size)?;
                    res.push_str(&self.to_string(&v));
                }
                Ok(XPathValue::String(res))
            }
            "substring" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(XmlError::XPathError("substring() expects 2 or 3 arguments".into()));
                }
                let s = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let raw_start = self.to_number(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                if raw_start.is_nan() || raw_start < 1.0 {
                    return Ok(XPathValue::String(String::new()));
                }
                let start_idx = (raw_start as i64) - 1;
                let chars: Vec<char> = s.chars().collect();
                if start_idx < 0 || (start_idx as usize) >= chars.len() {
                    return Ok(XPathValue::String(String::new()));
                }
                let start = start_idx as usize;
                let len = if args.len() == 3 {
                    let raw_len = self.to_number(&self.evaluate_internal(&args[2], ctx, pos, size)?);
                    if raw_len.is_nan() || raw_len <= 0.0 {
                        return Ok(XPathValue::String(String::new()));
                    }
                    raw_len as usize
                } else {
                    chars.len().saturating_sub(start)
                };
                let end = start.saturating_add(len).min(chars.len());
                let sub: String = chars[start..end].iter().collect();
                Ok(XPathValue::String(sub))
            }
            "substring-before" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("substring-before() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let s2 = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                if let Some(pos_idx) = s1.find(&s2) {
                    Ok(XPathValue::String(s1[..pos_idx].to_string()))
                } else {
                    Ok(XPathValue::String(String::new()))
                }
            }
            "substring-after" => {
                if args.len() != 2 {
                    return Err(XmlError::XPathError("substring-after() expects 2 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let s2 = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                if let Some(pos_idx) = s1.find(&s2) {
                    Ok(XPathValue::String(s1[pos_idx + s2.len()..].to_string()))
                } else {
                    Ok(XPathValue::String(String::new()))
                }
            }
            "normalize-space" => {
                let s = if args.is_empty() {
                    self.get_node_text(ctx)
                } else {
                    self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?)
                };
                let words: Vec<&str> = s.split_whitespace().collect();
                Ok(XPathValue::String(words.join(" ")))
            }
            "translate" => {
                if args.len() != 3 {
                    return Err(XmlError::XPathError("translate() expects 3 arguments".into()));
                }
                let s1 = self.to_string(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                let from = self.to_string(&self.evaluate_internal(&args[1], ctx, pos, size)?);
                let to = self.to_string(&self.evaluate_internal(&args[2], ctx, pos, size)?);
                let from_chars: Vec<char> = from.chars().collect();
                let to_chars: Vec<char> = to.chars().collect();

                let mut result = String::new();
                for ch in s1.chars() {
                    if let Some(pos_idx) = from_chars.iter().position(|&c| c == ch) {
                        if pos_idx < to_chars.len() {
                            result.push(to_chars[pos_idx]);
                        }
                    } else {
                        result.push(ch);
                    }
                }
                Ok(XPathValue::String(result))
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("floor() expects 1 argument".into()));
                }
                let num = self.to_number(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                Ok(XPathValue::Number(floor_f64(num)))
            }
            "ceiling" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("ceiling() expects 1 argument".into()));
                }
                let num = self.to_number(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                Ok(XPathValue::Number(ceil_f64(num)))
            }
            "round" => {
                if args.len() != 1 {
                    return Err(XmlError::XPathError("round() expects 1 argument".into()));
                }
                let num = self.to_number(&self.evaluate_internal(&args[0], ctx, pos, size)?);
                Ok(XPathValue::Number(round_f64(num)))
            }
            "true" => Ok(XPathValue::Boolean(true)),
            "false" => Ok(XPathValue::Boolean(false)),
            _ => {
                if let Some(custom_fn) = self.custom_functions.get(name) {
                    let mut eval_args = Vec::new();
                    for arg in args {
                        eval_args.push(self.evaluate_internal(arg, ctx, pos, size)?);
                    }
                    return custom_fn(&eval_args);
                }
                Err(XmlError::XPathError(format!("Unknown XPath function: '{name}'")))
            }
        }
    }

    fn get_node_text(&self, node_id: NodeId) -> String {
        let mut text = String::new();
        if let Some(node) = self.doc.get_node(node_id) {
            match &node.kind {
                NodeKind::Text(t) | NodeKind::CData(t) => text.push_str(t),
                NodeKind::Element { .. } | NodeKind::Root | NodeKind::Prolog => {
                    for &c_id in &node.children {
                        text.push_str(&self.get_node_text(c_id));
                    }
                }
                _ => {}
            }
        }
        text
    }

    /// Converts an [`XPathValue`] to boolean.
    pub fn to_bool(&self, val: &XPathValue) -> bool {
        match val {
            XPathValue::Boolean(b) => *b,
            XPathValue::Number(n) => *n != 0.0 && !n.is_nan(),
            XPathValue::String(s) => !s.is_empty(),
            XPathValue::NodeSet(ns) => !ns.is_empty(),
        }
    }

    /// Converts an [`XPathValue`] to floating-point number.
    pub fn to_number(&self, val: &XPathValue) -> f64 {
        match val {
            XPathValue::Number(n) => *n,
            XPathValue::Boolean(b) => if *b { 1.0 } else { 0.0 },
            XPathValue::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            XPathValue::NodeSet(ns) => {
                if let Some(&first) = ns.first() {
                    self.get_node_text(first).parse::<f64>().unwrap_or(f64::NAN)
                } else {
                    f64::NAN
                }
            }
        }
    }

    /// Converts an [`XPathValue`] to string representation.
    pub fn to_string(&self, val: &XPathValue) -> String {
        match val {
            XPathValue::String(s) => s.clone(),
            XPathValue::Boolean(b) => b.to_string(),
            XPathValue::Number(n) => n.to_string(),
            XPathValue::NodeSet(ns) => {
                if let Some(&first) = ns.first() {
                    self.get_node_text(first)
                } else {
                    String::new()
                }
            }
        }
    }
}

#[cfg(feature = "std")]
fn floor_f64(n: f64) -> f64 {
    n.floor()
}

#[cfg(not(feature = "std"))]
fn floor_f64(n: f64) -> f64 {
    (n as i64) as f64
}

#[cfg(feature = "std")]
fn ceil_f64(n: f64) -> f64 {
    n.ceil()
}

#[cfg(not(feature = "std"))]
fn ceil_f64(n: f64) -> f64 {
    if n > (n as i64) as f64 {
        (n as i64 + 1) as f64
    } else {
        (n as i64) as f64
    }
}

#[cfg(feature = "std")]
fn round_f64(n: f64) -> f64 {
    n.round()
}

#[cfg(not(feature = "std"))]
fn round_f64(n: f64) -> f64 {
    (n + 0.5) as i64 as f64
}
