//! # TOML Configuration Layering and Deep Merging Example
//!
//! Demonstrates a real-world configuration management pattern:
//! loading a base default configuration, merging environment-specific overrides,
//! and resolving the final active configuration document.

use babbel_toml::{from_str, to_string_pretty, Node};

/// Deep merges `overrides` table into `base` table.
fn deep_merge_tables(base: &mut Node, overrides: &Node) {
    if let (Node::Table(base_entries), Node::Table(override_entries)) = (base, overrides) {
        for (key, override_val) in override_entries {
            if let Some((_, existing_val)) = base_entries.iter_mut().find(|(k, _)| k == key) {
                // If both are tables, recurse
                if existing_val.is_table() && override_val.is_table() {
                    deep_merge_tables(existing_val, override_val);
                } else {
                    // Overwrite scalar, array, or mismatching container
                    *existing_val = override_val.clone();
                }
            } else {
                // New key in overrides
                base_entries.push((key.clone(), override_val.clone()));
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TOML Configuration Layering Example ===\n");

    // 1. Base Default Configuration
    let default_config = r#"
app_name = "babbel-gateway"
environment = "development"
debug = true

[server]
host = "127.0.0.1"
port = 3000
workers = 2
timeout_sec = 60

[database]
engine = "sqlite"
url = "sqlite::memory:"
pool_size = 5

[logging]
level = "debug"
json_format = false
"#;

    // 2. Production Override Layer
    let production_overrides = r#"
environment = "production"
debug = false

[server]
host = "0.0.0.0"
port = 8080
workers = 16

[database]
engine = "postgresql"
url = "postgres://db.prod.internal:5432/app"
pool_size = 50

[logging]
level = "info"
json_format = true
destination = "/var/log/babbel.log"
"#;

    println!("--- 1. Base Default Configuration ---");
    let mut config = from_str(default_config)?;
    println!("{}", to_string_pretty(&config)?);

    println!("--- 2. Applying Production Overrides ---");
    let overrides = from_str(production_overrides)?;
    deep_merge_tables(&mut config, &overrides);

    println!("--- 3. Effective Resolved Configuration ---");
    let resolved = to_string_pretty(&config)?;
    println!("{}", resolved);

    // Verify overridden fields
    assert_eq!(config.get("environment").and_then(|n| n.as_str()), Some("production"));
    assert_eq!(config.get("debug").and_then(|n| n.as_bool()), Some(false));
    assert_eq!(
        config.get("server").and_then(|s| s.get("port")).and_then(|n| n.as_i64()),
        Some(8080)
    );
    assert_eq!(
        config.get("server").and_then(|s| s.get("timeout_sec")).and_then(|n| n.as_i64()),
        Some(60) // Preserved from default!
    );
    assert_eq!(
        config.get("logging").and_then(|l| l.get("destination")).and_then(|n| n.as_str()),
        Some("/var/log/babbel.log") // Added from production!
    );

    println!("Configuration layering and deep merge verified successfully!");
    Ok(())
}
