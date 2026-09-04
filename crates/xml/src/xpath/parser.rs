//! # XPath 1.0 Parser
//!
//! Operator precedence and recursive descent parser transforming token streams into [`XPathExpr`] ASTs.

use crate::alloc_prelude::*;
use crate::error::{Result, XmlError};
use crate::xpath::ast::{Axis, NodeTest, XPathExpr, XPathOperator};
use crate::xpath::lexer::{Token, XPathLexer};

/// XPath AST parser instance.
pub struct XPathParser<'a> {
    lexer: XPathLexer<'a>,
    current: Token,
    depth: usize,
}

impl<'a> XPathParser<'a> {
    /// Maximum nesting depth for nested expressions / sub-queries (128 frames).
    pub const MAX_EXPR_DEPTH: usize = 128;

    /// Instantiates a new [`XPathParser`] for an XPath expression string slice.
    pub fn new(input: &'a str) -> Result<Self> {
        let mut lexer = XPathLexer::new(input);
        let current = lexer.next_token()?;
        Ok(Self {
            lexer,
            current,
            depth: 0,
        })
    }

    fn advance(&mut self) -> Result<Token> {
        let prev = core::mem::replace(&mut self.current, Token::Eof);
        self.current = self.lexer.next_token()?;
        Ok(prev)
    }

    fn enter_depth(&mut self) -> Result<()> {
        if self.depth >= Self::MAX_EXPR_DEPTH {
            return Err(XmlError::XPathError(
                "XPath expression exceeds maximum nesting depth (128)".into(),
            ));
        }
        self.depth += 1;
        Ok(())
    }

    fn leave_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Parses the top-level XPath expression into an [`XPathExpr`] AST.
    pub fn parse_expression(&mut self) -> Result<XPathExpr> {
        self.enter_depth()?;
        let res = self.parse_or_expr();
        self.leave_depth();
        res
    }

    fn parse_or_expr(&mut self) -> Result<XPathExpr> {
        let mut left = self.parse_and_expr()?;
        while self.current == Token::Name("or".into()) {
            self.advance()?;
            let right = self.parse_and_expr()?;
            left = XPathExpr::BinaryOp {
                op: XPathOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<XPathExpr> {
        let mut left = self.parse_equality_expr()?;
        while self.current == Token::Name("and".into()) {
            self.advance()?;
            let right = self.parse_equality_expr()?;
            left = XPathExpr::BinaryOp {
                op: XPathOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality_expr(&mut self) -> Result<XPathExpr> {
        let mut left = self.parse_relational_expr()?;
        while matches!(self.current, Token::Eq | Token::NotEq) {
            let op = if self.advance()? == Token::Eq {
                XPathOperator::Eq
            } else {
                XPathOperator::NotEq
            };
            let right = self.parse_relational_expr()?;
            left = XPathExpr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_relational_expr(&mut self) -> Result<XPathExpr> {
        let mut left = self.parse_additive_expr()?;
        while matches!(
            self.current,
            Token::Lt | Token::LtEq | Token::Gt | Token::GtEq
        ) {
            let token = self.advance()?;
            let op = match token {
                Token::Lt => XPathOperator::Lt,
                Token::LtEq => XPathOperator::LtEq,
                Token::Gt => XPathOperator::Gt,
                Token::GtEq => XPathOperator::GtEq,
                _ => unreachable!(),
            };
            let right = self.parse_additive_expr()?;
            left = XPathExpr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<XPathExpr> {
        let mut left = self.parse_multiplicative_expr()?;
        while matches!(self.current, Token::Plus | Token::Minus) {
            let op = if self.advance()? == Token::Plus {
                XPathOperator::Plus
            } else {
                XPathOperator::Minus
            };
            let right = self.parse_multiplicative_expr()?;
            left = XPathExpr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<XPathExpr> {
        let mut left = self.parse_unary_expr()?;
        while matches!(
            self.current,
            Token::Star | Token::Name(_)
        ) && (self.current == Token::Star || self.current == Token::Name("div".into()) || self.current == Token::Name("mod".into())) {
            let token = self.advance()?;
            let op = match token {
                Token::Star => XPathOperator::Multiply,
                Token::Name(n) if n == "div" => XPathOperator::Div,
                Token::Name(n) if n == "mod" => XPathOperator::Mod,
                _ => break,
            };
            let right = self.parse_unary_expr()?;
            left = XPathExpr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<XPathExpr> {
        if self.current == Token::Minus {
            self.advance()?;
            let operand = self.parse_unary_expr()?;
            return Ok(XPathExpr::BinaryOp {
                op: XPathOperator::Minus,
                left: Box::new(XPathExpr::LiteralNumber(0.0)),
                right: Box::new(operand),
            });
        }
        self.parse_path_expr()
    }

    fn parse_path_expr(&mut self) -> Result<XPathExpr> {
        let mut steps = Vec::new();
        let mut is_absolute = false;

        if self.current == Token::Slash {
            self.advance()?;
            is_absolute = true;
            if self.current == Token::Eof {
                return Ok(XPathExpr::Step {
                    axis: Axis::Root,
                    test: NodeTest::Node,
                    predicates: Vec::new(),
                });
            }
        } else if self.current == Token::DoubleSlash {
            self.advance()?;
            is_absolute = true;
            steps.push(XPathExpr::Step {
                axis: Axis::DescendantOrSelf,
                test: NodeTest::Node,
                predicates: Vec::new(),
            });
        }

        if is_absolute && (self.current == Token::Eof || self.current == Token::RightParen || self.current == Token::RightBracket) {
            return Ok(XPathExpr::Path(steps));
        }

        let first_step = self.parse_primary_or_step()?;
        steps.push(first_step);

        while self.current == Token::Slash || self.current == Token::DoubleSlash {
            let is_dsc = self.advance()? == Token::DoubleSlash;
            if is_dsc {
                steps.push(XPathExpr::Step {
                    axis: Axis::DescendantOrSelf,
                    test: NodeTest::Node,
                    predicates: Vec::new(),
                });
            }
            if self.current != Token::Eof {
                let next_step = self.parse_step()?;
                steps.push(next_step);
            }
        }

        if steps.len() == 1 {
            Ok(steps.remove(0))
        } else {
            Ok(XPathExpr::Path(steps))
        }
    }

    fn parse_primary_or_step(&mut self) -> Result<XPathExpr> {
        match &self.current {
            Token::Variable(v) => {
                let name = v.clone();
                self.advance()?;
                Ok(XPathExpr::VariableRef(name))
            }
            Token::LiteralString(s) => {
                let val = s.clone();
                self.advance()?;
                Ok(XPathExpr::LiteralString(val))
            }
            Token::LiteralNumber(n) => {
                let val = *n;
                self.advance()?;
                Ok(XPathExpr::LiteralNumber(val))
            }
            Token::LeftParen => {
                self.advance()?;
                let expr = self.parse_expression()?;
                if self.advance()? != Token::RightParen {
                    return Err(XmlError::XPathError("Expected ')' closing parenthesized expression".into()));
                }
                Ok(expr)
            }
            Token::Name(n) if self.lexer.peek() == Some('(') => {
                let fname = n.clone();
                self.advance()?; // consume function name
                self.advance()?; // consume '('
                let mut args = Vec::new();
                if self.current != Token::RightParen {
                    loop {
                        args.push(self.parse_expression()?);
                        if self.current == Token::Comma {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }
                if self.advance()? != Token::RightParen {
                    return Err(XmlError::XPathError(format!("Expected ')' in function call '{fname}'")));
                }
                Ok(XPathExpr::FunctionCall { name: fname, args })
            }
            _ => self.parse_step(),
        }
    }

    fn parse_step(&mut self) -> Result<XPathExpr> {
        let mut axis = Axis::Child;

        if self.current == Token::At {
            self.advance()?;
            axis = Axis::Attribute;
        } else if self.current == Token::Dot {
            self.advance()?;
            return Ok(XPathExpr::Step {
                axis: Axis::SelfAxis,
                test: NodeTest::Node,
                predicates: Vec::new(),
            });
        } else if self.current == Token::DoubleDot {
            self.advance()?;
            return Ok(XPathExpr::Step {
                axis: Axis::Parent,
                test: NodeTest::Node,
                predicates: Vec::new(),
            });
        } else if let Token::Name(name) = &self.current {
            let name_str = name.clone();
            let axis_opt = match name_str.as_str() {
                "child" => Some(Axis::Child),
                "descendant" => Some(Axis::Descendant),
                "parent" => Some(Axis::Parent),
                "ancestor" => Some(Axis::Ancestor),
                "following-sibling" => Some(Axis::FollowingSibling),
                "preceding-sibling" => Some(Axis::PrecedingSibling),
                "following" => Some(Axis::Following),
                "preceding" => Some(Axis::Preceding),
                "attribute" => Some(Axis::Attribute),
                "namespace" => Some(Axis::Namespace),
                "self" => Some(Axis::SelfAxis),
                "descendant-or-self" => Some(Axis::DescendantOrSelf),
                "ancestor-or-self" => Some(Axis::AncestorOrSelf),
                _ => None,
            };

            if let Some(ax) = axis_opt {
                self.advance()?;
                if self.current == Token::ColonColon {
                    self.advance()?;
                    axis = ax;
                } else {
                    return self.finish_step_with_name(axis, name_str);
                }
            }
        }

        let test = self.parse_node_test()?;
        let mut predicates = Vec::new();
        while self.current == Token::LeftBracket {
            self.advance()?;
            let pred_expr = self.parse_expression()?;
            if self.advance()? != Token::RightBracket {
                return Err(XmlError::XPathError("Expected ']' closing predicate".into()));
            }
            predicates.push(pred_expr);
        }

        Ok(XPathExpr::Step {
            axis,
            test,
            predicates,
        })
    }

    fn finish_step_with_name(&mut self, axis: Axis, name: String) -> Result<XPathExpr> {
        let test = if name == "*" {
            NodeTest::Wildcard
        } else {
            NodeTest::Name(name)
        };

        let mut predicates = Vec::new();
        while self.current == Token::LeftBracket {
            self.advance()?;
            let pred_expr = self.parse_expression()?;
            if self.advance()? != Token::RightBracket {
                return Err(XmlError::XPathError("Expected ']' closing predicate".into()));
            }
            predicates.push(pred_expr);
        }

        Ok(XPathExpr::Step {
            axis,
            test,
            predicates,
        })
    }

    fn parse_node_test(&mut self) -> Result<NodeTest> {
        match &self.current {
            Token::Star => {
                self.advance()?;
                Ok(NodeTest::Wildcard)
            }
            Token::Name(n) => {
                let name = n.clone();
                self.advance()?;
                if name == "text" && self.current == Token::LeftParen {
                    self.advance()?;
                    if self.advance()? != Token::RightParen {
                        return Err(XmlError::XPathError("Expected ')' after text(".into()));
                    }
                    Ok(NodeTest::Text)
                } else if name == "node" && self.current == Token::LeftParen {
                    self.advance()?;
                    if self.advance()? != Token::RightParen {
                        return Err(XmlError::XPathError("Expected ')' after node(".into()));
                    }
                    Ok(NodeTest::Node)
                } else if name == "comment" && self.current == Token::LeftParen {
                    self.advance()?;
                    if self.advance()? != Token::RightParen {
                        return Err(XmlError::XPathError("Expected ')' after comment(".into()));
                    }
                    Ok(NodeTest::Comment)
                } else {
                    Ok(NodeTest::Name(name))
                }
            }
            _ => Err(XmlError::XPathError("Expected node test in XPath step".into())),
        }
    }
}
