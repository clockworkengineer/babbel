//! # TOML Parse and Stringify Example
//!
//! Demonstrates parsing a TOML document, accessing tables and fields,
//! updating values in the DOM, and serializing back with standard and pretty stringifiers.

use babbel_toml::{from_str, to_string, to_string_pretty, Node};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Parse and Stringify Example ===\n");

    let source = r#"
# Core configuration
application = "Babbel TOML Engine"
version = "0.1.2"
debug_mode = false

[server]
host = "127.0.0.1"
port = 8080
max_workers = 8

[server.timeouts]
read_seconds = 30
write_seconds = 30

[metrics]
enabled = true
endpoints = [ "/metrics", "/stats" ]
"#;

    println!("--- 1. Parsing TOML document ---");
    let mut doc = from_str(source)?;
    println!("Parsed document successfully!");

    println!("\n--- 2. Accessing document fields ---");
    if let Some(app) = doc.get("application").and_then(|n| n.as_str()) {
        println!("Application Name: {}", app);
    }
    if let Some(ver) = doc.get("version").and_then(|n| n.as_str()) {
        println!("Version:          {}", ver);
    }
    if let Some(debug) = doc.get("debug_mode").and_then(|n| n.as_bool()) {
        println!("Debug Mode:       {}", debug);
    }

    if let Some(server) = doc.get("server") {
        let host = server.get("host").and_then(|n| n.as_str()).unwrap_or("unknown");
        let port = server.get("port").and_then(|n| n.as_i64()).unwrap_or(0);
        let workers = server.get("max_workers").and_then(|n| n.as_i64()).unwrap_or(1);
        println!("Server Host:      {}:{} (workers: {})", host, port, workers);

        if let Some(timeouts) = server.get("timeouts") {
            let read_s = timeouts.get("read_seconds").and_then(|n| n.as_i64()).unwrap_or(0);
            println!("Read Timeout:     {}s", read_s);
        }
    }

    if let Some(metrics) = doc.get("metrics") {
        if let Some(endpoints) = metrics.get("endpoints").and_then(|n| n.as_array()) {
            println!("Metric Endpoints: {} configured", endpoints.len());
            for (idx, ep) in endpoints.iter().enumerate() {
                if let Some(s) = ep.as_str() {
                    println!("  [{}] {}", idx, s);
                }
            }
        }
    }

    println!("\n--- 3. Modifying the DOM in-place ---");
    // Update top-level field
    doc.insert("debug_mode".to_string(), Node::from(true));
    // Insert new top-level field
    doc.insert("environment".to_string(), Node::from("staging"));

    // Modify nested server table
    if let Some(server) = doc.get_mut("server") {
        server.insert("port".to_string(), Node::from(9090));
        server.insert("tls_enabled".to_string(), Node::from(true));
    }

    println!("\n--- 4. Serializing (Standard TOML) ---");
    let serialized = to_string(&doc)?;
    println!("{}", serialized);

    println!("--- 5. Serializing (Pretty-Printed TOML) ---");
    let pretty = to_string_pretty(&doc)?;
    println!("{}", pretty);

    // Verify roundtrip fidelity
    let roundtrip_doc = from_str(&serialized)?;
    assert_eq!(
        roundtrip_doc.get("environment").and_then(|n| n.as_str()),
        Some("staging")
    );
    assert_eq!(
        roundtrip_doc
            .get("server")
            .and_then(|s| s.get("port"))
            .and_then(|n| n.as_i64()),
        Some(9090)
    );
    println!("Roundtrip verification confirmed!");

    Ok(())
}
