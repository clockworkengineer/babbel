//! # TOML Query and Tree Traversal Example
//!
//! Demonstrates navigating deeply nested TOML structures, safe query chains,
//! recursive tree traversal (visitor pattern), node counting, and in-place transformations.

use babbel_toml::{from_str, to_string_pretty, Node};

/// Recursively inspects and counts the various types of nodes in a TOML DOM tree.
#[derive(Default, Debug)]
struct TreeMetrics {
    tables: usize,
    arrays: usize,
    strings: usize,
    integers: usize,
    floats: usize,
    booleans: usize,
    datetimes: usize,
    total_nodes: usize,
    max_depth: usize,
}

impl TreeMetrics {
    fn collect(node: &Node) -> Self {
        let mut metrics = TreeMetrics::default();
        Self::traverse(node, 1, &mut metrics);
        metrics
    }

    fn traverse(node: &Node, depth: usize, metrics: &mut TreeMetrics) {
        metrics.total_nodes += 1;
        if depth > metrics.max_depth {
            metrics.max_depth = depth;
        }

        match node {
            Node::Table(entries) => {
                metrics.tables += 1;
                for (_, child) in entries {
                    Self::traverse(child, depth + 1, metrics);
                }
            }
            Node::Array(items) => {
                metrics.arrays += 1;
                for item in items {
                    Self::traverse(item, depth + 1, metrics);
                }
            }
            Node::String(_) => metrics.strings += 1,
            Node::Integer(_) => metrics.integers += 1,
            Node::Float(_) => metrics.floats += 1,
            Node::Boolean(_) => metrics.booleans += 1,
            Node::Datetime(_) => metrics.datetimes += 1,
        }
    }
}

/// Recursively traverses the tree and transforms all strings to uppercase.
fn uppercase_all_strings(node: &mut Node) {
    match node {
        Node::String(s) => *s = s.to_uppercase(),
        Node::Table(entries) => {
            for (_, child) in entries.iter_mut() {
                uppercase_all_strings(child);
            }
        }
        Node::Array(items) => {
            for item in items.iter_mut() {
                uppercase_all_strings(item);
            }
        }
        _ => {}
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Query & Tree Traversal Example ===\n");

    let complex_source = r#"
# Microservice Configuration
[service]
name = "orders-engine"
environment = "production"
active = true

[service.networking]
listen_address = "0.0.0.0"
port = 8443
ssl_enabled = true

[service.networking.rate_limit]
requests_per_minute = 12000
burst_size = 500

[[service.dependencies]]
name = "auth-service"
endpoint = "https://auth.internal:8080"
timeout_ms = 1500

[[service.dependencies]]
name = "inventory-service"
endpoint = "https://inventory.internal:8080"
timeout_ms = 3000

[telemetry]
sample_rate = 0.05
tags = [ "orders", "checkout", "v2" ]
"#;

    let mut doc = from_str(complex_source)?;

    // 1. Safe Query Chains
    println!("--- 1. Safe Query Chains ---");
    let rate_limit = doc
        .get("service")
        .and_then(|s| s.get("networking"))
        .and_then(|n| n.get("rate_limit"))
        .and_then(|r| r.get("requests_per_minute"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    println!("Rate Limit (req/min): {}", rate_limit);

    // Query dependency names
    if let Some(deps) = doc.get("service").and_then(|s| s.get("dependencies")).and_then(|d| d.as_array()) {
        println!("Dependencies ({} total):", deps.len());
        for (i, dep) in deps.iter().enumerate() {
            let name = dep.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
            let endpoint = dep.get("endpoint").and_then(|n| n.as_str()).unwrap_or("none");
            let timeout = dep.get("timeout_ms").and_then(|n| n.as_i64()).unwrap_or(0);
            println!("  [{}] {} -> {} (timeout: {}ms)", i, name, endpoint, timeout);
        }
    }

    // 2. Tree Metrics Collection
    println!("\n--- 2. Tree Metrics Collection ---");
    let metrics = TreeMetrics::collect(&doc);
    println!("{:#?}", metrics);

    // 3. In-Place Transformation
    println!("\n--- 3. In-Place Tree Transformation ---");
    uppercase_all_strings(&mut doc);
    println!("Document after uppercase string transformation:\n");
    println!("{}", to_string_pretty(&doc)?);

    Ok(())
}
