use json_lib::misc::print;
use json_lib::nodes::node::{Node, make_node};
use json_lib::BufferDestination;
use std::collections::HashMap;

fn main() {
    let mut destination = BufferDestination::new();

    // Create leaf nodes using make_node()
    let leaf_a1 = make_node(100); // Integer
    let leaf_a2 = make_node("leaf_a2"); // String
    let leaf_b1 = make_node(vec![1, 2, 3]); // Array

    // Create a deeper branch under A1 using make_node()
    let mut deep_branch_map = HashMap::new();
    deep_branch_map.insert("A1a1".to_string(), make_node(3.14)); // Float
    let deep_branch = Node::Object(deep_branch_map);

    // Branch A with children using make_node()
    let mut branch_a_map = HashMap::new();
    branch_a_map.insert("A1".to_string(), leaf_a1);
    branch_a_map.insert("A1a".to_string(), deep_branch);
    branch_a_map.insert("A2".to_string(), leaf_a2);
    let branch_a = Node::Object(branch_a_map);

    // Branch B with child using make_node()
    let mut branch_b_map = HashMap::new();
    branch_b_map.insert("B1".to_string(), leaf_b1);
    let branch_b = Node::Object(branch_b_map);

    // Root node with branches using make_node()
    let mut root_map = HashMap::new();
    root_map.insert("A".to_string(), branch_a);
    root_map.insert("B".to_string(), branch_b);
    let root = Node::Object(root_map);

    // Print the tree
    print(&root, &mut destination, 4);
    print!("{}", destination.to_string());
}   