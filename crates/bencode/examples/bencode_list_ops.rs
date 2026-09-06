//! Example demonstrating list manipulation operations.
//! This shows how to create, modify, iterate, search, and transform list nodes
//! in various scenarios when working with bencode data.

use bencode_lib::{make_node, Node};

fn main() {
    println!("=== List Manipulation Examples ===\n");

    // Example 1: Creating lists
    demonstrate_creation();

    // Example 2: Accessing list elements
    demonstrate_access();

    // Example 3: Modifying lists
    demonstrate_modification();

    // Example 4: Iterating and filtering
    demonstrate_iteration();

    // Example 5: List transformations
    demonstrate_transformations();

    // Example 6: Working with nested lists
    demonstrate_nested_lists();

    // Example 7: Practical use cases
    demonstrate_practical_cases();
}

/// Demonstrates various ways to create list nodes
fn demonstrate_creation() {
    println!("--- Creating Lists ---");

    // Method 1: Using make_node with vec
    let list1 = make_node(vec![make_node(1), make_node(2), make_node(3)]);
    println!("Method 1 (make_node): {}", list1);

    // Method 2: Direct Node::List construction
    let list2 = Node::List(vec![
        Node::Str("hello".to_string()),
        Node::Str("world".to_string()),
    ]);
    println!("Method 2 (direct): {}", list2);

    // Method 3: Empty list, then populate
    let mut list3 = Node::List(vec![]);
    if let Some(vec) = list3.as_list_mut() {
        vec.push(make_node("first"));
        vec.push(make_node("second"));
        vec.push(make_node("third"));
    }
    println!("Method 3 (build incrementally): {}", list3);

    // Method 4: Mixed types
    let list4 = make_node(vec![
        make_node(42),
        make_node("text"),
        make_node(vec![make_node(1), make_node(2)]),
    ]);
    println!("Method 4 (mixed types): {}\n", list4);
}

/// Demonstrates accessing list elements
fn demonstrate_access() {
    println!("--- Accessing List Elements ---");

    let list = make_node(vec![
        make_node("alpha"),
        make_node("beta"),
        make_node("gamma"),
        make_node("delta"),
    ]);

    println!("List: {}", list);

    if let Some(items) = list.as_list() {
        println!("Length: {}", items.len());
        println!("Is empty: {}", items.is_empty());

        // Access by index
        if let Some(first) = items.get(0) {
            println!("First element: {}", first);
        }

        if let Some(last) = items.last() {
            println!("Last element: {}", last);
        }

        // Safe index access
        match items.get(2) {
            Some(elem) => println!("Element at index 2: {}", elem),
            None => println!("Index 2 out of bounds"),
        }

        // Out of bounds check
        match items.get(10) {
            Some(elem) => println!("Element at index 10: {}", elem),
            None => println!("Index 10 out of bounds"),
        }
    }

    println!();
}

/// Demonstrates modifying list contents
fn demonstrate_modification() {
    println!("--- Modifying Lists ---");

    let mut list = make_node(vec![make_node(10), make_node(20), make_node(30)]);

    println!("Initial: {}", list);

    // Add elements
    if let Some(items) = list.as_list_mut() {
        items.push(make_node(40));
        items.push(make_node(50));
    }
    println!("After pushing: {}", list);

    // Modify element
    if let Some(items) = list.as_list_mut() {
        if let Some(elem) = items.get_mut(1) {
            *elem = make_node(25); // Change 20 to 25
        }
    }
    println!("After modifying index 1: {}", list);

    // Insert at position
    if let Some(items) = list.as_list_mut() {
        items.insert(0, make_node(5)); // Insert at beginning
    }
    println!("After inserting at start: {}", list);

    // Remove element
    if let Some(items) = list.as_list_mut() {
        items.remove(2); // Remove the element at index 2
    }
    println!("After removing index 2: {}", list);

    // Clear all
    if let Some(items) = list.as_list_mut() {
        items.clear();
    }
    println!("After clearing: {}\n", list);
}

/// Demonstrates iterating and filtering lists
fn demonstrate_iteration() {
    println!("--- Iterating and Filtering ---");

    let list = make_node(vec![
        make_node(5),
        make_node("text"),
        make_node(10),
        make_node(15),
        make_node("more text"),
        make_node(20),
    ]);

    println!("Original list: {}", list);

    if let Some(items) = list.as_list() {
        // Count by type
        let int_count = items.iter().filter(|n| n.is_integer()).count();
        let str_count = items.iter().filter(|n| n.is_string()).count();
        println!("Integers: {}, Strings: {}", int_count, str_count);

        // Collect integers only
        println!("Integer values:");
        for item in items.iter() {
            if let Some(val) = item.as_integer() {
                println!("  {}", val);
            }
        }

        // Sum all integers
        let sum: i64 = items.iter().filter_map(|n| n.as_integer()).sum();
        println!("Sum of integers: {}", sum);

        // Find max integer
        let max = items.iter().filter_map(|n| n.as_integer()).max();
        println!("Max integer: {:?}", max);

        // Collect strings
        let strings: Vec<_> = items.iter().filter_map(|n| n.as_string()).collect();
        println!("String values: {:?}", strings);
    }

    println!();
}

/// Demonstrates list transformations
fn demonstrate_transformations() {
    println!("--- List Transformations ---");

    let list = make_node(vec![
        make_node(1),
        make_node(2),
        make_node(3),
        make_node(4),
        make_node(5),
    ]);

    println!("Original: {}", list);

    // Double all integers
    if let Some(items) = list.as_list() {
        let doubled: Vec<Node> = items
            .iter()
            .filter_map(|n| n.as_integer())
            .map(|&val| make_node(val * 2))
            .collect();
        let doubled_list = Node::List(doubled);
        println!("Doubled: {}", doubled_list);
    }

    // Filter even numbers
    if let Some(items) = list.as_list() {
        let evens: Vec<Node> = items
            .iter()
            .filter_map(|n| n.as_integer())
            .filter(|&val| val % 2 == 0)
            .map(|&val| make_node(val))
            .collect();
        let evens_list = Node::List(evens);
        println!("Even numbers: {}", evens_list);
    }

    // Convert integers to strings
    if let Some(items) = list.as_list() {
        let as_strings: Vec<Node> = items
            .iter()
            .filter_map(|n| n.as_integer())
            .map(|val| make_node(format!("num_{}", val)))
            .collect();
        let strings_list = Node::List(as_strings);
        println!("As strings: {}", strings_list);
    }

    println!();
}

/// Demonstrates working with nested lists
fn demonstrate_nested_lists() {
    println!("--- Nested Lists ---");

    // Create nested list structure
    let nested = make_node(vec![
        make_node(vec![make_node(1), make_node(2), make_node(3)]),
        make_node(vec![make_node(4), make_node(5)]),
        make_node(vec![make_node(6), make_node(7), make_node(8), make_node(9)]),
    ]);

    println!("Nested list: {}", nested);

    if let Some(outer) = nested.as_list() {
        println!("Number of sublists: {}", outer.len());

        // Iterate through nested structure
        for (i, sublist) in outer.iter().enumerate() {
            if let Some(inner) = sublist.as_list() {
                println!("Sublist {}: {} items", i, inner.len());
                for item in inner {
                    if let Some(val) = item.as_integer() {
                        print!("{} ", val);
                    }
                }
                println!();
            }
        }

        // Flatten nested list
        let flattened: Vec<Node> = outer
            .iter()
            .filter_map(|n| n.as_list())
            .flat_map(|list| list.iter())
            .filter_map(|n| n.as_integer())
            .map(|&val| make_node(val))
            .collect();
        println!("Flattened: {}", Node::List(flattened));

        // Calculate total count of all nested integers
        let total_count: usize = outer
            .iter()
            .filter_map(|n| n.as_list())
            .map(|list| list.len())
            .sum();
        println!("Total nested items: {}", total_count);
    }

    println!();
}

/// Demonstrates practical use cases for list operations
fn demonstrate_practical_cases() {
    println!("--- Practical Use Cases ---");

    // Use case 1: Tracker announce list (common in torrents)
    let announce_list = make_node(vec![
        make_node(vec![
            make_node("udp://tracker1.example.com:6969"),
            make_node("http://tracker1.example.com/announce"),
        ]),
        make_node(vec![make_node("udp://tracker2.example.com:6969")]),
    ]);

    println!("Tracker announce list:");
    if let Some(tiers) = announce_list.as_list() {
        for (tier, trackers) in tiers.iter().enumerate() {
            println!("  Tier {}:", tier);
            if let Some(tracker_list) = trackers.as_list() {
                for tracker in tracker_list {
                    if let Some(url) = tracker.as_string() {
                        println!("    - {}", url);
                    }
                }
            }
        }
    }

    // Use case 2: File path components
    let file_path = make_node(vec![
        make_node("media"),
        make_node("videos"),
        make_node("example.mp4"),
    ]);

    println!("\nFile path components: {}", file_path);
    if let Some(parts) = file_path.as_list() {
        let path_str: Vec<_> = parts.iter().filter_map(|n| n.as_string()).collect();
        println!("Full path: {}", path_str.join("/"));
    }

    // Use case 3: Tag collection
    let tags = make_node(vec![
        make_node("rust"),
        make_node("bencode"),
        make_node("serialization"),
        make_node("torrent"),
    ]);

    println!("\nTags: {}", tags);
    if let Some(tag_list) = tags.as_list() {
        println!("Has {} tags", tag_list.len());

        // Check if specific tag exists
        let has_rust = tag_list.iter().any(|n| n.as_string() == Some("rust"));
        println!("Contains 'rust' tag: {}", has_rust);
    }

    println!();
}
