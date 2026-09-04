use super::types::Node;
use core::ops::{Index, IndexMut};

/// Default implementation returns Node::None
impl Default for Node {
    fn default() -> Self {
        Node::None
    }
}

/// Allows index access into Node::Array using array indexing: `node[0]`
impl Index<usize> for Node {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node::Array(arr) => arr.get(index).unwrap_or(&Node::None),
            _ => &Node::None,
        }
    }
}

/// Allows index access into Node::Object using key indexing: `node["key"]`
impl Index<&str> for Node {
    type Output = Node;

    fn index(&self, key: &str) -> &Self::Output {
        match self {
            Node::Object(map) => map.get(key).unwrap_or(&Node::None),
            _ => &Node::None,
        }
    }
}

/// Allows mutable index access into Node::Array: `node[0] = value`
impl IndexMut<usize> for Node {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Node::Array(arr) => &mut arr[index],
            _ => panic!("Cannot index non-array node with integer"),
        }
    }
}

/// Allows mutable index access into Node::Object: `node["key"] = value`
impl IndexMut<&str> for Node {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        match self {
            Node::Object(map) => map
                .get_mut(key)
                .expect("Key does not exist. Use insert() to add new keys."),
            _ => panic!("Cannot mutably index non-object node with string"),
        }
    }
}
