//! Demonstrates JSON merging and patching operations
//!
//! Shows how to merge JSON objects, apply configuration overlays,
//! and manage hierarchical configurations with deep merge strategies.

use json_lib::{parse, stringify, BufferDestination, BufferSource, Node};
use std::collections::HashMap;

fn main() {
    println!("=== JSON Merge and Patch Demo ===\n");

    // Example 1: Simple object merge
    println!("1. Simple object merge:");

    let mut base = Node::Object({
        let mut map = HashMap::new();
        map.insert("name".to_string(), Node::from("MyApp"));
        map.insert("version".to_string(), Node::from("1.0.0"));
        map.insert("debug".to_string(), Node::from(false));
        map
    });

    let overlay = Node::Object({
        let mut map = HashMap::new();
        map.insert("version".to_string(), Node::from("1.1.0"));
        map.insert("author".to_string(), Node::from("John Doe"));
        map
    });

    println!("  Base: {}", node_to_string(&base));
    println!("  Overlay: {}", node_to_string(&overlay));

    base.merge(overlay);

    println!("  Merged: {}", node_to_string(&base));

    // Example 2: Deep merge with nested objects
    println!("\n2. Deep merge with nested objects:");

    let mut config1 = Node::Object({
        let mut map = HashMap::new();
        map.insert(
            "app".to_string(),
            Node::Object({
                let mut inner = HashMap::new();
                inner.insert("name".to_string(), Node::from("Service"));
                inner.insert("port".to_string(), Node::from(8080));
                inner
            }),
        );
        map.insert(
            "database".to_string(),
            Node::Object({
                let mut inner = HashMap::new();
                inner.insert("host".to_string(), Node::from("localhost"));
                inner.insert("port".to_string(), Node::from(5432));
                inner
            }),
        );
        map
    });

    let config2 = Node::Object({
        let mut map = HashMap::new();
        map.insert(
            "app".to_string(),
            Node::Object({
                let mut inner = HashMap::new();
                inner.insert("port".to_string(), Node::from(9090));
                inner.insert("timeout".to_string(), Node::from(30));
                inner
            }),
        );
        map.insert(
            "cache".to_string(),
            Node::Object({
                let mut inner = HashMap::new();
                inner.insert("enabled".to_string(), Node::from(true));
                inner
            }),
        );
        map
    });

    println!("  Config 1:");
    print_pretty(&config1, 4);

    println!("  Config 2:");
    print_pretty(&config2, 4);

    config1.merge(config2);

    println!("  Merged config:");
    print_pretty(&config1, 4);

    // Example 3: Environment-specific configuration merging
    println!("\n3. Environment-specific configuration:");

    // Base configuration
    let base_config = Node::Object({
        let mut map = HashMap::new();
        map.insert("app_name".to_string(), Node::from("MyService"));
        map.insert("log_level".to_string(), Node::from("info"));
        map.insert(
            "features".to_string(),
            Node::Object({
                let mut inner = HashMap::new();
                inner.insert("feature_a".to_string(), Node::from(true));
                inner.insert("feature_b".to_string(), Node::from(false));
                inner
            }),
        );
        map
    });

    // Development environment overrides
    let dev_overrides = Node::Object({
        let mut map = HashMap::new();
        map.insert("log_level".to_string(), Node::from("debug"));
        map.insert(
            "features".to_string(),
            Node::Object({
                let mut inner = HashMap::new();
                inner.insert("feature_b".to_string(), Node::from(true));
                inner.insert("feature_c".to_string(), Node::from(true));
                inner
            }),
        );
        map
    });

    println!("  Base configuration:");
    print_pretty(&base_config, 4);

    let mut dev_config = base_config.clone();
    dev_config.merge(dev_overrides);

    println!("  Development configuration:");
    print_pretty(&dev_config, 4);

    // Example 4: Array handling in merge
    println!("\n4. Array replacement (not merge):");

    let mut node1 = Node::Object({
        let mut map = HashMap::new();
        map.insert(
            "items".to_string(),
            Node::Array(vec![Node::from(1), Node::from(2), Node::from(3)]),
        );
        map
    });

    let node2 = Node::Object({
        let mut map = HashMap::new();
        map.insert(
            "items".to_string(),
            Node::Array(vec![Node::from(10), Node::from(20)]),
        );
        map
    });

    println!("  Node 1: {}", node_to_string(&node1));
    println!("  Node 2: {}", node_to_string(&node2));

    node1.merge(node2);

    println!("  Merged (array replaced): {}", node_to_string(&node1));

    // Example 5: Configuration layers (cascading merge)
    println!("\n5. Cascading configuration layers:");

    let defaults = Node::Object({
        let mut map = HashMap::new();
        map.insert("timeout".to_string(), Node::from(30));
        map.insert("retries".to_string(), Node::from(3));
        map.insert("cache_size".to_string(), Node::from(100));
        map
    });

    let user_config = Node::Object({
        let mut map = HashMap::new();
        map.insert("timeout".to_string(), Node::from(60));
        map
    });

    let runtime_overrides = Node::Object({
        let mut map = HashMap::new();
        map.insert("retries".to_string(), Node::from(5));
        map
    });

    println!("  Layer 1 (defaults): {}", node_to_string(&defaults));
    println!("  Layer 2 (user): {}", node_to_string(&user_config));
    println!(
        "  Layer 3 (runtime): {}",
        node_to_string(&runtime_overrides)
    );

    let mut final_config = defaults.clone();
    final_config.merge(user_config);
    final_config.merge(runtime_overrides);

    println!("  Final config: {}", node_to_string(&final_config));

    // Example 6: Conditional merge based on content
    println!("\n6. Conditional merge:");

    let mut target = Node::Object({
        let mut map = HashMap::new();
        map.insert("status".to_string(), Node::from("active"));
        map.insert("count".to_string(), Node::from(10));
        map
    });

    let update1 = Node::Object({
        let mut map = HashMap::new();
        map.insert("count".to_string(), Node::from(20));
        map.insert("updated".to_string(), Node::from(true));
        map
    });

    let update2 = Node::Object({
        let mut map = HashMap::new();
        map.insert("status".to_string(), Node::from("inactive"));
        map
    });

    println!("  Initial: {}", node_to_string(&target));

    // Apply first update
    target.merge(update1.clone());
    println!("  After update 1: {}", node_to_string(&target));

    // Conditionally apply second update only if status is active
    let should_apply = target
        .get("status")
        .and_then(|s| s.as_str())
        .map(|s| s == "active")
        .unwrap_or(false);

    if should_apply {
        target.merge(update2);
        println!("  After conditional update 2: {}", node_to_string(&target));
    }

    // Example 7: Merge with JSON strings
    println!("\n7. Merging from JSON strings:");

    let json1 = r#"{"server": {"host": "localhost", "port": 8080}}"#;
    let json2 = r#"{"server": {"port": 9090, "ssl": true}, "logging": {"level": "info"}}"#;

    let mut source1 = BufferSource::new(json1.as_bytes());
    let mut node1 = parse(&mut source1).expect("Parse failed");

    let mut source2 = BufferSource::new(json2.as_bytes());
    let node2 = parse(&mut source2).expect("Parse failed");

    println!("  JSON 1: {}", json1);
    println!("  JSON 2: {}", json2);

    node1.merge(node2);

    println!("  Merged:");
    print_pretty(&node1, 4);

    // Example 8: Building a patch function
    println!("\n8. Custom patch function:");

    fn apply_patch(target: &mut Node, patch: &Node) {
        if let (Some(target_obj), Some(patch_obj)) = (target.as_object_mut(), patch.as_object()) {
            for (key, value) in patch_obj {
                if value.is_null() {
                    // null means remove the key
                    target_obj.remove(key);
                } else {
                    // Otherwise merge/set the value
                    target_obj
                        .entry(key.clone())
                        .and_modify(|existing| {
                            if existing.is_object() && value.is_object() {
                                existing.merge(value.clone());
                            } else {
                                *existing = value.clone();
                            }
                        })
                        .or_insert(value.clone());
                }
            }
        }
    }

    let mut document = Node::Object({
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::from("old_a"));
        map.insert("b".to_string(), Node::from("keep_b"));
        map.insert("c".to_string(), Node::from("remove_me"));
        map
    });

    let patch = Node::Object({
        let mut map = HashMap::new();
        map.insert("a".to_string(), Node::from("new_a"));
        map.insert("c".to_string(), Node::None); // Remove this key
        map.insert("d".to_string(), Node::from("add_d"));
        map
    });

    println!("  Document before: {}", node_to_string(&document));
    println!("  Patch: {}", node_to_string(&patch));

    apply_patch(&mut document, &patch);

    println!("  Document after: {}", node_to_string(&document));

    println!("\n=== Demo Complete ===");
}

// Helper function to convert Node to compact JSON string
fn node_to_string(node: &Node) -> String {
    let mut buffer = BufferDestination::new();
    stringify(node, &mut buffer).unwrap();
    buffer.to_string()
}

// Helper function to pretty print a Node
fn print_pretty(node: &Node, indent: usize) {
    let mut buffer = BufferDestination::new();
    json_lib::print(node, &mut buffer, indent);
    print!("{}", buffer.to_string());
}
