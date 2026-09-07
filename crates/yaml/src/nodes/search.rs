//! Tree search, visitor, and traversal methods for YAML AST nodes.
//!
//! Provides depth-first pre-order tree traversal, counting, depth calculation,
//! predicate-based node filtering, and immediate child iteration.

use super::node::Node;

/// Iterator over immediate child nodes of a YAML AST node
pub struct NodeChildIterator<'a> {
    node: &'a Node,
    index: usize,
    mapping_phase: usize, // 0 = keys, 1 = values, 2 = done
}

impl<'a> NodeChildIterator<'a> {
    /// Creates a new child iterator for `node`.
    pub fn new(node: &'a Node) -> Self {
        Self {
            node,
            index: 0,
            mapping_phase: 0,
        }
    }
}

impl<'a> Iterator for NodeChildIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.node {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                if self.index < items.len() {
                    let item = &items[self.index];
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            }
            Node::Mapping(pairs) => {
                if self.mapping_phase == 0 {
                    // Return keys first
                    if let Some((key, _)) = pairs.get(self.index) {
                        self.index += 1;
                        return Some(key);
                    } else {
                        // Done with keys, move to values
                        self.mapping_phase = 1;
                        self.index = 0;
                    }
                }
                if self.mapping_phase == 1 {
                    // Return values
                    if let Some((_, value)) = pairs.get(self.index) {
                        self.index += 1;
                        return Some(value);
                    } else {
                        self.mapping_phase = 2;
                    }
                }
                None
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                if self.index == 0 {
                    self.index = 1;
                    Some(inner.as_ref())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Node {
    /// Iterate over all immediate child nodes
    ///
    /// Returns an iterator over references to child nodes. Does not recurse.
    /// For recursive traversal, use `visit()` or `iter_all()`.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let array = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// let children: Vec<_> = array.children().collect();
    /// assert_eq!(children.len(), 2);
    /// ```
    pub fn children(&self) -> NodeChildIterator {
        NodeChildIterator::new(self)
    }

    /// Visit all nodes in the tree with a closure (depth-first, pre-order)
    ///
    /// The closure receives a reference to each node and its depth in the tree.
    /// Root node is at depth 0. Returns early if the closure returns false.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::Array(vec![Node::from(2), Node::from(3)])
    /// ]);
    ///
    /// let mut count = 0;
    /// doc.visit(|node, depth| {
    ///     count += 1;
    ///     true // continue traversal
    /// });
    /// assert_eq!(count, 5); // root array + 1 + nested array + 2 + 3
    /// ```
    pub fn visit<F>(&self, mut visitor: F)
    where
        F: FnMut(&Node, usize) -> bool,
    {
        self.visit_internal(&mut visitor, 0);
    }

    fn visit_internal<F>(&self, visitor: &mut F, depth: usize) -> bool
    where
        F: FnMut(&Node, usize) -> bool,
    {
        // Visit this node first (pre-order)
        if !visitor(self, depth) {
            return false;
        }

        // Then visit children
        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    if !item.visit_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    if !key.visit_internal(visitor, depth + 1) {
                        return false;
                    }
                    if !value.visit_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                return inner.visit_internal(visitor, depth + 1);
            }
            _ => {}
        }

        true
    }

    /// Visit all nodes in the tree with mutable access (depth-first, pre-order)
    ///
    /// The closure receives a mutable reference to each node and its depth.
    /// Allows modification of nodes during traversal.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// # use babbel_yaml::Numeric;
    /// let mut doc = Node::Array(vec![Node::from(1), Node::from(2)]);
    /// doc.visit_mut(|node, _depth| {
    ///     if let Node::Number(n) = node {
    ///         // Could modify number here
    ///         if let Numeric::Int32(val) = n {
    ///             *val *= 2; // Double the value
    ///         }
    ///     }
    ///     true
    /// });
    /// ```
    pub fn visit_mut<F>(&mut self, mut visitor: F)
    where
        F: FnMut(&mut Node, usize) -> bool,
    {
        self.visit_mut_internal(&mut visitor, 0);
    }

    fn visit_mut_internal<F>(&mut self, visitor: &mut F, depth: usize) -> bool
    where
        F: FnMut(&mut Node, usize) -> bool,
    {
        // Visit this node first (pre-order)
        if !visitor(self, depth) {
            return false;
        }

        // Then visit children
        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    if !item.visit_mut_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    if !key.visit_mut_internal(visitor, depth + 1) {
                        return false;
                    }
                    if !value.visit_mut_internal(visitor, depth + 1) {
                        return false;
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                return inner.visit_mut_internal(visitor, depth + 1);
            }
            _ => {}
        }

        true
    }

    /// Count all nodes in the tree (including this node)
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::Array(vec![Node::from(2)])
    /// ]);
    /// assert_eq!(doc.count_nodes(), 4); // array + 1 + nested array + 2
    /// ```
    pub fn count_nodes(&self) -> usize {
        let mut count = 0;
        self.visit(|_, _| {
            count += 1;
            true
        });
        count
    }

    /// Find the maximum depth of the tree
    ///
    /// Returns 0 for leaf nodes, 1+ for nodes with children.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let leaf = Node::from(42);
    /// assert_eq!(leaf.max_depth(), 0);
    ///
    /// let nested = Node::Array(vec![
    ///     Node::Array(vec![Node::from(1)])
    /// ]);
    /// assert_eq!(nested.max_depth(), 2);
    /// ```
    pub fn max_depth(&self) -> usize {
        let mut max = 0;
        self.visit(|_, depth| {
            if depth > max {
                max = depth;
            }
            true
        });
        max
    }

    /// Collect all nodes matching a predicate
    ///
    /// Returns a vector of references to nodes that match the predicate.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from(1),
    ///     Node::from("text"),
    ///     Node::from(2)
    /// ]);
    ///
    /// let numbers = doc.find_all(|node| node.is_number());
    /// assert_eq!(numbers.len(), 2);
    /// ```
    pub fn find_all<F>(&self, mut predicate: F) -> alloc::vec::Vec<&Node>
    where
        F: FnMut(&Node) -> bool,
    {
        let mut results = alloc::vec::Vec::new();
        self.find_all_internal(&mut predicate, &mut results);
        results
    }

    fn find_all_internal<'a, F>(
        &'a self,
        predicate: &mut F,
        results: &mut alloc::vec::Vec<&'a Node>,
    ) where
        F: FnMut(&Node) -> bool,
    {
        if predicate(self) {
            results.push(self);
        }

        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    item.find_all_internal(predicate, results);
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    key.find_all_internal(predicate, results);
                    value.find_all_internal(predicate, results);
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                inner.find_all_internal(predicate, results);
            }
            _ => {}
        }
    }

    /// Find the first node matching a predicate
    ///
    /// Returns None if no matching node is found.
    ///
    /// # Example
    /// ```
    /// # use babbel_yaml::Node;
    /// let doc = Node::Array(vec![
    ///     Node::from("text"),
    ///     Node::from(42)
    /// ]);
    ///
    /// let first_num = doc.find_first(|node| node.is_number());
    /// assert!(first_num.is_some());
    /// ```
    pub fn find_first<F>(&self, mut predicate: F) -> Option<&Node>
    where
        F: FnMut(&Node) -> bool,
    {
        self.find_first_internal(&mut predicate)
    }

    fn find_first_internal<F>(&self, predicate: &mut F) -> Option<&Node>
    where
        F: FnMut(&Node) -> bool,
    {
        if predicate(self) {
            return Some(self);
        }

        match self {
            Node::Array(items)
            | Node::Set(items)
            | Node::Document(items)
            | Node::Documents(items) => {
                for item in items {
                    if let Some(found) = item.find_first_internal(predicate) {
                        return Some(found);
                    }
                }
            }
            Node::Mapping(pairs) => {
                for (key, value) in pairs {
                    if let Some(found) = key.find_first_internal(predicate) {
                        return Some(found);
                    }
                    if let Some(found) = value.find_first_internal(predicate) {
                        return Some(found);
                    }
                }
            }
            Node::Anchored(inner, _) | Node::Tagged(inner, _) => {
                return inner.find_first_internal(predicate);
            }
            _ => {}
        }

        None
    }
}
