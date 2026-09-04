//! Visitor pattern abstraction for serializing bencode data into target formats.
//! Enables Open/Closed Principle (OCP) compliance: new target formats implement `BencodeVisitor`
//! without altering core AST data structures or parser logic.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::nodes::node::Node;

/// Trait implemented by format serializers to receive traversal events.
pub trait BencodeVisitor {
    type Error;

    /// Visit an integer value.
    fn visit_integer(&mut self, value: i64) -> Result<(), Self::Error>;
    /// Visit a string value.
    fn visit_string(&mut self, value: &str) -> Result<(), Self::Error>;
    /// Visit the start of a list.
    fn visit_list_start(&mut self) -> Result<(), Self::Error>;
    /// Visit the end of a list.
    fn visit_list_end(&mut self) -> Result<(), Self::Error>;
    /// Visit the start of a dictionary.
    fn visit_dict_start(&mut self) -> Result<(), Self::Error>;
    /// Visit a dictionary key.
    fn visit_dict_key(&mut self, key: &str) -> Result<(), Self::Error>;
    /// Visit the end of a dictionary.
    fn visit_dict_end(&mut self) -> Result<(), Self::Error>;
    /// Visit an uninitialized/None value.
    fn visit_none(&mut self) -> Result<(), Self::Error>;
}

/// Trait implemented by data structures that can be traversed by a `BencodeVisitor`.
pub trait BencodeVisitable {
    /// Accepts a visitor and traverses the internal structure.
    fn accept<V: BencodeVisitor>(&self, visitor: &mut V) -> Result<(), V::Error>;
}

impl BencodeVisitable for Node {
    fn accept<V: BencodeVisitor>(&self, visitor: &mut V) -> Result<(), V::Error> {
        match self {
            Node::Integer(val) => visitor.visit_integer(*val),
            Node::Str(val) => visitor.visit_string(val),
            Node::List(items) => {
                visitor.visit_list_start()?;
                for item in items {
                    item.accept(visitor)?;
                }
                visitor.visit_list_end()
            }
            Node::Dictionary(items) => {
                visitor.visit_dict_start()?;
                let mut sorted: Vec<_> = items.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));
                for (key, value) in sorted {
                    visitor.visit_dict_key(key)?;
                    value.accept(visitor)?;
                }
                visitor.visit_dict_end()
            }
            Node::None => visitor.visit_none(),
        }
    }
}
